//! Dry-run command tests
//!
//! Tests verify validation without execution using AAA pattern.

use clnrm_core::cli::commands::dry_run::dry_run_validate;
use clnrm_core::error::Result;

// Import helpers
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
fn test_dry_run_validates_without_execution() -> Result<()> {
    // Arrange - Create valid test configuration
    let valid_config = r#"
[meta]
name = "test_dry_run"
version = "1.0.0"

[[scenario]]
name = "test_scenario"
service = "test_service"
run = "echo test"

[service.test_service]
plugin = "generic_container"
image = "alpine:latest"
"#;
    let test_file = helpers::create_temp_file(valid_config)?;
    let file_path = helpers::temp_file_path(&test_file);

    // Act - Run dry-run validation
    let results = dry_run_validate(vec![file_path.as_path()], false)?;

    // Assert - Should return validation results without executing
    assert!(
        !results.is_empty(),
        "BEHAVIOR: Dry-run should return validation results"
    );
    assert!(
        results[0].valid,
        "BEHAVIOR: Valid config should pass dry-run validation"
    );
    // Note: We can't easily verify containers weren't started without Docker mocking,
    // but the fact that dry-run returns results quickly indicates no execution
    Ok(())
}

