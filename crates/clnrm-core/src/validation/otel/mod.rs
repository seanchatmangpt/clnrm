//! OpenTelemetry validation module
//!
//! This module provides comprehensive validation of OpenTelemetry instrumentation,
//! following the TTBD (Test That Backs Documentation) philosophy - ensuring that
//! observability claims are backed by verifiable telemetry data.

pub mod assertions;
pub mod config;
pub mod results;
pub mod span_processor;
pub mod validator;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use assertions::{SpanAssertion, TraceAssertion, span_assertion_from_toml, trace_assertion_from_toml};
pub use config::OtelValidationConfig;
pub use results::{SpanValidationResult, TraceValidationResult};
pub use span_processor::ValidationSpanProcessor;
pub use validator::OtelValidator;
