//! v0.7.0 CLI commands
//!
//! New commands introduced in v0.7.0:
//! - dev: Development mode with file watching
//! - fmt: TOML formatting
//! - dry-run: Shape validation without execution
//! - lint: Linting and static analysis
//! - diff: Trace comparison

pub mod dev;
pub mod dry_run;
pub mod fmt;
pub mod lint;
pub mod diff;
