//! OTEL validators for fake-green detection
//!
//! This module provides 7 production-ready validators that detect "fake-green" tests
//! (tests that report success without actually executing containers).
//!
//! ## Validators
//!
//! 1. **span** - Validates individual span expectations (name, parent, kind, attrs, events, duration)
//! 2. **graph** - Validates span graph structure (must_include, must_not_cross, acyclic)
//! 3. **counts** - Validates span counts (spans_total, events_total, errors_total, by_name)
//! 4. **window** - Validates temporal containment (outer span contains inner spans)
//! 5. **order** - Validates temporal ordering (must_precede, must_follow)
//! 6. **status** - Validates span status codes (all, by_name)
//! 7. **hermeticity** - Validates hermetic execution (no_external_services, resource_attrs, span_attrs)
//!
//! ## Usage
//!
//! ```rust
//! use clnrm_core::otel::validators::{span, graph, counts};
//! use clnrm_core::validation::span_validator::SpanData;
//!
//! // Create span expectation
//! let expectation = span::SpanExpectation::new("container.start")
//!     .with_kind(span::SpanKind::Internal);
//!
//! // Validate against spans
//! let spans: Vec<SpanData> = load_spans();
//! let result = expectation.validate(&spans, &span_by_id)?;
//!
//! assert!(result.passed);
//! ```
//!
//! ## Core Team Standards
//!
//! All validators follow production standards:
//! - NO .unwrap() or .expect() in production code
//! - Return Result<ValidationResult, CleanroomError>
//! - Sync functions (dyn compatible)
//! - Comprehensive unit tests (AAA pattern)
//! - Descriptive error messages with fake-green indicators

pub mod counts;
pub mod graph;
pub mod hermeticity;
pub mod order;
pub mod span;
pub mod status;
pub mod window;

// Re-export key types for convenience
pub use counts::{CountBound, CountExpectation};
pub use graph::GraphExpectation;
pub use hermeticity::{HermeticityExpectation, ViolationType};
pub use order::OrderExpectation;
pub use span::SpanExpectation;
pub use status::{StatusCode, StatusExpectation};
pub use window::WindowExpectation;
