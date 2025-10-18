//! Test Result Structures
//!
//! Core data structures for representing test execution results.
//! Used by all formatters to generate output.

use std::time::Duration;

/// Status of a test execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestStatus {
    /// Test passed successfully
    Passed,
    /// Test failed with error
    Failed,
    /// Test was skipped
    Skipped,
    /// Test status is unknown or pending
    Unknown,
}

/// Individual test result
#[derive(Debug, Clone)]
pub struct TestResult {
    /// Test name
    pub name: String,
    /// Test status
    pub status: TestStatus,
    /// Test duration
    pub duration: Option<Duration>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Standard output captured during test
    pub stdout: Option<String>,
    /// Standard error captured during test
    pub stderr: Option<String>,
    /// Test metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl TestResult {
    /// Create a new passed test result
    pub fn passed(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: TestStatus::Passed,
            duration: None,
            error: None,
            stdout: None,
            stderr: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Create a new failed test result
    pub fn failed(name: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: TestStatus::Failed,
            duration: None,
            error: Some(error.into()),
            stdout: None,
            stderr: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Create a new skipped test result
    pub fn skipped(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: TestStatus::Skipped,
            duration: None,
            error: None,
            stdout: None,
            stderr: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Set test duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Set stdout
    pub fn with_stdout(mut self, stdout: impl Into<String>) -> Self {
        self.stdout = Some(stdout.into());
        self
    }

    /// Set stderr
    pub fn with_stderr(mut self, stderr: impl Into<String>) -> Self {
        self.stderr = Some(stderr.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Check if test passed
    pub fn is_passed(&self) -> bool {
        self.status == TestStatus::Passed
    }

    /// Check if test failed
    pub fn is_failed(&self) -> bool {
        self.status == TestStatus::Failed
    }

    /// Check if test was skipped
    pub fn is_skipped(&self) -> bool {
        self.status == TestStatus::Skipped
    }
}

/// Test suite containing multiple test results
#[derive(Debug, Clone)]
pub struct TestSuite {
    /// Suite name
    pub name: String,
    /// Test results
    pub results: Vec<TestResult>,
    /// Suite duration
    pub duration: Option<Duration>,
    /// Suite metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl TestSuite {
    /// Create a new empty test suite
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            results: Vec::new(),
            duration: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Add a test result
    pub fn add_result(mut self, result: TestResult) -> Self {
        self.results.push(result);
        self
    }

    /// Set suite duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Get count of passed tests
    pub fn passed_count(&self) -> usize {
        self.results.iter().filter(|r| r.is_passed()).count()
    }

    /// Get count of failed tests
    pub fn failed_count(&self) -> usize {
        self.results.iter().filter(|r| r.is_failed()).count()
    }

    /// Get count of skipped tests
    pub fn skipped_count(&self) -> usize {
        self.results.iter().filter(|r| r.is_skipped()).count()
    }

    /// Get total test count
    pub fn total_count(&self) -> usize {
        self.results.len()
    }

    /// Check if all tests passed
    pub fn is_success(&self) -> bool {
        self.failed_count() == 0 && self.total_count() > 0
    }
}
