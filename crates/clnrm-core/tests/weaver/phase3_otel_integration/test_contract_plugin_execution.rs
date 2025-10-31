//! Test plugin_system schema contract compliance

#[cfg(test)]
mod plugin_execution_contract_tests {
    use crate::weaver::fixtures::*;

    #[test]
    fn test_plugin_execution_healthy_state() {
        // ARRANGE
        let contract = ContractFixtures::valid_plugin_execution();

        // ASSERT
        assert_eq!(contract.plugin_state, PluginState::Healthy);
        assert!(contract.plugin_health_check_performed);
        assert!(contract.plugin_health_check_passed);
    }

    #[test]
    fn test_plugin_execution_unhealthy_includes_error_details() {
        // ARRANGE
        let contract = ContractFixtures::plugin_execution_unhealthy();

        // ASSERT
        assert_eq!(contract.plugin_state, PluginState::Error);
        assert!(!contract.plugin_health_check_passed);
        assert!(contract.error_type.is_some());
        assert!(contract.error_message.is_some());
    }

    // TODO: Add tests for plugin state transitions
}
