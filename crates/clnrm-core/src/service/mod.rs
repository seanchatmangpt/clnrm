//! gVisor-native service management
//!
//! This module provides a production-ready service management layer
//! that replaces testcontainers-modules with gVisor-native container execution.

pub mod backend;
pub mod definition;
pub mod health;
pub mod logs;
pub mod network;
pub mod oci;
pub mod port_allocator;
pub mod registry;
pub mod templates;

pub use backend::{GvisorBackend, GvisorPlatform, NetworkMode};
pub use definition::{ImageRef, ServiceDefinition, ServiceSpec};
pub use health::{HealthCheck, HealthProbe, HealthStatus, ReadinessProbe};
pub use logs::{LogCollector, LogDestination, LogFormat};
pub use network::{NetworkConfig, PortMapping};
pub use oci::{OciBundle, OciImageManager};
pub use port_allocator::{AllocationStrategy, PortAllocator};
pub use registry::{ServiceMetadata, ServiceRegistry as GvisorServiceRegistry, ServiceState};
pub use templates::ServiceTemplates;
