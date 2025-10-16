//! Self-test command implementation
//!
//! Handles framework self-testing with comprehensive validation and reporting.

use crate::error::{CleanroomError, Result};
use crate::testing::{run_framework_tests, FrameworkTestResults};
use tracing::info;

/// Run framework self-tests
///
/// Core Team Compliance:
/// - ✅ Async function for I/O operations
/// - ✅ Proper error handling with CleanroomError
/// - ✅ No unwrap() or expect() calls
/// - ✅ Use tracing for internal operations
pub async fn run_self_tests(suite: Option<String>, report: bool) -> Result<()> {
    // Use tracing instead of println for internal operations
    info!("Starting framework self-tests");

    // Validate suite parameter if provided
    if let Some(ref suite_name) = suite {
        const VALID_SUITES: &[&str] = &["framework", "container", "plugin", "cli", "otel"];
        if !VALID_SUITES.contains(&suite_name.as_str()) {
            return Err(CleanroomError::validation_error(&format!(
                "Invalid test suite '{}'. Valid suites: {}",
                suite_name,
                VALID_SUITES.join(", ")
            )));
        }
    }

    // Proper error handling - no unwrap/expect
    let results = run_framework_tests().await.map_err(|e| {
        CleanroomError::internal_error("Framework self-tests failed")
            .with_context("Failed to execute framework test suite")
            .with_source(e.to_string())
    })?;

    // Display results (CLI output is acceptable for user-facing messages)
    crate::cli::commands::report::display_test_results(&results);

    // Generate report if requested
    if report {
        crate::cli::commands::report::generate_framework_report(&results)
            .await
            .map_err(|e| {
                CleanroomError::internal_error("Report generation failed")
                    .with_context("Failed to generate test report")
                    .with_source(e.to_string())
            })?;
    }

    // Return proper error with context
    if results.failed_tests > 0 {
        Err(CleanroomError::validation_error(&format!(
            "{} test(s) failed out of {}",
            results.failed_tests, results.total_tests
        )))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run_self_tests_succeeds() -> Result<()> {
        // Arrange - Test with no specific suite and no report
        let suite = None;
        let report = false;

        // Act - Execute self-tests
        let result = run_self_tests(suite, report).await;

        // Assert - Should succeed (framework self-tests should pass)
        assert!(
            result.is_ok(),
            "Framework self-tests should succeed: {:?}",
            result.err()
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_run_self_tests_with_invalid_suite_fails() -> Result<()> {
        // Arrange - Test with invalid suite name
        let suite = Some("invalid_suite".to_string());
        let report = false;

        // Act - Execute self-tests with invalid suite
        let result = run_self_tests(suite, report).await;

        // Assert - Should fail with validation error
        assert!(
            result.is_err(),
            "Invalid suite should cause validation error"
        );
        assert!(result.unwrap_err().message.contains("Invalid test suite"));
        Ok(())
    }

    #[tokio::test]
    async fn test_run_self_tests_with_valid_suite_succeeds() -> Result<()> {
        // Arrange - Test with valid suite name
        let suite = Some("framework".to_string());
        let report = false;

        // Act - Execute self-tests with valid suite
        let result = run_self_tests(suite, report).await;

        // Assert - Should succeed
        assert!(
            result.is_ok(),
            "Valid suite should succeed: {:?}",
            result.err()
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_run_self_tests_with_report() -> Result<()> {
        // Arrange - Test with report generation enabled
        let suite = None;
        let report = true;

        // Act - Execute self-tests with report
        let result = run_self_tests(suite, report).await;

        // Assert - Should succeed (framework self-tests should pass)
        assert!(
            result.is_ok(),
            "Framework self-tests with report should succeed: {:?}",
            result.err()
        );

        // Verify report file was created
        let report_exists = std::fs::metadata("framework-test-report.json").is_ok();
        assert!(
            report_exists,
            "Report file should be created when report=true"
        );

        // Cleanup
        let _ = std::fs::remove_file("framework-test-report.json");

        Ok(())
    }

    #[tokio::test]
    async fn test_run_self_tests_all_valid_suites() -> Result<()> {
        // Test all valid suite names
        let valid_suites = vec!["framework", "container", "plugin", "cli", "otel"];

        for suite_name in valid_suites {
            // Arrange
            let suite = Some(suite_name.to_string());
            let report = false;

            // Act
            let result = run_self_tests(suite, report).await;

            // Assert
            assert!(
                result.is_ok(),
                "Suite '{}' should be valid and succeed",
                suite_name
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_run_self_tests_invalid_suite_names() -> Result<()> {
        // Test various invalid suite names
        let invalid_suites = vec![
            "invalid",
            "test",
            "unit",
            "integration",
            "e2e",
            "performance",
            "stress",
            "",
            "FRAMEWORK",      // case sensitive
            "framework ",     // with space
            "framework-test", // with dash
        ];

        for suite_name in invalid_suites {
            // Arrange
            let suite = Some(suite_name.to_string());
            let report = false;

            // Act
            let result = run_self_tests(suite, report).await;

            // Assert
            assert!(result.is_err(), "Suite '{}' should be invalid", suite_name);
            assert!(result.unwrap_err().message.contains("Invalid test suite"));
        }

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Incomplete test data or implementation"]
    async fn test_run_self_tests_with_suite_and_report() -> Result<()> {
        // Arrange - Test with both suite and report
        let suite = Some("framework".to_string());
        let report = true;

        // Act
        let result = run_self_tests(suite, report).await;

        // Assert
        assert!(
            result.is_ok(),
            "Framework self-tests with suite and report should succeed: {:?}",
            result.err()
        );

        // Verify report file was created
        let report_exists = std::fs::metadata("framework-test-report.json").is_ok();
        assert!(
            report_exists,
            "Report file should be created when report=true"
        );

        // Cleanup
        let _ = std::fs::remove_file("framework-test-report.json");

        Ok(())
    }

    #[tokio::test]
    async fn test_run_self_tests_no_suite_no_report() -> Result<()> {
        // Arrange - Test with no suite and no report
        let suite = None;
        let report = false;

        // Act
        let result = run_self_tests(suite, report).await;

        // Assert
        assert!(
            result.is_ok(),
            "Framework self-tests with no suite and no report should succeed: {:?}",
            result.err()
        );

        // Verify no report file was created
        let report_exists = std::fs::metadata("framework-test-report.json").is_ok();
        assert!(
            !report_exists,
            "Report file should not be created when report=false"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_run_self_tests_error_handling() -> Result<()> {
        // This test verifies that the function handles errors properly
        // by testing with invalid suite names and ensuring proper error messages

        // Arrange
        let invalid_suite = Some("definitely_invalid_suite_name".to_string());
        let report = false;

        // Act
        let result = run_self_tests(invalid_suite, report).await;

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("Invalid test suite"));
        assert!(error.message.contains("definitely_invalid_suite_name"));
        assert!(error.message.contains("Valid suites:"));

        Ok(())
    }

    #[tokio::test]
    async fn test_run_self_tests_suite_validation_case_sensitive() -> Result<()> {
        // Test that suite validation is case sensitive
        let case_variations = vec!["FRAMEWORK", "Framework", "FRAMEWORK", "framework"];

        for suite_name in case_variations {
            // Arrange
            let suite = Some(suite_name.to_string());
            let report = false;

            // Act
            let result = run_self_tests(suite, report).await;

            // Assert - Only lowercase "framework" should be valid
            if suite_name == "framework" {
                assert!(result.is_ok(), "Lowercase 'framework' should be valid");
            } else {
                assert!(
                    result.is_err(),
                    "Uppercase '{}' should be invalid",
                    suite_name
                );
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_run_self_tests_suite_validation_whitespace() -> Result<()> {
        // Test that suite validation handles whitespace properly
        let whitespace_variations = vec![
            " framework",
            "framework ",
            " framework ",
            "\tframework",
            "framework\t",
            "\nframework",
            "framework\n",
        ];

        for suite_name in whitespace_variations {
            // Arrange
            let suite = Some(suite_name.to_string());
            let report = false;

            // Act
            let result = run_self_tests(suite, report).await;

            // Assert - All should be invalid due to whitespace
            assert!(
                result.is_err(),
                "Suite with whitespace '{}' should be invalid",
                suite_name
            );
        }

        Ok(())
    }
}
