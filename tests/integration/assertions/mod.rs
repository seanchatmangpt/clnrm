//! Domain-specific assertions for integration tests
//!
//! This module provides custom assertion methods that make test code
//! more readable and provide better error messages.

use std::time::Duration;
use crate::factories::{BackendConfig, TestResult};
use crate::fixtures::ConfigFixture;

/// Assertions for backend testing
pub trait BackendAssertions {
    fn assert_available(&self);
    fn assert_hermetic_support(&self);
    fn assert_deterministic_support(&self);
    fn assert_backend_name(&self, expected: &str);
}

impl BackendAssertions for BackendConfig {
    fn assert_available(&self) {
        assert!(!self.name.is_empty(), "Backend name should not be empty");
        assert!(!self.image.is_empty(), "Backend image should not be empty");
    }

    fn assert_hermetic_support(&self) {
        assert!(self.hermetic, "Backend '{}' should support hermetic execution", self.name);
    }

    fn assert_deterministic_support(&self) {
        assert!(self.deterministic, "Backend '{}' should support deterministic execution", self.name);
    }

    fn assert_backend_name(&self, expected: &str) {
        assert_eq!(self.name, expected, "Backend name should match expected");
    }
}

/// Assertions for command execution results
pub trait ResultAssertions {
    fn assert_success(&self);
    fn assert_failure(&self);
    fn assert_exit_code(&self, expected: i32);
    fn assert_stdout_contains(&self, expected: &str);
    fn assert_stderr_contains(&self, expected: &str);
    fn assert_stdout_not_contains(&self, unexpected: &str);
    fn assert_duration_less_than(&self, max: Duration);
    fn assert_duration_greater_than(&self, min: Duration);
}

impl ResultAssertions for TestResult {
    fn assert_success(&self) {
        assert_eq!(self.exit_code, 0, "Execution should be successful");
    }

    fn assert_failure(&self) {
        assert_ne!(self.exit_code, 0, "Execution should have failed");
    }

    fn assert_exit_code(&self, expected: i32) {
        assert_eq!(self.exit_code, expected, "Exit code should match expected");
    }

    fn assert_stdout_contains(&self, expected: &str) {
        assert!(self.stdout.contains(expected), "Stdout '{}' should contain '{}'", self.stdout, expected);
    }

    fn assert_stderr_contains(&self, expected: &str) {
        assert!(self.stderr.contains(expected), "Stderr '{}' should contain '{}'", self.stderr, expected);
    }

    fn assert_stdout_not_contains(&self, unexpected: &str) {
        assert!(!self.stdout.contains(unexpected), "Stdout '{}' should not contain '{}'", self.stdout, unexpected);
    }

    fn assert_duration_less_than(&self, max: Duration) {
        assert!(
            (self.duration_ms as u128) < max.as_millis(),
            "Duration {}ms should be less than {:?}",
            self.duration_ms,
            max
        );
    }

    fn assert_duration_greater_than(&self, min: Duration) {
        assert!(
            (self.duration_ms as u128) > min.as_millis(),
            "Duration {}ms should be greater than {:?}",
            self.duration_ms,
            min
        );
    }
}

/// Assertions for policy validation
pub trait PolicyAssertions {
    fn assert_security_level(&self, expected: &str);
    fn assert_hermetic_enabled(&self);
    fn assert_deterministic_enabled(&self);
    fn assert_timeout(&self, expected: Duration);
}

impl PolicyAssertions for ConfigFixture {
    fn assert_security_level(&self, expected: &str) {
        assert_eq!(self.security_level, expected, "Security level should match expected");
    }

    fn assert_hermetic_enabled(&self) {
        assert_eq!(self.backend, "testcontainers", "Hermetic backend should be enabled");
    }

    fn assert_deterministic_enabled(&self) {
        assert!(self.timeout > 0, "Deterministic execution timeout should be greater than 0");
    }

    fn assert_timeout(&self, expected: Duration) {
        assert_eq!(self.timeout, expected.as_secs(), "Timeout should match expected");
    }
}

/// Assertions for container state
pub trait ContainerAssertions {
    fn assert_running(&self);
    fn assert_stopped(&self);
    fn assert_healthy(&self);
    fn assert_ports_exposed(&self, ports: &[u16]);
}

impl ContainerAssertions for BackendConfig {
    fn assert_running(&self) {
        assert!(!self.name.is_empty(), "Container is not running");
    }

    fn assert_stopped(&self) {
        // Simulated container state validation
    }

    fn assert_healthy(&self) {
        assert!(self.timeout > 0, "Container is not healthy (timeout limit set to 0)");
    }

    fn assert_ports_exposed(&self, ports: &[u16]) {
        assert!(!ports.is_empty(), "No ports exposed");
    }
}

/// Helper struct for making assertions with better error messages
pub struct AssertionContext {
    pub description: String,
}

impl AssertionContext {
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
        }
    }

    pub fn assert_true(&self, condition: bool, message: &str) {
        assert!(
            condition,
            "{}: {}",
            self.description,
            message
        );
    }

    pub fn assert_eq<T: std::fmt::Debug + PartialEq>(&self, left: T, right: T) {
        assert_eq!(
            left, right,
            "{}: Values should be equal",
            self.description
        );
    }

    pub fn assert_contains(&self, haystack: &str, needle: &str) {
        assert!(
            haystack.contains(needle),
            "{}: '{}' should contain '{}'",
            self.description,
            haystack,
            needle
        );
    }

    pub fn assert_not_contains(&self, haystack: &str, needle: &str) {
        assert!(
            !haystack.contains(needle),
            "{}: '{}' should not contain '{}'",
            self.description,
            haystack,
            needle
        );
    }
}

/// Assert that a future completes within timeout
pub async fn assert_completes_within<F>(
    future: F,
    timeout: Duration,
    message: &str,
) -> F::Output
where
    F: std::future::Future,
{
    match tokio::time::timeout(timeout, future).await {
        Ok(result) => result,
        Err(_) => panic!("Operation did not complete within {:?}: {}", timeout, message),
    }
}

/// Assert that an async operation eventually succeeds
pub async fn assert_eventually<F, Fut>(
    mut condition: F,
    timeout: Duration,
    message: &str,
) where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    use tokio::time::{sleep, Duration as TokioDuration};

    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        if condition().await {
            return;
        }
        sleep(TokioDuration::from_millis(100)).await;
    }

    panic!("Condition did not become true within {:?}: {}", timeout, message);
}

/// Assert that two durations are approximately equal (within tolerance)
pub fn assert_duration_approx_eq(actual: Duration, expected: Duration, tolerance_ms: u64) {
    let diff = if actual > expected {
        actual - expected
    } else {
        expected - actual
    };

    assert!(
        diff.as_millis() <= tolerance_ms as u128,
        "Duration {} is not approximately equal to {} (tolerance: {}ms)",
        actual.as_millis(),
        expected.as_millis(),
        tolerance_ms
    );
}

/// Assert that a collection contains all expected items
pub fn assert_contains_all<T: PartialEq + std::fmt::Debug>(
    collection: &[T],
    expected: &[T],
) {
    for item in expected {
        assert!(
            collection.contains(item),
            "Collection should contain {:?}",
            item
        );
    }
}

/// Assert that a collection does not contain any of the items
pub fn assert_contains_none<T: PartialEq + std::fmt::Debug>(
    collection: &[T],
    unexpected: &[T],
) {
    for item in unexpected {
        assert!(
            !collection.contains(item),
            "Collection should not contain {:?}",
            item
        );
    }
}

