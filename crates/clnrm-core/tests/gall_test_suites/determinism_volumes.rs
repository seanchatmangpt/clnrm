//! Gall Test Suite for Determinism Volume Generation
//!
//! Validates pure, reproducible SHA-256 derivations for volume and container names.

use clnrm_core::determinism::volumes::{generate_container_name, generate_network_name, generate_volume_name};
use fake::faker::lorem::en::Word;
use fake::Fake;

#[test]
fn gall_test_deterministic_volume_naming() {
    // Arrange
    let test_name: String = Word().fake();
    let seed = 42;

    // Act
    let vol1 = generate_volume_name(&test_name, Some(seed));
    let vol2 = generate_volume_name(&test_name, Some(seed));
    
    // Assert
    assert_eq!(vol1, vol2, "Volume names must be strictly deterministic given the same seed");
    assert!(vol1.starts_with("clnrm-vol-"));
    assert_eq!(vol1.len(), "clnrm-vol-".len() + 12, "Should append exactly 12 chars of hash");
}

#[test]
fn gall_test_deterministic_container_naming_with_seed() {
    // Arrange
    let test_name: String = Word().fake();
    let step_name: String = Word().fake();
    let seed = 999;

    // Act
    let c1 = generate_container_name(&test_name, &step_name, Some(seed));
    let c2 = generate_container_name(&test_name, &step_name, Some(seed));

    // Assert
    assert_eq!(c1, c2, "Container names must be deterministic given the same seed");
    assert!(c1.contains(&test_name));
    assert!(c1.contains(&step_name));
}

#[test]
fn gall_test_deterministic_network_naming() {
    // Arrange
    let test_name: String = Word().fake();
    let seed = 12345;

    // Act
    let net1 = generate_network_name(&test_name, Some(seed));
    let net2 = generate_network_name(&test_name, Some(seed));

    // Assert
    assert_eq!(net1, net2, "Network names must be strictly deterministic given the same seed");
    assert!(net1.starts_with("clnrm-net-"));
}