//! clap-noun-verb - A framework for building composable CLI patterns
//!
//! This crate provides a high-level, ergonomic API for building noun-verb CLI patterns
//! on top of clap, similar to how Python's Typer provides a simpler interface over Click.
//!
//! ## Framework Philosophy
//!
//! Instead of providing specific compositions, this crate provides a framework that allows
//! users to compose their own CLI patterns. Key features:
//!
//! - **Composable Command Structure**: Easy composition of nouns and verbs
//! - **Framework-Level APIs**: APIs that make it easy to build CLI frameworks
//! - **Extensible Traits**: Traits that can be easily extended and customized
//! - **Hierarchical Command Support**: Support for complex nested command structures
//! - **Type-Safe Composition**: Compile-time verification of command structure
//!
//! ## API Stability
//!
//! This crate follows [Semantic Versioning](https://semver.org/). Version 1.0.0 and above
//! provide API stability guarantees:
//!
//! - **Public APIs** are stable and will not change in a breaking way within the same major version
//! - **Breaking changes** will only occur in major version bumps (2.0.0, 3.0.0, etc.)
//! - **Deprecations** will be announced at least one minor version before removal
//! - **Private APIs** (non-pub items) are not subject to stability guarantees
//!
//! All public types, traits, and functions documented in this crate are considered stable.

pub mod builder;
pub mod error;
pub mod macros;
pub mod noun;
pub mod registry;
pub mod router;
pub mod tree;
pub mod verb;

// Core framework types
pub use builder::{build_cli, run_cli, run_cli_with_args, CliBuilder};
pub use error::{NounVerbError, Result};
pub use noun::{CompoundNounCommand, NounCommand, NounContext};
pub use registry::CommandRegistry;
pub use router::CommandRouter;
pub use tree::{CommandTree, CommandTreeBuilder};
pub use verb::{VerbArgs, VerbCommand, VerbContext};

// Macros are exported at crate root via #[macro_export]

// Framework-level re-exports for easy composition
pub use builder::CliBuilder as Cli;
pub use registry::CommandRegistry as Registry;
pub use tree::CommandTree as Tree;
