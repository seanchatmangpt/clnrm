//! Capability-aware scenario framework
//!
//! This module provides the infrastructure for capability-based testing,
//! where scenarios explicitly declare what capabilities they exercise,
//! what effects they produce, and what constraints they must satisfy.
//!
//! # Architecture
//!
//! The capability framework consists of several layers:
//!
//! 1. **Effects** - Observable actions scenarios can perform (network, storage, etc.)
//! 2. **Constraints** - Quality requirements (hermeticity, latency bands, resource limits)
//! 3. **Scenarios** - Capability-aware test descriptors that tie it all together
//! 4. **Validation** - Cross-layer validation against capability registry
//!
//! # Example
//!
//! ```rust,no_run
//! use clnrm_core::capabilities::{
//!     CapabilityScenario, CapabilityScenarioBuilder,
//!     Effect, EffectSet, ConstraintSet,
//!     TelemetrySchemaRef,
//! };
//! use clnrm_core::backend::capabilities::BackendCapabilityRegistry;
//!
//! # fn example() -> clnrm_core::error::Result<()> {
//! // Create scenario with capabilities and effects
//! let scenario = CapabilityScenarioBuilder::new(
//!     "hermetic-db-test",
//!     "Hermetic Database Test"
//! )
//! .description("Test database operations in isolated environment")
//! .capability("hermetic_execution")
//! .capability("deterministic_execution")
//! .constraints(ConstraintSet::hot_path())
//! .telemetry_schema(TelemetrySchemaRef {
//!     registry_path: "registry/db-test-schema.yaml".to_string(),
//!     version: "1.0.0".to_string(),
//! })
//! .build();
//!
//! // Validate against capability registry
//! let registry = BackendCapabilityRegistry::new();
//! scenario.validate(&registry)?;
//! # Ok(())
//! # }
//! ```

pub mod constraints;
pub mod effects;
pub mod scenario;

// Re-export commonly used types
pub use constraints::{ConstraintSet, ExecutionMetrics, LatencyBand, ResourceLimits};
pub use effects::{Effect, EffectBudget, EffectSet, EffectUsage, PrivilegeType, StorageMode};
pub use scenario::{
    CapabilityId, CapabilityScenario, CapabilityScenarioBuilder, EnvironmentDescriptor,
    NetworkRequirement, PortMapping, ScenarioId, ServiceRequirement, TelemetrySchemaRef,
    VolumeRequirement,
};
