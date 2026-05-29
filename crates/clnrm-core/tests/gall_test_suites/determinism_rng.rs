//! Gall Test Suite for Determinism Random Number Generation (RNG)
//!
//! Validates `create_seeded_rng` provides strictly deterministic values.

use clnrm_core::determinism::rng::create_seeded_rng;
use fake::faker::number::en::NumberWithFormat;
use rand::RngCore;

#[test]
fn gall_test_seeded_rng_determinism() {
    // Arrange
    let seed = 123456789;

    // Act
    let mut rng1 = create_seeded_rng(seed);
    let mut rng2 = create_seeded_rng(seed);

    // Assert
    // Verify a sequence of 10 numbers is identical
    for _ in 0..10 {
        assert_eq!(
            rng1.next_u64(),
            rng2.next_u64(),
            "Seeded RNG must produce identical sequences"
        );
    }
}

#[test]
fn gall_test_different_seeds_diverge() {
    // Arrange
    let seed1 = 1111;
    let seed2 = 2222;

    // Act
    let mut rng1 = create_seeded_rng(seed1);
    let mut rng2 = create_seeded_rng(seed2);

    // Assert
    // It's technically possible (though astronomically improbable) for the first number to collide.
    // However, if the first 3 numbers collide, determinism logic is broken.
    let col1 = rng1.next_u64() == rng2.next_u64();
    let col2 = rng1.next_u64() == rng2.next_u64();
    let col3 = rng1.next_u64() == rng2.next_u64();

    assert!(
        !(col1 && col2 && col3),
        "Different seeds should produce divergent sequences"
    );
}
