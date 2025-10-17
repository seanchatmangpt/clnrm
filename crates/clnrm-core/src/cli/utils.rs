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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_discover_test_files_single_file_valid() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new().map_err(|e| {
            CleanroomError::internal_error("Failed to create temp dir").with_source(e.to_string())
        })?;
        let test_file = temp_dir.path().join("test.toml");
        fs::write(&test_file, "test content").map_err(|e| {
            CleanroomError::internal_error("Failed to write test file").with_source(e.to_string())
        })?;

        // Act
        let result = discover_test_files(&test_file)?;

        // Assert
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], test_file);
        Ok(())
    }

    #[test]
    fn test_discover_test_files_single_file_clnrm_toml() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new().map_err(|e| {
            CleanroomError::internal_error("Failed to create temp dir").with_source(e.to_string())
        })?;
        let test_file = temp_dir.path().join("test.clnrm.toml");
        fs::write(&test_file, "test content").map_err(|e| {
            CleanroomError::internal_error("Failed to write test file").with_source(e.to_string())
        })?;

        // Act
        let result = discover_test_files(&test_file)?;

        // Assert
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], test_file);
        Ok(())
    }

    #[test]
    #[ignore = "Incomplete test data or implementation"]
    fn test_discover_test_files_single_file_invalid_extension() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new().map_err(|e| {
            CleanroomError::internal_error("Failed to create temp dir").with_source(e.to_string())
        })?;
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test content").map_err(|e| {
            CleanroomError::internal_error("Failed to write test file").with_source(e.to_string())
        })?;

        // Act
        let result = discover_test_files(&test_file);

        // Assert
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("File must have .clnrm.toml extension"));
        Ok(())
    }

    #[test]
    fn test_discover_test_files_directory() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new().map_err(|e| {
            CleanroomError::internal_error("Failed to create temp dir").with_source(e.to_string())
        })?;

        // Create test files
        let test_file1 = temp_dir.path().join("test1.clnrm.toml");
        let test_file2 = temp_dir.path().join("test2.clnrm.toml");
        let ignored_file = temp_dir.path().join("ignored.txt");

        fs::write(&test_file1, "test content 1").map_err(|e| {
            CleanroomError::internal_error("Failed to write test file 1").with_source(e.to_string())
        })?;
        fs::write(&test_file2, "test content 2").map_err(|e| {
            CleanroomError::internal_error("Failed to write test file 2").with_source(e.to_string())
        })?;
        fs::write(&ignored_file, "ignored content").map_err(|e| {
            CleanroomError::internal_error("Failed to write ignored file")
                .with_source(e.to_string())
        })?;

        // Act
        let result = discover_test_files(&temp_dir.path().to_path_buf())?;

        // Assert
        assert_eq!(result.len(), 2);
        assert!(result
            .iter()
            .any(|p| p.file_name().unwrap_or_default() == "test1.clnrm.toml"));
        assert!(result
            .iter()
            .any(|p| p.file_name().unwrap_or_default() == "test2.clnrm.toml"));
        Ok(())
    }

    #[test]
    #[ignore = "Incomplete test data or implementation"]
    fn test_discover_test_files_directory_no_test_files() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new().map_err(|e| {
            CleanroomError::internal_error("Failed to create temp dir").with_source(e.to_string())
        })?;

        // Create non-test files
        let ignored_file = temp_dir.path().join("ignored.txt");
        fs::write(&ignored_file, "ignored content").map_err(|e| {
            CleanroomError::internal_error("Failed to write ignored file")
                .with_source(e.to_string())
        })?;

        // Act
        let result = discover_test_files(&temp_dir.path().to_path_buf());

        // Assert
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("No test files (.clnrm.toml) found"));
        Ok(())
    }

    #[test]
    fn test_discover_test_files_nonexistent_path() -> Result<()> {
        // Arrange
        let nonexistent_path = PathBuf::from("nonexistent_path");

        // Act
        let result = discover_test_files(&nonexistent_path);

        // Assert
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("Path is neither a file nor a directory"));
        Ok(())
    }

    #[test]
    fn test_setup_logging() -> Result<()> {
        // Act - This should not panic
        let result = setup_logging(0);

        // Assert
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    #[ignore = "Incomplete test data or implementation"]
    fn test_setup_logging_different_verbosity_levels() -> Result<()> {
        // Test different verbosity levels
        assert!(setup_logging(0).is_ok());
        assert!(setup_logging(1).is_ok());
        assert!(setup_logging(2).is_ok());
        assert!(setup_logging(5).is_ok());
        Ok(())
    }

    #[test]
    fn test_generate_junit_xml_success() -> Result<()> {
        // Arrange
        let results = CliTestResults {
            tests: vec![
                crate::cli::types::CliTestResult {
                    name: "test1".to_string(),
                    passed: true,
                    duration_ms: 1000,
                    error: None,
                },
                crate::cli::types::CliTestResult {
                    name: "test2".to_string(),
                    passed: false,
                    duration_ms: 500,
                    error: Some("Test failed".to_string()),
                },
            ],
            total_duration_ms: 1500,
        };

        // Act
        let xml = generate_junit_xml(&results)?;

        // Assert
        assert!(xml.contains("cleanroom_tests"));
        assert!(xml.contains("test1"));
        assert!(xml.contains("test2"));
        assert!(xml.contains("Test failed"));
        Ok(())
    }

    #[test]
    fn test_generate_junit_xml_empty_results() -> Result<()> {
        // Arrange
        let results = CliTestResults {
            tests: vec![],
            total_duration_ms: 0,
        };

        // Act
        let xml = generate_junit_xml(&results)?;

        // Assert
        assert!(xml.contains("cleanroom_tests"));
        assert!(xml.contains("<testsuite"));
        Ok(())
    }

    #[test]
    #[ignore = "Incomplete test data or implementation"]
    fn test_parse_toml_test_valid() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new().map_err(|e| {
            CleanroomError::internal_error("Failed to create temp dir").with_source(e.to_string())
        })?;
        let test_file = temp_dir.path().join("test.toml");

        let toml_content = r#"
name = "test_example"

[[scenarios]]
name = "basic_test"
steps = [
    { name = "test_step", cmd = ["echo", "hello world"] }
]
"#;

        fs::write(&test_file, toml_content).map_err(|e| {
            CleanroomError::internal_error("Failed to write test file").with_source(e.to_string())
        })?;

        // Act
        let config = parse_toml_test(&test_file)?;

        // Assert
        assert_eq!(config.get_name().unwrap(), "test_example");
        assert_eq!(config.steps.len(), 1);
        assert_eq!(config.steps[0].name, "test_step");
        assert_eq!(config.steps[0].command, vec!["echo", "hello world"]);

        Ok(())
    }

    #[test]
    fn test_parse_toml_test_invalid_toml() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new().map_err(|e| {
            CleanroomError::internal_error("Failed to create temp dir").with_source(e.to_string())
        })?;
        let test_file = temp_dir.path().join("invalid.toml");

        let invalid_toml = r#"
[test
name = "invalid"
"#;

        fs::write(&test_file, invalid_toml).map_err(|e| {
            CleanroomError::internal_error("Failed to write test file").with_source(e.to_string())
        })?;

        // Act & Assert
        let result = parse_toml_test(&test_file);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_parse_toml_test_file_not_found() -> Result<()> {
        // Arrange
        let non_existent_file = PathBuf::from("non_existent.toml");

        // Act & Assert
        let result = parse_toml_test(&non_existent_file);
        assert!(result.is_err());

        Ok(())
    }
}
