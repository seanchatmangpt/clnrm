//! SHA-256 digest for reproducibility
//!
//! Generates cryptographic hashes of span data to ensure reproducible test results.

use crate::error::{CleanroomError, Result};
use sha2::{Digest, Sha256};
use std::path::Path;

/// SHA-256 digest generator for reproducibility
pub struct DigestReporter;

impl DigestReporter {
    /// Write SHA-256 digest to file
    ///
    /// # Arguments
    /// * `path` - File path for digest output
    /// * `spans_json` - JSON string of spans to hash
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Errors
    /// Returns error if file write fails
    pub fn write(path: &Path, spans_json: &str) -> Result<()> {
        let digest = Self::compute_digest(spans_json);
        Self::write_file(path, &digest)
    }

    /// Compute SHA-256 digest of input string
    ///
    /// # Arguments
    /// * `spans_json` - JSON string to hash
    ///
    /// # Returns
    /// * Hexadecimal string representation of SHA-256 hash
    pub fn compute_digest(spans_json: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(spans_json.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Write digest to file with newline
    fn write_file(path: &Path, digest: &str) -> Result<()> {
        std::fs::write(path, format!("{}\n", digest))
            .map_err(|e| CleanroomError::report_error(format!("Failed to write digest: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_digest_reporter_basic() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new()
            .map_err(|e| CleanroomError::io_error(format!("Failed to create temp dir: {}", e)))?;
        let digest_path = temp_dir.path().join("digest.txt");
        let spans_json = r#"{"spans": []}"#;

        // Act
        DigestReporter::write(&digest_path, spans_json)?;

        // Assert
        let content = std::fs::read_to_string(&digest_path)
            .map_err(|e| CleanroomError::io_error(format!("Failed to read file: {}", e)))?;

        // SHA-256 produces 64 hex characters
        assert_eq!(content.trim().len(), 64);
        assert!(content.chars().all(|c| c.is_ascii_hexdigit() || c == '\n'));

        Ok(())
    }

    #[test]
    fn test_digest_reporter_deterministic() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new()
            .map_err(|e| CleanroomError::io_error(format!("Failed to create temp dir: {}", e)))?;
        let digest_path1 = temp_dir.path().join("digest1.txt");
        let digest_path2 = temp_dir.path().join("digest2.txt");
        let spans_json = r#"{"spans": [{"name": "test"}]}"#;

        // Act
        DigestReporter::write(&digest_path1, spans_json)?;
        DigestReporter::write(&digest_path2, spans_json)?;

        // Assert
        let content1 = std::fs::read_to_string(&digest_path1)
            .map_err(|e| CleanroomError::io_error(format!("Failed to read file: {}", e)))?;
        let content2 = std::fs::read_to_string(&digest_path2)
            .map_err(|e| CleanroomError::io_error(format!("Failed to read file: {}", e)))?;

        assert_eq!(content1, content2);

        Ok(())
    }

    #[test]
    fn test_digest_reporter_different_inputs() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new()
            .map_err(|e| CleanroomError::io_error(format!("Failed to create temp dir: {}", e)))?;
        let digest_path1 = temp_dir.path().join("digest1.txt");
        let digest_path2 = temp_dir.path().join("digest2.txt");
        let spans_json1 = r#"{"spans": []}"#;
        let spans_json2 = r#"{"spans": [{"name": "test"}]}"#;

        // Act
        DigestReporter::write(&digest_path1, spans_json1)?;
        DigestReporter::write(&digest_path2, spans_json2)?;

        // Assert
        let content1 = std::fs::read_to_string(&digest_path1)
            .map_err(|e| CleanroomError::io_error(format!("Failed to read file: {}", e)))?;
        let content2 = std::fs::read_to_string(&digest_path2)
            .map_err(|e| CleanroomError::io_error(format!("Failed to read file: {}", e)))?;

        assert_ne!(content1, content2);

        Ok(())
    }

    #[test]
    fn test_compute_digest_known_value() {
        // Arrange
        let input = "test";
        // Pre-computed SHA-256 hash of "test"
        let expected = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";

        // Act
        let digest = DigestReporter::compute_digest(input);

        // Assert
        assert_eq!(digest, expected);
    }

    #[test]
    fn test_compute_digest_empty_string() {
        // Arrange
        let input = "";
        // Pre-computed SHA-256 hash of empty string
        let expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

        // Act
        let digest = DigestReporter::compute_digest(input);

        // Assert
        assert_eq!(digest, expected);
    }

    #[test]
    fn test_compute_digest_complex_json() {
        // Arrange
        let input = r#"{"spans":[{"name":"root","span_id":"1"},{"name":"child","span_id":"2"}]}"#;

        // Act
        let digest = DigestReporter::compute_digest(input);

        // Assert
        assert_eq!(digest.len(), 64);
        assert!(digest.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_digest_sensitivity_to_whitespace() {
        // Arrange
        let input1 = r#"{"spans": []}"#;
        let input2 = r#"{"spans":[]}"#; // No space after colon

        // Act
        let digest1 = DigestReporter::compute_digest(input1);
        let digest2 = DigestReporter::compute_digest(input2);

        // Assert
        assert_ne!(digest1, digest2, "Digest should be sensitive to whitespace");
    }

    #[test]
    fn test_digest_file_format() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new()
            .map_err(|e| CleanroomError::io_error(format!("Failed to create temp dir: {}", e)))?;
        let digest_path = temp_dir.path().join("digest.txt");
        let spans_json = "test";

        // Act
        DigestReporter::write(&digest_path, spans_json)?;

        // Assert
        let content = std::fs::read_to_string(&digest_path)
            .map_err(|e| CleanroomError::io_error(format!("Failed to read file: {}", e)))?;

        // Should have exactly one newline at the end
        assert!(content.ends_with('\n'));
        assert_eq!(content.lines().count(), 1);

        Ok(())
    }
}
