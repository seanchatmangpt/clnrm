//! Test Runner
//!
//! Orchestrates test execution using Config and ContainerManager.
//! Executes steps in dependency order, validates assertions, and reports results.

use crate::config::spec::{Config, Step, StepAssertions};
use crate::error::{CleanroomError, Result};
use crate::executor::container_manager::{ContainerHandle, ContainerManager, ExecResult};
use regex::Regex;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Result of a single step execution
#[derive(Debug, Clone)]
pub struct StepResult {
    /// Step name
    pub name: String,

    /// Whether the step passed
    pub passed: bool,

    /// Exit code from command
    pub exit_code: i32,

    /// Standard output
    pub stdout: String,

    /// Standard error
    pub stderr: String,

    /// Execution duration
    pub duration: Duration,

    /// Failure reason if any
    pub failure_reason: Option<String>,

    /// Number of retry attempts
    pub retry_attempts: u32,
}

/// Result of a full test execution
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Test name
    pub test_name: String,

    /// Overall test passed
    pub passed: bool,

    /// Results for each step
    pub step_results: Vec<StepResult>,

    /// Total execution duration
    pub total_duration: Duration,

    /// Containers used in test
    pub containers_used: Vec<String>,

    /// Summary message
    pub summary: String,
}

/// Test runner that orchestrates execution
pub struct TestRunner<M: ContainerManager> {
    /// Container manager for lifecycle and execution
    manager: M,

    /// Active container handles
    handles: HashMap<String, ContainerHandle>,
}

impl<M: ContainerManager> TestRunner<M> {
    /// Create a new test runner
    pub fn new(manager: M) -> Self {
        Self {
            manager,
            handles: HashMap::new(),
        }
    }

    /// Run a complete test from config
    pub async fn run(&mut self, config: &Config) -> Result<ExecutionResult> {
        let start = Instant::now();
        let mut step_results = Vec::new();
        let mut all_passed = true;

        // Start all containers
        self.start_containers(config).await?;

        // Get execution order (respects depends_on)
        let execution_order = config.step_execution_order()?;

        // Execute steps in order
        for step_name in &execution_order {
            let step = config
                .steps
                .iter()
                .find(|s| &s.name == step_name)
                .ok_or_else(|| {
                    CleanroomError::internal_error(format!("Step '{}' not found", step_name))
                })?;

            let result = self.execute_step(step, config).await;

            match result {
                Ok(step_result) => {
                    if !step_result.passed {
                        all_passed = false;
                    }
                    step_results.push(step_result);
                }
                Err(e) => {
                    all_passed = false;
                    step_results.push(StepResult {
                        name: step.name.clone(),
                        passed: false,
                        exit_code: -1,
                        stdout: String::new(),
                        stderr: e.to_string(),
                        duration: Duration::ZERO,
                        failure_reason: Some(e.to_string()),
                        retry_attempts: 0,
                    });
                }
            }

            // Stop on first failure unless configured otherwise
            if !all_passed {
                break;
            }
        }

        // Stop all containers
        self.stop_containers().await?;

        let total_duration = start.elapsed();
        let containers_used: Vec<String> = config.containers.keys().cloned().collect();

        let passed_count = step_results.iter().filter(|r| r.passed).count();
        let total_count = step_results.len();
        let summary = if all_passed {
            format!(
                "PASSED: {}/{} steps in {:?}",
                passed_count, total_count, total_duration
            )
        } else {
            format!(
                "FAILED: {}/{} steps passed in {:?}",
                passed_count, total_count, total_duration
            )
        };

        Ok(ExecutionResult {
            test_name: config.test.name.clone(),
            passed: all_passed,
            step_results,
            total_duration,
            containers_used,
            summary,
        })
    }

    /// Start all containers defined in config
    async fn start_containers(&mut self, config: &Config) -> Result<()> {
        // Get container startup order (respects depends_on)
        let startup_order = self.container_startup_order(config)?;

        for name in startup_order {
            let spec = config.containers.get(&name).ok_or_else(|| {
                CleanroomError::internal_error(format!("Container '{}' not found", name))
            })?;

            let handle = self.manager.start(&name, spec).await?;
            self.handles.insert(name, handle);
        }

        Ok(())
    }

    /// Stop all containers
    async fn stop_containers(&mut self) -> Result<()> {
        // Stop in reverse order
        let names: Vec<String> = self.handles.keys().cloned().collect();
        for name in names.into_iter().rev() {
            if let Some(handle) = self.handles.remove(&name) {
                // Best effort stop - don't fail if already stopped
                let _ = self.manager.stop(&handle).await;
            }
        }
        Ok(())
    }

    /// Calculate container startup order based on depends_on
    fn container_startup_order(&self, config: &Config) -> Result<Vec<String>> {
        let mut order = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut visiting = std::collections::HashSet::new();

        fn visit(
            name: &str,
            config: &Config,
            order: &mut Vec<String>,
            visited: &mut std::collections::HashSet<String>,
            visiting: &mut std::collections::HashSet<String>,
        ) -> Result<()> {
            if visited.contains(name) {
                return Ok(());
            }
            if visiting.contains(name) {
                return Err(CleanroomError::validation_error(format!(
                    "Circular container dependency detected: {}",
                    name
                )));
            }

            visiting.insert(name.to_string());

            if let Some(spec) = config.containers.get(name) {
                for dep in &spec.depends_on {
                    visit(dep, config, order, visited, visiting)?;
                }
            }

            visiting.remove(name);
            visited.insert(name.to_string());
            order.push(name.to_string());

            Ok(())
        }

        for name in config.containers.keys() {
            visit(name, config, &mut order, &mut visited, &mut visiting)?;
        }

        Ok(order)
    }

    /// Execute a single step with retry logic
    async fn execute_step(&self, step: &Step, config: &Config) -> Result<StepResult> {
        let handle = self.handles.get(&step.container).ok_or_else(|| {
            CleanroomError::internal_error(format!(
                "Container '{}' not running for step '{}'",
                step.container, step.name
            ))
        })?;

        // Merge container env with step env (step takes precedence)
        let mut env = config
            .containers
            .get(&step.container)
            .map(|c| c.env.clone())
            .unwrap_or_default();
        env.extend(step.env.clone());

        // Determine retry configuration
        let max_attempts = step.retry.as_ref().map(|r| r.attempts).unwrap_or(1);
        let retry_delay = step
            .retry
            .as_ref()
            .and_then(|r| parse_duration(&r.delay).ok())
            .unwrap_or(Duration::from_secs(1));

        let mut last_result: Option<ExecResult> = None;
        let mut attempt = 0;
        let start = Instant::now();

        // Retry loop
        while attempt < max_attempts {
            attempt += 1;

            let result = self.manager.exec(handle, &step.exec, &env).await?;

            // Check if step passed
            if let Some(assertions) = &step.assert {
                if self.check_assertions(&result, assertions)? {
                    return Ok(StepResult {
                        name: step.name.clone(),
                        passed: true,
                        exit_code: result.exit_code,
                        stdout: result.stdout,
                        stderr: result.stderr,
                        duration: start.elapsed(),
                        failure_reason: None,
                        retry_attempts: attempt - 1,
                    });
                }
            } else {
                // No assertions - just check exit code is 0
                if result.exit_code == 0 {
                    return Ok(StepResult {
                        name: step.name.clone(),
                        passed: true,
                        exit_code: result.exit_code,
                        stdout: result.stdout,
                        stderr: result.stderr,
                        duration: start.elapsed(),
                        failure_reason: None,
                        retry_attempts: attempt - 1,
                    });
                }
            }

            last_result = Some(result);

            // Wait before retry (unless last attempt)
            if attempt < max_attempts {
                tokio::time::sleep(retry_delay).await;
            }
        }

        // All retries exhausted - return failure
        let result = last_result.unwrap_or(ExecResult {
            exit_code: -1,
            stdout: String::new(),
            stderr: "No execution result".to_string(),
            duration: Duration::ZERO,
        });

        let failure_reason = self.get_failure_reason(&result, step.assert.as_ref());

        Ok(StepResult {
            name: step.name.clone(),
            passed: false,
            exit_code: result.exit_code,
            stdout: result.stdout,
            stderr: result.stderr,
            duration: start.elapsed(),
            failure_reason: Some(failure_reason),
            retry_attempts: attempt - 1,
        })
    }

    /// Check if assertions pass
    fn check_assertions(&self, result: &ExecResult, assertions: &StepAssertions) -> Result<bool> {
        // Check exit code
        if let Some(expected_exit) = assertions.exit_code {
            if result.exit_code != expected_exit {
                return Ok(false);
            }
        }

        // Check stdout contains
        if let Some(expected) = &assertions.stdout_contains {
            if !result.stdout.contains(expected) {
                return Ok(false);
            }
        }

        // Check stdout not contains
        if let Some(not_expected) = &assertions.stdout_not_contains {
            if result.stdout.contains(not_expected) {
                return Ok(false);
            }
        }

        // Check stdout regex
        if let Some(pattern) = &assertions.stdout_regex {
            let re = Regex::new(pattern).map_err(|e| {
                CleanroomError::validation_error(format!("Invalid regex '{}': {}", pattern, e))
            })?;
            if !re.is_match(&result.stdout) {
                return Ok(false);
            }
        }

        // Check stderr contains
        if let Some(expected) = &assertions.stderr_contains {
            if !result.stderr.contains(expected) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Get human-readable failure reason
    fn get_failure_reason(
        &self,
        result: &ExecResult,
        assertions: Option<&StepAssertions>,
    ) -> String {
        let Some(assertions) = assertions else {
            return format!("Command exited with code {} (expected 0)", result.exit_code);
        };

        let mut reasons = Vec::new();

        if let Some(expected_exit) = assertions.exit_code {
            if result.exit_code != expected_exit {
                reasons.push(format!(
                    "Exit code {} (expected {})",
                    result.exit_code, expected_exit
                ));
            }
        }

        if let Some(expected) = &assertions.stdout_contains {
            if !result.stdout.contains(expected) {
                reasons.push(format!("stdout missing '{}'", expected));
            }
        }

        if let Some(not_expected) = &assertions.stdout_not_contains {
            if result.stdout.contains(not_expected) {
                reasons.push(format!("stdout unexpectedly contains '{}'", not_expected));
            }
        }

        if let Some(pattern) = &assertions.stdout_regex {
            if let Ok(re) = Regex::new(pattern) {
                if !re.is_match(&result.stdout) {
                    reasons.push(format!("stdout doesn't match regex '{}'", pattern));
                }
            }
        }

        if let Some(expected) = &assertions.stderr_contains {
            if !result.stderr.contains(expected) {
                reasons.push(format!("stderr missing '{}'", expected));
            }
        }

        if reasons.is_empty() {
            "Unknown failure".to_string()
        } else {
            reasons.join("; ")
        }
    }
}

/// Parse duration string like "30s", "5m", "1h"
fn parse_duration(s: &str) -> Result<Duration> {
    let s = s.trim();
    if s.is_empty() {
        return Err(CleanroomError::validation_error("Empty duration string"));
    }

    let (num_str, unit) = if let Some(stripped) = s.strip_suffix("ms") {
        (stripped, "ms")
    } else if let Some(stripped) = s.strip_suffix('s') {
        (stripped, "s")
    } else if let Some(stripped) = s.strip_suffix('m') {
        (stripped, "m")
    } else if let Some(stripped) = s.strip_suffix('h') {
        (stripped, "h")
    } else {
        return Err(CleanroomError::validation_error(format!(
            "Invalid duration format '{}' (expected 30s, 5m, 1h, etc.)",
            s
        )));
    };

    let num: u64 = num_str.parse().map_err(|_| {
        CleanroomError::validation_error(format!("Invalid duration number: {}", num_str))
    })?;

    match unit {
        "ms" => Ok(Duration::from_millis(num)),
        "s" => Ok(Duration::from_secs(num)),
        "m" => Ok(Duration::from_secs(num * 60)),
        "h" => Ok(Duration::from_secs(num * 3600)),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration_seconds() {
        let dur = parse_duration("30s").unwrap();
        assert_eq!(dur, Duration::from_secs(30));
    }

    #[test]
    fn test_parse_duration_minutes() {
        let dur = parse_duration("5m").unwrap();
        assert_eq!(dur, Duration::from_secs(300));
    }

    #[test]
    fn test_parse_duration_hours() {
        let dur = parse_duration("2h").unwrap();
        assert_eq!(dur, Duration::from_secs(7200));
    }

    #[test]
    fn test_parse_duration_milliseconds() {
        let dur = parse_duration("500ms").unwrap();
        assert_eq!(dur, Duration::from_millis(500));
    }

    #[test]
    fn test_parse_duration_invalid() {
        assert!(parse_duration("").is_err());
        assert!(parse_duration("30").is_err());
        assert!(parse_duration("abc").is_err());
    }

    #[test]
    fn test_step_result_creation() {
        let result = StepResult {
            name: "test_step".to_string(),
            passed: true,
            exit_code: 0,
            stdout: "hello".to_string(),
            stderr: String::new(),
            duration: Duration::from_secs(1),
            failure_reason: None,
            retry_attempts: 0,
        };

        assert!(result.passed);
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_execution_result_summary() {
        let result = ExecutionResult {
            test_name: "my_test".to_string(),
            passed: true,
            step_results: vec![],
            total_duration: Duration::from_secs(5),
            containers_used: vec!["alpine".to_string()],
            summary: "PASSED: 2/2 steps".to_string(),
        };

        assert!(result.passed);
        assert!(result.summary.contains("PASSED"));
    }
}
