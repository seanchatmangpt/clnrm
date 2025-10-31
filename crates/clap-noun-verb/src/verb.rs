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
    /// Parent matches for accessing global arguments (e.g., --verbose, --config)
    pub parent_matches: Option<ArgMatches>,
    /// Context information
    pub context: VerbContext,
}

impl VerbArgs {
    /// Create new verb arguments
    pub fn new(matches: ArgMatches) -> Self {
        Self {
            matches,
            parent_matches: None,
            context: VerbContext::new(""),
        }
    }

    /// Create verb arguments with parent matches for global args access
    pub fn with_parent(mut self, parent: ArgMatches) -> Self {
        self.parent_matches = Some(parent);
        self
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

    /// Get a required string argument
    pub fn get_one_str(&self, name: &str) -> Result<String> {
        self.matches
            .get_one::<String>(name)
            .cloned()
            .ok_or_else(|| {
                crate::error::NounVerbError::argument_error(format!(
                    "Required argument '{}' is missing",
                    name
                ))
            })
    }

    /// Get an optional string argument
    pub fn get_one_str_opt(&self, name: &str) -> Option<String> {
        self.matches.get_one::<String>(name).cloned()
    }

    /// Get a required typed argument (e.g., usize, PathBuf)
    pub fn get_one<T>(&self, name: &str) -> Result<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.matches
            .get_one::<T>(name)
            .cloned()
            .ok_or_else(|| {
                crate::error::NounVerbError::argument_error(format!(
                    "Required argument '{}' is missing or has invalid type",
                    name
                ))
            })
    }

    /// Get an optional typed argument
    pub fn get_one_opt<T>(&self, name: &str) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.matches.get_one::<T>(name).cloned()
    }

    /// Get multiple values of a typed argument
    pub fn get_many<T>(&self, name: &str) -> Result<Vec<T>>
    where
        T: Clone + Send + Sync + 'static,
    {
        let values: Vec<T> = self
            .matches
            .get_many::<T>(name)
            .map(|iter| iter.cloned().collect())
            .unwrap_or_default();

        if values.is_empty() {
            Err(crate::error::NounVerbError::argument_error(format!(
                "Required argument '{}' is missing or has no values",
                name
            )))
        } else {
            Ok(values)
        }
    }

    /// Get multiple values of a typed argument (optional, returns empty vec if missing)
    pub fn get_many_opt<T>(&self, name: &str) -> Vec<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.matches
            .get_many::<T>(name)
            .map(|iter| iter.cloned().collect())
            .unwrap_or_default()
    }

    /// Check if a flag is set
    pub fn is_flag_set(&self, name: &str) -> bool {
        self.matches.get_flag(name)
    }

    /// Get the count of a flag (for -v, -vv, -vvv patterns)
    pub fn get_flag_count(&self, name: &str) -> u8 {
        self.matches.get_count(name)
    }

    /// Get all argument names
    pub fn arg_names(&self) -> Vec<String> {
        self.matches.ids().map(|id| id.as_str().to_string()).collect()
    }

    /// Get a required PathBuf argument
    pub fn get_path(&self, name: &str) -> Result<std::path::PathBuf> {
        self.get_one::<std::path::PathBuf>(name)
    }

    /// Get an optional PathBuf argument
    pub fn get_path_opt(&self, name: &str) -> Option<std::path::PathBuf> {
        self.get_one_opt::<std::path::PathBuf>(name)
    }

    /// Get a global argument from parent matches (e.g., --verbose, --config)
    pub fn get_global<T>(&self, name: &str) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.parent_matches
            .as_ref()
            .and_then(|parent| parent.get_one::<T>(name).cloned())
    }

    /// Get a global string argument from parent matches
    pub fn get_global_str(&self, name: &str) -> Option<String> {
        self.get_global::<String>(name)
    }

    /// Check if a global flag is set (e.g., --verbose)
    pub fn is_global_flag_set(&self, name: &str) -> bool {
        self.parent_matches
            .as_ref()
            .map(|parent| parent.get_flag(name))
            .unwrap_or(false)
    }

    /// Get global flag count (e.g., -v, -vv, -vvv)
    pub fn get_global_flag_count(&self, name: &str) -> u8 {
        self.parent_matches
            .as_ref()
            .map(|parent| parent.get_count(name))
            .unwrap_or(0)
    }
}

/// Trait for defining verb commands (e.g., "status", "logs", "restart")
///
/// # Examples
///
/// Implementing `VerbCommand` directly:
///
/// ```rust
/// use clap_noun_verb::{VerbCommand, VerbArgs, Result};
///
/// struct StatusCommand;
///
/// impl VerbCommand for StatusCommand {
///     fn name(&self) -> &'static str { "status" }
///     fn about(&self) -> &'static str { "Show status" }
///     fn run(&self, _args: &VerbArgs) -> Result<()> {
///         println!("All services running");
///         Ok(())
///     }
/// }
/// ```
///
/// Using the `verb!` macro (recommended):
///
/// ```rust,no_run
/// use clap_noun_verb::{verb, VerbArgs, Result};
///
/// let _status = verb!("status", "Show status", |_args: &VerbArgs| -> Result<()> {
///     println!("All services running");
///     Ok(())
/// });
/// ```
pub trait VerbCommand: Send + Sync {
    /// The name of the verb command
    fn name(&self) -> &'static str;

    /// Description of what this verb command does
    fn about(&self) -> &'static str;

    /// Execute the verb command
    ///
    /// # Errors
    ///
    /// Returns `Result::Err` if command execution fails.
    fn run(&self, args: &VerbArgs) -> Result<()>;

    /// Build the clap command for this verb
    ///
    /// Default implementation creates a basic command with name and description.
    /// Override to customize command building.
    fn build_command(&self) -> Command {
        Command::new(self.name()).about(self.about())
    }

    /// Get additional arguments for this verb (override to add custom args)
    ///
    /// Returns an empty vector by default. Override to provide custom arguments.
    fn additional_args(&self) -> Vec<clap::Arg> {
        Vec::new()
    }
}
