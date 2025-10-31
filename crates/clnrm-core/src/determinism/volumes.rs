//! Deterministic volume naming for reproducible tests
//!
//! Provides hash-based volume naming to ensure consistent volume names
//! across multiple test runs.

use sha2::{Digest, Sha256};

/// Generate deterministic volume name from test name and optional seed
///
/// Creates a hash-based volume name that will be identical across runs
/// with the same test name and seed.
///
/// # Arguments
/// * `test_name` - Name of the test
/// * `seed` - Optional seed for additional uniqueness
///
/// # Returns
/// * String - Deterministic volume name in format "clnrm-vol-{hash}"
///
/// # Examples
/// ```
/// use clnrm_core::determinism::volumes::generate_volume_name;
///
/// let vol_name = generate_volume_name("my_test", Some(42));
/// assert!(vol_name.starts_with("clnrm-vol-"));
///
/// // Same inputs always produce same name
/// let vol_name2 = generate_volume_name("my_test", Some(42));
/// assert_eq!(vol_name, vol_name2);
/// ```
pub fn generate_volume_name(test_name: &str, seed: Option<u64>) -> String {
    let mut hasher = Sha256::new();

    // Hash test name
    hasher.update(test_name.as_bytes());

    // Include seed if present
    if let Some(s) = seed {
        hasher.update(s.to_le_bytes());
    }

    let result = hasher.finalize();

    // Use first 12 chars of hex digest for readability
    let hash_prefix = format!("{:x}", result);
    format!("clnrm-vol-{}", &hash_prefix[..12])
}

/// Generate deterministic container name from test name, step name, and optional seed
///
/// Creates a hash-based container name that will be identical across runs.
///
/// # Arguments
/// * `test_name` - Name of the test
/// * `step_name` - Name of the test step
/// * `seed` - Optional seed for additional uniqueness
///
/// # Returns
/// * String - Deterministic container name
///
/// # Examples
/// ```
/// use clnrm_core::determinism::volumes::generate_container_name;
///
/// let name = generate_container_name("my_test", "setup", Some(42));
/// assert!(name.contains("my_test"));
/// assert!(name.contains("setup"));
/// ```
pub fn generate_container_name(test_name: &str, step_name: &str, seed: Option<u64>) -> String {
    let base_name = format!("{}-step-{}", test_name, step_name);

    if seed.is_some() {
        // With seed, add hash suffix for additional determinism
        let mut hasher = Sha256::new();
        hasher.update(base_name.as_bytes());
        if let Some(s) = seed {
            hasher.update(s.to_le_bytes());
        }

        let result = hasher.finalize();
        let hash_suffix = format!("{:x}", result);

        format!("{}-{}", base_name, &hash_suffix[..8])
    } else {
        // Without seed, use simple name (current behavior)
        base_name
    }
}

/// Generate deterministic network name from test name and optional seed
///
/// # Arguments
/// * `test_name` - Name of the test
/// * `seed` - Optional seed for additional uniqueness
///
/// # Returns
/// * String - Deterministic network name in format "clnrm-net-{hash}"
pub fn generate_network_name(test_name: &str, seed: Option<u64>) -> String {
    let mut hasher = Sha256::new();

    hasher.update(test_name.as_bytes());

    if let Some(s) = seed {
        hasher.update(s.to_le_bytes());
    }

    let result = hasher.finalize();
    let hash_prefix = format!("{:x}", result);

    format!("clnrm-net-{}", &hash_prefix[..12])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_name_deterministic() {
        let name1 = generate_volume_name("test1", Some(42));
        let name2 = generate_volume_name("test1", Some(42));
        assert_eq!(name1, name2);
    }

    #[test]
    fn test_volume_name_different_with_different_seed() {
        let name1 = generate_volume_name("test1", Some(42));
        let name2 = generate_volume_name("test1", Some(43));
        assert_ne!(name1, name2);
    }

    #[test]
    fn test_volume_name_different_with_different_test() {
        let name1 = generate_volume_name("test1", Some(42));
        let name2 = generate_volume_name("test2", Some(42));
        assert_ne!(name1, name2);
    }

    #[test]
    fn test_container_name_format() {
        let name = generate_container_name("my_test", "step1", Some(42));
        assert!(name.contains("my_test"));
        assert!(name.contains("step1"));
    }

    #[test]
    fn test_container_name_deterministic() {
        let name1 = generate_container_name("test", "step", Some(42));
        let name2 = generate_container_name("test", "step", Some(42));
        assert_eq!(name1, name2);
    }

    #[test]
    fn test_network_name_format() {
        let name = generate_network_name("test1", Some(42));
        assert!(name.starts_with("clnrm-net-"));
    }
}
