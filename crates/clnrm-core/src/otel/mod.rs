//! OpenTelemetry integration for fake-green detection
//!
//! This module provides comprehensive OTEL validation to detect "fake-green" tests
//! (tests that report success without actually executing containers).
//!
//! ## Fake-Green Detection
//!
//! A fake-green test is one that:
//! - Reports success without creating expected spans
//! - Has incorrect parent-child span relationships
//! - Has insufficient span/event counts
//! - Has incorrect temporal ordering
//! - Shows timing anomalies
//! - Reports wrong status codes
//! - Violates hermetic execution
//!
//! ## Validators
//!
//! This module provides 7 production-ready validators:
//!
//! 1. **span** - Individual span validation (name, parent, kind, attrs, events, duration)
//! 2. **graph** - Graph structure validation (must_include, must_not_cross, acyclic)
//! 3. **counts** - Cardinality validation (spans_total, events_total, errors_total, by_name)
//! 4. **window** - Temporal containment validation (outer contains inner)
//! 5. **order** - Temporal ordering validation (must_precede, must_follow)
//! 6. **status** - Status code validation (all, by_name)
//! 7. **hermeticity** - Hermetic execution validation (no external services)
//!
//! ## Usage Example
//!
//! ### Parsing Spans from Container Stdout
//!
//! ```rust
//! use clnrm_core::otel::StdoutSpanParser;
//!
//! // Container stdout containing OTEL spans mixed with logs
//! let stdout = r#"
//! Starting test...
//! {"name":"clnrm.run","trace_id":"abc123","span_id":"s1","parent_span_id":null,"attributes":{"result":"pass"}}
//! Container created: alpine:latest
//! {"name":"clnrm.step:setup","trace_id":"abc123","span_id":"s2","parent_span_id":"s1","events":["container.start"]}
//! Test completed
//! "#;
//!
//! // Parse spans from stdout
//! let spans = StdoutSpanParser::parse(stdout)?;
//! assert_eq!(spans.len(), 2);
//! assert_eq!(spans[0].name, "clnrm.run");
//! ```
//!
//! ### Validating Spans for Fake-Green Detection
//!
//! ```rust
//! use clnrm_core::otel::validators;
//! use clnrm_core::validation::span_validator::SpanValidator;
//!
//! // Load spans from OTEL collector export
//! let validator = SpanValidator::from_file("spans.json")?;
//! let spans = validator.spans();
//!
//! // Validate span creation
//! let span_expectation = validators::SpanExpectation::new("container.start");
//! let result = span_expectation.validate(spans, &span_by_id)?;
//! assert!(result.passed, "Fake-green detected: container never started");
//!
//! // Validate graph structure
//! let graph_expectation = validators::GraphExpectation::new(vec![
//!     ("container.start".into(), "container.exec".into()),
//! ]);
//! let result = graph_expectation.validate(spans)?;
//! assert!(result.passed, "Fake-green detected: exec never ran as child of start");
//!
//! // Validate counts
//! let count_expectation = validators::CountExpectation::new()
//!     .with_spans_total(validators::CountBound::gte(3));
//! let result = count_expectation.validate(spans)?;
//! assert!(result.passed, "Fake-green detected: insufficient spans created");
//! ```
//!
//! ## Core Team Standards
//!
//! All validators follow FAANG-level standards:
//! - NO .unwrap() or .expect() in production code
//! - Return Result<ValidationResult, CleanroomError>
//! - Sync functions (dyn compatible)
//! - Comprehensive unit tests (AAA pattern)
//! - Descriptive error messages

pub mod stdout_parser;
pub mod validators;

// Re-export validators for convenience
pub use validators::{
    counts, graph, hermeticity, order, span, status, window, CountBound, CountExpectation,
    GraphExpectation, HermeticityExpectation, OrderExpectation, SpanExpectation, StatusCode,
    StatusExpectation, ViolationType, WindowExpectation,
};

// Re-export stdout parser for convenience
pub use stdout_parser::StdoutSpanParser;
