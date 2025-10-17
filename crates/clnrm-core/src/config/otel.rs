//! OpenTelemetry configuration types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OTEL configuration (v0.6.0)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OtelConfig {
    /// OTEL exporter type
    pub exporter: String,
    /// Sample ratio
    pub sample_ratio: Option<f64>,
    /// Resource attributes
    pub resources: Option<HashMap<String, String>>,
    /// OTEL headers
    pub headers: Option<HashMap<String, String>>,
    /// OTEL propagators
    pub propagators: Option<OtelPropagatorsConfig>,
}

/// Expectations configuration (v0.6.0)
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ExpectationsConfig {
    /// Span expectations
    #[serde(default)]
    pub span: Vec<SpanExpectationConfig>,
    /// Order expectations
    #[serde(default)]
    pub order: Option<OrderExpectationConfig>,
    /// Status expectations
    #[serde(default)]
    pub status: Option<StatusExpectationConfig>,
    /// Count expectations
    #[serde(default)]
    pub counts: Option<CountExpectationConfig>,
    /// Window expectations
    #[serde(default)]
    pub window: Vec<WindowExpectationConfig>,
    /// Graph expectations
    #[serde(default)]
    pub graph: Option<GraphExpectationConfig>,
    /// Hermeticity expectations
    #[serde(default)]
    pub hermeticity: Option<HermeticityExpectationConfig>,
}

/// Span expectation configuration (v0.6.0)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SpanExpectationConfig {
    /// Span name (can be glob pattern)
    pub name: String,
    /// Span kind
    pub kind: Option<String>,
    /// Attribute expectations
    pub attrs: Option<SpanAttributesConfig>,
}

/// Span attributes configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SpanAttributesConfig {
    /// All attributes must match
    pub all: Option<HashMap<String, String>>,
    /// Any attribute must match
    pub any: Option<HashMap<String, String>>,
}

/// OpenTelemetry validation section in TOML
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OtelValidationSection {
    /// Enable OTEL validation
    pub enabled: bool,
    /// Validate spans
    #[serde(default)]
    pub validate_spans: Option<bool>,
    /// Validate traces
    #[serde(default)]
    pub validate_traces: Option<bool>,
    /// Validate exports
    #[serde(default)]
    pub validate_exports: Option<bool>,
    /// Validate performance overhead
    #[serde(default)]
    pub validate_performance: Option<bool>,
    /// Maximum allowed performance overhead in milliseconds
    #[serde(default)]
    pub max_overhead_ms: Option<f64>,
    /// Expected spans configuration
    #[serde(default)]
    pub expected_spans: Option<Vec<ExpectedSpanConfig>>,
    /// Expected traces configuration
    #[serde(default)]
    pub expected_traces: Option<Vec<ExpectedTraceConfig>>,
    /// Graph topology expectations
    #[serde(default)]
    pub expect_graph: Option<GraphExpectationConfig>,
    /// Count/cardinality expectations
    #[serde(default)]
    pub expect_counts: Option<CountExpectationConfig>,
    /// Temporal window expectations
    #[serde(default)]
    pub expect_windows: Option<Vec<WindowExpectationConfig>>,
    /// Hermeticity expectations
    #[serde(default)]
    pub expect_hermeticity: Option<HermeticityExpectationConfig>,
    /// Temporal ordering expectations (v0.6.0)
    #[serde(default)]
    pub expect_order: Option<OrderExpectationConfig>,
    /// Status code expectations (v0.6.0)
    #[serde(default)]
    pub expect_status: Option<StatusExpectationConfig>,
}

/// Expected span configuration from TOML
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ExpectedSpanConfig {
    /// Span name (operation name)
    pub name: String,
    /// Expected attributes
    pub attributes: Option<HashMap<String, String>>,
    /// Whether span is required
    pub required: Option<bool>,
    /// Minimum duration in milliseconds
    pub min_duration_ms: Option<f64>,
    /// Maximum duration in milliseconds
    pub max_duration_ms: Option<f64>,
}

/// Expected trace configuration from TOML
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ExpectedTraceConfig {
    /// Trace ID (optional, for specific trace validation)
    pub trace_id: Option<String>,
    /// Expected span names in the trace
    pub span_names: Vec<String>,
    /// Whether all spans must be present
    pub complete: Option<bool>,
    /// Parent-child relationships (parent_name -> child_name)
    pub parent_child: Option<Vec<(String, String)>>,
}

/// Graph topology expectation from TOML
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GraphExpectationConfig {
    /// Edges that must be present in the span graph (parent, child)
    pub must_include: Vec<(String, String)>,
    /// Edges that must not exist in the span graph (forbidden crossings)
    #[serde(default)]
    pub must_not_cross: Option<Vec<(String, String)>>,
    /// Whether the graph must be acyclic
    #[serde(default)]
    pub acyclic: Option<bool>,
}

/// Count bound configuration for cardinality expectations
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CountBoundConfig {
    /// Greater than or equal to (>=)
    #[serde(default)]
    pub gte: Option<usize>,
    /// Less than or equal to (<=)
    #[serde(default)]
    pub lte: Option<usize>,
    /// Equal to (==)
    #[serde(default)]
    pub eq: Option<usize>,
}

/// Count expectations from TOML for span cardinalities
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CountExpectationConfig {
    /// Total span count bounds
    #[serde(default)]
    pub spans_total: Option<CountBoundConfig>,
    /// Total event count bounds
    #[serde(default)]
    pub events_total: Option<CountBoundConfig>,
    /// Total error count bounds
    #[serde(default)]
    pub errors_total: Option<CountBoundConfig>,
    /// Per-span-name count bounds
    #[serde(default)]
    pub by_name: Option<HashMap<String, CountBoundConfig>>,
}

/// Temporal window expectation from TOML
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WindowExpectationConfig {
    /// Outer span name that defines the temporal window
    pub outer: String,
    /// Span names that must be temporally contained within the outer span
    pub contains: Vec<String>,
}

/// Hermeticity expectation from TOML
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HermeticityExpectationConfig {
    /// Whether external service calls are forbidden
    #[serde(default)]
    pub no_external_services: Option<bool>,
    /// Resource attributes that must match exactly
    #[serde(default)]
    pub resource_attrs_must_match: Option<HashMap<String, String>>,
    /// Span attribute keys that are forbidden (e.g., "net.peer.name")
    #[serde(default)]
    pub span_attrs_forbid_keys: Option<Vec<String>>,
}

/// Temporal ordering expectations (v0.6.0)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OrderExpectationConfig {
    /// Edges where first must temporally precede second
    #[serde(default)]
    pub must_precede: Option<Vec<(String, String)>>,
    /// Edges where first must temporally follow second
    #[serde(default)]
    pub must_follow: Option<Vec<(String, String)>>,
}

/// Status code expectations (v0.6.0)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StatusExpectationConfig {
    /// Expected status for all spans ("OK", "ERROR", "UNSET")
    #[serde(default)]
    pub all: Option<String>,
    /// Expected status by span name pattern (supports globs)
    #[serde(default)]
    pub by_name: Option<HashMap<String, String>>,
}

/// OTEL headers configuration (v0.6.0)
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct OtelHeadersConfig {
    /// Custom OTLP headers (e.g., Authorization)
    #[serde(flatten)]
    pub headers: HashMap<String, String>,
}

/// OTEL propagators configuration (v0.6.0)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OtelPropagatorsConfig {
    /// Propagators to use (e.g., ["tracecontext", "baggage"])
    pub r#use: Vec<String>,
}
