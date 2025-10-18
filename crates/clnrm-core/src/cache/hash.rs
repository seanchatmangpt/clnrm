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
