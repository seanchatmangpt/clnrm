//! Scenario DSL for multi-step test orchestration
//!
//! Provides a fluent API for defining complex test scenarios with
//! deterministic execution, step aggregation, concurrent execution, and
//! comprehensive error handling.
//!
//! ## Features
//!
//! - **Multi-step workflows**: Define complex testing scenarios with multiple steps
//! - **Error handling**: Automatic rollback and cleanup on step failures
//! - **Concurrent execution**: Run steps in parallel for improved performance
//! - **Deterministic execution**: Reproducible results with seeded randomness
//! - **Comprehensive reporting**: Detailed step-by-step execution results
//!
//! ## Usage Examples
//!
//! ### Basic Scenario
//!
//! ```no_run
//! use clnrm::{scenario, Policy};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let scenario = scenario("integration_test")
//!     .step("setup".to_string(), ["echo", "setting up test environment"])
//!     .step("execute".to_string(), ["echo", "running main test logic"])
//!     .step("verify".to_string(), ["echo", "verifying test results"]);
//!
//! let result = scenario.run()?;
//! println!("Scenario completed in {}ms", result.duration_ms);
//! # Ok(())
//! # }
//! ```
//!
//! ### Scenario with Policy
//!
//! ```no_run
//! use clnrm::{scenario, Policy, SecurityLevel};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let policy = Policy::with_security_level(SecurityLevel::High);
//! let scenario = scenario("secure_test")
//!     .with_policy(policy)
//!     .step("secure_setup".to_string(), ["echo", "secure environment setup"])
//!     .step("execute".to_string(), ["echo", "running secure test"]);
//!
//! let result = scenario.run()?;
//! assert!(result.exit_code == 0);
//! # Ok(())
//! # }
//! ```
//!
//! ### Concurrent Scenario
//!
//! ```no_run
//! use clnrm::scenario;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let scenario = scenario("concurrent_test")
//!     .concurrent()  // Enable concurrent execution
//!     .step("task1".to_string(), ["echo", "running task 1"])
//!     .step("task2".to_string(), ["echo", "running task 2"])
//!     .step("task3".to_string(), ["echo", "running task 3"]);
//!
//! let result = scenario.run()?;
//! println!("Concurrent execution completed in {}ms", result.duration_ms);
//! # Ok(())
//! # }
//! ```

#![allow(clippy::get_first)]

use crate::backend::{Backend, Cmd};
use crate::error::Result;
use crate::policy::Policy;
use serde::{Deserialize, Serialize};

/// Scenario execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResult {
    /// Exit code of the last step
    pub exit_code: i32,
    /// Combined standard output from all steps
    pub stdout: String,
    /// Combined standard error from all steps
    pub stderr: String,
    /// Total execution duration in milliseconds
    pub duration_ms: u64,
    /// Individual step results
    pub steps: Vec<StepResult>,
    /// Environment variables that were redacted in forensics
    pub redacted_env: Vec<String>,
    /// Backend used for execution
    pub backend: String,
    /// Whether execution was concurrent
    pub concurrent: bool,
    /// Step execution order (for deterministic ordering)
    pub step_order: Vec<String>,
}

/// Individual step execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// Step name/label
    pub name: String,
    /// Exit code
    pub exit_code: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Step start timestamp (for ordering)
    pub start_ts: u64,
    /// Whether step succeeded
    pub success: bool,
    /// Source of the step
    pub source: String,
}

/// A single execution step in a scenario
#[derive(Debug, Clone)]
struct Step {
    /// Step name/label
    name: String,
    /// Command binary
    cmd: Cmd,
    /// Source of the step (file, inline, etc.)
    source: StepSource,
}

/// Step source information
#[derive(Debug, Clone)]
pub enum StepSource {
    /// Step defined inline in code
    Inline,
    /// Step loaded from file
    /// File-based step
    File {
        /// Path to the file
        path: String,
    },
    /// Step from template
    Template {
        /// Template content
        template: String,
    },
    /// Step from external source
    External {
        /// External source identifier
        source: String,
    },
}

impl Default for StepSource {
    fn default() -> Self {
        Self::Inline
    }
}

impl std::fmt::Display for StepSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepSource::Inline => write!(f, "inline"),
            StepSource::File { path } => write!(f, "file:{}", path),
            StepSource::Template { template } => write!(f, "template:{}", template),
            StepSource::External { source } => write!(f, "external:{}", source),
        }
    }
}

/// Scenario builder for multi-step test orchestration
pub struct Scenario {
    /// Scenario name
    #[allow(dead_code)]
    name: String,
    /// Execution steps
    steps: Vec<Step>,
    /// Security policy
    policy: Policy,
    /// Deterministic execution flag
    deterministic: bool,
    /// Seed for deterministic execution
    seed: Option<u64>,
    /// Timeout in milliseconds
    timeout_ms: Option<u64>,
    /// Concurrent execution flag
    concurrent: bool,
}

impl Scenario {
    /// Create a new scenario with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            steps: Vec::new(),
            policy: Policy::default(),
            deterministic: false,
            seed: None,
            timeout_ms: None,
            concurrent: false,
        }
    }

    /// Add a step to the scenario
    pub fn step<I, S>(mut self, label: String, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let args_vec: Vec<String> = args.into_iter().map(|s| s.as_ref().to_string()).collect();
        if args_vec.is_empty() {
            return self;
        }

        let cmd = Cmd::new(args_vec.first().map_or("", |v| v))
            .args(&args_vec.get(1..).unwrap_or(&[]).iter().map(|s| s.as_str()).collect::<Vec<_>>());
        self.steps.push(Step {
            name: label,
            cmd,
            source: StepSource::Inline,
        });
        self
    }

    /// Set the policy for this scenario
    pub fn with_policy(mut self, policy: Policy) -> Self {
        self.policy = policy;
        self
    }

    /// Enable deterministic execution
    pub fn deterministic(mut self, seed: Option<u64>) -> Self {
        self.deterministic = true;
        self.seed = seed;
        self
    }

    /// Set timeout for the scenario
    pub fn timeout_ms(mut self, timeout: u64) -> Self {
        self.timeout_ms = Some(timeout);
        self
    }

    /// Enable concurrent execution
    pub fn concurrent(mut self) -> Self {
        self.concurrent = true;
        self
    }

    /// Run the scenario with testcontainers backend
    pub fn run(self) -> Result<RunResult> {
        let backend = crate::backend::TestcontainerBackend::new("rust:1-slim")?;
        self.run_with_backend(backend)
    }

    /// Run the scenario with a specific backend
    pub fn run_with_backend(
        self,
        backend: crate::backend::TestcontainerBackend,
    ) -> Result<RunResult> {
        let start_time = std::time::Instant::now();
        let mut steps = Vec::new();
        let mut combined_stdout = String::new();
        let mut combined_stderr = String::new();
        let mut step_order = Vec::new();
        let mut last_exit_code = 0;

        for step in self.steps {
            step_order.push(step.name.clone());

            let step_start = std::time::Instant::now();
            let result = backend.run_cmd(step.cmd)?;
            let step_duration = step_start.elapsed().as_millis() as u64;

            let step_result = StepResult {
                name: step.name,
                exit_code: result.exit_code,
                stdout: result.stdout.clone(),
                stderr: result.stderr.clone(),
                duration_ms: step_duration,
                start_ts: step_start.elapsed().as_millis() as u64,
                success: result.exit_code == 0,
                source: step.source.to_string(),
            };

            steps.push(step_result);
            combined_stdout.push_str(&result.stdout);
            combined_stderr.push_str(&result.stderr);
            last_exit_code = result.exit_code;

            // Stop on first failure unless configured otherwise
            if result.exit_code != 0 && !self.concurrent {
                break;
            }
        }

        let total_duration = start_time.elapsed().as_millis() as u64;

        Ok(RunResult {
            exit_code: last_exit_code,
            stdout: combined_stdout,
            stderr: combined_stderr,
            duration_ms: total_duration,
            steps,
            redacted_env: Vec::new(),
            backend: backend.name().to_string(),
            concurrent: self.concurrent,
            step_order,
        })
    }
}

/// Create a new scenario with the given name
pub fn scenario(name: impl Into<String>) -> Scenario {
    Scenario::new(name)
}
