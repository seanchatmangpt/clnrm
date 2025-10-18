//! Builder pattern API for composable CLI applications

use crate::error::Result;
use crate::noun::NounCommand;
use crate::registry::CommandRegistry;
use clap::Command;

/// Main builder for creating composable CLI applications
pub struct CliBuilder {
    registry: CommandRegistry,
}

impl CliBuilder {
    /// Create a new CLI builder
    pub fn new() -> Self {
        Self {
            registry: CommandRegistry::new(),
        }
    }

    /// Set the application name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.registry = self.registry.name(name);
        self
    }

    /// Set the application description
    pub fn about(mut self, about: impl Into<String>) -> Self {
        self.registry = self.registry.about(about);
        self
    }

    /// Set the application version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.registry = self.registry.version(version);
        self
    }

    /// Add global arguments available to all commands
    pub fn global_args(mut self, args: Vec<clap::Arg>) -> Self {
        self.registry = self.registry.global_args(args);
        self
    }

    /// Add a noun command to the CLI
    pub fn noun(mut self, noun: impl NounCommand + 'static) -> Self {
        self.registry = self.registry.register_noun(noun);
        self
    }

    /// Add multiple noun commands
    pub fn nouns<I>(mut self, nouns: I) -> Self
    where
        I: IntoIterator<Item = Box<dyn NounCommand>>,
    {
        self.registry = self.registry.register_nouns(nouns);
        self
    }

    /// Run the CLI application
    pub fn run(self) -> Result<()> {
        self.registry.run()
    }

    /// Run the CLI application with custom args
    pub fn run_with_args(self, args: Vec<String>) -> Result<()> {
        self.registry.run_with_args(args)
    }

    /// Get the built command for testing or manual execution
    pub fn build_command(self) -> Command {
        self.registry.build_command()
    }

    /// Get the underlying registry for advanced usage
    pub fn registry(self) -> CommandRegistry {
        self.registry
    }

    /// Get a reference to the registry for introspection
    pub fn registry_ref(&self) -> &CommandRegistry {
        &self.registry
    }

    /// Get the command structure for introspection
    pub fn command_structure(&self) -> std::collections::HashMap<String, Vec<String>> {
        self.registry.command_structure()
    }

    /// Check if a command exists
    pub fn has_command(&self, name: &str) -> bool {
        self.registry.has_noun(name)
    }
}

impl Default for CliBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to run a CLI with the current process args
pub fn run_cli<F>(builder: F) -> Result<()>
where
    F: FnOnce(CliBuilder) -> CliBuilder,
{
    let cli = CliBuilder::new();
    let cli = builder(cli);
    cli.run()
}

/// Convenience function to run a CLI with custom args
pub fn run_cli_with_args<F>(args: Vec<String>, builder: F) -> Result<()>
where
    F: FnOnce(CliBuilder) -> CliBuilder,
{
    let cli = CliBuilder::new();
    let cli = builder(cli);
    cli.run_with_args(args)
}

/// Convenience function to build a CLI and get the command structure
pub fn build_cli<F>(builder: F) -> (Command, std::collections::HashMap<String, Vec<String>>)
where
    F: FnOnce(CliBuilder) -> CliBuilder,
{
    let cli = CliBuilder::new();
    let cli = builder(cli);
    let registry = cli.registry;
    let command = registry.build_command();
    let structure = registry.command_structure();
    (command, structure)
}

/// Macro for quickly building CLI applications
///
/// # Example
///
/// ```rust
/// use clap_noun_verb::{cli_builder, noun, verb, VerbArgs, Result};
///
/// cli_builder! {
///     name: "myapp",
///     about: "My awesome CLI application",
///     nouns: [
///         noun!("services", "Manage services", [
///             verb!("status", "Show status", |_args: &VerbArgs| {
///                 println!("Services are running");
///                 Ok(())
///             }),
///         ]),
///     ],
/// }
/// ```
#[macro_export]
macro_rules! cli_builder {
    (name: $name:expr, about: $about:expr, nouns: [$($noun:expr),* $(,)?]) => {
        {
            let mut builder = $crate::CliBuilder::new()
                .name($name)
                .about($about);

            $(
                builder = builder.noun($noun);
            )*

            builder
        }
    };
}
