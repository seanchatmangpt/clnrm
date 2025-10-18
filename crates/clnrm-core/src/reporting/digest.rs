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
