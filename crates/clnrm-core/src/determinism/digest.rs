//! Digest generation for trace verification
//!
//! Provides SHA-256 digest generation for trace verification.

use sha2::{Sha256, Digest};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_digest_deterministic() {
        let data = b"test data";
        let digest1 = generate_digest(data);
        let digest2 = generate_digest(data);
        assert_eq!(digest1, digest2);
    }

    #[test]
    fn test_verify_digest_valid() {
        let data = b"test data";
        let digest = generate_digest(data);
        let valid = verify_digest(data, &digest);
        assert!(valid);
    }
}
