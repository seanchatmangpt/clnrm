//! Self-test command implementation with OTEL export support
//!
//! Handles framework self-testing with comprehensive validation, reporting, and OpenTelemetry export.

use crate::error::{CleanroomError, Result};
use tracing::{info, span, Level};

#[cfg(feature = "otel-traces")]
use crate::telemetry::{init_otel, Export, OtelConfig, OtelGuard};

/// Run framework self-tests with optional OTEL export
///
/// Core Team Compliance:
/// - ✅ Async function for I/O operations
/// - ✅ Proper error handling with CleanroomError
/// - ✅ No unwrap() or expect() calls
/// - ✅ Use tracing for internal operations
pub async fn run_self_tests(
    suite: Option<String>,
    report: bool,
    otel_exporter: String,
    _otel_endpoint: Option<String>,
) -> Result<()> {
    // Initialize OTEL if requested
    #[cfg(feature = "otel-traces")]
    let _guard = if otel_exporter != "none" {
        Some(init_otel_for_self_test(
            &otel_exporter,
            _otel_endpoint.as_deref(),
        )?)
    } else {
        None
    };

    #[cfg(not(feature = "otel-traces"))]
    if otel_exporter != "none" {
        return Err(CleanroomError::validation_error(
            "OTEL export requires the 'otel-traces' feature. Build with --features otel-traces",
        ));
    }

    // Use tracing instead of println for internal operations
    info!("Starting framework self-tests");

    #[cfg(feature = "otel-traces")]
    let root_span = if otel_exporter != "none" {
        span!(
            Level::INFO,
            "clnrm.self_test",
            clnrm.version = env!("CARGO_PKG_VERSION"),
            test.suite = suite.as_deref().unwrap_or("all"),
            otel.exporter = %otel_exporter,
        )
    } else {
        span!(Level::INFO, "clnrm.self_test")
    };

    #[cfg(not(feature = "otel-traces"))]
    let root_span = span!(Level::INFO, "clnrm.self_test");

    let _enter = root_span.enter();

    // Validate suite parameter if provided
    if let Some(ref suite_name) = suite {
        const VALID_SUITES: &[&str] = &["framework", "container", "plugin", "cli", "otel"];
        if !VALID_SUITES.contains(&suite_name.as_str()) {
            #[cfg(feature = "otel-traces")]
            {
                root_span.record("result", "error");
                root_span.record("error.type", "validation_error");
            }

            return Err(CleanroomError::validation_error(format!(
                "Invalid test suite '{}'. Valid suites: {}",
                suite_name,
                VALID_SUITES.join(", ")
            )));
        }
    }

    // Run basic self-tests
    info!("🧪 Running framework self-tests");

    // Run framework tests with optional suite filter
    use crate::testing::run_framework_tests_by_suite;
    let test_results = run_framework_tests_by_suite(suite.as_deref())
        .await
        .map_err(|e| {
            CleanroomError::internal_error("Framework self-tests failed")
                .with_context("Failed to execute framework test suite")
                .with_source(e.to_string())
        })?;

    // Display results (CLI output is acceptable for user-facing messages)
    crate::cli::commands::report::display_test_results(&test_results);

    // Generate report if requested
    if report {
        crate::cli::commands::report::generate_framework_report(&test_results)
            .await
            .map_err(|e| {
                CleanroomError::internal_error("Report generation failed")
                    .with_context("Failed to generate test report")
                    .with_source(e.to_string())
            })?;
    }

    #[cfg(feature = "otel-traces")]
    {
        if test_results.failed_tests > 0 {
            root_span.record("result", "fail");
            root_span.record("failed_tests", test_results.failed_tests);
        } else {
            root_span.record("result", "pass");
        }
        root_span.record("total_tests", test_results.total_tests);
    }

    // Return proper error with context
    if test_results.failed_tests > 0 {
        Err(CleanroomError::validation_error(format!(
            "{} test(s) failed out of {}",
            test_results.failed_tests, test_results.total_tests
        )))
    } else {
        info!("✅ All self-tests passed");
        Ok(())
    }
}

/// Initialize OTEL for self-test with proper error handling
#[cfg(feature = "otel-traces")]
fn init_otel_for_self_test(exporter: &str, endpoint: Option<&str>) -> Result<OtelGuard> {
    let export = match exporter {
        "stdout" => Export::Stdout,
        "otlp-http" => {
            let endpoint = endpoint.ok_or_else(|| {
                CleanroomError::validation_error("OTEL endpoint required for otlp-http exporter")
            })?;
            // Convert to static string by leaking (acceptable for test setup)
            let static_endpoint: &'static str = Box::leak(endpoint.to_string().into_boxed_str());
            Export::OtlpHttp {
                endpoint: static_endpoint,
            }
        }
        "otlp-grpc" => {
            let endpoint = endpoint.ok_or_else(|| {
                CleanroomError::validation_error("OTEL endpoint required for otlp-grpc exporter")
            })?;
            // Convert to static string by leaking (acceptable for test setup)
            let static_endpoint: &'static str = Box::leak(endpoint.to_string().into_boxed_str());
            Export::OtlpGrpc {
                endpoint: static_endpoint,
            }
        }
        _ => {
            return Err(CleanroomError::validation_error(format!(
                "Invalid OTEL exporter '{}'. Valid: none, stdout, otlp-http, otlp-grpc",
                exporter
            )))
        }
    };

    let config = OtelConfig {
        service_name: "clnrm-self-test",
        deployment_env: "test",
        sample_ratio: 1.0,
        export,
        enable_fmt_layer: false,
        headers: None,
    };

    init_otel(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_run_self_tests_succeeds() -> Result<()> {
        // Arrange - Test with no specific suite and no report
        let suite = None;
        let report = false;
        let otel_exporter = "none".to_string();
        let otel_endpoint = None;

        // Act - Execute self-tests
        let result = run_self_tests(suite, report, otel_exporter, otel_endpoint).await;

        // Assert - Should succeed (framework self-tests should pass)
        assert!(
            result.is_ok(),
            "Framework self-tests should succeed: {:?}",
            result.err()
        );
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_run_self_tests_with_invalid_suite_fails() -> Result<()> {
        // Arrange - Test with invalid suite name
        let suite = Some("invalid_suite".to_string());
        let report = false;
        let otel_exporter = "none".to_string();
        let otel_endpoint = None;

        // Act - Execute self-tests with invalid suite
        let result = run_self_tests(suite, report, otel_exporter, otel_endpoint).await;

        // Assert - Should fail with validation error
        assert!(
            result.is_err(),
            "Invalid suite should cause validation error"
        );
        assert!(result.unwrap_err().message.contains("Invalid test suite"));
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_run_self_tests_with_valid_suite_succeeds() -> Result<()> {
        // Arrange - Test with valid suite name
        let suite = Some("framework".to_string());
        let report = false;
        let otel_exporter = "none".to_string();
        let otel_endpoint = None;

        // Act - Execute self-tests with valid suite
        let result = run_self_tests(suite, report, otel_exporter, otel_endpoint).await;

        // Assert - Should succeed
        assert!(
            result.is_ok(),
            "Valid suite should succeed: {:?}",
            result.err()
        );
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    #[cfg(feature = "otel-stdout")]
    async fn test_run_self_tests_with_stdout_otel() -> Result<()> {
        // Arrange - Test with OTEL stdout export
        let suite = None;
        let report = false;
        let otel_exporter = "stdout".to_string();
        let otel_endpoint = None;

        // Act - Execute self-tests with OTEL export
        let result = run_self_tests(suite, report, otel_exporter, otel_endpoint).await;

        // Assert - Should succeed
        assert!(
            result.is_ok(),
            "Self-tests with OTEL stdout should succeed: {:?}",
            result.err()
        );
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_run_self_tests_with_invalid_otel_exporter() -> Result<()> {
        // Arrange - Test with invalid OTEL exporter
        let suite = None;
        let report = false;
        let otel_exporter = "invalid-exporter".to_string();
        let otel_endpoint = None;

        // Act - Execute self-tests with invalid exporter
        let result = run_self_tests(suite, report, otel_exporter, otel_endpoint).await;

        // Assert - Should fail with validation error
        #[cfg(feature = "otel-traces")]
        {
            assert!(result.is_err(), "Invalid OTEL exporter should fail");
            assert!(result
                .unwrap_err()
                .message
                .contains("Invalid OTEL exporter"));
        }

        #[cfg(not(feature = "otel-traces"))]
        {
            assert!(result.is_err(), "OTEL without feature should fail");
        }

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_run_self_tests_otlp_http_without_endpoint() -> Result<()> {
        // Arrange - Test with otlp-http but no endpoint
        let suite = None;
        let report = false;
        let otel_exporter = "otlp-http".to_string();
        let otel_endpoint = None;

        // Act - Execute self-tests without endpoint
        let result = run_self_tests(suite, report, otel_exporter, otel_endpoint).await;

        // Assert - Should fail with validation error
        #[cfg(feature = "otel-traces")]
        {
            assert!(result.is_err(), "OTLP-HTTP without endpoint should fail");
            assert!(result.unwrap_err().message.contains("endpoint required"));
        }

        #[cfg(not(feature = "otel-traces"))]
        {
            assert!(result.is_err(), "OTEL without feature should fail");
        }

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_run_self_tests_all_valid_suites() -> Result<()> {
        // Test all valid suite names
        let valid_suites = vec!["framework", "container", "plugin", "cli", "otel"];

        for suite_name in valid_suites {
            // Arrange
            let suite = Some(suite_name.to_string());
            let report = false;
            let otel_exporter = "none".to_string();
            let otel_endpoint = None;

            // Act
            let result = run_self_tests(suite, report, otel_exporter, otel_endpoint).await;

            // Assert
            assert!(
                result.is_ok(),
                "Suite '{}' should be valid and succeed",
                suite_name
            );
        }

        Ok(())
    }
}
