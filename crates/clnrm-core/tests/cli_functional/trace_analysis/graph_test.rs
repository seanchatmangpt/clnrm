//! Graph command tests
//!
//! Tests verify trace visualization using AAA pattern.

use clnrm_core::cli::commands::graph::visualize_graph;
use clnrm_core::cli::types::GraphFormat;
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
fn test_graph_generates_ascii_visualization() -> Result<()> {
    // Arrange - Create trace file
    let trace_content = r#"
{
  "spans": [
    {
      "name": "test.span",
      "span_id": "span-1",
      "trace_id": "trace-1",
      "parent_span_id": null,
      "kind": "internal"
    }
  ]
}
"#;
    let trace_file = helpers::create_temp_file(trace_content)?;
    let trace_path = helpers::temp_file_path(&trace_file);

    // Act - Generate ASCII visualization
    // Note: This will print to stdout, but we can verify it doesn't error
    let result = visualize_graph(&trace_path, &GraphFormat::Ascii, false, None);

    // Assert - Should succeed
    assert!(
        result.is_ok(),
        "BEHAVIOR: Graph visualization should succeed for valid trace"
    );

    Ok(())
}

#[test]
fn test_graph_generates_dot_format() -> Result<()> {
    // Arrange - Create trace file
    let trace_content = r#"
{
  "spans": [
    {
      "name": "test.span",
      "span_id": "span-1",
      "trace_id": "trace-1",
      "parent_span_id": null,
      "kind": "internal"
    }
  ]
}
"#;
    let trace_file = helpers::create_temp_file(trace_content)?;
    let trace_path = helpers::temp_file_path(&trace_file);

    // Act - Generate DOT format
    let result = visualize_graph(&trace_path, &GraphFormat::Dot, false, None);

    // Assert - Should succeed
    assert!(
        result.is_ok(),
        "BEHAVIOR: DOT graph generation should succeed"
    );

    Ok(())
}

#[test]
fn test_graph_handles_invalid_trace_file() -> Result<()> {
    // Arrange - Create invalid trace file
    let invalid_content = "invalid json";
    let trace_file = helpers::create_temp_file(invalid_content)?;
    let trace_path = helpers::temp_file_path(&trace_file);

    // Act - Try to visualize
    let result = visualize_graph(&trace_path, &GraphFormat::Ascii, false, None);

    // Assert - Should fail gracefully
    assert!(
        result.is_err(),
        "BEHAVIOR: Graph should fail with proper error for invalid trace"
    );

    Ok(())
}

