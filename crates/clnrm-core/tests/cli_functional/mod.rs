//! CLI Functional Tests
//!
//! Comprehensive end-to-end testing of all CLI commands following core team best practices:
//! - AAA Pattern (Arrange, Act, Assert)
//! - Behavior-focused testing
//! - Proper async handling
//! - No false positives (verify actual work)

mod helpers;

pub mod file_ops;
pub mod execution;
pub mod trace_analysis;
pub mod baseline;
pub mod services;
pub mod reporting;
pub mod dev;

