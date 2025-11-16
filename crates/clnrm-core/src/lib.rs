//! Cleanroom Testing Platform - Hermetic Integration Testing
//!
//! A framework for reliable, hermetic integration testing with automatic
//! container lifecycle management and comprehensive observability.
//!
//! This library provides a complete testing platform that tests itself
//! through the "eat your own dog food" principle - the framework validates
//! its own functionality using its own capabilities.

pub mod assertions;
pub mod backend;
pub mod cache;
pub mod capabilities; // v1.7.0: Capability-aware scenario framework
pub mod chaos;
pub mod chicago_tdd; // v1.5.0: Chicago-TDD-Tools integration framework
pub mod cleanroom;
pub mod environment; // v1.7.0: Σ*-aware environment compiler (Phase 2)
pub mod receipts;    // v1.7.0: Test receipt infrastructure (Γₜ) (Phase 3)
pub mod timing;      // v1.7.0: μ-Kernel timing & τ validation (Phase 4)
pub mod synthesis;   // v1.7.0: Scenario synthesis engine (Phase 5)
pub mod scheduler;   // v1.7.0: Swarm-scale scheduler & resource governance (Phase 6)
pub mod phases;      // v1.8.0: Phases 8-10 determinism, conformance, & contracts
pub mod cli;
pub mod config;
pub mod coverage;
pub mod determinism;
pub mod error;
pub mod formatting;
pub mod macros;
pub mod metrics;
pub mod otel;
pub mod policy;
pub mod reporting;
pub mod sbom; // v1.5.0: SBOM generation
pub mod scenario;
pub mod services;
pub mod stress_test;
pub mod telemetry;
pub mod utils;
pub mod validation;
pub mod watch;

// Testing utilities (includes property-based test generators)
pub mod testing;

// Re-export test suite types
pub use testing::{FrameworkTestResults, SuiteResult, TestResult as TestingTestResult};

pub use error::{CleanroomError, Result};
pub use policy::{Policy, SecurityLevel, SecurityPolicy};
pub use scenario::scenario;

pub use telemetry::weaver_controller::{
    ValidationReport as WeaverValidationReport, ValidationStatus, WeaverConfig, WeaverController,
};
pub use telemetry::{Export, OtelConfig, OtelGuard};
// Type-safe Weaver coordination exports
pub use telemetry::weaver_coordination::{
    Running, Stopped, Unstarted, WeaverConfig as TypeSafeWeaverConfig,
    WeaverController as TypeSafeWeaverController,
};

// Phase 8-10 exports (Infrastructure for Determinism, Conformance, & Resource Contracts)
pub use phases::{
    ExecutionOutcome, ReplayMode, ScheduleCertificate, ScheduleLedger, ScheduleLedgerEntry,
    BackendConformanceHarness, BackendConformanceReport, BackendExecutionResult,
    BackendInvariantChecker, EquivalenceViolation, EquivalenceStatus, ExhaustionOutcome,
    ResourceAccountingEntry, ResourceAccountingLedger, ResourceContract, ResourceContractBuilder,
    ResourceContractError, CpuNanos, MemoryBytes, NetworkBytes,
};

pub use assertions::{cache, database, email_service, UserAssertions};
pub use cache::{Cache, CacheManager, CacheStats, FileCache, MemoryCache};
pub use cleanroom::{
    CleanroomEnvironment, ExecutionResult, HealthStatus, ServiceHandle, ServicePlugin,
    ServiceRegistry,
};
pub use config::{
    load_cleanroom_config, load_cleanroom_config_from_file, load_config_from_file,
    parse_toml_config, CleanroomConfig, DeterminismConfig, ScenarioConfig, StepConfig, TestConfig,
};
pub use determinism::DeterminismEngine;
pub use formatting::{
    format_test_results, format_toml_content, format_toml_file, needs_formatting, Formatter,
    FormatterType, HumanFormatter, JsonFormatter, JunitFormatter, TapFormatter, TestResult,
    TestStatus, TestSuite,
};
pub use macros::{with_cache, with_database, with_message_queue, with_web_server};
pub use metrics::{AtomicMetrics, MetricsSnapshot};
pub use reporting::{generate_reports, DigestReporter, JsonReporter, JunitReporter, ReportConfig};
pub use services::generic::GenericContainerPlugin;
pub use services::surrealdb::SurrealDbPlugin;

// Re-export template functionality from clnrm-template
pub use clnrm_template::{
    get_cached_template_renderer, is_template, render_template, render_template_file,
    DeterminismConfig as TemplateDeterminismConfig, TemplateContext, TemplateError,
    TemplateRenderer,
};

pub use validation::otel::{OtelValidationConfig, OtelValidator, SpanAssertion, TraceAssertion};
pub use validation::{PrdExpectations, ShapeValidator, ValidationReport};
pub use watch::{debouncer::FileDebouncer, WatchConfig};

// Coverage tracking and reporting
pub use coverage::manifest::{BehaviorManifest, Dimensions, SystemInfo};
pub use coverage::report::{ReportFormat, ReportGenerator};
pub use coverage::tracker::CoverageTracker;
pub use coverage::{
    BehaviorCoverage, BehaviorCoverageReport, DimensionCoverage, DimensionWeights, StateTransition,
    UncoveredBehaviors,
};

// The cleanroom_test macro is already exported via #[macro_export] in macros.rs

/// Result of a cleanroom run
#[derive(Debug)]
pub struct RunResult {
    pub success: bool,
    pub duration_ms: u64,
    pub output: String,
    pub error: Option<String>,
}
