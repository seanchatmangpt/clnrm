//! Spans command tests
//!
//! Tests verify span filtering using AAA pattern.

use clnrm_core::cli::commands::spans::filter_spans;
use clnrm_core::cli::types::OutputFormat;
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
fn test_spans_filters_by_grep_pattern() -> Result<()> {
    // Arrange - Create trace file with multiple spans
    let trace_content = r#"
{
  "spans": [
    {
      "name": "test.span1",
      "span_id": "span-1",
      "trace_id": "trace-1",
      "kind": "internal"
    },
    {
      "name": "other.span",
      "span_id": "span-2",
      "trace_id": "trace-1",
      "kind": "internal"
    }
  ]
}
"#;
    let trace_file = helpers::create_temp_file(trace_content)?;
    let trace_path = helpers::temp_file_path(&trace_file);

    // Act - Filter spans with grep pattern
    let result = filter_spans(&trace_path, Some("test"), &OutputFormat::Human, false, false);

    // Assert - Should succeed
    assert!(
        result.is_ok(),
        "BEHAVIOR: Span filtering should succeed with grep pattern"
    );

    Ok(())
}

#[test]
fn test_spans_outputs_json_format() -> Result<()> {
    // Arrange - Create trace file
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
    let trace_file = helpers::create_temp_file(trace_content)?;
    let trace_path = helpers::temp_file_path(&trace_file);

    // Act - Output in JSON format
    let result = filter_spans(&trace_path, None, &OutputFormat::Json, false, false);

    // Assert - Should succeed
    assert!(
        result.is_ok(),
        "BEHAVIOR: JSON format output should succeed"
    );

    Ok(())
}

