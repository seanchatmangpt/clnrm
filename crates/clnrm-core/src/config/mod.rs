//! Configuration system for cleanroom testing
//!
//! Provides TOML-based configuration parsing and validation for test files
//! and cleanroom environment settings.
//!
//! This module is organized into several submodules:
//! - `types` - Core configuration structures (TestConfig, MetaConfig, etc.)
//! - `services` - Service and volume configurations
//! - `otel` - OpenTelemetry-related structures
//! - `project` - Project-level cleanroom configuration
//! - `loader` - File loading and parsing functions
//! - `deserializers` - Custom serde deserializers

pub mod deserializers;
pub mod loader;
pub mod otel;
pub mod project;
pub mod services;
pub mod types;

// Re-export commonly used types for backward compatibility
pub use types::{
    ArtifactsConfig, DeterminismConfig, LimitsConfig, MetaConfig, PolicyConfig, ReportConfig,
    ScenarioConfig, StepConfig, TestConfig, TestMetadata, TestMetadataSection, TimeoutConfig,
};

pub use services::{HealthCheckConfig, ServiceConfig, VolumeConfig};

pub use otel::{
    CountBoundConfig, CountExpectationConfig, DurationBoundConfig, ExpectationsConfig,
    ExpectedSpanConfig, ExpectedTraceConfig, GraphExpectationConfig, HermeticityExpectationConfig,
    OrderExpectationConfig, OtelConfig, OtelHeadersConfig, OtelPropagatorsConfig,
    OtelValidationSection, ResourceAttrsConfig, SpanAttributesConfig, SpanAttrsConfig,
    SpanEventsConfig, SpanExpectationConfig, StatusExpectationConfig, WindowExpectationConfig,
};

pub use project::{
    load_cleanroom_config, load_cleanroom_config_from_file, CleanroomConfig, CliConfig,
    ContainerConfig, ObservabilityConfig, PerformanceConfig, PluginConfig, ProjectConfig,
    ReportingConfig, SecurityConfig, ServiceDefaultsConfig, TestExecutionConfig,
};

pub use loader::{load_config_from_file, parse_toml_config};
