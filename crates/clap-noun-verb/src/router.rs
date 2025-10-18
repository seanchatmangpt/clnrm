//! Command routing logic for noun-verb CLI

use crate::error::{NounVerbError, Result};
use crate::noun::NounCommand;
use crate::verb::{VerbArgs, VerbContext};
use clap::{ArgMatches, Command};
use std::collections::HashMap;

/// Router for dispatching noun-verb commands
pub struct CommandRouter {
    nouns: HashMap<String, Box<dyn NounCommand>>,
}

impl CommandRouter {
    /// Create a new command router
    pub fn new() -> Self {
        Self {
            nouns: HashMap::new(),
        }
    }

    /// Register a noun command
    pub fn register_noun(&mut self, noun: Box<dyn NounCommand>) {
        self.nouns.insert(noun.name().to_string(), noun);
    }

    /// Route a command based on clap matches
    pub fn route(&self, matches: &ArgMatches) -> Result<()> {
        // Get the top-level subcommand (noun)
        let (noun_name, noun_matches) = matches.subcommand()
            .ok_or_else(|| NounVerbError::invalid_structure("No subcommand found"))?;

        // Find the noun command
        let noun = self.nouns.get(noun_name)
            .ok_or_else(|| NounVerbError::command_not_found(noun_name))?;

        // Route the command recursively
        self.route_recursive(noun.as_ref(), noun_name, noun_matches)
    }

    /// Recursively route commands through nested noun-verb structure
    fn route_recursive(&self, noun: &dyn NounCommand, noun_name: &str, matches: &ArgMatches) -> Result<()> {
        // Check if there's a subcommand (either verb or sub-noun)
        if let Some((sub_name, sub_matches)) = matches.subcommand() {
            // First check if it's a verb
            if let Some(verb) = noun.verbs().iter().find(|v| v.name() == sub_name) {
                // Execute the verb
                let context = VerbContext::new(sub_name).with_noun(noun_name);
                let args = VerbArgs::new(sub_matches.clone())
                    .with_context(context);

                verb.run(&args)
            } else if let Some(sub_noun) = noun.sub_nouns().iter().find(|n| n.name() == sub_name) {
                // Recursively route to sub-noun
                self.route_recursive(sub_noun.as_ref(), sub_name, sub_matches)
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

    /// Build the complete clap command structure
    pub fn build_command(&self, app_name: &'static str, about: &'static str) -> Command {
        let mut cmd = Command::new(app_name).about(about);

        for noun in self.nouns.values() {
            cmd = cmd.subcommand(noun.build_command());
        }

        cmd
    }

    /// Get all registered noun names
    pub fn noun_names(&self) -> Vec<&str> {
        self.nouns.keys().map(|s| s.as_str()).collect()
    }

    /// Get verbs for a specific noun
    pub fn get_verbs(&self, noun_name: &str) -> Result<Vec<String>> {
        let noun = self.nouns.get(noun_name)
            .ok_or_else(|| NounVerbError::command_not_found(noun_name))?;
        
        Ok(noun.verbs().iter().map(|v| v.name().to_string()).collect())
    }
}

impl Default for CommandRouter {
    fn default() -> Self {
        Self::new()
    }
}
