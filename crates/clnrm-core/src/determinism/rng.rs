//! Seeded random number generation for deterministic tests
//!
//! Provides deterministic RNG using StdRng with fixed seeds.

use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};

/// Create a seeded RNG for deterministic random number generation
///
/// Uses StdRng which is a cryptographically secure RNG that can be seeded
/// for deterministic behavior.
///
/// # Arguments
/// * `seed` - 64-bit seed value
///
/// # Returns
/// * Boxed RngCore trait object that can be used across threads
pub fn create_seeded_rng(seed: u64) -> Box<dyn RngCore + Send> {
    // Create a 32-byte seed from the u64 seed
    // We use a simple scheme: repeat the u64 4 times to fill 32 bytes with variation
    let mut full_seed = [0u8; 32];
    for (i, chunk) in full_seed.chunks_mut(8).enumerate() {
        // XOR with index to provide variation across the seed
        let modified_seed = seed.wrapping_add(i as u64);
        chunk.copy_from_slice(&modified_seed.to_le_bytes());
    }

    Box::new(StdRng::from_seed(full_seed))
}
