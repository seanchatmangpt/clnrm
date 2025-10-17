//! OpenTelemetry configuration types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OTEL configuration (v0.6.0 - v1.0)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OtelConfig {
    /// OTEL exporter type (e.g., "stdout", "otlp")
    pub exporter: String,
    /// OTLP endpoint URL (e.g., "http://localhost:4318")
    #[serde(default)]
    pub endpoint: Option<String>,
    /// Protocol to use (e.g., "http/protobuf", "grpc")
    #[serde(default)]
    pub protocol: Option<String>,
    /// Sample ratio (0.0 to 1.0)
    #[serde(default)]
    pub sample_ratio: Option<f64>,
    /// Resource attributes
    #[serde(default)]
    pub resources: Option<HashMap<String, String>>,
    /// OTEL headers
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
    /// OTEL propagators
    #[serde(default)]
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

/// Span expectation configuration (v0.6.0 - v1.0)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SpanExpectationConfig {
    /// Span name (can be glob pattern)
    pub name: String,
    /// Parent span name
    #[serde(default)]
    pub parent: Option<String>,
    /// Span kind (e.g., "internal", "client", "server")
    #[serde(default)]
    pub kind: Option<String>,
    /// Attribute expectations
    #[serde(default)]
    pub attrs: Option<SpanAttributesConfig>,
    /// Event expectations
    #[serde(default)]
    pub events: Option<SpanEventsConfig>,
    /// Duration expectations
    #[serde(default)]
    pub duration_ms: Option<DurationBoundConfig>,
}

/// Span events configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SpanEventsConfig {
    /// Any of these events must be present
    #[serde(default)]
    pub any: Option<Vec<String>>,
    /// All of these events must be present
    #[serde(default)]
    pub all: Option<Vec<String>>,
}

/// Duration bound configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DurationBoundConfig {
    /// Minimum duration in milliseconds
    #[serde(default)]
    pub min: Option<f64>,
    /// Maximum duration in milliseconds
    #[serde(default)]
    pub max: Option<f64>,
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

/// Graph topology expectation from TOML (v1.0 schema)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GraphExpectationConfig {
    /// Edges that must be present in the span graph (parent, child)
    /// Format: [["parent", "child"], ...]
    #[serde(default)]
    pub must_include: Option<Vec<Vec<String>>>,
    /// Edges that must not exist in the span graph (forbidden crossings)
    /// Format: [["a", "b"], ...]
    #[serde(default)]
    pub must_not_cross: Option<Vec<Vec<String>>>,
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

/// Hermeticity expectation from TOML (v1.0 schema)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HermeticityExpectationConfig {
    /// Whether external service calls are forbidden
    #[serde(default)]
    pub no_external_services: Option<bool>,
    /// Resource attributes that must match exactly
    #[serde(default, alias = "resource_attrs_must_match")]
    pub resource_attrs: Option<ResourceAttrsConfig>,
    /// Span attribute keys that are forbidden (e.g., "net.peer.name")
    #[serde(default, alias = "span_attrs_forbid_keys")]
    pub span_attrs: Option<SpanAttrsConfig>,
}

/// Resource attributes configuration for hermeticity validation
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResourceAttrsConfig {
    /// Attributes that must match exactly
    #[serde(default)]
    pub must_match: Option<HashMap<String, String>>,
}

/// Span attributes configuration for hermeticity validation
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SpanAttrsConfig {
    /// Attribute keys that are forbidden
    #[serde(default)]
    pub forbid_keys: Option<Vec<String>>,
}

/// Temporal ordering expectations (v1.0 schema)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OrderExpectationConfig {
    /// Edges where first must temporally precede second
    /// Format: [["first", "second"], ...]
    #[serde(default)]
    pub must_precede: Option<Vec<Vec<String>>>,
    /// Edges where first must temporally follow second
    /// Format: [["first", "second"], ...]
    #[serde(default)]
    pub must_follow: Option<Vec<Vec<String>>>,
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

impl OtelConfig {
    /// Validate the OTEL configuration
    pub fn validate(&self) -> crate::error::Result<()> {
        // Validate exporter type
        match self.exporter.to_lowercase().as_str() {
            "stdout" | "otlp" | "jaeger" | "zipkin" => {}
            _ => {
                return Err(crate::error::CleanroomError::validation_error(format!(
                    "Invalid exporter type '{}'. Must be one of: stdout, otlp, jaeger, zipkin",
                    self.exporter
                )))
            }
        }

        // Validate sample ratio if present
        if let Some(ratio) = self.sample_ratio {
            if !(0.0..=1.0).contains(&ratio) {
                return Err(crate::error::CleanroomError::validation_error(
                    "Sample ratio must be between 0.0 and 1.0",
                ));
            }
        }

        // Validate protocol if present
        if let Some(ref protocol) = self.protocol {
            match protocol.to_lowercase().as_str() {
                "http/protobuf" | "grpc" | "http/json" => {}
                _ => {
                    return Err(crate::error::CleanroomError::validation_error(format!(
                        "Invalid protocol '{}'. Must be one of: http/protobuf, grpc, http/json",
                        protocol
                    )))
                }
            }
        }

        Ok(())
    }
}

impl GraphExpectationConfig {
    /// Validate the graph expectation configuration
    pub fn validate(&self) -> crate::error::Result<()> {
        // Validate must_include edges
        if let Some(ref edges) = self.must_include {
            for edge in edges {
                if edge.len() != 2 {
                    return Err(crate::error::CleanroomError::validation_error(
                        "Graph edges must have exactly 2 elements [parent, child]",
                    ));
                }
                if edge[0].is_empty() || edge[1].is_empty() {
                    return Err(crate::error::CleanroomError::validation_error(
                        "Graph edge names cannot be empty",
                    ));
                }
            }
        }

        // Validate must_not_cross edges
        if let Some(ref edges) = self.must_not_cross {
            for edge in edges {
                if edge.len() != 2 {
                    return Err(crate::error::CleanroomError::validation_error(
                        "Graph edges must have exactly 2 elements [source, target]",
                    ));
                }
                if edge[0].is_empty() || edge[1].is_empty() {
                    return Err(crate::error::CleanroomError::validation_error(
                        "Graph edge names cannot be empty",
                    ));
                }
            }
        }

        Ok(())
    }
}

impl StatusExpectationConfig {
    /// Validate the status expectation configuration
    pub fn validate(&self) -> crate::error::Result<()> {
        // Validate 'all' status if present
        if let Some(ref status) = self.all {
            Self::validate_status_value(status)?;
        }

        // Validate 'by_name' statuses if present
        if let Some(ref by_name) = self.by_name {
            for (span_name, status) in by_name {
                if span_name.is_empty() {
                    return Err(crate::error::CleanroomError::validation_error(
                        "Span name in status expectations cannot be empty",
                    ));
                }
                Self::validate_status_value(status)?;
            }
        }

        Ok(())
    }

    fn validate_status_value(status: &str) -> crate::error::Result<()> {
        match status.to_uppercase().as_str() {
            "OK" | "ERROR" | "UNSET" => Ok(()),
            _ => Err(crate::error::CleanroomError::validation_error(format!(
                "Invalid status value '{}'. Must be one of: OK, ERROR, UNSET",
                status
            ))),
        }
    }
}
