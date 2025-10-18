//! Digest generation for trace verification
//!
//! Provides SHA-256 digest generation for trace verification.

use sha2::{Digest, Sha256};

/// Generate SHA-256 digest from byte data
///
/// # Arguments
/// * `data` - Input data to hash
///
/// # Returns
/// * Hex-encoded SHA-256 digest string
pub fn generate_digest(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Verify that data matches expected digest
///
/// # Arguments
/// * `data` - Data to verify
/// * `expected_digest` - Expected hex-encoded SHA-256 digest
///
/// # Returns
/// * true if digest matches, false otherwise
pub fn verify_digest(data: &[u8], expected_digest: &str) -> bool {
    let actual_digest = generate_digest(data);
    actual_digest == expected_digest
}
