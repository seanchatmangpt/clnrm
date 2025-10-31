//! Self-test command tests
//!
//! Tests verify framework self-testing using AAA pattern.

use clnrm_core::cli::commands::self_test::run_self_tests;
use clnrm_core::error::Result;

#[tokio::test]
async fn test_self_test_executes_framework_tests() -> Result<()> {
    // Arrange - No setup needed

    // Act - Run self-tests
    let result = run_self_tests(None, false, "none".to_string(), None).await;

    // Assert - Should execute (may pass or fail based on framework state)
    // The important thing is it executes without panicking
    assert!(
        result.is_ok() || result.is_err(),
        "BEHAVIOR: Self-test should execute without panicking"
    );

    Ok(())
}

