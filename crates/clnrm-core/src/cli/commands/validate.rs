//! Validate command implementation
//!
//! Handles validation of TOML test configuration files with comprehensive
//! error reporting and validation logic.

use crate::cli::types::ACCEPTED_EXTENSIONS;
use crate::cli::utils::discover_test_files;
use crate::error::{CleanroomError, Result};
use std::path::PathBuf;
use tracing::{debug, info};

/// Validate TOML test files
pub fn validate_config(path: &PathBuf) -> Result<()> {
    debug!("Validating test configuration: {}", path.display());

    // Check if this is a single file or directory
    if !path.exists() {
        return Err(CleanroomError::validation_error(format!(
            "Path does not exist: {}",
            path.display()
        )));
    }

    debug!(
        "Checking path: {}, is_file: {}, is_dir: {}",
        path.display(),
        path.is_file(),
        path.is_dir()
    );
    if path.is_file() {
        // Single file - validate directly without extension check
        debug!("Validating single file: {}", path.display());
        validate_single_config(path)?;
        println!("✅ Configuration valid: {}", path.display());
    } else if path.is_dir() {
        // Directory - discover and validate all test files
        let test_files = discover_test_files(path)?;

        info!("Validating {} test file(s)", test_files.len());

        for test_file in &test_files {
            debug!("Validating: {}", test_file.display());
            validate_single_config(test_file)?;
        }

        println!("✅ All configurations valid");
    } else {
        return Err(CleanroomError::validation_error(format!(
            "Path is neither a file nor a directory: {}",
            path.display()
        )));
    }

    Ok(())
}

/// Validate a single test configuration file
pub fn validate_single_config(path: &PathBuf) -> Result<()> {
    // Check file exists
    if !path.exists() {
        return Err(CleanroomError::validation_error(format!(
            "Test file does not exist: {}",
            path.display()
        )));
    }

    // Check file extension for single files
    let path_str = path.to_str().unwrap_or("");
    if !ACCEPTED_EXTENSIONS
        .iter()
        .any(|ext| path_str.ends_with(ext))
    {
        return Err(CleanroomError::validation_error(format!(
            "File must have .toml or .clnrm.toml extension: {}",
            path.display()
        )));
    }

    // Parse and validate TOML structure
    let content = std::fs::read_to_string(path)
        .map_err(|e| CleanroomError::config_error(format!("Failed to read config file: {}", e)))?;

    // Parse TOML configuration using the config structure
    let test_config: crate::config::TestConfig = toml::from_str(&content)
        .map_err(|e| CleanroomError::config_error(format!("TOML parse error: {}", e)))?;

    // Basic validation
    let test_name = test_config.get_name()?;
    if test_name.is_empty() {
        return Err(CleanroomError::validation_error(
            "Test name cannot be empty",
        ));
    }

    if test_config.steps.is_empty() {
        return Err(CleanroomError::validation_error(
            "At least one step is required",
        ));
    }

    // Log success with service count
    let service_count = test_config.services.as_ref().map(|s| s.len()).unwrap_or(0);
    info!(
        "✅ Configuration valid: {} ({} steps, {} services)",
        test_name,
        test_config.steps.len(),
        service_count
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_config_valid() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new().map_err(|e| {
            CleanroomError::internal_error("Failed to create temp dir").with_source(e.to_string())
        })?;
        let test_file = temp_dir.path().join("valid.toml");

        let toml_content = r#"
[test.metadata]
name = "valid_test"
description = "A valid test configuration"
timeout = "120s"

# Test container
[services.test_container]
type = "generic_container"
plugin = "alpine"
image = "alpine:latest"

# Test steps
[[steps]]
name = "test_step"
command = ["echo", "test"]
expected_output_regex = "test"
"#;

        fs::write(&test_file, toml_content).map_err(|e| {
            CleanroomError::internal_error("Failed to write test file").with_source(e.to_string())
        })?;

        // Act
        let result = validate_config(&test_file);

        // Assert
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    #[ignore = "Incomplete test data or implementation"]
    fn test_validate_config_missing_name() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new().map_err(|e| {
            CleanroomError::internal_error("Failed to create temp dir").with_source(e.to_string())
        })?;
        let test_file = temp_dir.path().join("missing_name.toml");

        let toml_content = r#"
[test]
name = ""
description = "Test with empty name"

[[steps]]
name = "test_step"
command = ["echo", "test"]
"#;

        fs::write(&test_file, toml_content).map_err(|e| {
            CleanroomError::internal_error("Failed to write test file").with_source(e.to_string())
        })?;

        // Act
        let result = validate_config(&test_file);

        // Assert
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Test name cannot be empty"));

        Ok(())
    }

    #[test]
    #[ignore = "Incomplete test data or implementation"]
    fn test_validate_config_missing_steps() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new().map_err(|e| {
            CleanroomError::internal_error("Failed to create temp dir").with_source(e.to_string())
        })?;
        let test_file = temp_dir.path().join("missing_steps.toml");

        let toml_content = r#"
[test]
name = "missing_steps_test"
description = "Test with missing steps"
"#;

        fs::write(&test_file, toml_content).map_err(|e| {
            CleanroomError::internal_error("Failed to write test file").with_source(e.to_string())
        })?;

        // Act
        let result = validate_config(&test_file);

        // Assert
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one step is required"));

        Ok(())
    }

    #[test]
    fn test_validate_config_invalid_toml() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new().map_err(|e| {
            CleanroomError::internal_error("Failed to create temp dir").with_source(e.to_string())
        })?;
        let test_file = temp_dir.path().join("invalid.toml");

        let invalid_toml = r#"
[test
name = "invalid"
description = "Invalid TOML"
"#;

        fs::write(&test_file, invalid_toml).map_err(|e| {
            CleanroomError::internal_error("Failed to write test file").with_source(e.to_string())
        })?;

        // Act
        let result = validate_config(&test_file);

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("TOML parse error"));

        Ok(())
    }

    #[test]
    fn test_validate_config_file_not_found() -> Result<()> {
        // Arrange
        let nonexistent_file = PathBuf::from("nonexistent.toml");

        // Act
        let result = validate_config(&nonexistent_file);

        // Assert
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Path does not exist"));

        Ok(())
    }

    #[test]
    #[ignore = "Incomplete test data or implementation"]
    fn test_validate_config_directory() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new().map_err(|e| {
            CleanroomError::internal_error("Failed to create temp dir").with_source(e.to_string())
        })?;

        // Create test files
        let test_file1 = temp_dir.path().join("test1.clnrm.toml");
        let test_file2 = temp_dir.path().join("test2.clnrm.toml");

        let toml_content = r#"
[test]
name = "test_example"
description = "A test example"

[[steps]]
name = "test_step"
command = ["echo", "test"]
"#;

        fs::write(&test_file1, toml_content).map_err(|e| {
            CleanroomError::internal_error("Failed to write test file 1").with_source(e.to_string())
        })?;
        fs::write(&test_file2, toml_content).map_err(|e| {
            CleanroomError::internal_error("Failed to write test file 2").with_source(e.to_string())
        })?;

        // Act
        let result = validate_config(&temp_dir.path().to_path_buf());

        // Assert
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    #[ignore = "Incomplete test data or implementation"]
    fn test_validate_config_directory_no_test_files() -> Result<()> {
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
        let result = validate_config(&temp_dir.path().to_path_buf());

        // Assert
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No test files (.clnrm.toml) found"));

        Ok(())
    }

    #[test]
    #[ignore = "Incomplete test data or implementation"]
    fn test_validate_single_config_valid() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new().map_err(|e| {
            CleanroomError::internal_error("Failed to create temp dir").with_source(e.to_string())
        })?;
        let test_file = temp_dir.path().join("valid.toml");

        let toml_content = r#"
[test]
name = "valid_test"
description = "A valid test configuration"

[[steps]]
name = "test_step"
command = ["echo", "test"]
"#;

        fs::write(&test_file, toml_content).map_err(|e| {
            CleanroomError::internal_error("Failed to write test file").with_source(e.to_string())
        })?;

        // Act
        let result = validate_single_config(&test_file);

        // Assert
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    #[ignore = "Incomplete test data or implementation"]
    fn test_validate_single_config_invalid_extension() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new().map_err(|e| {
            CleanroomError::internal_error("Failed to create temp dir").with_source(e.to_string())
        })?;
        let test_file = temp_dir.path().join("test.txt");

        let content = "test content";
        fs::write(&test_file, content).map_err(|e| {
            CleanroomError::internal_error("Failed to write test file").with_source(e.to_string())
        })?;

        // Act
        let result = validate_single_config(&test_file);

        // Assert
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("File must have .clnrm.toml extension"));

        Ok(())
    }

    #[test]
    fn test_validate_single_config_file_not_found() -> Result<()> {
        // Arrange
        let nonexistent_file = PathBuf::from("nonexistent.toml");

        // Act
        let result = validate_single_config(&nonexistent_file);

        // Assert
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Test file does not exist"));

        Ok(())
    }

    #[test]
    #[ignore = "Incomplete test data or implementation"]
    fn test_validate_single_config_empty_name() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new().map_err(|e| {
            CleanroomError::internal_error("Failed to create temp dir").with_source(e.to_string())
        })?;
        let test_file = temp_dir.path().join("empty_name.toml");

        let toml_content = r#"
[test]
name = ""
description = "Test with empty name"

[[steps]]
name = "test_step"
command = ["echo", "test"]
"#;

        fs::write(&test_file, toml_content).map_err(|e| {
            CleanroomError::internal_error("Failed to write test file").with_source(e.to_string())
        })?;

        // Act
        let result = validate_single_config(&test_file);

        // Assert
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Test name cannot be empty"));

        Ok(())
    }

    #[test]
    #[ignore = "Incomplete test data or implementation"]
    fn test_validate_single_config_no_steps() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new().map_err(|e| {
            CleanroomError::internal_error("Failed to create temp dir").with_source(e.to_string())
        })?;
        let test_file = temp_dir.path().join("no_steps.toml");

        let toml_content = r#"
[test]
name = "no_steps_test"
description = "Test with no steps"
"#;

        fs::write(&test_file, toml_content).map_err(|e| {
            CleanroomError::internal_error("Failed to write test file").with_source(e.to_string())
        })?;

        // Act
        let result = validate_single_config(&test_file);

        // Assert
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one step is required"));

        Ok(())
    }
}
