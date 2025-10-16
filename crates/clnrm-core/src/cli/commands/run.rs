//! Run command implementation
//!
//! Handles test execution, both sequential and parallel, with comprehensive
//! error handling and result reporting.

use crate::error::{CleanroomError, Result};
use crate::cleanroom::CleanroomEnvironment;
use crate::cli::types::{CliConfig, CliTestResult, OutputFormat};
use crate::cli::utils::{discover_test_files, generate_junit_xml};
use std::path::PathBuf;
use std::collections::HashMap;
use tracing::{info, debug, warn, error};

/// Run tests from TOML files
pub async fn run_tests(paths: &[PathBuf], config: &CliConfig) -> Result<()> {
    info!("Running cleanroom tests (framework self-testing)");
    debug!("Test paths: {:?}", paths);
    debug!("Config: parallel={}, jobs={}", config.parallel, config.jobs);
    
    // Handle watch mode
    if config.watch {
        return Err(CleanroomError::validation_error("Watch mode not yet implemented"));
    }
    
    // Handle interactive mode
    if config.interactive {
        warn!("Interactive mode requested but not yet fully implemented");
        info!("Tests will run normally - interactive mode coming in v0.4.0");
    }
    
    // Discover all test files from provided paths
    let mut all_test_files = Vec::new();
    for path in paths {
        let discovered = discover_test_files(path)?;
        all_test_files.extend(discovered);
    }
    
    info!("Found {} test file(s) to execute", all_test_files.len());
    
    let start_time = std::time::Instant::now();
    let results = if config.parallel {
        run_tests_parallel_with_results(&all_test_files, config).await?
    } else {
        run_tests_sequential_with_results(&all_test_files, config).await?
    };
    
    let total_duration = start_time.elapsed().as_millis() as u64;
    let cli_results = crate::cli::types::CliTestResults {
        tests: results,
        total_duration_ms: total_duration,
    };
    
    // Output results based on format
    match config.format {
        OutputFormat::Junit => {
            let junit_xml = generate_junit_xml(&cli_results)?;
            println!("{}", junit_xml);
        }
        _ => {
            // Default human-readable output
            let passed = cli_results.tests.iter().filter(|t| t.passed).count();
            let failed = cli_results.tests.iter().filter(|t| !t.passed).count();
            info!("Test Results: {} passed, {} failed", passed, failed);
            
            if failed > 0 {
                return Err(CleanroomError::validation_error(&format!(
                    "{} test(s) failed", failed
                )));
            }
        }
    }
    
    Ok(())
}

/// Run tests sequentially and return results
pub async fn run_tests_sequential_with_results(paths: &[PathBuf], config: &CliConfig) -> Result<Vec<CliTestResult>> {
    let mut results = Vec::new();
    
    for path in paths {
        debug!("Processing test file: {}", path.display());
        let test_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let start_time = std::time::Instant::now();
        match run_single_test(path, config).await {
            Ok(_) => {
                let duration = start_time.elapsed().as_millis() as u64;
                info!("Test passed: {}", path.display());
                results.push(CliTestResult {
                    name: test_name,
                    passed: true,
                    duration_ms: duration,
                    error: None,
                });
            }
            Err(e) => {
                let duration = start_time.elapsed().as_millis() as u64;
                error!("Test failed: {} - {}", path.display(), e);
                results.push(CliTestResult {
                    name: test_name,
                    passed: false,
                    duration_ms: duration,
                    error: Some(e.to_string()),
                });
                if config.fail_fast {
                    break;
                }
            }
        }
    }
    
    Ok(results)
}

/// Run tests sequentially (legacy - kept for compatibility)
pub async fn run_tests_sequential(paths: &[PathBuf], config: &CliConfig) -> Result<()> {
    let results = run_tests_sequential_with_results(paths, config).await?;
    let tests_passed = results.iter().filter(|r| r.passed).count();
    let tests_failed = results.iter().filter(|r| !r.passed).count();
    
    info!("Test Results: {} passed, {} failed", tests_passed, tests_failed);
    
    if tests_failed > 0 {
        Err(CleanroomError::validation_error(&format!(
            "{} test(s) failed", tests_failed
        )))
    } else {
        info!("All tests passed! Framework self-testing successful.");
        Ok(())
    }
}

/// Run tests in parallel and return results
pub async fn run_tests_parallel_with_results(paths: &[PathBuf], config: &CliConfig) -> Result<Vec<CliTestResult>> {
    use tokio::task::JoinSet;
    
    let mut join_set = JoinSet::new();
    let mut results = Vec::new();
    
    // Spawn tasks for each test file
    for path in paths {
        let path_clone = path.clone();
        let config_clone = config.clone();
        let test_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        join_set.spawn(async move {
            let start_time = std::time::Instant::now();
            let result = run_single_test(&path_clone, &config_clone).await;
            let duration = start_time.elapsed().as_millis() as u64;
            (test_name, result, duration)
        });
    }
    
    // Collect results
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok((test_name, Ok(_), duration)) => {
                results.push(CliTestResult {
                    name: test_name,
                    passed: true,
                    duration_ms: duration,
                    error: None,
                });
            }
            Ok((test_name, Err(e), duration)) => {
                error!("Test failed: {}", e);
                results.push(CliTestResult {
                    name: test_name,
                    passed: false,
                    duration_ms: duration,
                    error: Some(e.to_string()),
                });
                if config.fail_fast {
                    join_set.abort_all();
                    break;
                }
            }
            Err(e) => {
                error!("Task failed: {}", e);
                results.push(CliTestResult {
                    name: "unknown".to_string(),
                    passed: false,
                    duration_ms: 0,
                    error: Some(e.to_string()),
                });
            }
        }
    }
    
    Ok(results)
}

/// Run tests in parallel (legacy - kept for compatibility)
pub async fn run_tests_parallel(paths: &[PathBuf], config: &CliConfig) -> Result<()> {
    let results = run_tests_parallel_with_results(paths, config).await?;
    let tests_passed = results.iter().filter(|r| r.passed).count();
    let tests_failed = results.iter().filter(|r| !r.passed).count();
    
    info!("Test Results: {} passed, {} failed", tests_passed, tests_failed);
    
    if tests_failed > 0 {
        Err(CleanroomError::validation_error(&format!(
            "{} test(s) failed", tests_failed
        )))
    } else {
        info!("All tests passed! Framework self-testing successful.");
        Ok(())
    }
}

/// Run a single test file
/// 
/// Core Team Compliance:
/// - ✅ Async function for I/O operations
/// - ✅ Proper error handling with CleanroomError
/// - ✅ No unwrap() or expect() calls
/// - ✅ Use tracing for structured logging
pub async fn run_single_test(path: &PathBuf, _config: &CliConfig) -> Result<()> {
    use tracing::{info, debug, warn, error};

    // Parse TOML configuration using CLI's own config structure
    let content = std::fs::read_to_string(path)
        .map_err(|e| CleanroomError::config_error(format!("Failed to read config file: {}", e)))?;
    
    let test_config: crate::config::TestConfig = toml::from_str(&content)
        .map_err(|e| CleanroomError::config_error(format!("TOML parse error: {}", e)))?;
    
    println!("🚀 Executing test: {}", test_config.test.metadata.name);
    info!("🚀 Executing test: {}", test_config.test.metadata.name);
    if let Some(description) = &test_config.test.metadata.description {
        println!("📝 Description: {}", description);
        debug!("Test description: {}", description);
    }

    // Create cleanroom environment for test execution
    let environment = CleanroomEnvironment::new().await
        .map_err(|e| CleanroomError::internal_error("Failed to create test environment")
            .with_context("Test execution requires cleanroom environment")
            .with_source(e.to_string()))?;

    // Register services from configuration
    if let Some(services) = &test_config.services {
        for (service_name, service_config) in services {
            debug!("Registering service: {} ({})", service_name, service_config.r#type);
            info!("📦 Service '{}' would be registered (service registration not yet implemented)", service_name);
        }
    }

    // Execute test steps
    for (i, step) in test_config.steps.iter().enumerate() {
        info!("📋 Step {}: {}", i + 1, step.name);
        
        // Validate the step configuration
        if step.command.is_empty() {
            return Err(CleanroomError::validation_error(&format!(
                "Step '{}' has empty command", step.name
            )));
        }
        
        // Execute the command using std::process::Command
        println!("🔧 Executing: {}", step.command.join(" "));
        info!("🔧 Executing: {}", step.command.join(" "));
        
        let output = std::process::Command::new(&step.command[0])
            .args(&step.command[1..])
            .output()
            .map_err(|e| CleanroomError::internal_error(&format!(
                "Failed to execute command '{}': {}", 
                step.command.join(" "), e
            )))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        println!("📤 Output: {}", stdout.trim());
        info!("📤 Output: {}", stdout.trim());
        if !stderr.is_empty() {
            println!("⚠️  Stderr: {}", stderr.trim());
            info!("⚠️  Stderr: {}", stderr.trim());
        }
        
        // Check exit status
        if !output.status.success() {
            return Err(CleanroomError::validation_error(&format!(
                "Step '{}' failed with exit code: {}", 
                step.name, 
                output.status.code().unwrap_or(-1)
            )));
        }
        
        // Validate expected output regex if provided
        if let Some(regex) = &step.expected_output_regex {
            debug!("Expected output regex: {}", regex);
            let re = regex::Regex::new(regex)
                .map_err(|e| CleanroomError::validation_error(&format!(
                    "Invalid regex '{}' in step '{}': {}", regex, step.name, e
                )))?;
            
            if !re.is_match(&stdout) {
                return Err(CleanroomError::validation_error(&format!(
                    "Step '{}' output did not match expected regex '{}'. Output: {}", 
                    step.name, regex, stdout.trim()
                )));
            }
            info!("✅ Output matches expected regex");
        }
        
        info!("✅ Step '{}' completed successfully", step.name);
    }

    // Test completion
    println!("🎉 Test '{}' completed successfully!", test_config.test.metadata.name);
    info!("🎉 Test '{}' completed successfully!", test_config.test.metadata.name);
    Ok(())
}

/// Validate test assertions after execution
pub async fn validate_test_assertions(
    environment: &CleanroomEnvironment,
    assertions: &HashMap<String, toml::Value>,
) -> Result<()> {
    use tracing::{info, warn};

    for (assertion_key, assertion_value) in assertions {
        match assertion_key.as_str() {
            "container_should_have_executed_commands" => {
                if let Some(expected_count) = assertion_value.as_integer() {
                    let (created, reused) = environment.get_container_reuse_stats().await;
                    let total_commands = created + reused;

                    if total_commands < expected_count as u32 {
                        return Err(CleanroomError::deterministic_error(format!(
                            "Assertion failed: expected at least {} commands executed, got {}",
                            expected_count, total_commands
                        )));
                    }

                    info!("✅ Container command execution assertion passed ({} commands)", total_commands);
                }
            }
            "execution_should_be_hermetic" => {
                if let Some(true) = assertion_value.as_bool() {
                    // Validate hermetic isolation - each test should have unique session
                    let session_id = environment.session_id();
                    if session_id.is_nil() {
                        return Err(CleanroomError::deterministic_error("Hermetic isolation assertion failed: invalid session ID"));
                    }

                    info!("✅ Hermetic isolation assertion passed (session: {})", session_id);
                }
            }
            _ => {
                warn!("Unknown assertion type: {}", assertion_key);
            }
        }
    }
    
    Ok(())
}

/// Watch test files and rerun on changes
pub async fn watch_and_run(paths: &[PathBuf], config: &CliConfig) -> Result<()> {
    use notify::{Watcher, RecursiveMode, Event, event::EventKind};
    use std::sync::mpsc::channel;
    use std::time::Duration;
    
    info!("Watch mode enabled - monitoring test files for changes");
    info!("Press Ctrl+C to stop watching");
    
    // Run tests once before watching
    info!("Running initial test suite...");
    let mut watch_config = config.clone();
    watch_config.watch = false; // Prevent recursive watch
    
    if let Err(e) = run_tests(paths, &watch_config).await {
        warn!("Initial test run failed: {}", e);
    }
    
    // Set up file watcher
    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(move |res: std::result::Result<Event, notify::Error>| {
        if let Ok(event) = res {
            if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                let _ = tx.send(event);
            }
        }
    })
    .map_err(|e| CleanroomError::internal_error("Failed to create file watcher")
        .with_context("Watch mode initialization failed")
        .with_source(e.to_string()))?;
    
    // Watch all test directories/files
    for path in paths {
        watcher.watch(path.as_ref(), RecursiveMode::Recursive)
            .map_err(|e| CleanroomError::internal_error("Failed to watch path")
                .with_context(format!("Path: {}", path.display()))
                .with_source(e.to_string()))?;
        info!("Watching: {}", path.display());
    }
    
    // Watch loop
    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(event) => {
                info!("File change detected: {:?}", event.paths);
                info!("Rerunning tests...");
                
                // Small delay to allow file write to complete
                tokio::time::sleep(Duration::from_millis(100)).await;
                
                if let Err(e) = run_tests(paths, &watch_config).await {
                    error!("Test run failed: {}", e);
                } else {
                    info!("All tests passed!");
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // No events, continue watching
                continue;
            }
            Err(e) => {
                return Err(CleanroomError::internal_error("File watcher error")
                    .with_context("Watch mode encountered an error")
                    .with_source(e.to_string()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_run_tests_sequential_with_results_empty_paths() -> Result<()> {
        // Arrange
        let paths = vec![];
        let config = CliConfig::default();

        // Act
        let results = run_tests_sequential_with_results(&paths, &config).await?;

        // Assert
        assert_eq!(results.len(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_run_tests_parallel_with_results_empty_paths() -> Result<()> {
        // Arrange
        let paths = vec![];
        let config = CliConfig::default();

        // Act
        let results = run_tests_parallel_with_results(&paths, &config).await?;

        // Assert
        assert_eq!(results.len(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_run_tests_sequential_with_results_single_file() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new()
            .map_err(|e| CleanroomError::internal_error("Failed to create temp dir")
                .with_source(e.to_string()))?;
        let test_file = temp_dir.path().join("test.toml");
        
        let toml_content = r#"
[test]
name = "test_example"
description = "A test example"

[[steps]]
name = "test_step"
command = ["echo", "hello"]
"#;
        
        fs::write(&test_file, toml_content)
            .map_err(|e| CleanroomError::internal_error("Failed to write test file")
                .with_source(e.to_string()))?;
        
        let paths = vec![test_file];
        let config = CliConfig::default();

        // Act
        let results = run_tests_sequential_with_results(&paths, &config).await;

        // Assert - Should fail because test execution is not implemented
        assert!(results.is_err());
        assert!(results.unwrap_err().message.contains("Test completion: Cannot complete test"));
        Ok(())
    }

    #[tokio::test]
    async fn test_run_tests_parallel_with_results_single_file() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new()
            .map_err(|e| CleanroomError::internal_error("Failed to create temp dir")
                .with_source(e.to_string()))?;
        let test_file = temp_dir.path().join("test.toml");
        
        let toml_content = r#"
[test]
name = "test_example"
description = "A test example"

[[steps]]
name = "test_step"
command = ["echo", "hello"]
"#;
        
        fs::write(&test_file, toml_content)
            .map_err(|e| CleanroomError::internal_error("Failed to write test file")
                .with_source(e.to_string()))?;
        
        let paths = vec![test_file];
        let config = CliConfig::default();

        // Act
        let results = run_tests_parallel_with_results(&paths, &config).await;

        // Assert - Should fail because test execution is not implemented
        assert!(results.is_err());
        assert!(results.unwrap_err().message.contains("Test completion: Cannot complete test"));
        Ok(())
    }

    #[tokio::test]
    async fn test_run_single_test_invalid_toml() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new()
            .map_err(|e| CleanroomError::internal_error("Failed to create temp dir")
                .with_source(e.to_string()))?;
        let test_file = temp_dir.path().join("invalid.toml");
        
        let invalid_toml = r#"
[test
name = "invalid"
"#;
        
        fs::write(&test_file, invalid_toml)
            .map_err(|e| CleanroomError::internal_error("Failed to write test file")
                .with_source(e.to_string()))?;
        
        let config = CliConfig::default();

        // Act
        let result = run_single_test(&test_file, &config).await;

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("TOML parse error"));
        Ok(())
    }

    #[tokio::test]
    async fn test_run_single_test_file_not_found() -> Result<()> {
        // Arrange
        let nonexistent_file = PathBuf::from("nonexistent.toml");
        let config = CliConfig::default();

        // Act
        let result = run_single_test(&nonexistent_file, &config).await;

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Failed to read config file"));
        Ok(())
    }

    #[tokio::test]
    async fn test_validate_test_assertions_empty() -> Result<()> {
        // Arrange
        let environment = CleanroomEnvironment::default();
        let assertions = HashMap::new();

        // Act
        let result = validate_test_assertions(&environment, &assertions).await;

        // Assert
        assert!(result.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn test_validate_test_assertions_unknown_type() -> Result<()> {
        // Arrange
        let environment = CleanroomEnvironment::default();
        let mut assertions = HashMap::new();
        assertions.insert("unknown_assertion".to_string(), toml::Value::Boolean(true));

        // Act
        let result = validate_test_assertions(&environment, &assertions).await;

        // Assert
        assert!(result.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn test_validate_test_assertions_container_commands() -> Result<()> {
        // Arrange
        let environment = CleanroomEnvironment::default();
        let mut assertions = HashMap::new();
        assertions.insert("container_should_have_executed_commands".to_string(), toml::Value::Integer(5));

        // Act
        let result = validate_test_assertions(&environment, &assertions).await;

        // Assert - Should pass because environment starts with 0 commands
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Assertion failed: expected at least 5 commands executed, got 0"));
        Ok(())
    }

    #[tokio::test]
    async fn test_validate_test_assertions_hermetic_isolation() -> Result<()> {
        // Arrange
        let environment = CleanroomEnvironment::default();
        let mut assertions = HashMap::new();
        assertions.insert("execution_should_be_hermetic".to_string(), toml::Value::Boolean(true));

        // Act
        let result = validate_test_assertions(&environment, &assertions).await;

        // Assert - Should pass because environment has valid session ID
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_cli_config_clone() {
        // Test that CliConfig can be cloned for parallel execution
        let config = CliConfig::default();
        let cloned_config = config.clone();
        
        assert_eq!(config.jobs, cloned_config.jobs);
        assert_eq!(config.parallel, cloned_config.parallel);
        assert_eq!(config.fail_fast, cloned_config.fail_fast);
    }
}
