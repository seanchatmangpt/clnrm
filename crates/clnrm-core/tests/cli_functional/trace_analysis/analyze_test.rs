//! Analyze command tests
//!
//! Tests verify OTEL trace analysis using AAA pattern.

use clnrm_core::cli::commands::analyze::analyze_traces;
use clnrm_core::error::Result;
use tempfile::NamedTempFile;

// Helper module
mod helpers {
    use clnrm_core::error::{CleanroomError, Result};
    use tempfile::NamedTempFile;

    pub fn create_temp_file(contents: &str) -> Result<NamedTempFile> {
        let mut file = tempfile::NamedTempFile::new().map_err(|e| {
            CleanroomError::io_error(format!("Failed to create temp file: {}", e))
        })?;
        
        std::io::Write::write_all(&mut file, contents.as_bytes()).map_err(|e| {
            CleanroomError::io_error(format!("Failed to write temp file: {}", e))
        })?;
        
        file.flush().map_err(|e| {
            CleanroomError::io_error(format!("Failed to flush temp file: {}", e))
        })?;
        
        Ok(file)
    }

    pub fn temp_file_path(file: &NamedTempFile) -> std::path::PathBuf {
        file.path().to_path_buf()
    }
}

#[test]
fn test_analyze_loads_traces_and_runs_validators() -> Result<()> {
    // Arrange - Create test config and trace file
    let test_config = r#"
[meta]
name = "test_analyze"
version = "1.0.0"

[[expect.span]]
name = "test.span"
kind = "internal"
"#;

    let trace_content = r#"
{
  "spans": [
    {
      "name": "test.span",
      "span_id": "span-1",
      "trace_id": "trace-1",
      "kind": "internal"
    }
  ]
}
"#;

    let test_file = helpers::create_temp_file(test_config)?;
    let trace_file = helpers::create_temp_file(trace_content)?;
    let test_path = helpers::temp_file_path(&test_file);
    let trace_path = helpers::temp_file_path(&trace_file);

    // Act - Analyze traces
    let result = analyze_traces(&test_path, Some(&trace_path));

    // Assert - Should produce analysis report
    assert!(
        result.is_ok(),
        "BEHAVIOR: Analyze should succeed with valid inputs"
    );

    let report = result?;
    assert!(
        !report.validators.is_empty(),
        "BEHAVIOR: Should run validators on traces"
    );

    Ok(())
}

#[test]
fn test_analyze_fails_with_missing_trace_file() -> Result<()> {
    // Arrange - Create test config but no trace file
    let test_config = r#"
[meta]
name = "test_analyze"
version = "1.0.0"
"#;

    let test_file = helpers::create_temp_file(test_config)?;
    let test_path = helpers::temp_file_path(&test_file);
    let missing_path = std::path::PathBuf::from("/nonexistent/trace.json");

    // Act - Try to analyze with missing trace
    let result = analyze_traces(&test_path, Some(&missing_path));

    // Assert - Should fail gracefully
    assert!(
        result.is_err(),
        "BEHAVIOR: Should fail with proper error for missing trace file"
    );

    Ok(())
}

