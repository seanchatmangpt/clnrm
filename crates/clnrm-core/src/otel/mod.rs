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
//! ## OpenTelemetry Integration
//!
//! This module provides OpenTelemetry integration for fake-green detection,
//! focusing on span parsing and processing capabilities.
//!
//! ## Core Functionality
//!
//! - **Span Parsing**: Extract OTEL spans from container stdout
//! - **Validation Integration**: Works with the main validation system in `/validation/`
//! - **Fake-Green Detection**: Identifies tests that report success without actual execution
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
//! let spans = StdoutSpanParser::parse(stdout).unwrap(); // OK: doc example
//! assert_eq!(spans.len(), 2);
//! assert_eq!(spans[0].name, "clnrm.run");
//! ```
//!
//! ### Integration with Validation System
//!
//! The OTEL module integrates with the main validation system:
//!
//! ```rust
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use clnrm_core::otel::StdoutSpanParser;
//! use clnrm_core::validation::{PrdExpectations, ValidationReport};
//!
//! # let container_stdout = String::from(r#"{"name":"clnrm.run","trace_id":"abc123","span_id":"s1","parent_span_id":null,"attributes":{"result":"pass"}}"#);
//! // Parse spans from container output
//! let spans = StdoutSpanParser::parse(&container_stdout)?;
//!
//! // Use main validation system
//! let expectations = PrdExpectations::new();
//! let report = expectations.validate_all(&spans)?;
//!
//! if report.is_success() {
//!     println!("✅ All validations passed");
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Architecture
//!
//! The OTEL module provides span parsing capabilities that feed into the main
//! validation system located in `/validation/`. This separation ensures:
//! - Clean separation of concerns
//! - Reusable span parsing logic
//! - Integration with the comprehensive validation framework

pub mod stdout_parser;

// Re-export stdout parser for convenience
pub use stdout_parser::StdoutSpanParser;
