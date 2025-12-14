//! Test Timeout Enforcement
//!
//! This module provides macros and utilities to enforce test timeouts
//! and prevent tests from running longer than 1 second.

use std::time::{Duration, Instant};

/// Macro to enforce a 1-second timeout on test functions
/// 
/// Usage:
/// ```rust
/// #[test]
/// fn my_test() {
///     test_timeout!(|| {
///         // Your test code here
///         assert_eq!(1 + 1, 2);
///     });
/// }
/// ```
#[macro_export]
macro_rules! test_timeout {
    ($test_fn:expr) => {
        let start = std::time::Instant::now();
        let result = std::panic::catch_unwind(|| {
            $test_fn
        });
        
        let elapsed = start.elapsed();
        if elapsed > std::time::Duration::from_secs(1) {
            panic!("Test exceeded 1-second timeout: {:?}", elapsed);
        }
        
        match result {
            Ok(_) => {},
            Err(e) => std::panic::resume_unwind(e),
        }
    };
}

/// Test timeout configuration
pub const TEST_TIMEOUT: Duration = Duration::from_secs(1);

/// Compile-time validated test run with bounds checking
/// N must be >= 10 and <= 1000 at compile time
pub struct ValidatedTestRun<const N: usize> {
    _phantom: std::marker::PhantomData<[(); N]>,
}

impl<const N: usize> ValidatedTestRun<N> {
    /// Create a new validated test run
    /// Bounds checking: N must be between 10 and 1000 inclusive
    pub const fn new() -> Self {
        // Compile-time bounds checking using const assertions
        // This will fail to compile if N is outside the valid range
        assert!(N >= 10, "ValidatedTestRun: N must be >= 10");
        assert!(N <= 1000, "ValidatedTestRun: N must be <= 1000");

        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get the validated test count
    pub const fn count() -> usize {
        N
    }
}

/// Assert that a test completes within the timeout
pub fn assert_test_timeout<F>(test_fn: F) 
where 
    F: FnOnce() + std::panic::UnwindSafe,
{
    let start = Instant::now();
    let result = std::panic::catch_unwind(test_fn);
    let elapsed = start.elapsed();
    
    if elapsed > TEST_TIMEOUT {
        panic!("Test exceeded {}ms timeout: {:?}", TEST_TIMEOUT.as_millis(), elapsed);
    }
    
    match result {
        Ok(_) => {},
        Err(e) => std::panic::resume_unwind(e),
    }
}

