//! Self-test command implementation with OTEL export support
//!
//! Handles framework self-testing with comprehensive validation, reporting, and OpenTelemetry export.

use crate::error::{CleanroomError, Result};
use tracing::{info, span, Level};

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
    let _guard = if otel_exporter != "none" {
        Some(init_otel_for_self_test(
            &otel_exporter,
            _otel_endpoint.as_deref(),
        )?)
    } else {
        None
    };

    // Use tracing instead of println for internal operations
    info!("Starting framework self-tests");

    let _root_span = if otel_exporter != "none" {
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

    let _enter = _root_span.enter();

    // Validate suite parameter if provided
    if let Some(ref suite_name) = suite {
        const VALID_SUITES: &[&str] = &["framework", "container", "plugin", "cli", "otel"];
        if !VALID_SUITES.contains(&suite_name.as_str()) {
            {
                _root_span.record("result", "error");
                _root_span.record("error.type", "validation_error");
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

    {
        if test_results.failed_tests > 0 {
            _root_span.record("result", "fail");
            _root_span.record("failed_tests", test_results.failed_tests);
        } else {
            _root_span.record("result", "pass");
        }
        _root_span.record("total_tests", test_results.total_tests);
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
