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
//! PRD v1.0 additional commands (stubs):
//! - pull: Image pre-pulling
//! - graph: Trace visualization
//! - repro: Baseline reproduction
//! - redgreen: TDD validation
//! - render: Template rendering
//! - spans: Span filtering
//! - collector: OTEL collector management

pub mod dev;
pub mod dry_run;
pub mod fmt;
pub mod lint;
pub mod diff;
pub mod record;
pub mod prd_commands;
