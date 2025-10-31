//! Mock Implementations for London TDD
//!
//! This module provides mock implementations of external dependencies,
//! designed from schema contracts rather than implementations.

pub mod weaver_process;
pub mod docker_backend;
pub mod otel_exporter;
pub mod port_discovery;

pub use weaver_process::*;
pub use docker_backend::*;
pub use otel_exporter::*;
pub use port_discovery::*;
