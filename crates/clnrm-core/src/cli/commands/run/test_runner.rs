//! Test Runner
//!
//! Executes tests using Config and proper `docker exec` semantics.
//! This runner uses `DockerContainerManager` and `TestRunner` from the executor module.
//!
//! # Key Features
//!
//! - Parses clnrm config format
//! - Uses `docker exec` for step execution (not new containers!)
//! - Container lifecycle: start → exec steps → stop
//! - Proper environment variable propagation
//!
//! # Config Format
//!
//! ```toml
//! [test]
//! name = "my_test"
//! timeout = "60s"
//!
//! [containers.alpine]
//! image = "alpine:latest"
//! env = { MY_VAR = "hello" }
//!
//! [[steps]]
//! name = "verify_env"
//! container = "alpine"
//! exec = ["sh", "-c", "echo $MY_VAR"]
//! assert.stdout_contains = "hello"
//! ```

use crate::config::spec::Config;
use crate::error::{CleanroomError, Result};
use crate::executor::{DockerContainerManager, ExecutionResult, TestRunner};
use std::path::Path;
use tracing::{debug, error, info};

/// Run a test using the clnrm config format and executor
///
/// Returns `Ok(ExecutionResult)` with all step results
pub async fn run_test(path: &Path) -> Result<ExecutionResult> {
    // Read config file
    let content = std::fs::read_to_string(path).map_err(|e| {
        CleanroomError::config_error(format!(
            "Failed to read config file '{}': {}",
            path.display(),
            e
        ))
    })?;

    // Parse config
    let config: Config = toml::from_str(&content).map_err(|e| {
        CleanroomError::config_error(format!("TOML parse error in '{}': {}", path.display(), e))
    })?;

    // Validate at parse time (fail fast) - includes reference validation
    config.validate()?;

    info!("🚀 Executing test: {}", config.test.name);
    debug!(
        "Containers: {:?}, Steps: {}",
        config.containers.keys().collect::<Vec<_>>(),
        config.steps.len()
    );

    // Create container manager and runner
    let manager = DockerContainerManager::new();
    let mut runner = TestRunner::new(manager);

    // Execute test
    let result = runner.run(&config).await?;

    // Log summary
    if result.passed {
        info!("✅ {}", result.summary);
    } else {
        error!("❌ {}", result.summary);
        for step_result in &result.step_results {
            if !step_result.passed {
                if let Some(reason) = &step_result.failure_reason {
                    error!("  Step '{}' failed: {}", step_result.name, reason);
                }
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_parse_valid() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "[test]").unwrap();
        writeln!(file, r#"name = "test""#).unwrap();
        writeln!(file, "[containers.alpine]").unwrap();
        writeln!(file, r#"image = "alpine:latest""#).unwrap();
        writeln!(file, "[[steps]]").unwrap();
        writeln!(file, r#"name = "step1""#).unwrap();
        writeln!(file, r#"container = "alpine""#).unwrap();
        writeln!(file, r#"exec = ["echo", "hello"]"#).unwrap();

        // Just verify it parses without error
        let content = std::fs::read_to_string(file.path()).unwrap();
        let config: Config = toml::from_str(&content).unwrap();
        assert_eq!(config.test.name, "test");
        assert!(config.containers.contains_key("alpine"));
        assert_eq!(config.steps.len(), 1);
    }
}
