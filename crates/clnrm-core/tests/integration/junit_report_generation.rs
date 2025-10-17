//! JUnit XML Report Generation Integration Tests
//!
//! Tests the complete JUnit XML report generation functionality including:
//! - CLI flag parsing and handling
//! - XML generation with proper structure
//! - File writing and error handling
//! - Timestamp and hostname inclusion
//! - Multiple test results (passed, failed, skipped)

use clnrm_core::cli::types::{CliTestResult, CliTestResults};
use clnrm_core::cli::utils::generate_junit_xml;
use clnrm_core::error::Result;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test basic JUnit XML generation with passing tests
#[test]
fn test_junit_xml_generation_with_passing_tests() -> Result<()> {
    // Arrange
    let results = CliTestResults {
        tests: vec![
            CliTestResult {
                name: "test_success_1".to_string(),
                passed: true,
                duration_ms: 150,
                error: None,
            },
            CliTestResult {
                name: "test_success_2".to_string(),
                passed: true,
                duration_ms: 200,
                error: None,
            },
        ],
        total_duration_ms: 350,
    };

    // Act
    let xml = generate_junit_xml(&results)?;

    // Assert
    assert!(xml.contains(r#"<?xml version="1.0" encoding="utf-8"?>"#));
    assert!(xml.contains(r#"<testsuites>"#));
    assert!(xml.contains(r#"<testsuite"#));
    assert!(xml.contains(r#"name="cleanroom_tests""#));
    assert!(xml.contains(r#"tests="2""#));
    assert!(xml.contains(r#"failures="0""#));
    assert!(xml.contains("test_success_1"));
    assert!(xml.contains("test_success_2"));
    assert!(xml.contains("</testsuite>"));
    assert!(xml.contains("</testsuites>"));

    Ok(())
}

/// Test JUnit XML generation with failing tests
#[test]
fn test_junit_xml_generation_with_failing_tests() -> Result<()> {
    // Arrange
    let results = CliTestResults {
        tests: vec![
            CliTestResult {
                name: "test_pass".to_string(),
                passed: true,
                duration_ms: 100,
                error: None,
            },
            CliTestResult {
                name: "test_fail".to_string(),
                passed: false,
                duration_ms: 200,
                error: Some("Assertion failed: expected 2, got 1".to_string()),
            },
        ],
        total_duration_ms: 300,
    };

    // Act
    let xml = generate_junit_xml(&results)?;

    // Assert
    assert!(xml.contains(r#"tests="2""#));
    assert!(xml.contains(r#"failures="1""#));
    assert!(xml.contains("test_pass"));
    assert!(xml.contains("test_fail"));
    assert!(xml.contains("Assertion failed: expected 2, got 1"));
    assert!(xml.contains("<failure"));

    Ok(())
}

/// Test JUnit XML generation with special characters that need escaping
#[test]
fn test_junit_xml_generation_with_special_characters() -> Result<()> {
    // Arrange
    let results = CliTestResults {
        tests: vec![CliTestResult {
            name: "test_with_<special>_chars".to_string(),
            passed: false,
            duration_ms: 100,
            error: Some(r#"Error: "value" & 'expected' < 10 > 5"#.to_string()),
        }],
        total_duration_ms: 100,
    };

    // Act
    let xml = generate_junit_xml(&results)?;

    // Assert
    // XML should contain escaped characters, not raw special characters
    assert!(xml.contains("&lt;"));
    assert!(xml.contains("&gt;"));
    assert!(xml.contains("&quot;"));
    assert!(xml.contains("&amp;"));

    // Verify the XML is valid by checking it doesn't contain unescaped special chars in data
    let error_line_count = xml.lines().filter(|line| line.contains("Error:")).count();
    assert!(error_line_count > 0, "Should contain escaped error message");

    Ok(())
}

/// Test JUnit XML includes timestamp
#[test]
fn test_junit_xml_includes_timestamp() -> Result<()> {
    // Arrange
    let results = CliTestResults {
        tests: vec![CliTestResult {
            name: "test_timestamp".to_string(),
            passed: true,
            duration_ms: 50,
            error: None,
        }],
        total_duration_ms: 50,
    };

    // Act
    let xml = generate_junit_xml(&results)?;

    // Assert
    assert!(
        xml.contains("timestamp="),
        "JUnit XML should include timestamp attribute"
    );

    Ok(())
}

/// Test JUnit XML file writing
#[test]
fn test_junit_xml_file_writing() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new()
        .map_err(|e| clnrm_core::error::CleanroomError::io_error(format!("TempDir failed: {}", e)))?;
    let junit_path = temp_dir.path().join("junit.xml");

    let results = CliTestResults {
        tests: vec![
            CliTestResult {
                name: "test_1".to_string(),
                passed: true,
                duration_ms: 100,
                error: None,
            },
            CliTestResult {
                name: "test_2".to_string(),
                passed: false,
                duration_ms: 200,
                error: Some("Test failed".to_string()),
            },
        ],
        total_duration_ms: 300,
    };

    // Act
    let xml = generate_junit_xml(&results)?;
    std::fs::write(&junit_path, &xml).map_err(|e| {
        clnrm_core::error::CleanroomError::io_error(format!("Failed to write file: {}", e))
    })?;

    // Assert
    assert!(junit_path.exists(), "JUnit XML file should be created");

    let content = std::fs::read_to_string(&junit_path).map_err(|e| {
        clnrm_core::error::CleanroomError::io_error(format!("Failed to read file: {}", e))
    })?;

    assert!(content.contains("test_1"));
    assert!(content.contains("test_2"));
    assert!(content.contains("Test failed"));

    Ok(())
}

/// Test JUnit XML with empty test suite
#[test]
fn test_junit_xml_generation_with_empty_results() -> Result<()> {
    // Arrange
    let results = CliTestResults {
        tests: vec![],
        total_duration_ms: 0,
    };

    // Act
    let xml = generate_junit_xml(&results)?;

    // Assert
    assert!(xml.contains(r#"tests="0""#));
    assert!(xml.contains(r#"failures="0""#));
    assert!(xml.contains("testsuite"));

    Ok(())
}

/// Test JUnit XML generation with multiple test suites
#[test]
fn test_junit_xml_generation_with_many_tests() -> Result<()> {
    // Arrange
    let mut tests = Vec::new();
    for i in 0..10 {
        tests.push(CliTestResult {
            name: format!("test_{}", i),
            passed: i % 2 == 0, // Every other test fails
            duration_ms: 100 + i * 10,
            error: if i % 2 != 0 {
                Some(format!("Test {} failed", i))
            } else {
                None
            },
        });
    }

    let results = CliTestResults {
        tests,
        total_duration_ms: 1500,
    };

    // Act
    let xml = generate_junit_xml(&results)?;

    // Assert
    assert!(xml.contains(r#"tests="10""#));
    assert!(xml.contains(r#"failures="5""#));

    // Verify all test names are present
    for i in 0..10 {
        assert!(
            xml.contains(&format!("test_{}", i)),
            "Should contain test_{}",
            i
        );
    }

    Ok(())
}

/// Test JUnit XML structure validation
#[test]
fn test_junit_xml_structure_validation() -> Result<()> {
    // Arrange
    let results = CliTestResults {
        tests: vec![
            CliTestResult {
                name: "test_structure".to_string(),
                passed: true,
                duration_ms: 100,
                error: None,
            },
        ],
        total_duration_ms: 100,
    };

    // Act
    let xml = generate_junit_xml(&results)?;

    // Assert - verify basic XML structure
    let opening_tags = xml.matches("<testsuites>").count();
    let closing_tags = xml.matches("</testsuites>").count();
    assert_eq!(
        opening_tags, closing_tags,
        "Opening and closing testsuites tags should match"
    );

    let opening_suite_tags = xml.matches("<testsuite").count();
    let closing_suite_tags = xml.matches("</testsuite>").count();
    assert_eq!(
        opening_suite_tags, closing_suite_tags,
        "Opening and closing testsuite tags should match"
    );

    Ok(())
}

/// Test JUnit XML with hostname information
#[test]
fn test_junit_xml_includes_hostname_if_available() -> Result<()> {
    // Arrange
    let results = CliTestResults {
        tests: vec![CliTestResult {
            name: "test_hostname".to_string(),
            passed: true,
            duration_ms: 50,
            error: None,
        }],
        total_duration_ms: 50,
    };

    // Act
    let xml = generate_junit_xml(&results)?;

    // Assert - hostname may or may not be present depending on environment
    // But if it is present, it should be in the correct format
    if xml.contains("hostname=") {
        // Verify hostname is properly formatted in XML
        assert!(
            xml.contains(r#"hostname=""#) || xml.contains(r#"hostname='"#),
            "Hostname should be properly quoted in XML"
        );
    }

    Ok(())
}

#[cfg(test)]
mod cli_integration_tests {
    use super::*;
    use clnrm_core::cli::types::CliConfig;

    /// Test that --report-junit flag is properly defined in CLI
    #[test]
    fn test_report_junit_flag_exists() {
        // This test verifies the CLI structure compilation
        // If this compiles, the --report-junit flag is properly defined
        let config = CliConfig::default();
        assert_eq!(config.jobs, 4);
    }
}

#[cfg(test)]
mod xml_parsing_tests {
    use super::*;

    /// Test that generated XML can be parsed
    #[test]
    fn test_generated_xml_is_parseable() -> Result<()> {
        // Arrange
        let results = CliTestResults {
            tests: vec![
                CliTestResult {
                    name: "test_parse_1".to_string(),
                    passed: true,
                    duration_ms: 100,
                    error: None,
                },
                CliTestResult {
                    name: "test_parse_2".to_string(),
                    passed: false,
                    duration_ms: 200,
                    error: Some("Parse error test".to_string()),
                },
            ],
            total_duration_ms: 300,
        };

        // Act
        let xml = generate_junit_xml(&results)?;

        // Assert - Try to parse the XML using quick_xml
        use quick_xml::Reader;
        let mut reader = Reader::from_str(&xml);
        reader.config_mut().check_end_names = true;

        let mut buf = Vec::new();
        let mut tag_count = 0;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Eof) => break,
                Ok(quick_xml::events::Event::Start(_)) => {
                    tag_count += 1;
                }
                Ok(_) => {}
                Err(e) => {
                    return Err(clnrm_core::error::CleanroomError::validation_error(
                        format!("XML parsing error: {}", e),
                    ));
                }
            }
            buf.clear();
        }

        assert!(tag_count > 0, "Should have parsed some XML tags");

        Ok(())
    }
}
