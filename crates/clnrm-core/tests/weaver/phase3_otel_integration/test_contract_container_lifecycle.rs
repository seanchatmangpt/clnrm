//! Test container_lifecycle schema contract compliance

#[cfg(test)]
mod container_lifecycle_contract_tests {
    use crate::weaver::fixtures::*;

    #[test]
    fn test_container_lifecycle_requires_destroyed_timestamp() {
        // ARRANGE - Valid contract
        let contract = ContractFixtures::valid_container_lifecycle();

        // ASSERT
        assert_eq!(contract.container_state, ContainerState::Destroyed);
        assert!(!contract.container_destroyed_at.is_empty());
        assert!(contract.cleanup_success);
    }

    #[test]
    fn test_container_lifecycle_detects_resource_leak() {
        // ARRANGE - INVALID contract (leaked container)
        let contract = ContractFixtures::container_lifecycle_leaked();

        // ASSERT
        assert!(contract.container_destroyed_at.is_empty());  // Missing!
        assert!(!contract.cleanup_success);
        assert!(contract.cleanup_orphaned_resources > 0);
    }

    // TODO: Add tests verifying lifecycle state transitions
}
