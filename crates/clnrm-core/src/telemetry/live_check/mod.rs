//! Live check support for OpenTelemetry Weaver validation
//!
//! This module provides infrastructure for running live checks against
//! OpenTelemetry schemas using the Weaver validator.

pub mod config;
pub mod orchestrator;
pub mod validation;
pub mod weaver_manager;

// Re-export key types for convenience
pub use config::{
    AttributeCriticality, Complete80_20Config, CoverageThresholds, CriticalSpan,
    EightyTwentyConfig, ValidationConfig, ValidationMode,
};
pub use orchestrator::{
    FallbackMode, GracefulFallbackResult, LiveCheckGuard, LiveCheckOrchestrator,
    OrchestrationMode, run_with_graceful_fallback, Completed, Uninitialized, WeaverRunning,
};
pub use validation::{
    ConformanceReport, ConformanceValidator, CoverageBreakdown, ValidationResult, Violation,
};
pub use weaver_manager::{WeaverProcessManager, WeaverPorts};
