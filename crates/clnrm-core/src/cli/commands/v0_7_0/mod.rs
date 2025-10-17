//! v0.7.0 CLI commands
//!
//! New commands introduced in v0.7.0:
//! - dev: Development mode with file watching
//! - fmt: TOML formatting
//! - dry-run: Shape validation without execution
//! - lint: Linting and static analysis
//! - diff: Trace comparison
//! - record: Baseline recording for test runs
//!
//! PRD v1.0 additional commands:
//! - pull: Image pre-pulling (implemented)
//! - graph: Trace visualization (fully implemented)
//! - repro: Baseline reproduction (implemented)
//! - redgreen: TDD validation (FULLY IMPLEMENTED)
//! - render: Template rendering (implemented)
//! - spans: Span filtering (IMPLEMENTED)
//! - collector: OTEL collector management (stub)

pub mod collector;
pub mod dev;
pub mod diff;
pub mod dry_run;
pub mod fmt;
pub mod graph;
pub mod lint;
pub mod prd_commands;
pub mod pull;
pub mod record;
pub mod redgreen;
pub mod redgreen_impl;
pub mod repro;
pub mod spans;

