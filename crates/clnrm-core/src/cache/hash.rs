//! SHA-256 file hashing for cache invalidation
//!
//! Provides content-based hashing for detecting file changes.
//! Uses SHA-256 for cryptographic strength and collision resistance.

use crate::error::{CleanroomError, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use tracing::debug;

/// Hash content using SHA-256 and return hex string
///
/// Core Team Compliance:
/// - Proper error handling with Result<String, CleanroomError>
/// - No unwrap() or expect() calls
/// - Efficient for small-to-medium sized files
///
/// # Arguments
/// * `content` - String content to hash (typically rendered TOML)
///
/// # Returns
/// Hex-encoded SHA-256 hash (64 characters)
///
/// # Performance
/// - Hash calculation: <50ms per file for typical TOML files
/// - Memory efficient: processes content in-place
pub fn hash_content(content: &str) -> Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    let hex = format!("{:x}", result);

    debug!("Hashed content ({} bytes) -> {}", content.len(), &hex[..16]);
    Ok(hex)
}

/// Hash a file's content using SHA-256
///
/// Reads file from disk and computes hash.
/// For cache management, prefer `hash_content` with rendered content.
///
/// # Arguments
/// * `path` - Path to file to hash
///
/// # Returns
/// Hex-encoded SHA-256 hash (64 characters)
///
/// # Errors
/// - File read errors
/// - Invalid file path
pub fn hash_file(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path).map_err(|e| {
        CleanroomError::io_error(format!(
            "Failed to read file '{}' for hashing: {}",
            path.display(),
            e
        ))
    })?;

    hash_content(&content)
}

/// Compute hash from multiple content parts (for composite hashing)
///
/// Useful for hashing configuration that depends on multiple files
/// or sections. Parts are concatenated before hashing.
///
/// # Arguments
/// * `parts` - Content parts to hash together
///
/// # Returns
/// Hex-encoded SHA-256 hash of combined content
pub fn hash_parts(parts: &[&str]) -> Result<String> {
    let combined = parts.join("");
    hash_content(&combined)
}

/// Verify if content matches expected hash
///
/// # Arguments
/// * `content` - Content to verify
/// * `expected_hash` - Expected hex-encoded SHA-256 hash
///
/// # Returns
/// true if hashes match, false otherwise
pub fn verify_hash(content: &str, expected_hash: &str) -> Result<bool> {
    let actual_hash = hash_content(content)?;
    Ok(actual_hash == expected_hash)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic
    )]

    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hash_content_basic() -> Result<()> {
        // Arrange
        let content = "test content";

        // Act
        let hash = hash_content(content)?;

        // Assert
        assert_eq!(hash.len(), 64); // SHA-256 produces 64 hex characters
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

        Ok(())
    }

    #[test]
    fn test_hash_content_deterministic() -> Result<()> {
        // Arrange
        let content = "deterministic content";

        // Act
        let hash1 = hash_content(content)?;
        let hash2 = hash_content(content)?;

        // Assert
        assert_eq!(hash1, hash2, "Same content should produce same hash");

        Ok(())
    }

    #[test]
    fn test_hash_content_different_for_different_content() -> Result<()> {
        // Arrange
        let content1 = "content 1";
        let content2 = "content 2";

        // Act
        let hash1 = hash_content(content1)?;
        let hash2 = hash_content(content2)?;

        // Assert
        assert_ne!(
            hash1, hash2,
            "Different content should produce different hashes"
        );

        Ok(())
    }

    #[test]
    fn test_hash_content_empty_string() -> Result<()> {
        // Arrange
        let content = "";

        // Act
        let hash = hash_content(content)?;

        // Assert
        assert_eq!(hash.len(), 64);
        // Empty string has known SHA-256 hash
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );

        Ok(())
    }

    #[test]
    fn test_hash_file_basic() -> Result<()> {
        // Arrange
        let mut temp_file = NamedTempFile::new().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to create temp file: {}", e))
        })?;
        writeln!(temp_file, "test file content").map_err(|e| {
            CleanroomError::io_error(format!("Failed to write temp file: {}", e))
        })?;

        // Act
        let hash = hash_file(temp_file.path())?;

        // Assert
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

        Ok(())
    }

    #[test]
    fn test_hash_parts_matches_combined() -> Result<()> {
        // Arrange
        let parts = vec!["hello", " ", "world"];
        let combined = "hello world";

        // Act
        let parts_hash = hash_parts(&parts)?;
        let combined_hash = hash_content(combined)?;

        // Assert
        assert_eq!(parts_hash, combined_hash, "Parts hash should match combined hash");

        Ok(())
    }

    #[test]
    fn test_verify_hash_valid() -> Result<()> {
        // Arrange
        let content = "verify this content";
        let expected_hash = hash_content(content)?;

        // Act
        let is_valid = verify_hash(content, &expected_hash)?;

        // Assert
        assert!(is_valid, "Hash verification should succeed for matching content");

        Ok(())
    }

    #[test]
    fn test_verify_hash_invalid() -> Result<()> {
        // Arrange
        let content = "verify this content";
        let wrong_hash = hash_content("different content")?;

        // Act
        let is_valid = verify_hash(content, &wrong_hash)?;

        // Assert
        assert!(
            !is_valid,
            "Hash verification should fail for non-matching content"
        );

        Ok(())
    }
}
