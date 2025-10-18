//! Validation module for cleanroom testing framework
//!
//! Provides validation capabilities for test assertions, including
//! OpenTelemetry validation for observability testing.

pub mod common;
pub mod count_validator;
pub mod graph_validator;
pub mod hermeticity_validator;
pub mod orchestrator;
pub mod order_validator;
pub mod otel;
pub mod shape;
pub mod span_validator;
pub mod status_validator;
pub mod window_validator;

pub use count_validator::{CountBound, CountExpectation};
pub use graph_validator::{GraphExpectation, GraphValidator};
pub use hermeticity_validator::{
    HermeticityExpectation, HermeticityValidator, HermeticityViolation, ViolationType,
};
pub use orchestrator::{PrdExpectations, ValidationReport};
pub use order_validator::OrderExpectation;
pub use otel::{
    OtelValidationConfig, OtelValidator, SpanAssertion as OtelSpanAssertion, TraceAssertion,
    ValidationSpanProcessor, SpanValidationResult, TraceValidationResult,
};
pub use shape::{ErrorCategory, ShapeValidationError, ShapeValidationResult, ShapeValidator};
pub use span_validator::{
    FailureDetails, SpanAssertion, SpanData, SpanKind, SpanValidator, ValidationResult,
};
pub use status_validator::{StatusCode, StatusExpectation};
pub use window_validator::{WindowExpectation, WindowValidator};
