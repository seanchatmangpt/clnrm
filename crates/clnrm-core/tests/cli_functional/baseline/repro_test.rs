//! Repro command tests
//!
//! Tests verify baseline reproduction using AAA pattern.

use clnrm_core::cli::commands::repro::reproduce_baseline;
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

#[tokio::test]
async fn test_repro_fails_with_invalid_baseline_file() -> Result<()> {
    // Arrange - Create invalid baseline file
    let invalid_baseline = "invalid json";
    let baseline_file = helpers::create_temp_file(invalid_baseline)?;
    let baseline_path = helpers::temp_file_path(&baseline_file);

    // Act - Try to reproduce
    let result = reproduce_baseline(&baseline_path, false, None).await;

    // Assert - Should fail gracefully
    assert!(
        result.is_err(),
        "BEHAVIOR: Should fail with proper error for invalid baseline file"
    );

    Ok(())
}

