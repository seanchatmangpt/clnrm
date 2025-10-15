//! Utility functions and helpers for the cleanroom framework
//!
//! Contains common utilities used throughout the framework.

use crate::error::Result;

/// Validate a file path exists and is readable
pub fn validate_file_path(path: &str) -> Result<()> {
    use std::path::Path;
    
    let path_obj = Path::new(path);
    
    // Check if path exists
    if !path_obj.exists() {
        return Err(crate::error::CleanroomError::validation_error(&format!(
            "Path does not exist: {}", path
        )));
    }
    
    // Check if path is readable (file) or accessible (directory)
    if path_obj.is_file() {
        // For files, check if we can read metadata
        std::fs::metadata(path_obj)?;
    } else if path_obj.is_dir() {
        // For directories, check if we can read the directory
        std::fs::read_dir(path_obj)?;
    } else {
        return Err(crate::error::CleanroomError::validation_error(&format!(
            "Path is neither a file nor directory: {}", path
        )));
    }
    
    Ok(())
}

/// Parse a TOML configuration file
pub fn parse_toml_config(content: &str) -> Result<serde_json::Value> {
    // Parse TOML content
    let toml_value: toml::Value = toml::from_str(content)
        .map_err(|e| crate::error::CleanroomError::validation_error(&format!(
            "Invalid TOML syntax: {}", e
        )))?;
    
    // Convert TOML to JSON for consistent handling
    let json_value = serde_json::to_value(toml_value)
        .map_err(|e| crate::error::CleanroomError::internal_error(&format!(
            "Failed to convert TOML to JSON: {}", e
        )))?;
    
    Ok(json_value)
}

/// Generate a unique session ID
pub fn generate_session_id() -> String {
    format!("session_{}", uuid::Uuid::new_v4())
}

/// Format duration for display
pub fn format_duration(duration: std::time::Duration) -> String {
    if duration.as_secs() > 0 {
        format!("{:.2}s", duration.as_secs_f64())
    } else if duration.as_millis() > 0 {
        format!("{}ms", duration.as_millis())
    } else {
        format!("{}μs", duration.as_micros())
    }
}

/// Validate regex pattern
pub fn validate_regex(pattern: &str) -> Result<()> {
    use regex::Regex;
    
    // Try to compile the regex pattern
    Regex::new(pattern)
        .map_err(|e| crate::error::CleanroomError::validation_error(&format!(
            "Invalid regex pattern '{}': {}", pattern, e
        )))?;
    
    Ok(())
}

/// Execute regex pattern matching
pub fn execute_regex_match(text: &str, pattern: &str) -> Result<bool> {
    use regex::Regex;
    
    // Compile and execute regex
    let regex = Regex::new(pattern)
        .map_err(|e| crate::error::CleanroomError::validation_error(&format!(
            "Invalid regex pattern '{}': {}", pattern, e
        )))?;
    
    Ok(regex.is_match(text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_file_path() {
        // Test with existing file (current directory)
        let result = validate_file_path(".");
        assert!(result.is_ok());
        
        // Test with nonexistent file
        let result = validate_file_path("nonexistent.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_toml_config() {
        // Test with valid TOML
        let result = parse_toml_config("[test]\nname = \"example\"");
        assert!(result.is_ok());
        
        // Test with invalid TOML
        let result = parse_toml_config("invalid toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_session_id() {
        let id = generate_session_id();
        assert!(id.starts_with("session_"));
        assert!(id.len() > 8); // Should contain UUID
    }

    #[test]
    fn test_format_duration() {
        let duration = std::time::Duration::from_millis(1500);
        let formatted = format_duration(duration);
        assert!(formatted.contains("1.50s"));

        let duration = std::time::Duration::from_millis(500);
        let formatted = format_duration(duration);
        assert!(formatted.contains("500ms"));
    }
}
