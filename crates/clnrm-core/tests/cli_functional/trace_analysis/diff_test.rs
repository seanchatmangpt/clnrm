//! Diff command tests
//!
//! Tests verify trace comparison using AAA pattern.

use clnrm_core::cli::commands::diff::diff_traces;
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
fn test_diff_detects_differences_between_traces() -> Result<()> {
    // Arrange - Create two different trace files
    let baseline_content = r#"
{
  "spans": [
    {"name": "span1", "span_id": "1"}
  ]
}
"#;
    let current_content = r#"
{
  "spans": [
    {"name": "span1", "span_id": "1"},
    {"name": "span2", "span_id": "2"}
  ]
}
"#;

    let baseline_file = helpers::create_temp_file(baseline_content)?;
    let current_file = helpers::create_temp_file(current_content)?;
    let baseline_path = helpers::temp_file_path(&baseline_file);
    let current_path = helpers::temp_file_path(&current_file);

    // Act - Compare traces
    let result = diff_traces(&baseline_path, &current_path, "human", false);

    // Assert - Should detect differences
    assert!(
        result.is_ok(),
        "BEHAVIOR: Diff should succeed and detect differences"
    );

    let diff_result = result?;
    assert!(
        diff_result.added_count > 0,
        "BEHAVIOR: Should detect added spans"
    );

    Ok(())
}

