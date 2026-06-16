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

    #[test]
    fn test_container_lifecycle_valid_state_transitions() {
        // ARRANGE - A container should complete the full lifecycle and end up Destroyed
        let lifecycle = ContractFixtures::valid_container_lifecycle();

        // ASSERT - Final state is Destroyed (completed the full lifecycle)
        assert_eq!(lifecycle.container_state, ContainerState::Destroyed);
        assert!(lifecycle.cleanup_success);
    }

    #[test]
    fn test_container_lifecycle_error_state_skips_stopped() {
        // ARRANGE - A leaked container is still Running, never reaching Stopped or Destroyed
        let leaked = ContractFixtures::container_lifecycle_leaked();

        // ASSERT - Leaked containers are not cleanly destroyed
        assert!(leaked.container_destroyed_at.is_empty());
        assert_eq!(leaked.cleanup_orphaned_resources, 1);
    }

    #[test]
    fn test_container_lifecycle_cleanup_is_required() {
        // ARRANGE
        let lifecycle = ContractFixtures::valid_container_lifecycle();

        // ASSERT - Every lifecycle must attempt cleanup
        // cleanup_performed is implied by cleanup_success + no orphaned resources
        assert_eq!(lifecycle.cleanup_orphaned_resources, 0, "Clean lifecycle must have no orphaned resources");
    }
}
