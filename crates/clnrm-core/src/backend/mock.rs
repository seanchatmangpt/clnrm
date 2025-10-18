//! Mock backend for high-performance testing
//!
//! Provides instant command execution for testing without Docker overhead.
//! Following core team best practices for fast, reliable test execution.

use crate::backend::{Backend, Cmd, RunResult};
use crate::error::Result;
use std::collections::HashMap;

/// High-performance mock backend for fast testing
/// Replaces slow Docker operations with instant responses
#[derive(Debug, Clone)]
pub struct MockBackend {
    /// Mock responses for different commands
    responses: HashMap<String, MockResponse>,
}

#[derive(Debug, Clone)]
pub struct MockResponse {
    stdout: String,
    stderr: String,
    exit_code: i32,
}

impl MockBackend {
    /// Create a new mock backend with default responses
    pub fn new() -> Self {
        let mut responses = HashMap::new();

        // Pre-configure common mock responses for fast testing
        responses.insert(
            "echo".to_string(),
            MockResponse {
                stdout: "mock echo output".to_string(),
                stderr: "".to_string(),
                exit_code: 0,
            },
        );

        responses.insert(
            "cat".to_string(),
            MockResponse {
                stdout: "mock file content".to_string(),
                stderr: "".to_string(),
                exit_code: 0,
            },
        );

        responses.insert(
            "test".to_string(),
            MockResponse {
                stdout: "".to_string(),
                stderr: "".to_string(),
                exit_code: 1, // File doesn't exist
            },
        );

        responses.insert(
            "uname".to_string(),
            MockResponse {
                stdout: "Linux\n".to_string(),
                stderr: "".to_string(),
                exit_code: 0,
            },
        );

        responses.insert(
            "whoami".to_string(),
            MockResponse {
                stdout: "root\n".to_string(),
                stderr: "".to_string(),
                exit_code: 0,
            },
        );

        responses.insert("env".to_string(), MockResponse {
            stdout: "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin\nHOSTNAME=mock-container\nHOME=/root\n".to_string(),
            stderr: "".to_string(),
            exit_code: 0,
        });

        responses.insert("ls".to_string(), MockResponse {
            stdout: "bin\nboot\ndev\netc\nhome\nlib\nlib64\nmedia\nmnt\nopt\nproc\nroot\nrun\nsbin\nsrv\nsys\ntmp\nusr\nvar\n".to_string(),
            stderr: "".to_string(),
            exit_code: 0,
        });

        Self { responses }
    }

    /// Add a custom mock response for a command
    pub fn add_response(mut self, command: &str, response: MockResponse) -> Self {
        self.responses.insert(command.to_string(), response);
        self
    }

    /// Ultra-fast command execution (microseconds instead of seconds)
    fn execute_mock_cmd(&self, cmd: &Cmd) -> Result<RunResult> {
        let cmd_key = cmd.bin.clone();

        // Instant response - no Docker overhead
        if let Some(response) = self.responses.get(&cmd_key) {
            Ok(RunResult {
                exit_code: response.exit_code,
                stdout: response.stdout.clone(),
                stderr: response.stderr.clone(),
                duration_ms: 1, // 1ms for realistic timing
                steps: Vec::new(),
                redacted_env: Vec::new(),
                backend: "mock".to_string(),
                concurrent: false,
                step_order: Vec::new(),
            })
        } else {
            // Default success for unknown commands - simulates container behavior
            Ok(RunResult {
                exit_code: 0,
                stdout: format!("mock output for: {}", cmd.bin),
                stderr: "".to_string(),
                duration_ms: 1,
                steps: Vec::new(),
                redacted_env: Vec::new(),
                backend: "mock".to_string(),
                concurrent: false,
                step_order: Vec::new(),
            })
        }
    }
}

impl Backend for MockBackend {
    /// Run a command in the mock backend
    fn run_cmd(&self, cmd: Cmd) -> Result<RunResult> {
        self.execute_mock_cmd(&cmd)
    }

    /// Get the name of the backend
    fn name(&self) -> &str {
        "mock"
    }

    /// Mock backend is always available for testing
    fn is_available(&self) -> bool {
        true
    }

    /// Mock backend supports hermetic execution (simulated)
    fn supports_hermetic(&self) -> bool {
        true
    }

    /// Mock backend supports deterministic execution (by design)
    fn supports_deterministic(&self) -> bool {
        true
    }
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new()
    }
}
