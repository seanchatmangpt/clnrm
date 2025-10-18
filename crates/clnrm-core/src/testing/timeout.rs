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

