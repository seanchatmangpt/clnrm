//! Test fixtures for integration tests
//!
//! Fixtures provide pre-defined test data that can be loaded and used
//! across multiple tests for consistency.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Test configuration fixture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFixture {
    pub name: String,
    pub backend: String,
    pub security_level: String,
    pub timeout: u64,
    pub env_vars: HashMap<String, String>,
}

impl ConfigFixture {
    pub fn default_alpine() -> Self {
        Self {
            name: "test-alpine".to_string(),
            backend: "testcontainers".to_string(),
            security_level: "medium".to_string(),
            timeout: 30,
            env_vars: HashMap::new(),
        }
    }

    pub fn default_ubuntu() -> Self {
        Self {
            name: "test-ubuntu".to_string(),
            backend: "testcontainers".to_string(),
            security_level: "medium".to_string(),
            timeout: 30,
            env_vars: HashMap::new(),
        }
    }

    pub fn high_security() -> Self {
        Self {
            name: "high-security-test".to_string(),
            backend: "testcontainers".to_string(),
            security_level: "high".to_string(),
            timeout: 60,
            env_vars: HashMap::new(),
        }
    }
}

/// Test command fixture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandFixture {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub expected_exit_code: i32,
    pub expected_stdout_contains: Option<String>,
    pub expected_stderr_contains: Option<String>,
}

impl CommandFixture {
    pub fn echo_hello() -> Self {
        Self {
            name: "echo-hello".to_string(),
            command: "echo".to_string(),
            args: vec!["Hello, World!".to_string()],
            expected_exit_code: 0,
            expected_stdout_contains: Some("Hello, World!".to_string()),
            expected_stderr_contains: None,
        }
    }

    pub fn list_files() -> Self {
        Self {
            name: "list-files".to_string(),
            command: "ls".to_string(),
            args: vec!["-la".to_string()],
            expected_exit_code: 0,
            expected_stdout_contains: None,
            expected_stderr_contains: None,
        }
    }

    pub fn failing_command() -> Self {
        Self {
            name: "failing-command".to_string(),
            command: "false".to_string(),
            args: vec![],
            expected_exit_code: 1,
            expected_stdout_contains: None,
            expected_stderr_contains: None,
        }
    }

    pub fn with_env_vars() -> Self {
        Self {
            name: "env-vars".to_string(),
            command: "env".to_string(),
            args: vec![],
            expected_exit_code: 0,
            expected_stdout_contains: Some("TEST_VAR".to_string()),
            expected_stderr_contains: None,
        }
    }
}

/// Test result fixture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultFixture {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub backend: String,
}

impl ResultFixture {
    pub fn successful_execution() -> Self {
        Self {
            exit_code: 0,
            stdout: "Success output".to_string(),
            stderr: String::new(),
            duration_ms: 100,
            backend: "testcontainers".to_string(),
        }
    }

    pub fn failed_execution() -> Self {
        Self {
            exit_code: 1,
            stdout: String::new(),
            stderr: "Error: Command failed".to_string(),
            duration_ms: 50,
            backend: "testcontainers".to_string(),
        }
    }
}

/// Load fixture from JSON file
pub fn load_fixture<T: for<'de> Deserialize<'de>>(name: &str) -> anyhow::Result<T> {
    let fixture_path = format!("tests/integration/fixtures/{}.json", name);
    let content = std::fs::read_to_string(&fixture_path)?;
    let fixture = serde_json::from_str(&content)?;
    Ok(fixture)
}

/// Save fixture to JSON file
pub fn save_fixture<T: Serialize>(name: &str, data: &T) -> anyhow::Result<()> {
    let fixture_path = format!("tests/integration/fixtures/{}.json", name);
    let content = serde_json::to_string_pretty(data)?;
    std::fs::write(&fixture_path, content)?;
    Ok(())
}
