//! Validation module for cleanroom testing framework
//!
//! Provides validation capabilities for test assertions, including
//! OpenTelemetry validation for observability testing.

pub mod count_validator;
pub mod graph_validator;
pub mod hermeticity_validator;
pub mod orchestrator;
pub mod otel;
pub mod span_validator;
pub mod window_validator;

pub use count_validator::CountExpectation;
pub use graph_validator::{GraphExpectation, GraphValidator};
pub use hermeticity_validator::{HermeticityExpectation, HermeticityValidator, HermeticityViolation, ViolationType};
pub use orchestrator::{PrdExpectations, ValidationReport};
pub use otel::{OtelValidationConfig, OtelValidator, SpanAssertion as OtelSpanAssertion, TraceAssertion};
pub use span_validator::{SpanAssertion, SpanData, SpanKind, SpanValidator};
pub use window_validator::{WindowExpectation, WindowValidator};
