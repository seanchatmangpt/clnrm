//! Report Generation Integration Tests
//!
//! This file consolidates critical report generation tests focusing on:
//! - JSON report generation
//! - JUnit XML report generation
//! - SHA-256 digest generation
//! - Multi-format reporting
//! - Report integrity validation
//!
//! Testing Philosophy: 80/20 Principle
//! - Focus on critical report formats (JSON, JUnit XML)
//! - Test actual file generation, not mocked reporters
//! - Verify report structure and content
//!
//! Core Team Compliance:
//! - ✅ AAA pattern (Arrange, Act, Assert)
//! - ✅ Descriptive test names
//! - ✅ Proper error handling

#![allow(clippy::unwrap_used)] // Test code only

use clnrm_core::error::{CleanroomError, Result};
use clnrm_core::reporting::{DigestReporter, JsonReporter, JunitReporter};
use clnrm_core::validation::ValidationReport;
use std::path::Path;
use tempfile::TempDir;

// ============================================================================
// JSON Report Generation Tests
// ============================================================================

#[tokio::test]
async fn test_json_report_generation_produces_valid_json() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new()
        .map_err(|e| CleanroomError::internal_error(format!("Failed to create temp dir: {}", e)))?;
    let report_path = temp_dir.path().join("report.json");

    let mut validation_report = ValidationReport::new();
    validation_report.add_pass("test_successful_scenario");
    validation_report.add_fail("test_failed_scenario", "Assertion failed".to_string());

    // Act
    JsonReporter::write(Path::new(&report_path), &validation_report)?;

    // Assert
    assert!(report_path.exists());

    let json_content = std::fs::read_to_string(&report_path)
        .map_err(|e| CleanroomError::io_error(format!("Failed to read JSON report: {}", e)))?;

    let parsed: serde_json::Value = serde_json::from_str(&json_content)
        .map_err(|e| CleanroomError::serialization_error(format!("Failed to parse JSON: {}", e)))?;

    assert!(parsed.is_object());
    assert!(json_content.contains("test_successful_scenario"));
    assert!(json_content.contains("test_failed_scenario"));

    Ok(())
}

// ============================================================================
// JUnit XML Report Generation Tests
// ============================================================================

#[tokio::test]
async fn test_junit_xml_generation_produces_valid_xml() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new()
        .map_err(|e| CleanroomError::internal_error(format!("Failed to create temp dir: {}", e)))?;
    let junit_path = temp_dir.path().join("junit.xml");

    let mut validation_report = ValidationReport::new();
    validation_report.add_pass("test_integration_success");
    validation_report.add_fail("test_integration_failure", "Expected value not found".to_string());

    // Act
    JunitReporter::write(Path::new(&junit_path), &validation_report)?;

    // Assert
    assert!(junit_path.exists());

    let xml_content = std::fs::read_to_string(&junit_path)
        .map_err(|e| CleanroomError::io_error(format!("Failed to read JUnit XML: {}", e)))?;

    assert!(xml_content.contains("<?xml"));
    assert!(xml_content.contains("<testsuite"));
    assert!(xml_content.contains("<testcase"));
    assert!(xml_content.contains("test_integration_success"));
    assert!(xml_content.contains("test_integration_failure"));
    assert!(xml_content.contains("<failure"));

    Ok(())
}

// ============================================================================
// SHA-256 Digest Generation Tests
// ============================================================================

#[tokio::test]
async fn test_sha256_digest_generation_for_spans() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new()
        .map_err(|e| CleanroomError::internal_error(format!("Failed to create temp dir: {}", e)))?;
    let digest_path = temp_dir.path().join("digest.txt");

    let spans_json = r#"{"spans": [{"name": "test.span", "trace_id": "abc123"}]}"#;

    // Act
    DigestReporter::write(Path::new(&digest_path), spans_json)?;

    // Assert
    assert!(digest_path.exists());

    let digest_content = std::fs::read_to_string(&digest_path)
        .map_err(|e| CleanroomError::io_error(format!("Failed to read digest: {}", e)))?;

    let digest = digest_content.trim();

    assert_eq!(digest.len(), 64, "SHA-256 hex digest should be 64 characters");
    assert!(digest.chars().all(|c| c.is_ascii_hexdigit()));

    Ok(())
}

#[tokio::test]
async fn test_digest_reproducibility_across_runs() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new()
        .map_err(|e| CleanroomError::internal_error(format!("Failed to create temp dir: {}", e)))?;

    let spans_json = r#"{"spans": [{"name": "reproducible.span"}]}"#;

    // Act
    let digest_path1 = temp_dir.path().join("digest1.txt");
    DigestReporter::write(Path::new(&digest_path1), spans_json)?;
    let digest1 = std::fs::read_to_string(&digest_path1)
        .map_err(|e| CleanroomError::io_error(format!("Failed to read digest1: {}", e)))?;

    let digest_path2 = temp_dir.path().join("digest2.txt");
    DigestReporter::write(Path::new(&digest_path2), spans_json)?;
    let digest2 = std::fs::read_to_string(&digest_path2)
        .map_err(|e| CleanroomError::io_error(format!("Failed to read digest2: {}", e)))?;

    // Assert
    assert_eq!(digest1.trim(), digest2.trim());

    Ok(())
}

// ============================================================================
// Report Structure Validation Tests
// ============================================================================

#[tokio::test]
async fn test_json_report_contains_test_results() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new()
        .map_err(|e| CleanroomError::internal_error(format!("Failed to create temp dir: {}", e)))?;
    let report_path = temp_dir.path().join("report.json");

    let mut validation_report = ValidationReport::new();
    validation_report.add_pass("test_one");
    validation_report.add_pass("test_two");
    validation_report.add_fail("test_three", "Failed assertion".to_string());

    // Act
    JsonReporter::write(Path::new(&report_path), &validation_report)?;

    // Assert
    let json_content = std::fs::read_to_string(&report_path)
        .map_err(|e| CleanroomError::io_error(format!("Failed to read JSON: {}", e)))?;

    assert!(json_content.contains("test_one"));
    assert!(json_content.contains("test_two"));
    assert!(json_content.contains("test_three"));
    assert!(json_content.contains("Failed assertion"));

    Ok(())
}

#[tokio::test]
async fn test_junit_xml_includes_test_counts() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new()
        .map_err(|e| CleanroomError::internal_error(format!("Failed to create temp dir: {}", e)))?;
    let junit_path = temp_dir.path().join("junit.xml");

    let mut validation_report = ValidationReport::new();
    validation_report.add_pass("test_1");
    validation_report.add_pass("test_2");
    validation_report.add_fail("test_3", "Failure".to_string());

    // Act
    JunitReporter::write(Path::new(&junit_path), &validation_report)?;

    // Assert
    let xml_content = std::fs::read_to_string(&junit_path)
        .map_err(|e| CleanroomError::io_error(format!("Failed to read XML: {}", e)))?;

    assert!(xml_content.contains("tests="));
    assert!(xml_content.contains("failures="));

    Ok(())
}

// ============================================================================
// Multi-Report Generation Tests
// ============================================================================

#[tokio::test]
async fn test_generate_multiple_report_formats_simultaneously() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new()
        .map_err(|e| CleanroomError::internal_error(format!("Failed to create temp dir: {}", e)))?;
    let json_path = temp_dir.path().join("report.json");
    let junit_path = temp_dir.path().join("junit.xml");

    let mut validation_report = ValidationReport::new();
    validation_report.add_pass("multi_format_test");

    // Act
    JsonReporter::write(Path::new(&json_path), &validation_report)?;
    JunitReporter::write(Path::new(&junit_path), &validation_report)?;

    // Assert
    assert!(json_path.exists());
    assert!(junit_path.exists());

    Ok(())
}

// ============================================================================
// Edge Cases
// ============================================================================

#[tokio::test]
async fn test_json_report_with_empty_results() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new()
        .map_err(|e| CleanroomError::internal_error(format!("Failed to create temp dir: {}", e)))?;
    let report_path = temp_dir.path().join("empty.json");

    let validation_report = ValidationReport::new();

    // Act
    JsonReporter::write(Path::new(&report_path), &validation_report)?;

    // Assert
    assert!(report_path.exists());
    let json_content = std::fs::read_to_string(&report_path)
        .map_err(|e| CleanroomError::io_error(format!("Failed to read JSON: {}", e)))?;

    let parsed: serde_json::Value = serde_json::from_str(&json_content)
        .map_err(|e| CleanroomError::serialization_error(format!("Parse failed: {}", e)))?;

    assert!(parsed.is_object());

    Ok(())
}

#[tokio::test]
async fn test_junit_xml_with_special_characters_in_test_names() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new()
        .map_err(|e| CleanroomError::internal_error(format!("Failed to create temp dir: {}", e)))?;
    let junit_path = temp_dir.path().join("junit.xml");

    let mut validation_report = ValidationReport::new();
    validation_report.add_fail("test_with_<special>_&_chars", "Error: \"quoted\" value".to_string());

    // Act
    JunitReporter::write(Path::new(&junit_path), &validation_report)?;

    // Assert
    let xml_content = std::fs::read_to_string(&junit_path)
        .map_err(|e| CleanroomError::io_error(format!("Failed to read XML: {}", e)))?;

    // XML should escape special characters
    assert!(xml_content.contains("&lt;") || xml_content.contains("&gt;") || xml_content.contains("&quot;"));

    Ok(())
}

#[tokio::test]
async fn test_digest_generation_with_empty_input() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new()
        .map_err(|e| CleanroomError::internal_error(format!("Failed to create temp dir: {}", e)))?;
    let digest_path = temp_dir.path().join("empty_digest.txt");

    let empty_json = "";

    // Act
    DigestReporter::write(Path::new(&digest_path), empty_json)?;

    // Assert
    assert!(digest_path.exists());
    let digest_content = std::fs::read_to_string(&digest_path)
        .map_err(|e| CleanroomError::io_error(format!("Failed to read digest: {}", e)))?;

    let digest = digest_content.trim();
    assert_eq!(digest.len(), 64);

    Ok(())
}
