//! Command registry for composable CLI patterns
//!
//! The CommandRegistry provides a central hub for registering and composing
//! commands in a flexible, extensible way. This allows users to build their
//! own CLI patterns by composing commands together.

use crate::error::{NounVerbError, Result};
use crate::noun::NounCommand;
use crate::verb::{VerbArgs, VerbContext};
use clap::{ArgMatches, Command};
use std::collections::HashMap;

/// Central registry for managing all CLI commands
///
/// This registry allows users to:
/// - Register nouns and verbs in any order
/// - Compose command hierarchies dynamically
/// - Query command structure for introspection
/// - Build complete CLI applications from registered commands
pub struct CommandRegistry {
    /// Map of noun name to noun command
    nouns: HashMap<String, Box<dyn NounCommand>>,
    /// Global configuration for the CLI
    config: RegistryConfig,
}

/// Configuration for the command registry
#[derive(Debug, Clone)]
pub struct RegistryConfig {
    /// Application name
    pub name: String,
    /// Application description
    pub about: String,
    /// Version string
    pub version: Option<String>,
    /// Global arguments available to all commands
    pub global_args: Vec<clap::Arg>,
    /// Auto-validate command structure on build/run
    pub auto_validate: bool,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            name: "cli".to_string(),
            about: "A command-line application".to_string(),
            version: None,
            global_args: Vec::new(),
            auto_validate: false,
        }
    }
}

impl CommandRegistry {
    /// Create a new command registry
    pub fn new() -> Self {
        Self {
            nouns: HashMap::new(),
            config: RegistryConfig::default(),
        }
    }

    /// Create a new registry with configuration
    pub fn with_config(config: RegistryConfig) -> Self {
        Self {
            nouns: HashMap::new(),
            config,
        }
    }

    /// Set the application name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.config.name = name.into();
        self
    }

    /// Set the application description
    pub fn about(mut self, about: impl Into<String>) -> Self {
        self.config.about = about.into();
        self
    }

    /// Set the application version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.config.version = Some(version.into());
        self
    }

    /// Add global arguments available to all commands
    pub fn global_args(mut self, args: Vec<clap::Arg>) -> Self {
        self.config.global_args = args;
        self
    }

    /// Enable automatic validation of command structure
    pub fn auto_validate(mut self, enable: bool) -> Self {
        self.config.auto_validate = enable;
        self
    }

    /// Register a noun command
    pub fn register_noun(mut self, noun: impl NounCommand + 'static) -> Self {
        self.nouns.insert(noun.name().to_string(), Box::new(noun));
        self
    }

    /// Register multiple noun commands
    pub fn register_nouns<I>(mut self, nouns: I) -> Self
    where
        I: IntoIterator<Item = Box<dyn NounCommand>>,
    {
        for noun in nouns {
            self.nouns.insert(noun.name().to_string(), noun);
        }
        self
    }

    /// Get a noun command by name
    pub fn get_noun(&self, name: &str) -> Option<&dyn NounCommand> {
        self.nouns.get(name).map(|n| n.as_ref())
    }

    /// Get all registered noun names
    pub fn noun_names(&self) -> Vec<&str> {
        self.nouns.keys().map(|s| s.as_str()).collect()
    }

    /// Get all registered nouns
    pub fn nouns(&self) -> Vec<&dyn NounCommand> {
        self.nouns.values().map(|n| n.as_ref()).collect()
    }

    /// Check if a noun is registered
    pub fn has_noun(&self, name: &str) -> bool {
        self.nouns.contains_key(name)
    }

    /// Remove a noun command
    pub fn remove_noun(&mut self, name: &str) -> Option<Box<dyn NounCommand>> {
        self.nouns.remove(name)
    }

    /// Clear all registered commands
    pub fn clear(&mut self) {
        self.nouns.clear();
    }

    /// Get the complete command structure for introspection
    pub fn command_structure(&self) -> HashMap<String, Vec<String>> {
        let mut structure = HashMap::new();

        for (noun_name, noun) in &self.nouns {
            let verbs: Vec<String> = noun.verbs().iter().map(|v| v.name().to_string()).collect();
            structure.insert(noun_name.clone(), verbs);
        }

        structure
    }

    /// Validate the command registry structure
    pub fn validate(&self) -> Result<()> {
        // Check for duplicate noun names
        let mut seen_nouns = std::collections::HashSet::new();
        for noun_name in self.nouns.keys() {
            if !seen_nouns.insert(noun_name) {
                return Err(NounVerbError::invalid_structure(format!(
                    "Duplicate noun name: '{}'",
                    noun_name
                )));
            }
        }

        // Validate each noun structure
        for (noun_name, noun) in &self.nouns {
            // Check for empty nouns (no verbs or sub-nouns)
            if noun.verbs().is_empty() && noun.sub_nouns().is_empty() {
                return Err(NounVerbError::invalid_structure(format!(
                    "Noun '{}' has no verbs or sub-nouns",
                    noun_name
                )));
            }

            // Check for duplicate verb names within a noun
            let mut seen_verbs = std::collections::HashSet::new();
            for verb in noun.verbs() {
                let verb_name = verb.name();
                if !seen_verbs.insert(verb_name) {
                    return Err(NounVerbError::invalid_structure(format!(
                        "Duplicate verb name '{}' in noun '{}'",
                        verb_name, noun_name
                    )));
                }
            }

            // Check for duplicate sub-noun names within a noun
            let mut seen_sub_nouns = std::collections::HashSet::new();
            for sub_noun in noun.sub_nouns() {
                let sub_noun_name = sub_noun.name();
                if !seen_sub_nouns.insert(sub_noun_name) {
                    return Err(NounVerbError::invalid_structure(format!(
                        "Duplicate sub-noun name '{}' in noun '{}'",
                        sub_noun_name, noun_name
                    )));
                }
            }

            // Check for verb/sub-noun name conflicts
            let verb_names: std::collections::HashSet<_> = noun.verbs().iter().map(|v| v.name()).collect();
            for sub_noun in noun.sub_nouns() {
                let sub_noun_name = sub_noun.name();
                if verb_names.contains(sub_noun_name) {
                    return Err(NounVerbError::invalid_structure(format!(
                        "Verb and sub-noun cannot have the same name '{}' in noun '{}'",
                        sub_noun_name, noun_name
                    )));
                }
            }
        }

        Ok(())
    }

    /// Build the complete clap command structure
    pub fn build_command(&self) -> Command {
        // Auto-validate if enabled
        if self.config.auto_validate {
            if let Err(e) = self.validate() {
                // Log validation error but continue building for API compatibility
                eprintln!("Warning: Command structure validation failed: {}", e);
            }
        }
        
        // Clone to owned strings and convert to static lifetime for clap
        // Note: This leaks memory but is acceptable for CLI construction (happens once per run)
        // The leaked strings live for the duration of the program which is fine for CLI apps
        let name: &'static str = Box::leak(self.config.name.clone().into_boxed_str());
        let about: &'static str = Box::leak(self.config.about.clone().into_boxed_str());
        let mut cmd = Command::new(name)
            .about(about);

        if let Some(version) = &self.config.version {
            let version_str: &'static str = Box::leak(version.to_string().into_boxed_str());
            cmd = cmd.version(version_str);
        }

        // Add global arguments
        for arg in &self.config.global_args {
            cmd = cmd.arg(arg.clone());
        }

        // Add noun subcommands
        for noun in self.nouns.values() {
            cmd = cmd.subcommand(noun.build_command());
        }

        cmd
    }

    /// Route a command based on clap matches
    pub fn route(&self, matches: &ArgMatches) -> Result<()> {
        // Get the top-level subcommand (noun)
        let (noun_name, noun_matches) = matches.subcommand()
            .ok_or_else(|| NounVerbError::invalid_structure("No subcommand found"))?;

        // Find the noun command
        let noun = self.nouns.get(noun_name)
            .ok_or_else(|| NounVerbError::command_not_found(noun_name))?;

        // Route the command recursively with root matches for global args access
        self.route_recursive(noun.as_ref(), noun_name, noun_matches, matches)
    }

    /// Recursively route commands through nested noun-verb structure
    #[allow(clippy::only_used_in_recursion)]
    fn route_recursive(&self, noun: &dyn NounCommand, noun_name: &str, matches: &ArgMatches, root_matches: &ArgMatches) -> Result<()> {
        // Check if there's a subcommand (either verb or sub-noun)
        if let Some((sub_name, sub_matches)) = matches.subcommand() {
            // First check if it's a verb
            if let Some(verb) = noun.verbs().iter().find(|v| v.name() == sub_name) {
                // Execute the verb with root matches for global args access
                let context = VerbContext::new(sub_name).with_noun(noun_name);
                let args = VerbArgs::new(sub_matches.clone())
                    .with_parent(root_matches.clone())
                    .with_context(context);

                verb.run(&args)
            } else if let Some(sub_noun) = noun.sub_nouns().iter().find(|n| n.name() == sub_name) {
                // Recursively route to sub-noun, passing root matches for global args
                self.route_recursive(sub_noun.as_ref(), sub_name, sub_matches, root_matches)
            } else {
                // Neither verb nor sub-noun found
                Err(NounVerbError::verb_not_found(noun_name, sub_name))
            }
        } else {
            // No subcommand, try direct noun execution
            let context = VerbContext::new("").with_noun(noun_name);
            let args = VerbArgs::new(matches.clone())
                .with_context(context);

            noun.handle_direct(&args)
        }
    }

    /// Run the CLI with the current process arguments
    pub fn run(self) -> Result<()> {
        // Auto-validate if enabled
        if self.config.auto_validate {
            self.validate()?;
        }
        
        let cmd = self.build_command();
        let matches = cmd.try_get_matches()
            .map_err(|e| NounVerbError::argument_error(e.to_string()))?;

        self.route(&matches)
    }

    /// Run the CLI with custom arguments
    pub fn run_with_args(self, args: Vec<String>) -> Result<()> {
        let cmd = self.build_command();
        let matches = cmd.try_get_matches_from(args)
            .map_err(|e| NounVerbError::argument_error(e.to_string()))?;

        self.route(&matches)
    }

    /// Get the built command for testing or manual execution
    pub fn command(self) -> Command {
        self.build_command()
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}
