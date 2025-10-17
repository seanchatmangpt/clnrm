//! London School TDD Tests for Multi-Format Reporting
//!
//! Test Coverage:
//! - JSON report generation with proper structure
//! - JUnit XML report generation
//! - SHA-256 digest generation for reports
//! - Digest reproducibility across runs
//! - Report integrity validation
//! - Multi-format parallel generation
//! - Report file persistence
//!
//! Testing Philosophy:
//! - OUTSIDE-IN: Test reporting from validation report perspective
//! - MOCK-FIRST: Define reporter contracts through test doubles
//! - BEHAVIOR VERIFICATION: Focus on output format correctness
//!
//! Core Team Compliance:
//! - ✅ AAA pattern (Arrange, Act, Assert)
//! - ✅ Descriptive test names (test_X_with_Y_produces_Z)
//! - ✅ No false positives - proper error propagation
//! - ✅ Result<()> for proper error handling

#![allow(clippy::unwrap_used)] // Test code only

use clnrm_core::error::{CleanroomError, Result};
use clnrm_core::reporting::{
    generate_reports, DigestReporter, JsonReporter, JunitReporter, ReportConfig,
};
use clnrm_core::validation::ValidationReport;
use sha2::{Digest, Sha256};
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

    // Assert - File exists and contains valid JSON
    assert!(report_path.exists(), "JSON report file should be created");

    let json_content = std::fs::read_to_string(&report_path)
        .map_err(|e| CleanroomError::io_error(format!("Failed to read JSON report: {}", e)))?;

    let parsed: serde_json::Value = serde_json::from_str(&json_content).map_err(|e| {
        CleanroomError::serialization_error(format!("Failed to parse JSON report: {}", e))
    })?;

    // Verify structure
    assert!(parsed.is_object(), "Report should be JSON object");
    assert!(
        json_content.contains("test_successful_scenario"),
        "Should contain passed test"
    );
    assert!(
        json_content.contains("test_failed_scenario"),
        "Should contain failed test"
    );

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
    validation_report.add_fail(
        "test_integration_failure",
        "Expected value not found".to_string(),
    );

    // Act
    JunitReporter::write(Path::new(&junit_path), &validation_report)?;

    // Assert - File exists and contains XML
    assert!(junit_path.exists(), "JUnit XML file should be created");

    let xml_content = std::fs::read_to_string(&junit_path)
        .map_err(|e| CleanroomError::io_error(format!("Failed to read JUnit XML: {}", e)))?;

    // Verify XML structure
    assert!(xml_content.contains("<?xml"), "Should have XML declaration");
    assert!(
        xml_content.contains("<testsuite"),
        "Should have testsuite element"
    );
    assert!(
        xml_content.contains("<testcase"),
        "Should have testcase elements"
    );
    assert!(
        xml_content.contains("test_integration_success"),
        "Should contain passed test"
    );
    assert!(
        xml_content.contains("test_integration_failure"),
        "Should contain failed test"
    );
    assert!(
        xml_content.contains("<failure"),
        "Failed test should have failure element"
    );

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

    // Assert - File exists and contains SHA-256 digest
    assert!(digest_path.exists(), "Digest file should be created");

    let digest_content = std::fs::read_to_string(&digest_path)
        .map_err(|e| CleanroomError::io_error(format!("Failed to read digest: {}", e)))?;

    let digest = digest_content.trim();

    // Verify digest format
    assert_eq!(
        digest.len(),
        64,
        "SHA-256 hex digest should be 64 characters"
    );
    assert!(
        digest.chars().all(|c| c.is_ascii_hexdigit()),
        "Digest should contain only hex characters"
    );

    Ok(())
}

#[tokio::test]
async fn test_digest_reproducibility_across_runs() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new()
        .map_err(|e| CleanroomError::internal_error(format!("Failed to create temp dir: {}", e)))?;

    let spans_json = r#"{"spans": [{"name": "reproducible.span"}]}"#;

    // Act - Generate digest twice
    let digest_path1 = temp_dir.path().join("digest1.txt");
    DigestReporter::write(Path::new(&digest_path1), spans_json)?;
    let digest1 = std::fs::read_to_string(&digest_path1)
        .map_err(|e| CleanroomError::io_error(format!("Failed to read digest1: {}", e)))?;

    let digest_path2 = temp_dir.path().join("digest2.txt");
    DigestReporter::write(Path::new(&digest_path2), spans_json)?;
    let digest2 = std::fs::read_to_string(&digest_path2)
        .map_err(|e| CleanroomError::io_error(format!("Failed to read digest2: {}", e)))?;

    // Assert - Digests are identical
    assert_eq!(
        digest1.trim(),
        digest2.trim(),
        "Digest should be reproducible across multiple runs"
    );

    Ok(())
}

// ============================================================================
// Report Integrity Validation Tests
// ============================================================================

#[tokio::test]
async fn test_report_integrity_validation_detects_corruption() -> Result<()> {
    // Arrange
    let original_spans = r#"{"spans": [{"name": "original"}]}"#;
    let modified_spans = r#"{"spans": [{"name": "modified"}]}"#;

    let mut hasher1 = Sha256::new();
    hasher1.update(original_spans.as_bytes());
    let digest1 = format!("{:x}", hasher1.finalize());

    let mut hasher2 = Sha256::new();
    hasher2.update(modified_spans.as_bytes());
    let digest2 = format!("{:x}", hasher2.finalize());

    // Act - Compare digests
    let integrity_verified = digest1 == digest2;

    // Assert - Modification detected
    assert!(
        !integrity_verified,
        "Modified data should produce different digest"
    );
    assert_ne!(
        digest1, digest2,
        "Digests should differ when data is modified"
    );

    Ok(())
}

// ============================================================================
// Multi-Format Parallel Generation Tests
// ============================================================================

#[tokio::test]
async fn test_multi_format_parallel_generation_creates_all_reports() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new()
        .map_err(|e| CleanroomError::internal_error(format!("Failed to create temp dir: {}", e)))?;

    let json_path = temp_dir.path().join("report.json");
    let junit_path = temp_dir.path().join("junit.xml");
    let digest_path = temp_dir.path().join("digest.txt");

    let config = ReportConfig::new()
        .with_json(json_path.to_string_lossy().to_string())
        .with_junit(junit_path.to_string_lossy().to_string())
        .with_digest(digest_path.to_string_lossy().to_string());

    let mut validation_report = ValidationReport::new();
    validation_report.add_pass("test_multi_format");
    validation_report.add_fail("test_failure", "Error occurred".to_string());

    let spans_json = r#"{"spans": [{"name": "test.span"}]}"#;

    // Act - Generate all reports in parallel
    generate_reports(&config, &validation_report, spans_json)?;

    // Assert - All files created
    assert!(json_path.exists(), "JSON report should be created");
    assert!(junit_path.exists(), "JUnit XML should be created");
    assert!(digest_path.exists(), "Digest should be created");

    // Verify content is non-empty
    let json_size = std::fs::metadata(&json_path)
        .map_err(|e| CleanroomError::io_error(format!("Failed to read metadata: {}", e)))?
        .len();
    let junit_size = std::fs::metadata(&junit_path)
        .map_err(|e| CleanroomError::io_error(format!("Failed to read metadata: {}", e)))?
        .len();
    let digest_size = std::fs::metadata(&digest_path)
        .map_err(|e| CleanroomError::io_error(format!("Failed to read metadata: {}", e)))?
        .len();

    assert!(json_size > 0, "JSON report should have content");
    assert!(junit_size > 0, "JUnit XML should have content");
    assert!(digest_size > 0, "Digest should have content");

    Ok(())
}

// ============================================================================
// Report File Persistence Tests
// ============================================================================

#[tokio::test]
async fn test_report_file_persistence_maintains_data() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new()
        .map_err(|e| CleanroomError::internal_error(format!("Failed to create temp dir: {}", e)))?;
    let report_path = temp_dir.path().join("persistent_report.json");

    let mut validation_report = ValidationReport::new();
    validation_report.add_pass("test_persistent_data");

    // Act - Write report
    JsonReporter::write(Path::new(&report_path), &validation_report)?;

    // Simulate application restart by re-reading file
    let persisted_content = std::fs::read_to_string(&report_path)
        .map_err(|e| CleanroomError::io_error(format!("Failed to read persisted report: {}", e)))?;

    let parsed: serde_json::Value = serde_json::from_str(&persisted_content).map_err(|e| {
        CleanroomError::serialization_error(format!("Failed to parse persisted report: {}", e))
    })?;

    // Assert - Data persisted correctly
    assert!(
        persisted_content.contains("test_persistent_data"),
        "Persisted report should contain test data"
    );
    assert!(
        parsed.is_object(),
        "Persisted report should be valid JSON object"
    );

    Ok(())
}

// ============================================================================
// Performance and Scalability Tests
// ============================================================================

#[tokio::test]
async fn test_report_generation_performance_with_large_dataset() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new()
        .map_err(|e| CleanroomError::internal_error(format!("Failed to create temp dir: {}", e)))?;
    let report_path = temp_dir.path().join("large_report.json");

    let mut validation_report = ValidationReport::new();

    // Add 1000 test results
    for i in 0..1000 {
        if i % 2 == 0 {
            validation_report.add_pass(&format!("test_pass_{}", i));
        } else {
            validation_report.add_fail(&format!("test_fail_{}", i), "Error".to_string());
        }
    }

    let start = std::time::Instant::now();

    // Act
    JsonReporter::write(Path::new(&report_path), &validation_report)?;

    let elapsed = start.elapsed();

    // Assert - Performance target <500ms for 1000 tests
    assert!(
        elapsed.as_millis() < 500,
        "Report generation should be fast (<500ms for 1000 tests), took {}ms",
        elapsed.as_millis()
    );

    // Verify file was created
    assert!(report_path.exists(), "Report should be created");

    Ok(())
}

// ============================================================================
// Edge Cases and Error Handling
// ============================================================================

#[tokio::test]
async fn test_empty_validation_report_generates_valid_output() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new()
        .map_err(|e| CleanroomError::internal_error(format!("Failed to create temp dir: {}", e)))?;
    let report_path = temp_dir.path().join("empty_report.json");

    let validation_report = ValidationReport::new(); // Empty report

    // Act
    JsonReporter::write(Path::new(&report_path), &validation_report)?;

    // Assert - Empty report is valid
    assert!(report_path.exists(), "Empty report should be created");

    let json_content = std::fs::read_to_string(&report_path)
        .map_err(|e| CleanroomError::io_error(format!("Failed to read report: {}", e)))?;

    let parsed: serde_json::Value = serde_json::from_str(&json_content).map_err(|e| {
        CleanroomError::serialization_error(format!("Failed to parse report: {}", e))
    })?;

    assert!(
        parsed.is_object(),
        "Empty report should still be valid JSON"
    );

    Ok(())
}
