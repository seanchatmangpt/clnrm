//! Environment Compiler & Ontology System (Phase 2)
//!
//! Σ*-aware environment compilation for capability-based testing.
//!
//! # Architecture
//!
//! - **Σ* (SigmaBase)**: Immutable environment ontology snapshots
//! - **ΔΣ (SigmaDelta)**: Overlay/delta operations on base ontologies
//! - **Compiler**: Transforms Σ* + ΔΣ + Q → executable environments
//! - **Store**: Content-addressable ontology storage
//!
//! # Example
//!
//! ```rust,no_run
//! use clnrm_core::environment::{
//!     SigmaBase, SigmaDelta, EnvironmentCompiler, OntologyStore,
//! };
//! use clnrm_core::capabilities::ConstraintSet;
//! use std::sync::Arc;
//!
//! # fn example() -> clnrm_core::error::Result<()> {
//! // Create ontology store
//! let store = Arc::new(OntologyStore::new());
//!
//! // Create and store base ontology
//! // let sigma = SigmaBase { ... };
//! // let hash = store.put(sigma)?;
//!
//! // Create compiler
//! let compiler = EnvironmentCompiler::new(store);
//!
//! // Compile environment
//! // let env = compiler.compile(&hash, None, &ConstraintSet::default())?;
//! # Ok(())
//! # }
//! ```

pub mod compiler;
pub mod delta;
pub mod sigma;
pub mod store;

// Re-export commonly used types
pub use compiler::{
    CompiledEnvironment, ContainerGraph, ContainerNode, DependencyEdge, DependencyType,
    EnvironmentCompiler, HealthCheck, NetworkConfig, ProofMetadata, ResourceLimits as CompilerResourceLimits,
    ServiceInstrumentation, TelemetryConfig, VolumeConfig, VolumeMount,
};
pub use delta::{
    NetworkModification, ServiceModification, SigmaDelta, SigmaDeltaBuilder, VolumeModification,
};
pub use sigma::{
    ContentHash, HealthCheckDef, InstrumentationDef, NetworkDef, NetworkId, OtelCollectorDef,
    ResourcesDef, SemVer, ServiceDef, ServiceId, SigmaBase, TelemetryDef, VolumeDef, VolumeId,
    VolumeMountDef, WeaverDef,
};
pub use store::OntologyStore;
