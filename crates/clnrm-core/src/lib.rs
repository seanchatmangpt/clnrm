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
pub mod cleanroom;
pub mod cli;
pub mod config;
pub mod coverage;
pub mod determinism;
pub mod error;
pub mod formatting;
pub mod macros;
pub mod otel;
pub mod policy;
pub mod reporting;
pub mod scenario;
pub mod services;
pub mod telemetry;
pub mod template;
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

pub use telemetry::{Export, OtelConfig, OtelGuard};

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
pub use reporting::{generate_reports, DigestReporter, JsonReporter, JunitReporter, ReportConfig};
pub use services::generic::GenericContainerPlugin;
pub use services::surrealdb::SurrealDbPlugin;
pub use template::{TemplateContext, TemplateRenderer};
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
