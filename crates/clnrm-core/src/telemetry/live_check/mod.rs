//! Live check support for OpenTelemetry Weaver validation
//!
//! This module provides infrastructure for running live checks against
//! OpenTelemetry schemas using the Weaver validator.

pub mod config;
pub mod diagnostics;
pub mod orchestrator;
pub mod stop_coordinator;
pub mod validation;
pub mod weaver_manager;

// Re-export key types for convenience
pub use config::{
    AttributeCriticality, Complete80_20Config, CoverageThresholds, CriticalSpan,
    EightyTwentyConfig, ValidationConfig, ValidationMode,
};
pub use diagnostics::{
    detect_format, AnsiConfig, AnsiFormatter, AttributeValidation,
    ConformanceReport as DiagnosticConformanceReport, DiagnosticConfig, DiagnosticFormat,
    DiagnosticFormatter, DiagnosticProcessor, EnvironmentInfo, GithubConfig,
    GithubWorkflowFormatter, JsonConfig, JsonFormatter, SpanValidation,
    ValidationStatus as DiagnosticValidationStatus, Violation as DiagnosticViolation,
};
pub use orchestrator::{
    run_with_graceful_fallback, Completed, FallbackMode, GracefulFallbackResult, LiveCheckConfig,
    LiveCheckGuard, LiveCheckOrchestrator, OrchestrationMode, Uninitialized, ValidationDetail,
    ValidationReport, ValidationStatus, WeaverRunning,
};
pub use stop_coordinator::{StopConfig, StopCoordinator, StopReason};
pub use validation::{
    ConformanceReport, ConformanceValidator, CoverageBreakdown, ValidationResult, Violation,
};
pub use weaver_manager::{WeaverPorts, WeaverProcessManager};
