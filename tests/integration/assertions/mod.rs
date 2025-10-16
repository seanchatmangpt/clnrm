//! Domain-specific assertions for integration tests
//!
//! This module provides custom assertion methods that make test code
//! more readable and provide better error messages.

use std::time::Duration;

/// Assertions for backend testing
pub trait BackendAssertions {
    fn assert_available(&self);
    fn assert_hermetic_support(&self);
    fn assert_deterministic_support(&self);
    fn assert_backend_name(&self, expected: &str);
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

/// Assertions for policy validation
pub trait PolicyAssertions {
    fn assert_security_level(&self, expected: &str);
    fn assert_hermetic_enabled(&self);
    fn assert_deterministic_enabled(&self);
    fn assert_timeout(&self, expected: Duration);
}

/// Assertions for container state
pub trait ContainerAssertions {
    fn assert_running(&self);
    fn assert_stopped(&self);
    fn assert_healthy(&self);
    fn assert_ports_exposed(&self, ports: &[u16]);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assertion_context() {
        let ctx = AssertionContext::new("Test context");
        ctx.assert_true(true, "Should be true");
        ctx.assert_eq(1, 1);
        ctx.assert_contains("hello world", "world");
        ctx.assert_not_contains("hello world", "foo");
    }

    #[test]
    fn test_duration_approx_eq() {
        let d1 = Duration::from_millis(100);
        let d2 = Duration::from_millis(105);
        assert_duration_approx_eq(d1, d2, 10);
    }

    #[test]
    #[should_panic]
    fn test_duration_approx_eq_fails() {
        let d1 = Duration::from_millis(100);
        let d2 = Duration::from_millis(200);
        assert_duration_approx_eq(d1, d2, 10);
    }

    #[test]
    fn test_contains_all() {
        let collection = vec![1, 2, 3, 4, 5];
        let expected = vec![2, 4];
        assert_contains_all(&collection, &expected);
    }

    #[test]
    fn test_contains_none() {
        let collection = vec![1, 2, 3];
        let unexpected = vec![4, 5, 6];
        assert_contains_none(&collection, &unexpected);
    }

    #[tokio::test]
    async fn test_assert_completes_within() {
        let future = async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            42
        };

        let result = assert_completes_within(
            future,
            Duration::from_secs(1),
            "Should complete quickly"
        ).await;

        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_assert_eventually() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = Arc::clone(&flag);

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            flag_clone.store(true, Ordering::SeqCst);
        });

        assert_eventually(
            || {
                let f = Arc::clone(&flag);
                async move { f.load(Ordering::SeqCst) }
            },
            Duration::from_secs(1),
            "Flag should become true"
        ).await;
    }
}
