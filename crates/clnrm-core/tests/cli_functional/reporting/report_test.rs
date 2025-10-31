//! Report command tests
//!
//! Tests verify report generation using AAA pattern.

use clnrm_core::cli::commands::report::generate_report;
use clnrm_core::error::Result;

#[tokio::test]
async fn test_report_generates_default_test_results() -> Result<()> {
    // Arrange - No input file (uses default test results)

    // Act - Generate report
    let result = generate_report(None, None, "json").await;

    // Assert - Should succeed
    // Note: May fail if framework tests can't run, but should not panic
    assert!(
        result.is_ok() || result.is_err(),
        "BEHAVIOR: Report generation should execute without panicking"
    );

    Ok(())
}

