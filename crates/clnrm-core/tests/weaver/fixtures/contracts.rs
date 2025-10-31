//! Contract Builders from OTel Schemas
//!
//! These builders generate test data that conforms to (or intentionally violates)
//! the semantic convention schemas in registry/*.yaml

/// Test execution contract from test_execution.yaml
#[derive(Debug, Clone)]
pub struct TestExecutionContract {
    pub test_name: String,
    pub test_suite: String,
    pub test_isolated: bool,
    pub test_result: TestResult,
    pub test_duration_ms: f64,
    pub test_start_timestamp: i64,
    pub test_end_timestamp: i64,
    pub container_id: String,
    pub container_image_name: String,
    pub container_image_tag: Option<String>,
    pub container_exit_code: i32,
    pub test_assertion_count: Option<i32>,
    pub test_cleanup_performed: bool,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestResult {
    Pass,
    Fail,
    Error,
}

/// Container lifecycle contract from container_lifecycle.yaml
#[derive(Debug, Clone)]
pub struct ContainerLifecycleContract {
    pub container_id: String,
    pub container_image: String,
    pub container_name: Option<String>,
    pub container_state: ContainerState,
    pub container_created_at: String,
    pub container_started_at: String,
    pub container_destroyed_at: String,
    pub container_exit_code: Option<i32>,
    pub container_runtime: Option<String>,
    pub container_backend: String,
    pub cleanup_success: bool,
    pub cleanup_orphaned_resources: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContainerState {
    Creating,
    Running,
    Stopped,
    Error,
    Destroyed,
}

/// Plugin execution contract from plugin_system.yaml
#[derive(Debug, Clone)]
pub struct PluginExecutionContract {
    pub plugin_name: String,
    pub plugin_type: String,
    pub plugin_version: Option<String>,
    pub plugin_state: PluginState,
    pub service_name: String,
    pub service_type: String,
    pub container_id: String,
    pub plugin_health_check_performed: bool,
    pub plugin_health_check_passed: bool,
    pub plugin_health_check_duration_ms: Option<f64>,
    pub plugin_startup_duration_ms: Option<f64>,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PluginState {
    Registered,
    Starting,
    Running,
    Healthy,
    Stopping,
    Stopped,
    Error,
}

/// Test event contracts from test_events.yaml
#[derive(Debug, Clone)]
pub struct TestStartedEvent {
    pub test_name: String,
    pub test_suite: String,
    pub container_id: String,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct TestCompletedEvent {
    pub test_name: String,
    pub test_suite: String,
    pub container_id: String,
    pub test_result: String,
    pub test_assertions_passed: Option<i32>,
    pub test_assertions_failed: Option<i32>,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct TestFailedEvent {
    pub test_name: String,
    pub test_suite: String,
    pub container_id: Option<String>,
    pub error_type: String,
    pub error_message: String,
    pub error_stack_trace: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct ContainerLeakedEvent {
    pub container_id: String,
    pub container_image: String,
    pub test_name: String,
    pub container_age_seconds: i32,
    pub timestamp: String,
}

/// Builder for contract fixtures
pub struct ContractFixtures;

impl ContractFixtures {
    /// Generate valid test execution contract (all REQUIRED attributes present)
    pub fn valid_test_execution() -> TestExecutionContract {
        TestExecutionContract {
            test_name: "test_container_creation".to_string(),
            test_suite: "integration_tests".to_string(),
            test_isolated: true,  // Schema REQUIRES true
            test_result: TestResult::Pass,
            test_duration_ms: 125.5,  // Schema REQUIRES > 0
            test_start_timestamp: 1730250000000,
            test_end_timestamp: 1730250125500,
            container_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            container_image_name: "alpine:latest".to_string(),
            container_image_tag: Some("latest".to_string()),
            container_exit_code: 0,
            test_assertion_count: Some(5),
            test_cleanup_performed: true,  // Schema REQUIRES true
            error_type: None,
            error_message: None,
        }
    }

    /// Generate INVALID contract (missing container_id - REQUIRED attribute)
    pub fn invalid_test_execution_missing_container_id() -> TestExecutionContract {
        let mut contract = Self::valid_test_execution();
        contract.container_id = "".to_string();  // VIOLATES schema
        contract
    }

    /// Generate INVALID contract (test.isolated = false)
    pub fn invalid_test_execution_not_isolated() -> TestExecutionContract {
        let mut contract = Self::valid_test_execution();
        contract.test_isolated = false;  // VIOLATES schema requirement
        contract
    }

    /// Generate INVALID contract (zero duration)
    pub fn invalid_test_execution_zero_duration() -> TestExecutionContract {
        let mut contract = Self::valid_test_execution();
        contract.test_duration_ms = 0.0;  // VIOLATES schema (must be > 0)
        contract
    }

    /// Generate INVALID contract (cleanup not performed)
    pub fn invalid_test_execution_no_cleanup() -> TestExecutionContract {
        let mut contract = Self::valid_test_execution();
        contract.test_cleanup_performed = false;  // VIOLATES schema
        contract
    }

    /// Generate valid container lifecycle contract
    pub fn valid_container_lifecycle() -> ContainerLifecycleContract {
        ContainerLifecycleContract {
            container_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            container_image: "docker.io/library/alpine:latest".to_string(),
            container_name: Some("test_container_1".to_string()),
            container_state: ContainerState::Destroyed,  // Final state MUST be destroyed
            container_created_at: "2025-10-30T14:23:45.123Z".to_string(),
            container_started_at: "2025-10-30T14:23:46.456Z".to_string(),
            container_destroyed_at: "2025-10-30T14:25:12.789Z".to_string(),
            container_exit_code: Some(0),
            container_runtime: Some("docker".to_string()),
            container_backend: "testcontainers".to_string(),
            cleanup_success: true,  // Schema REQUIRES true
            cleanup_orphaned_resources: 0,  // Schema REQUIRES 0
        }
    }

    /// Generate INVALID contract (resource leak - missing destroyed_at)
    pub fn container_lifecycle_leaked() -> ContainerLifecycleContract {
        ContainerLifecycleContract {
            container_id: "leaked-container-123".to_string(),
            container_image: "postgres:15".to_string(),
            container_name: Some("leaked_postgres".to_string()),
            container_state: ContainerState::Running,  // Still running!
            container_created_at: "2025-10-30T14:23:45.123Z".to_string(),
            container_started_at: "2025-10-30T14:23:46.456Z".to_string(),
            container_destroyed_at: "".to_string(),  // MISSING - VIOLATES schema
            container_exit_code: None,
            container_runtime: Some("docker".to_string()),
            container_backend: "testcontainers".to_string(),
            cleanup_success: false,  // VIOLATES schema
            cleanup_orphaned_resources: 1,  // Should be 0
        }
    }

    /// Generate valid plugin execution contract
    pub fn valid_plugin_execution() -> PluginExecutionContract {
        PluginExecutionContract {
            plugin_name: "postgres".to_string(),
            plugin_type: "database".to_string(),
            plugin_version: Some("1.0.0".to_string()),
            plugin_state: PluginState::Healthy,  // Successful lifecycle
            service_name: "test_postgres".to_string(),
            service_type: "database".to_string(),
            container_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            plugin_health_check_performed: true,
            plugin_health_check_passed: true,
            plugin_health_check_duration_ms: Some(25.3),
            plugin_startup_duration_ms: Some(1234.5),
            error_type: None,
            error_message: None,
        }
    }

    /// Generate INVALID contract (health check failed)
    pub fn plugin_execution_unhealthy() -> PluginExecutionContract {
        PluginExecutionContract {
            plugin_name: "postgres".to_string(),
            plugin_type: "database".to_string(),
            plugin_version: Some("1.0.0".to_string()),
            plugin_state: PluginState::Error,
            service_name: "test_postgres".to_string(),
            service_type: "database".to_string(),
            container_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            plugin_health_check_performed: true,
            plugin_health_check_passed: false,  // Health check FAILED
            plugin_health_check_duration_ms: Some(150.7),
            plugin_startup_duration_ms: None,
            error_type: Some("HealthCheckFailed".to_string()),
            error_message: Some("Service failed to respond within 30s".to_string()),
        }
    }

    /// Generate test.started event
    pub fn test_started_event(test_name: &str, container_id: &str) -> TestStartedEvent {
        TestStartedEvent {
            test_name: test_name.to_string(),
            test_suite: "integration_tests".to_string(),
            container_id: container_id.to_string(),
            timestamp: "2025-10-30T14:23:45.123Z".to_string(),
        }
    }

    /// Generate test.completed event
    pub fn test_completed_event(test_name: &str, container_id: &str) -> TestCompletedEvent {
        TestCompletedEvent {
            test_name: test_name.to_string(),
            test_suite: "integration_tests".to_string(),
            container_id: container_id.to_string(),
            test_result: "pass".to_string(),
            test_assertions_passed: Some(5),
            test_assertions_failed: Some(0),
            timestamp: "2025-10-30T14:23:47.456Z".to_string(),
        }
    }

    /// Generate container.leaked event (CRITICAL - should never occur)
    pub fn container_leaked_event(container_id: &str, test_name: &str) -> ContainerLeakedEvent {
        ContainerLeakedEvent {
            container_id: container_id.to_string(),
            container_image: "postgres:15".to_string(),
            test_name: test_name.to_string(),
            container_age_seconds: 300,
            timestamp: "2025-10-30T14:28:45.123Z".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_test_execution_contract_has_all_required_fields() {
        let contract = ContractFixtures::valid_test_execution();

        assert!(!contract.test_name.is_empty());
        assert!(!contract.test_suite.is_empty());
        assert!(contract.test_isolated);  // Must be true
        assert!(contract.test_duration_ms > 0.0);  // Must be > 0
        assert!(!contract.container_id.is_empty());
        assert!(contract.test_cleanup_performed);  // Must be true
    }

    #[test]
    fn test_valid_container_lifecycle_ends_in_destroyed_state() {
        let contract = ContractFixtures::valid_container_lifecycle();

        assert_eq!(contract.container_state, ContainerState::Destroyed);
        assert!(!contract.container_destroyed_at.is_empty());
        assert!(contract.cleanup_success);
        assert_eq!(contract.cleanup_orphaned_resources, 0);
    }

    #[test]
    fn test_leaked_container_violates_cleanup_requirements() {
        let contract = ContractFixtures::container_lifecycle_leaked();

        assert_ne!(contract.container_state, ContainerState::Destroyed);
        assert!(contract.container_destroyed_at.is_empty());  // Missing!
        assert!(!contract.cleanup_success);
        assert!(contract.cleanup_orphaned_resources > 0);
    }

    #[test]
    fn test_unhealthy_plugin_has_error_details() {
        let contract = ContractFixtures::plugin_execution_unhealthy();

        assert_eq!(contract.plugin_state, PluginState::Error);
        assert!(!contract.plugin_health_check_passed);
        assert!(contract.error_type.is_some());
        assert!(contract.error_message.is_some());
    }
}
