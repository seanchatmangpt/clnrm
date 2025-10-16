//! Validation module for cleanroom testing framework
//!
//! Provides validation capabilities for test assertions, including
//! OpenTelemetry validation for observability testing.

pub mod otel;

pub use otel::{OtelValidationConfig, OtelValidator, SpanAssertion, TraceAssertion};
