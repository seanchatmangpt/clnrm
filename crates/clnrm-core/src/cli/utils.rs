//! Utility functions for the CLI module
//!
//! Contains shared utility functions used across CLI commands.

use crate::cli::types::{CliTestResults, ACCEPTED_EXTENSIONS};
use crate::config::load_config_from_file;
use crate::error::{CleanroomError, Result};
use std::path::{Path, PathBuf};
use tracing::{debug, info};
use walkdir::WalkDir;

/// Discover all .clnrm.toml test files in a directory
///
/// Core Team Compliance:
/// - ✅ Proper error handling with CleanroomError
/// - ✅ No unwrap() or expect() calls
/// - ✅ Sync function for file system operations
pub fn discover_test_files(path: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut test_files = Vec::new();

    if path.is_file() {
        // If single file, check extension - accept both .toml and .clnrm.toml
        let path_str = path.to_str().unwrap_or("");
        if ACCEPTED_EXTENSIONS
            .iter()
            .any(|ext| path_str.ends_with(ext))
        {
            test_files.push(path.clone());
        } else {
            return Err(CleanroomError::validation_error(format!(
                "File must have .toml or .clnrm.toml extension: {}",
                path.display()
            )));
        }
    } else if path.is_dir() {
        // Search recursively for test files with accepted extensions
        info!("Discovering test files in: {}", path.display());

        for entry in WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let entry_path = entry.path();
            let path_str = entry_path.to_str().unwrap_or("");

            // Accept both .toml and .clnrm.toml files
            if ACCEPTED_EXTENSIONS
                .iter()
                .any(|ext| path_str.ends_with(ext))
                && entry_path.is_file()
            {
                test_files.push(entry_path.to_path_buf());
                debug!("Found test file: {}", entry_path.display());
            }
        }

        if test_files.is_empty() {
            return Err(CleanroomError::validation_error(format!(
                "No test files (.toml or .clnrm.toml) found in directory: {}",
                path.display()
            )));
        }

        info!("Discovered {} test file(s)", test_files.len());
    } else {
        return Err(CleanroomError::validation_error(format!(
            "Path is neither a file nor a directory: {}",
            path.display()
        )));
    }

    Ok(test_files)
}

/// Parse a TOML test configuration file
pub fn parse_toml_test(path: &Path) -> Result<crate::config::TestConfig> {
    load_config_from_file(path)
}

/// Set up logging based on verbosity level
pub fn setup_logging(verbosity: u8) -> Result<()> {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = match verbosity {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    let subscriber = fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::new(filter))
        .finish();

    tracing::subscriber::set_global_default(subscriber).map_err(|e| {
        CleanroomError::internal_error("Failed to set up logging").with_source(e.to_string())
    })?;

    Ok(())
}

/// Generate JUnit XML output for CI/CD integration
///
/// # Core Team Compliance
/// - ✅ Proper error handling with CleanroomError
/// - ✅ No unwrap() or expect() calls
/// - ✅ Returns Result<String, CleanroomError>
/// - ✅ Includes timestamp information
pub fn generate_junit_xml(results: &CliTestResults) -> Result<String> {
    use junit_report::{Duration, OffsetDateTime, Report, TestCase, TestSuite};

    let mut test_suite = TestSuite::new("cleanroom_tests");
    test_suite.set_timestamp(OffsetDateTime::now_utc());

    for test in &results.tests {
        let duration_secs = test.duration_ms as f64 / 1000.0;
        let test_case = if !test.passed {
            if let Some(error) = &test.error {
                TestCase::failure(
                    &test.name,
                    Duration::seconds(duration_secs as i64),
                    "test_failure",
                    error,
                )
            } else {
                TestCase::failure(
                    &test.name,
                    Duration::seconds(duration_secs as i64),
                    "test_failure",
                    "Test failed without error message",
                )
            }
        } else {
            TestCase::success(&test.name, Duration::seconds(duration_secs as i64))
        };

        test_suite.add_testcase(test_case);
    }

    let mut report = Report::new();
    report.add_testsuite(test_suite);

    let mut xml_output = Vec::new();
    report.write_xml(&mut xml_output).map_err(|e| {
        CleanroomError::internal_error("JUnit XML generation failed")
            .with_context("Failed to serialize test results to JUnit XML")
            .with_source(e.to_string())
    })?;

    String::from_utf8(xml_output).map_err(|e| {
        CleanroomError::internal_error("JUnit XML encoding failed")
            .with_context("Failed to convert JUnit XML to UTF-8 string")
            .with_source(e.to_string())
    })
}
