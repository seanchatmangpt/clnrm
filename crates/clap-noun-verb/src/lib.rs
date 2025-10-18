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

pub mod builder;
pub mod error;
pub mod macros;
pub mod noun;
pub mod registry;
pub mod router;
pub mod tree;
pub mod verb;

// Core framework types
pub use builder::{CliBuilder, run_cli, run_cli_with_args};
pub use error::{NounVerbError, Result};
pub use noun::{NounCommand, NounContext};
pub use registry::CommandRegistry;
pub use router::CommandRouter;
pub use tree::{CommandTree, CommandTreeBuilder};
pub use verb::{VerbArgs, VerbCommand, VerbContext};

// Macros are exported at crate root via #[macro_export]

// Framework-level re-exports for easy composition
pub use builder::CliBuilder as Cli;
pub use registry::CommandRegistry as Registry;
pub use tree::CommandTree as Tree;
