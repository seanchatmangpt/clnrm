//! Policy enforcement for cleanroom testing
//!
//! This module provides comprehensive policy enforcement following core team best practices:
//! - Security boundaries and isolation controls
//! - Resource limits and constraints enforcement
//! - Execution policies and compliance rules
//! - Audit trails and compliance reporting
//!
//! ## Security Levels
//!
//! Cleanroom supports multiple security levels:
//!
//! - **Low**: Minimal isolation for development and debugging
//! - **Medium**: Balanced security for most testing scenarios
//! - **High**: Enhanced security for sensitive applications
//! - **Maximum**: Maximum security with strict isolation
//! - **Locked**: Maximum security with additional restrictions
//!
//! ## Usage Examples
//!
//! ### Basic Policy Creation
//!
//! ```no_run
//! use clnrm::{Policy, SecurityLevel};
//!
//! // Create policy with specific security level
//! let policy = Policy::with_security_level(SecurityLevel::High);
//!
//! // Create custom policy with resource limits
//! let policy = Policy::with_resource_limits(
//!     80.0,  // max CPU %
//!     1024 * 1024 * 1024,  // max memory bytes (1GB)
//!     10 * 1024 * 1024 * 1024,  // max disk bytes (10GB)
//! );
//! ```
//!
//! ### Security Policy Configuration
//!
//! ```no_run
//! use clnrm::{Policy, SecurityPolicy};
//!
//! let mut policy = Policy::default();
//! policy.security.enable_network_isolation = true;
//! policy.security.enable_filesystem_isolation = true;
//! policy.security.allowed_ports = vec![5432, 6379]; // PostgreSQL, Redis
//! policy.security.blocked_addresses = vec!["127.0.0.1".to_string()];
//! ```
//!
//! ### Policy Validation
//!
//! ```no_run
//! use clnrm::Policy;
//!
//! let policy = Policy::locked();
//!
//! // Validate policy configuration
//! if let Err(e) = policy.validate() {
//!     eprintln!("Policy validation failed: {}", e);
//!     // Handle validation errors
//! }
//! ```

use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Policy configuration for cleanroom testing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Policy {
    /// Security policy
    pub security: SecurityPolicy,
    /// Resource policy
    pub resources: ResourcePolicy,
    /// Execution policy
    pub execution: ExecutionPolicy,
    /// Compliance policy
    pub compliance: CompliancePolicy,
}

/// Security policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// Enable network isolation
    pub enable_network_isolation: bool,
    /// Enable filesystem isolation
    pub enable_filesystem_isolation: bool,
    /// Enable process isolation
    pub enable_process_isolation: bool,
    /// Allowed network ports
    pub allowed_ports: Vec<u16>,
    /// Blocked network addresses
    pub blocked_addresses: Vec<String>,
    /// Enable sensitive data redaction
    pub enable_data_redaction: bool,
    /// Redaction patterns
    pub redaction_patterns: Vec<String>,
    /// Enable audit logging
    pub enable_audit_logging: bool,
    /// Security level
    pub security_level: SecurityLevel,
}

/// Resource policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePolicy {
    /// Maximum CPU usage percentage
    pub max_cpu_usage_percent: f64,
    /// Maximum memory usage bytes
    pub max_memory_usage_bytes: u64,
    /// Maximum disk usage bytes
    pub max_disk_usage_bytes: u64,
    /// Maximum network bandwidth bytes per second
    pub max_network_bandwidth_bytes_per_sec: u64,
    /// Maximum container count
    pub max_container_count: u32,
    /// Maximum test execution time
    pub max_test_execution_time: Duration,
    /// Enable resource monitoring
    pub enable_resource_monitoring: bool,
    /// Resource cleanup timeout
    pub resource_cleanup_timeout: Duration,
}

/// Execution policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPolicy {
    /// Enable deterministic execution
    pub enable_deterministic_execution: bool,
    /// Fixed seed for deterministic runs
    pub deterministic_seed: Option<u64>,
    /// Enable parallel execution
    pub enable_parallel_execution: bool,
    /// Maximum parallel tasks
    pub max_parallel_tasks: u32,
    /// Enable test isolation
    pub enable_test_isolation: bool,
    /// Test timeout
    pub test_timeout: Duration,
    /// Enable retry on failure
    pub enable_retry_on_failure: bool,
    /// Maximum retry attempts
    pub max_retry_attempts: u32,
    /// Retry delay
    pub retry_delay: Duration,
}

/// Compliance policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompliancePolicy {
    /// Enable compliance reporting
    pub enable_compliance_reporting: bool,
    /// Compliance standards
    pub compliance_standards: Vec<ComplianceStandard>,
    /// Enable audit trails
    pub enable_audit_trails: bool,
    /// Audit retention period
    pub audit_retention_period: Duration,
    /// Enable policy validation
    pub enable_policy_validation: bool,
    /// Policy validation rules
    pub policy_validation_rules: Vec<PolicyValidationRule>,
}

/// Security level enumeration
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Low security level
    Low,
    /// Medium security level
    Medium,
    /// High security level
    High,
    /// Maximum security level
    Maximum,
    /// Standard security level
    Standard,
    /// Locked security level
    Locked,
}

/// Audit level enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AuditLevel {
    /// Debug audit level
    Debug,
    /// Info audit level
    Info,
    /// Warning audit level
    Warn,
    /// Error audit level
    Error,
}

/// Compliance standard enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceStandard {
    /// SOC 2 compliance
    Soc2,
    /// ISO 27001 compliance
    Iso27001,
    /// PCI DSS compliance
    PciDss,
    /// HIPAA compliance
    Hipaa,
    /// GDPR compliance
    Gdpr,
    /// Custom compliance standard
    Custom(String),
}

/// Policy validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyValidationRule {
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Rule condition
    pub condition: String,
    /// Rule action
    pub action: PolicyValidationAction,
    /// Rule severity
    pub severity: PolicyValidationSeverity,
}

/// Policy validation action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyValidationAction {
    /// Allow operation
    Allow,
    /// Deny operation
    Deny,
    /// Warn about operation
    Warn,
    /// Require approval
    RequireApproval,
}

/// Policy validation severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyValidationSeverity {
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

impl SecurityPolicy {
    /// Create a new security policy with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a security policy with a specific security level
    pub fn with_security_level(security_level: SecurityLevel) -> Self {
        let mut policy = Self {
            security_level,
            ..Default::default()
        };

        // Adjust isolation settings based on security level
        match security_level {
            SecurityLevel::Low => {
                policy.enable_network_isolation = false;
                policy.enable_filesystem_isolation = false;
                policy.enable_process_isolation = false;
                policy.enable_data_redaction = false;
                policy.enable_audit_logging = false;
            }
            SecurityLevel::Medium | SecurityLevel::Standard => {
                // Keep default settings
            }
            SecurityLevel::High | SecurityLevel::Maximum | SecurityLevel::Locked => {
                // Keep all isolation enabled (default)
            }
        }

        policy
    }
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            enable_network_isolation: true,
            enable_filesystem_isolation: true,
            enable_process_isolation: true,
            allowed_ports: vec![5432, 6379, 8080, 9090],
            blocked_addresses: vec!["127.0.0.1".to_string()],
            enable_data_redaction: true,
            redaction_patterns: vec![
                r"password\s*=\s*[^\s]+".to_string(),
                r"token\s*=\s*[^\s]+".to_string(),
                r"key\s*=\s*[^\s]+".to_string(),
            ],
            enable_audit_logging: true,
            security_level: SecurityLevel::Standard,
        }
    }
}

impl ResourcePolicy {
    /// Create a new resource policy with default settings
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for ResourcePolicy {
    fn default() -> Self {
        Self {
            max_cpu_usage_percent: 80.0,
            max_memory_usage_bytes: 1024 * 1024 * 1024, // 1GB
            max_disk_usage_bytes: 10 * 1024 * 1024 * 1024, // 10GB
            max_network_bandwidth_bytes_per_sec: 100 * 1024 * 1024, // 100MB/s
            max_container_count: 10,
            max_test_execution_time: Duration::from_secs(300), // 5 minutes
            enable_resource_monitoring: true,
            resource_cleanup_timeout: Duration::from_secs(30),
        }
    }
}

impl ExecutionPolicy {
    /// Create a new execution policy with default settings
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for ExecutionPolicy {
    fn default() -> Self {
        Self {
            enable_deterministic_execution: true,
            deterministic_seed: Some(42),
            enable_parallel_execution: true,
            max_parallel_tasks: 4,
            enable_test_isolation: true,
            test_timeout: Duration::from_secs(60),
            enable_retry_on_failure: true,
            max_retry_attempts: 3,
            retry_delay: Duration::from_secs(1),
        }
    }
}

impl CompliancePolicy {
    /// Create a new compliance policy with default settings
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for CompliancePolicy {
    fn default() -> Self {
        Self {
            enable_compliance_reporting: true,
            compliance_standards: vec![ComplianceStandard::Soc2],
            enable_audit_trails: true,
            audit_retention_period: Duration::from_secs(30 * 24 * 60 * 60), // 30 days
            enable_policy_validation: true,
            policy_validation_rules: vec![],
        }
    }
}

impl Policy {
    /// Create a new policy with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a policy with custom security level
    pub fn with_security_level(security_level: SecurityLevel) -> Self {
        Self {
            security: SecurityPolicy::with_security_level(security_level),
            ..Default::default()
        }
    }

    /// Create a policy with custom resource limits
    pub fn with_resource_limits(
        max_cpu_percent: f64,
        max_memory_bytes: u64,
        max_disk_bytes: u64,
    ) -> Self {
        let mut policy = Self::default();
        policy.resources.max_cpu_usage_percent = max_cpu_percent;
        policy.resources.max_memory_usage_bytes = max_memory_bytes;
        policy.resources.max_disk_usage_bytes = max_disk_bytes;
        policy
    }

    /// Create a locked-down policy with maximum security
    pub fn locked() -> Self {
        Self::with_security_level(SecurityLevel::Locked)
    }

    /// Create a high security policy
    pub fn high_security() -> Self {
        Self::with_security_level(SecurityLevel::High)
    }

    /// Create a standard security policy
    pub fn standard() -> Self {
        Self::with_security_level(SecurityLevel::Standard)
    }

    /// Create a low security policy
    pub fn low_security() -> Self {
        Self::with_security_level(SecurityLevel::Low)
    }

    /// Disable network access
    pub fn with_network_disabled(mut self) -> Self {
        self.security.enable_network_isolation = true;
        self
    }

    /// Set network isolation
    pub fn with_network_isolation(mut self, enable: bool) -> Self {
        self.security.enable_network_isolation = enable;
        self
    }

    /// Check if network access is allowed
    pub fn allows_network(&self) -> bool {
        !self.security.enable_network_isolation
    }

    /// Validate policy configuration
    pub fn validate(&self) -> Result<()> {
        // Validate security policy
        if self.security.allowed_ports.is_empty() {
            return Err(CleanroomError::policy_violation_error(
                "No allowed ports configured",
            ));
        }

        // Validate resource policy
        if self.resources.max_cpu_usage_percent <= 0.0
            || self.resources.max_cpu_usage_percent > 100.0
        {
            return Err(CleanroomError::policy_violation_error(
                "Invalid CPU usage percentage",
            ));
        }

        if self.resources.max_memory_usage_bytes == 0 {
            return Err(CleanroomError::policy_violation_error(
                "Invalid memory usage limit",
            ));
        }

        if self.resources.max_disk_usage_bytes == 0 {
            return Err(CleanroomError::policy_violation_error(
                "Invalid disk usage limit",
            ));
        }

        // Validate execution policy
        if self.execution.max_parallel_tasks == 0 {
            return Err(CleanroomError::policy_violation_error(
                "Invalid parallel task count",
            ));
        }

        if self.execution.max_retry_attempts == 0 {
            return Err(CleanroomError::policy_violation_error(
                "Invalid retry attempt count",
            ));
        }

        Ok(())
    }

    /// Check if operation is allowed by policy
    pub fn is_operation_allowed(
        &self,
        _operation: &str,
        context: &HashMap<String, String>,
    ) -> Result<bool> {
        // Check security policy
        if self.security.enable_network_isolation {
            if let Some(port) = context.get("port") {
                if let Ok(port_num) = port.parse::<u16>() {
                    if !self.security.allowed_ports.contains(&port_num) {
                        return Ok(false);
                    }
                }
            }
        }

        // Check resource policy
        if let Some(cpu_usage) = context.get("cpu_usage") {
            if let Ok(cpu_percent) = cpu_usage.parse::<f64>() {
                if cpu_percent > self.resources.max_cpu_usage_percent {
                    return Ok(false);
                }
            }
        }

        if let Some(memory_usage) = context.get("memory_usage") {
            if let Ok(memory_bytes) = memory_usage.parse::<u64>() {
                if memory_bytes > self.resources.max_memory_usage_bytes {
                    return Ok(false);
                }
            }
        }

        // Check execution policy
        if let Some(parallel_tasks) = context.get("parallel_tasks") {
            if let Ok(task_count) = parallel_tasks.parse::<u32>() {
                if task_count > self.execution.max_parallel_tasks {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// Get environment variables for policy enforcement
    pub fn to_env(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();

        // Security policy environment variables
        env.insert(
            "CLEANROOM_SECURITY_LEVEL".to_string(),
            format!("{:?}", self.security.security_level),
        );
        env.insert(
            "CLEANROOM_NETWORK_ISOLATION".to_string(),
            self.security.enable_network_isolation.to_string(),
        );
        env.insert(
            "CLEANROOM_FILESYSTEM_ISOLATION".to_string(),
            self.security.enable_filesystem_isolation.to_string(),
        );
        env.insert(
            "CLEANROOM_PROCESS_ISOLATION".to_string(),
            self.security.enable_process_isolation.to_string(),
        );
        env.insert(
            "CLEANROOM_ALLOWED_PORTS".to_string(),
            self.security
                .allowed_ports
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(","),
        );

        // Resource policy environment variables
        env.insert(
            "CLEANROOM_MAX_CPU_PERCENT".to_string(),
            self.resources.max_cpu_usage_percent.to_string(),
        );
        env.insert(
            "CLEANROOM_MAX_MEMORY_BYTES".to_string(),
            self.resources.max_memory_usage_bytes.to_string(),
        );
        env.insert(
            "CLEANROOM_MAX_DISK_BYTES".to_string(),
            self.resources.max_disk_usage_bytes.to_string(),
        );
        env.insert(
            "CLEANROOM_MAX_CONTAINER_COUNT".to_string(),
            self.resources.max_container_count.to_string(),
        );

        // Execution policy environment variables
        env.insert(
            "CLEANROOM_DETERMINISTIC_EXECUTION".to_string(),
            self.execution.enable_deterministic_execution.to_string(),
        );
        env.insert(
            "CLEANROOM_PARALLEL_EXECUTION".to_string(),
            self.execution.enable_parallel_execution.to_string(),
        );
        env.insert(
            "CLEANROOM_MAX_PARALLEL_TASKS".to_string(),
            self.execution.max_parallel_tasks.to_string(),
        );
        env.insert(
            "CLEANROOM_TEST_ISOLATION".to_string(),
            self.execution.enable_test_isolation.to_string(),
        );

        env
    }

    /// Get policy summary
    pub fn summary(&self) -> String {
        format!(
            "Policy Summary:\n\
            Security Level: {:?}\n\
            Network Isolation: {}\n\
            Filesystem Isolation: {}\n\
            Process Isolation: {}\n\
            Max CPU Usage: {}%\n\
            Max Memory Usage: {} bytes\n\
            Max Disk Usage: {} bytes\n\
            Max Container Count: {}\n\
            Deterministic Execution: {}\n\
            Parallel Execution: {}\n\
            Max Parallel Tasks: {}\n\
            Test Isolation: {}",
            self.security.security_level,
            self.security.enable_network_isolation,
            self.security.enable_filesystem_isolation,
            self.security.enable_process_isolation,
            self.resources.max_cpu_usage_percent,
            self.resources.max_memory_usage_bytes,
            self.resources.max_disk_usage_bytes,
            self.resources.max_container_count,
            self.execution.enable_deterministic_execution,
            self.execution.enable_parallel_execution,
            self.execution.max_parallel_tasks,
            self.execution.enable_test_isolation
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Allow unwrap/expect in tests as they are expected to panic on failure
    #[allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic
    )]
    #[test]
    fn test_policy_creation() {
        let policy = Policy::new();
        assert!(policy.validate().is_ok());
    }

    #[test]
    fn test_policy_with_security_level() {
        let policy = Policy::with_security_level(SecurityLevel::High);
        assert_eq!(policy.security.security_level, SecurityLevel::High);
    }

    #[test]
    fn test_policy_with_resource_limits() {
        let policy = Policy::with_resource_limits(50.0, 512 * 1024 * 1024, 5 * 1024 * 1024 * 1024);
        assert_eq!(policy.resources.max_cpu_usage_percent, 50.0);
        assert_eq!(policy.resources.max_memory_usage_bytes, 512 * 1024 * 1024);
        assert_eq!(
            policy.resources.max_disk_usage_bytes,
            5 * 1024 * 1024 * 1024
        );
    }

    #[test]
    fn test_policy_validation() {
        let policy = Policy::new();
        assert!(policy.validate().is_ok());
    }

    #[test]
    fn test_policy_operation_allowed() -> Result<()> {
        let policy = Policy::new();
        let mut context = HashMap::new();
        context.insert("port".to_string(), "5432".to_string());
        context.insert("cpu_usage".to_string(), "50.0".to_string());
        context.insert("memory_usage".to_string(), "256000000".to_string());

        assert!(policy.is_operation_allowed("test_operation", &context)?);
        Ok(())
    }

    #[test]
    fn test_policy_operation_denied() -> Result<()> {
        let policy = Policy::new();
        let mut context = HashMap::new();
        context.insert("port".to_string(), "9999".to_string()); // Not in allowed ports
        context.insert("cpu_usage".to_string(), "90.0".to_string()); // Exceeds limit

        assert!(!policy.is_operation_allowed("test_operation", &context)?);
        Ok(())
    }

    #[test]
    fn test_policy_env_variables() {
        let policy = Policy::new();
        let env = policy.to_env();

        assert!(env.contains_key("CLEANROOM_SECURITY_LEVEL"));
        assert!(env.contains_key("CLEANROOM_NETWORK_ISOLATION"));
        assert!(env.contains_key("CLEANROOM_MAX_CPU_PERCENT"));
        assert!(env.contains_key("CLEANROOM_MAX_MEMORY_BYTES"));
    }

    #[test]
    fn test_policy_summary() {
        let policy = Policy::new();
        let summary = policy.summary();

        assert!(summary.contains("Policy Summary"));
        assert!(summary.contains("Security Level"));
        assert!(summary.contains("Network Isolation"));
    }

    #[test]
    fn test_policy_default() {
        let policy = Policy::default();

        assert_eq!(policy.security.security_level, SecurityLevel::Standard);
        assert!(policy.security.enable_network_isolation);
        assert!(policy.security.enable_filesystem_isolation);
        assert!(policy.security.enable_process_isolation);
        assert!(policy.security.enable_data_redaction);
        assert!(policy.security.enable_audit_logging);
    }

    #[test]
    fn test_policy_new() {
        let policy = Policy::new();

        assert_eq!(policy.security.security_level, SecurityLevel::Standard);
        assert!(policy.security.enable_network_isolation);
        assert!(policy.security.enable_filesystem_isolation);
        assert!(policy.security.enable_process_isolation);
        assert!(policy.security.enable_data_redaction);
        assert!(policy.security.enable_audit_logging);
    }

    #[test]
    fn test_policy_serialization() -> Result<()> {
        let policy = Policy::new();

        let json = serde_json::to_string(&policy)
            .map_err(|e| CleanroomError::internal_error(format!("Serialization failed: {}", e)))?;
        let deserialized: Policy = serde_json::from_str(&json).map_err(|e| {
            CleanroomError::internal_error(format!("Deserialization failed: {}", e))
        })?;

        assert_eq!(
            policy.security.security_level,
            deserialized.security.security_level
        );
        assert_eq!(
            policy.security.enable_network_isolation,
            deserialized.security.enable_network_isolation
        );
        assert_eq!(
            policy.security.enable_filesystem_isolation,
            deserialized.security.enable_filesystem_isolation
        );
        Ok(())
    }

    #[test]
    fn test_security_policy() {
        let security = SecurityPolicy::new();

        assert_eq!(security.security_level, SecurityLevel::Standard);
        assert!(security.enable_network_isolation);
        assert!(security.enable_filesystem_isolation);
        assert!(security.enable_process_isolation);
        assert!(security.enable_data_redaction);
        assert!(security.enable_audit_logging);
        assert!(security.allowed_ports.contains(&5432)); // PostgreSQL
        assert!(security.allowed_ports.contains(&6379)); // Redis
    }

    #[test]
    fn test_security_policy_with_level() {
        let security = SecurityPolicy::with_security_level(SecurityLevel::High);

        assert_eq!(security.security_level, SecurityLevel::High);
        assert!(security.enable_network_isolation);
        assert!(security.enable_filesystem_isolation);
        assert!(security.enable_process_isolation);
        assert!(security.enable_data_redaction);
        assert!(security.enable_audit_logging);
    }

    #[test]
    fn test_security_level_serialization() -> Result<()> {
        let levels = vec![
            SecurityLevel::Low,
            SecurityLevel::Standard,
            SecurityLevel::High,
            SecurityLevel::Locked,
        ];

        for level in levels {
            let json = serde_json::to_string(&level).map_err(|e| {
                CleanroomError::internal_error(format!("Serialization failed: {}", e))
            })?;
            let deserialized: SecurityLevel = serde_json::from_str(&json).map_err(|e| {
                CleanroomError::internal_error(format!("Deserialization failed: {}", e))
            })?;
            assert_eq!(level, deserialized);
        }
        Ok(())
    }

    #[test]
    fn test_resource_policy() {
        let resources = ResourcePolicy::new();

        assert_eq!(resources.max_cpu_usage_percent, 80.0);
        assert_eq!(resources.max_memory_usage_bytes, 1024 * 1024 * 1024); // 1GB
        assert_eq!(resources.max_disk_usage_bytes, 10 * 1024 * 1024 * 1024); // 10GB
        assert_eq!(
            resources.max_network_bandwidth_bytes_per_sec,
            100 * 1024 * 1024
        ); // 100MB/s
        assert_eq!(resources.max_container_count, 10);
        assert_eq!(resources.max_test_execution_time, Duration::from_secs(300));
    }

    #[test]
    fn test_execution_policy() {
        let execution = ExecutionPolicy::new();

        assert!(execution.enable_deterministic_execution);
        assert!(execution.enable_parallel_execution);
        assert!(execution.enable_test_isolation);
        assert!(execution.enable_retry_on_failure);
        assert_eq!(execution.max_parallel_tasks, 4);
        assert_eq!(execution.max_retry_attempts, 3);
        assert_eq!(execution.test_timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_compliance_policy() {
        let compliance = CompliancePolicy::new();

        assert!(compliance.enable_compliance_reporting);
        assert!(compliance.enable_audit_trails);
        assert!(compliance.enable_policy_validation);
        assert_eq!(
            compliance.audit_retention_period,
            Duration::from_secs(30 * 24 * 60 * 60)
        ); // 30 days
    }

    #[test]
    fn test_audit_level_serialization() -> Result<()> {
        let levels = vec![
            AuditLevel::Debug,
            AuditLevel::Info,
            AuditLevel::Warn,
            AuditLevel::Error,
        ];

        for level in levels {
            let json = serde_json::to_string(&level).map_err(|e| {
                CleanroomError::internal_error(format!("Serialization failed: {}", e))
            })?;
            let deserialized: AuditLevel = serde_json::from_str(&json).map_err(|e| {
                CleanroomError::internal_error(format!("Deserialization failed: {}", e))
            })?;
            assert_eq!(level, deserialized);
        }
        Ok(())
    }

    #[test]
    fn test_policy_locked() {
        let policy = Policy::locked();

        assert_eq!(policy.security.security_level, SecurityLevel::Locked);
        assert!(policy.security.enable_network_isolation);
        assert!(policy.security.enable_filesystem_isolation);
        assert!(policy.security.enable_process_isolation);
        assert!(policy.security.enable_data_redaction);
        assert!(policy.security.enable_audit_logging);
    }

    #[test]
    fn test_policy_high_security() {
        let policy = Policy::high_security();

        assert_eq!(policy.security.security_level, SecurityLevel::High);
        assert!(policy.security.enable_network_isolation);
        assert!(policy.security.enable_filesystem_isolation);
        assert!(policy.security.enable_process_isolation);
        assert!(policy.security.enable_data_redaction);
        assert!(policy.security.enable_audit_logging);
    }

    #[test]
    fn test_policy_standard() {
        let policy = Policy::standard();

        assert_eq!(policy.security.security_level, SecurityLevel::Standard);
        assert!(policy.security.enable_network_isolation);
        assert!(policy.security.enable_filesystem_isolation);
        assert!(policy.security.enable_process_isolation);
        assert!(policy.security.enable_data_redaction);
        assert!(policy.security.enable_audit_logging);
    }

    #[test]
    fn test_policy_low_security() {
        let policy = Policy::low_security();

        assert_eq!(policy.security.security_level, SecurityLevel::Low);
        assert!(!policy.security.enable_network_isolation);
        assert!(!policy.security.enable_filesystem_isolation);
        assert!(!policy.security.enable_process_isolation);
        assert!(!policy.security.enable_data_redaction);
        assert!(!policy.security.enable_audit_logging);
    }

    #[test]
    fn test_policy_operation_allowed_with_valid_context() -> Result<()> {
        let policy = Policy::new();
        let mut context = HashMap::new();
        context.insert("port".to_string(), "5432".to_string());
        context.insert("cpu_usage".to_string(), "50.0".to_string());
        context.insert("memory_usage".to_string(), "256000000".to_string());

        assert!(policy.is_operation_allowed("test_operation", &context)?);
        Ok(())
    }

    #[test]
    fn test_policy_operation_denied_with_invalid_port() -> Result<()> {
        let policy = Policy::new();
        let mut context = HashMap::new();
        context.insert("port".to_string(), "9999".to_string()); // Not in allowed ports
        context.insert("cpu_usage".to_string(), "50.0".to_string());
        context.insert("memory_usage".to_string(), "256000000".to_string());

        assert!(!policy.is_operation_allowed("test_operation", &context)?);
        Ok(())
    }

    #[test]
    fn test_policy_operation_denied_with_high_cpu() -> Result<()> {
        let policy = Policy::new();
        let mut context = HashMap::new();
        context.insert("port".to_string(), "5432".to_string());
        context.insert("cpu_usage".to_string(), "90.0".to_string()); // Exceeds limit
        context.insert("memory_usage".to_string(), "256000000".to_string());

        assert!(!policy.is_operation_allowed("test_operation", &context)?);
        Ok(())
    }

    #[test]
    fn test_policy_operation_denied_with_high_memory() -> Result<()> {
        let policy = Policy::new();
        let mut context = HashMap::new();
        context.insert("port".to_string(), "5432".to_string());
        context.insert("cpu_usage".to_string(), "50.0".to_string());
        context.insert("memory_usage".to_string(), "2000000000".to_string()); // Exceeds limit

        assert!(!policy.is_operation_allowed("test_operation", &context)?);
        Ok(())
    }

    #[test]
    fn test_policy_env_variables_values() {
        let policy = Policy::new();
        let env = policy.to_env();

        assert_eq!(
            env.get("CLEANROOM_SECURITY_LEVEL"),
            Some(&"Standard".to_string())
        );
        assert_eq!(
            env.get("CLEANROOM_NETWORK_ISOLATION"),
            Some(&"true".to_string())
        );
        assert_eq!(
            env.get("CLEANROOM_MAX_CPU_PERCENT"),
            Some(&"80".to_string())
        );
        assert_eq!(
            env.get("CLEANROOM_MAX_MEMORY_BYTES"),
            Some(&"1073741824".to_string())
        );
    }
}
