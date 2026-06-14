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

    #[test]
    fn test_plugin_state_transitions_registered_to_healthy() {
        // A plugin should complete startup and reach the Healthy state
        let contract = ContractFixtures::valid_plugin_execution();

        // ASSERT - Final state is Healthy
        assert_eq!(contract.plugin_state, PluginState::Healthy);
        assert!(contract.plugin_health_check_performed);
        assert!(contract.plugin_health_check_passed);
    }

    #[test]
    fn test_plugin_state_transitions_starting_to_error() {
        // A plugin can fail during startup, transitioning to Error
        let contract = ContractFixtures::plugin_execution_unhealthy();

        // ASSERT - Error state has required error details
        assert_eq!(contract.plugin_state, PluginState::Error);
        assert!(contract.error_type.is_some(), "Error state must include error_type");
        assert!(contract.error_message.is_some(), "Error state must include error_message");
        assert!(!contract.plugin_health_check_passed, "Unhealthy plugin must fail health check");
    }

    #[test]
    fn test_plugin_health_check_duration_recorded_on_success() {
        // Successful plugin must record health check duration
        let contract = ContractFixtures::valid_plugin_execution();

        // ASSERT - Duration is recorded when health check performed and passed
        assert!(
            contract.plugin_health_check_duration_ms.is_some(),
            "plugin_health_check_duration_ms must be recorded on successful health check"
        );
    }
}
