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
pub mod cli; // CLI types and utilities (commands moved to clnrm-cli)
pub mod config;
pub mod constants;
pub mod coverage;
pub mod dao;
pub mod determinism;
pub mod environment; // v1.7.0: Σ*-aware environment compiler (Phase 2)
pub mod error;
pub mod executor; // v2.0: New executor with docker exec support
pub mod formatting;
pub mod macros;
pub mod market;
pub mod metrics;
pub mod otel;
pub mod phases; // v1.8.0: Phases 8-10 determinism, conformance, & contracts
pub mod policy;
pub mod pqc;
pub mod template; // Template generation functions
pub mod poka_yoke {
    //! Poka-Yoke (Error-Proofing) Mechanisms
    //!
    //! Trait-based abstractions for error-proofing mechanisms that prevent
    //! the highest-priority failure modes identified in the FMEA audit.
    pub mod globals;
    pub mod impls;
    pub mod traits;

    // Re-export for convenience
    pub use globals::*;
    pub use impls::*;
    pub use traits::*;
}
pub mod receipts; // v1.7.0: Test receipt infrastructure (Γₜ) (Phase 3)
pub mod reporting;
pub mod sbom; // v1.5.0: SBOM generation
pub mod scenario;
pub mod scheduler; // v1.7.0: Swarm-scale scheduler & resource governance (Phase 6)
pub mod service;
pub mod services;
pub mod stress_test;
pub mod synthesis; // v1.7.0: Scenario synthesis engine (Phase 5)
pub mod telemetry;
pub mod timing; // v1.7.0: μ-Kernel timing & τ validation (Phase 4)
pub mod truex;
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
    BackendConformanceHarness, BackendConformanceReport, BackendExecutionResult,
    BackendInvariantChecker, CpuNanos, EquivalenceStatus, EquivalenceViolation, ExecutionOutcome,
    ExhaustionOutcome, MemoryBytes, NetworkBytes, ReplayMode, ResourceAccountingEntry,
    ResourceAccountingLedger, ResourceContract, ResourceContractBuilder, ResourceContractError,
    ScheduleCertificate, ScheduleLedger, ScheduleLedgerEntry,
};

pub use assertions::{cache, database, email_service, UserAssertions};
pub use cache::{Cache, CacheManager, CacheStats, FileCache, MemoryCache};
pub use cleanroom::{
    CleanroomEnvironment, ExecutionResult, HealthStatus, ServiceHandle, ServicePlugin,
    ServiceRegistry,
};
pub use config::{
    load_cleanroom_config, load_cleanroom_config_from_file, load_config_from_file,
    parse_toml_config, CleanroomConfig, ContainerConfig, DeterminismConfig, ScenarioConfig,
    StepConfig, TestConfig,
};

// Type safety improvements - newtypes for IDs and counts
pub mod types {
    //! Type-safe wrappers for common types to prevent ID mixups and improve safety

    use serde::{Deserialize, Serialize};
    use std::fmt;

    /// A container identifier with type safety
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct ContainerId(pub String);

    impl ContainerId {
        pub fn new(id: impl Into<String>) -> Self {
            Self(id.into())
        }

        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl fmt::Display for ContainerId {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl From<String> for ContainerId {
        fn from(s: String) -> Self {
            Self(s)
        }
    }

    impl From<&str> for ContainerId {
        fn from(s: &str) -> Self {
            Self(s.to_string())
        }
    }

    /// A test count with type safety to prevent mixing up different kinds of counts
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
    pub struct TestCount(pub u64);

    impl TestCount {
        pub fn new(count: u64) -> Self {
            Self(count)
        }

        pub fn as_u64(&self) -> u64 {
            self.0
        }
    }

    impl fmt::Display for TestCount {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl From<u64> for TestCount {
        fn from(n: u64) -> Self {
            Self(n)
        }
    }

    impl From<usize> for TestCount {
        fn from(n: usize) -> Self {
            Self(n as u64)
        }
    }

    /// A step identifier with type safety
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct StepId(pub String);

    impl StepId {
        pub fn new(id: impl Into<String>) -> Self {
            Self(id.into())
        }

        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl fmt::Display for StepId {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl From<String> for StepId {
        fn from(s: String) -> Self {
            Self(s)
        }
    }

    impl From<&str> for StepId {
        fn from(s: &str) -> Self {
            Self(s.to_string())
        }
    }
}
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

// v2.0 Executor exports
pub use executor::{
    ContainerHandle, ContainerManager, DockerContainerManager, ExecutionResult as ExecutorResult,
    StepResult, TestRunner,
};

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
