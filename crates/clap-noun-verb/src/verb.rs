//! Verb command trait and types for composable CLI patterns

use crate::error::Result;
use clap::{ArgMatches, Command};
use std::collections::HashMap;

/// Context information passed to verb commands
#[derive(Debug, Clone)]
pub struct VerbContext {
    /// The verb name being executed
    pub verb: String,
    /// The noun this verb belongs to
    pub noun: Option<String>,
    /// Additional context data
    pub data: HashMap<String, String>,
}

impl VerbContext {
    /// Create a new verb context
    pub fn new(verb: impl Into<String>) -> Self {
        Self {
            verb: verb.into(),
            noun: None,
            data: HashMap::new(),
        }
    }

    /// Set the noun this verb belongs to
    pub fn with_noun(mut self, noun: impl Into<String>) -> Self {
        self.noun = Some(noun.into());
        self
    }

    /// Add context data
    pub fn with_data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.data.insert(key.into(), value.into());
        self
    }

    /// Get context data
    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }
}

/// Arguments passed to a verb command
#[derive(Debug, Clone)]
pub struct VerbArgs {
    /// The raw clap matches for this verb
    pub matches: ArgMatches,
    /// Context information
    pub context: VerbContext,
}

impl VerbArgs {
    /// Create new verb arguments
    pub fn new(matches: ArgMatches) -> Self {
        Self {
            matches,
            context: VerbContext::new(""),
        }
    }

    /// Create verb arguments with context
    pub fn with_context(mut self, context: VerbContext) -> Self {
        self.context = context;
        self
    }

    /// Add context data to existing arguments
    pub fn add_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context = self.context.with_data(key, value);
        self
    }

    /// Get context value
    pub fn get_context(&self, key: &str) -> Option<&String> {
        self.context.get_data(key)
    }

    /// Get the verb name
    pub fn verb(&self) -> &str {
        &self.context.verb
    }

    /// Get the noun name (if available)
    pub fn noun(&self) -> Option<&str> {
        self.context.noun.as_deref()
    }
}

/// Trait for defining verb commands (e.g., "status", "logs", "restart")
pub trait VerbCommand: Send + Sync {
    /// The name of the verb command
    fn name(&self) -> &'static str;

    /// Description of what this verb command does
    fn about(&self) -> &'static str;

    /// Execute the verb command
    fn run(&self, args: &VerbArgs) -> Result<()>;

    /// Build the clap command for this verb
    fn build_command(&self) -> Command {
        Command::new(self.name()).about(self.about())
    }

    /// Get additional arguments for this verb (override to add custom args)
    fn additional_args(&self) -> Vec<clap::Arg> {
        Vec::new()
    }
}
