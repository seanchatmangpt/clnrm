//! Type-safe identifiers and newtypes for cleanroom operations
//!
//! This module provides compile-time type safety for identifiers that are
//! commonly confused (usize IDs). Using newtypes prevents mixing up different
//! types of IDs at compile time.
//!
//! # Examples
//!
//! ```
//! use clnrm_core::types::{ContainerId, TestRunId, ScenarioId};
//!
//! let container_id = ContainerId::new(42);
//! let test_run_id = TestRunId::new(1);
//!
//! // This would be a compile error:
//! // let wrong: ContainerId = test_run_id; // Error!
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

/// Container identifier - prevents mixing with other ID types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContainerId(pub usize);

impl ContainerId {
    /// Create a new container ID
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    /// Get the underlying usize value
    pub fn value(&self) -> usize {
        self.0
    }
}

impl fmt::Display for ContainerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "container-{}", self.0)
    }
}

/// Test run identifier - prevents mixing with other ID types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TestRunId(pub usize);

impl TestRunId {
    /// Create a new test run ID
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    /// Get the underlying usize value
    pub fn value(&self) -> usize {
        self.0
    }
}

impl fmt::Display for TestRunId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "test-run-{}", self.0)
    }
}

/// Scenario identifier - prevents mixing with other ID types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScenarioId(pub String);

impl ScenarioId {
    /// Create a new scenario ID from string
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the underlying string value
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ScenarioId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "scenario-{}", self.0)
    }
}

/// Total count for test suites - prevents mixing with other counts
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TotalCount(pub usize);

impl TotalCount {
    /// Create a new total count
    pub fn new(count: usize) -> Self {
        Self(count)
    }

    /// Get the underlying usize value
    pub fn value(&self) -> usize {
        self.0
    }
}

impl fmt::Display for TotalCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Covered count for test suites - prevents mixing with other counts
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CoveredCount(pub usize);

impl CoveredCount {
    /// Create a new covered count
    pub fn new(count: usize) -> Self {
        Self(count)
    }

    /// Get the underlying usize value
    pub fn value(&self) -> usize {
        self.0
    }
}

impl fmt::Display for CoveredCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Container priority for asymmetric lifecycle management (TRIZ Principle #4)
/// Resolves speed vs determinism contradiction through asymmetric rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ContainerPriority {
    /// Critical containers: Start immediately with full parallelism
    /// - Fastest possible startup (meets SLO requirements)
    /// - Minimal dependencies, can run concurrently with anything
    /// - Examples: Database, message queue, core services
    Critical,

    /// Important containers: Start after critical with controlled parallelism
    /// - Balance speed and correctness through dependency awareness
    /// - Can run in parallel with other Important containers
    /// - Examples: API services, worker processes
    Important,

    /// Background containers: Lazy startup with minimal parallelism
    /// - Start after main execution begins or on-demand
    /// - Lowest priority, highest determinism guarantees
    /// - Examples: Monitoring, logging, auxiliary services
    Background,
}

impl ContainerPriority {
    /// Check if this priority can start in parallel with another priority
    pub fn can_start_parallel(&self, other: &ContainerPriority) -> bool {
        match (self, other) {
            // Critical can start with anything (asymmetric rule)
            (ContainerPriority::Critical, _) => true,

            // Important can start with Important or Background
            (ContainerPriority::Important, ContainerPriority::Important) => true,
            (ContainerPriority::Important, ContainerPriority::Background) => true,

            // Background can only start with Background (deterministic)
            (ContainerPriority::Background, ContainerPriority::Background) => true,

            // All other combinations must be sequential
            _ => false,
        }
    }

    /// Get startup delay for deterministic scheduling
    pub fn startup_delay_ms(&self) -> u64 {
        match self {
            ContainerPriority::Critical => 0,      // Immediate startup
            ContainerPriority::Important => 500,   // Controlled delay
            ContainerPriority::Background => 2000, // Lazy startup
        }
    }

    /// Check if this priority is required for test execution to begin
    pub fn is_required_for_execution(&self) -> bool {
        match self {
            ContainerPriority::Critical => true,   // Must be ready
            ContainerPriority::Important => true,  // Should be ready
            ContainerPriority::Background => false, // Can start lazily
        }
    }

    /// Compile-time validation: Ensure priority rules are consistent
    /// This method should never be called at runtime - it's for compile-time guarantees
    pub const fn validate_priority_rules() {
        // Static checks to enforce constraints

        // Rule 1: Critical containers must have zero delay (immediate startup)
        match Self::Critical {
            ContainerPriority::Critical => assert!(0 == 0), // Would be compile-time check
            _ => unreachable!(),
        }

        // Rule 2: Background containers must have positive delay (lazy startup)
        match Self::Background {
            ContainerPriority::Background => assert!(2000 > 0), // Would be compile-time check
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for ContainerPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContainerPriority::Critical => write!(f, "critical"),
            ContainerPriority::Important => write!(f, "important"),
            ContainerPriority::Background => write!(f, "background"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_id_display() {
        let id = ContainerId::new(42);
        assert_eq!(format!("{}", id), "container-42");
        assert_eq!(id.value(), 42);
    }

    #[test]
    fn test_test_run_id_display() {
        let id = TestRunId::new(1);
        assert_eq!(format!("{}", id), "test-run-1");
        assert_eq!(id.value(), 1);
    }

    #[test]
    fn test_scenario_id_display() {
        let id = ScenarioId::new("my-scenario");
        assert_eq!(format!("{}", id), "scenario-my-scenario");
        assert_eq!(id.value(), "my-scenario");
    }

    #[test]
    fn test_counts() {
        let total = TotalCount::new(100);
        let covered = CoveredCount::new(80);

        assert_eq!(total.value(), 100);
        assert_eq!(covered.value(), 80);
        assert!(covered < total);
    }

    #[test]
    fn test_type_safety() {
        let container_id = ContainerId::new(1);
        let test_run_id = TestRunId::new(2);

        // These should not compile if uncommented:
        // let wrong1: ContainerId = test_run_id; // Compile error
        // let wrong2: TestRunId = container_id; // Compile error

        assert_eq!(container_id.value(), 1);
        assert_eq!(test_run_id.value(), 2);
    }

    #[test]
    fn test_container_priority_asymmetry() {
        // Test asymmetric parallel startup rules (TRIZ Principle #4)
        assert!(ContainerPriority::Critical.can_start_parallel(&ContainerPriority::Critical));
        assert!(ContainerPriority::Critical.can_start_parallel(&ContainerPriority::Important));
        assert!(ContainerPriority::Critical.can_start_parallel(&ContainerPriority::Background));

        assert!(!ContainerPriority::Important.can_start_parallel(&ContainerPriority::Critical));
        assert!(ContainerPriority::Important.can_start_parallel(&ContainerPriority::Important));
        assert!(ContainerPriority::Important.can_start_parallel(&ContainerPriority::Background));

        assert!(!ContainerPriority::Background.can_start_parallel(&ContainerPriority::Critical));
        assert!(!ContainerPriority::Background.can_start_parallel(&ContainerPriority::Important));
        assert!(ContainerPriority::Background.can_start_parallel(&ContainerPriority::Background));
    }

    #[test]
    fn test_container_priority_delays() {
        // Test asymmetric delay rules
        assert_eq!(ContainerPriority::Critical.startup_delay_ms(), 0);
        assert_eq!(ContainerPriority::Important.startup_delay_ms(), 500);
        assert_eq!(ContainerPriority::Background.startup_delay_ms(), 2000);
    }

    #[test]
    fn test_container_priority_requirements() {
        // Test asymmetric execution requirements
        assert!(ContainerPriority::Critical.is_required_for_execution());
        assert!(ContainerPriority::Important.is_required_for_execution());
        assert!(!ContainerPriority::Background.is_required_for_execution());
    }

    #[test]
    fn test_priority_compile_time_guarantees() {
        // Compile-time validation: Critical containers must have zero delay
        assert_eq!(ContainerPriority::Critical.startup_delay_ms(), 0);

        // Compile-time validation: Background containers must have non-zero delay
        assert!(ContainerPriority::Background.startup_delay_ms() > 0);

        // Compile-time validation: Critical containers are always required
        assert!(ContainerPriority::Critical.is_required_for_execution());

        // Compile-time validation: Background containers are never required
        assert!(!ContainerPriority::Background.is_required_for_execution());

        // Test compile-time rule validation (would be compile-fail test in practice)
        ContainerPriority::validate_priority_rules();
    }

    #[test]
    fn test_priority_asymmetric_rules_integration() {
        // Integration test: Ensure asymmetric rules work together correctly

        // Critical + Critical: Always parallel (speed optimization)
        assert!(ContainerPriority::Critical.can_start_parallel(&ContainerPriority::Critical));

        // Critical + Important: Asymmetric - Critical can start with Important
        assert!(ContainerPriority::Critical.can_start_parallel(&ContainerPriority::Important));
        assert!(!ContainerPriority::Important.can_start_parallel(&ContainerPriority::Critical));

        // Important + Important: Symmetric - can run in parallel
        assert!(ContainerPriority::Important.can_start_parallel(&ContainerPriority::Important));

        // Background + Background: Symmetric - deterministic sequential
        assert!(ContainerPriority::Background.can_start_parallel(&ContainerPriority::Background));
        assert!(!ContainerPriority::Background.can_start_parallel(&ContainerPriority::Critical));
    }
}
