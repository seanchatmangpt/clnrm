//! Noun command trait and types for composable CLI patterns

use crate::error::Result;
use crate::verb::{VerbArgs, VerbCommand};
use clap::Command;
use std::collections::HashMap;

/// Context information passed to noun commands
#[derive(Debug, Clone)]
pub struct NounContext {
    /// The noun name being executed
    pub noun: String,
    /// Additional context data
    pub data: HashMap<String, String>,
}

impl NounContext {
    /// Create a new noun context
    pub fn new(noun: impl Into<String>) -> Self {
        Self {
            noun: noun.into(),
            data: HashMap::new(),
        }
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

/// Trait for defining noun commands (e.g., "services", "collector")
pub trait NounCommand: Send + Sync {
    /// The name of the noun command
    fn name(&self) -> &'static str;

    /// Description of what this noun command does
    fn about(&self) -> &'static str;

    /// Get all verb commands associated with this noun
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>>;

    /// Get all sub-noun commands (for nested command groups)
    fn sub_nouns(&self) -> Vec<Box<dyn NounCommand>> {
        Vec::new()
    }

    /// Build the clap command for this noun
    fn build_command(&self) -> Command {
        let mut cmd = Command::new(self.name())
            .about(self.about());

        // Add verb subcommands
        for verb in self.verbs() {
            cmd = cmd.subcommand(verb.build_command());
        }

        // Add sub-noun commands (for nested command groups)
        for sub_noun in self.sub_nouns() {
            cmd = cmd.subcommand(sub_noun.build_command());
        }

        cmd
    }

    /// Handle the noun command if it has no verbs or sub-nouns (direct execution)
    fn handle_direct(&self, _args: &VerbArgs) -> Result<()> {
        Err(crate::error::NounVerbError::invalid_structure(format!(
            "Noun '{}' has no verbs or sub-nouns and cannot be executed directly",
            self.name()
        )))
    }

    /// Handle a verb command for this noun
    fn handle_verb(&self, verb_name: &str, args: &VerbArgs) -> Result<()> {
        let verb = self.verbs()
            .into_iter()
            .find(|v| v.name() == verb_name)
            .ok_or_else(|| crate::error::NounVerbError::verb_not_found(self.name(), verb_name))?;

        verb.run(args)
    }

    /// Handle a sub-noun command for this noun
    fn handle_sub_noun(&self, sub_noun_name: &str, args: &VerbArgs) -> Result<()> {
        let sub_noun = self.sub_nouns()
            .into_iter()
            .find(|n| n.name() == sub_noun_name)
            .ok_or_else(|| crate::error::NounVerbError::command_not_found(sub_noun_name))?;

        sub_noun.handle_direct(args)
    }
}

/// Helper trait for creating compound commands (nouns that contain other nouns)
pub trait CompoundNounCommand: NounCommand {
    /// Get all nested nouns recursively
    fn all_nouns(&self) -> Vec<String> {
        let mut nouns = vec![self.name().to_string()];
        for sub_noun in self.sub_nouns() {
            nouns.push(sub_noun.name().to_string());
            // For compound sub-nouns, we can't easily recurse without dynamic dispatch
            // This is a limitation of the current trait design
        }
        nouns
    }

    /// Get all verbs recursively
    fn all_verbs(&self) -> HashMap<String, Vec<String>> {
        let mut verbs = HashMap::new();
        verbs.insert(self.name().to_string(), self.verbs().iter().map(|v| v.name().to_string()).collect());

        for sub_noun in self.sub_nouns() {
            verbs.insert(sub_noun.name().to_string(), sub_noun.verbs().iter().map(|v| v.name().to_string()).collect());
        }

        verbs
    }
}
