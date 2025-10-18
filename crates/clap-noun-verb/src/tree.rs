//! Command tree for hierarchical CLI composition
//!
//! The CommandTree provides a tree-based structure for organizing commands
//! hierarchically, making it easy to build complex nested command structures.

use crate::error::{NounVerbError, Result};
use crate::verb::VerbArgs;
use clap::{ArgMatches, Command};

/// Tree-based command structure for hierarchical CLI composition
pub struct CommandTree {
    /// Root commands in the tree
    roots: Vec<TreeNode>,
}

/// A node in the command tree
pub struct TreeNode {
    /// The command name
    pub name: String,
    /// The command description
    pub about: String,
    /// Child commands (verbs or sub-nouns)
    pub children: Vec<TreeNode>,
    /// Command handler if this is a leaf node
    pub handler: Option<CommandHandler>,
}

/// Command handler for leaf nodes
pub struct CommandHandler {
    /// Handler function
    pub handler: Box<dyn Fn(&VerbArgs) -> Result<()> + Send + Sync>,
}

/// Builder for creating command trees
pub struct CommandTreeBuilder {
    roots: Vec<TreeNode>,
}

impl CommandTree {
    /// Create a new empty command tree
    pub fn new() -> Self {
        Self {
            roots: Vec::new(),
        }
    }

    /// Create a tree from a builder
    pub fn from_builder(builder: CommandTreeBuilder) -> Self {
        builder.build()
    }

    /// Add a root command to the tree
    pub fn add_root(mut self, node: TreeNode) -> Self {
        self.roots.push(node);
        self
    }

    /// Add multiple root commands
    pub fn add_roots<I>(mut self, nodes: I) -> Self
    where
        I: IntoIterator<Item = TreeNode>,
    {
        self.roots.extend(nodes);
        self
    }

    /// Get all root command names
    pub fn root_names(&self) -> Vec<&str> {
        self.roots.iter().map(|n| n.name.as_str()).collect()
    }

    /// Find a command in the tree by path
    pub fn find_command(&self, path: &[&str]) -> Option<&TreeNode> {
        if path.is_empty() {
            return None;
        }

        // Find root command
        let mut current = self.roots.iter().find(|n| n.name == path[0])?;

        // Traverse the path
        for &segment in &path[1..] {
            current = current.children.iter().find(|n| n.name == segment)?;
        }

        Some(current)
    }

    /// Build the complete clap command structure
    pub fn build_command(&self) -> Command {
        let mut cmd = Command::new("cli").about("Command tree application");

        for root in &self.roots {
            cmd = cmd.subcommand(root.build_command());
        }

        cmd
    }

    /// Route a command based on clap matches
    pub fn route(&self, matches: &ArgMatches) -> Result<()> {
        let (cmd_name, cmd_matches) = matches.subcommand()
            .ok_or_else(|| NounVerbError::invalid_structure("No subcommand found"))?;

        // Find the root command
        let root = self.roots.iter().find(|n| n.name == cmd_name)
            .ok_or_else(|| NounVerbError::command_not_found(cmd_name))?;

        // Route recursively
        self.route_recursive(root, cmd_matches)
    }

    /// Recursively route commands through the tree
    fn route_recursive(&self, node: &TreeNode, matches: &ArgMatches) -> Result<()> {
        if let Some((child_name, child_matches)) = matches.subcommand() {
            // Find the child command
            let child = node.children.iter().find(|n| n.name == child_name)
                .ok_or_else(|| NounVerbError::command_not_found(child_name))?;

            // Recursively route
            self.route_recursive(child, child_matches)
        } else {
            // Leaf node, execute handler
            if let Some(handler) = &node.handler {
                let args = VerbArgs::new(matches.clone());
                (handler.handler)(&args)
            } else {
                Err(NounVerbError::invalid_structure("No handler for command"))
            }
        }
    }
}

impl TreeNode {
    /// Create a new tree node
    pub fn new(name: impl Into<String>, about: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            about: about.into(),
            children: Vec::new(),
            handler: None,
        }
    }

    /// Add a child command
    pub fn add_child(mut self, child: TreeNode) -> Self {
        self.children.push(child);
        self
    }

    /// Add multiple child commands
    pub fn add_children<I>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = TreeNode>,
    {
        self.children.extend(children);
        self
    }

    /// Set the command handler
    pub fn with_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(&VerbArgs) -> Result<()> + Send + Sync + 'static,
    {
        self.handler = Some(CommandHandler {
            handler: Box::new(handler),
        });
        self
    }

    /// Build the clap command for this node
    pub fn build_command(&self) -> Command {
        let mut cmd = Command::new(self.name.as_str())
            .about(self.about.as_str());

        for child in &self.children {
            cmd = cmd.subcommand(child.build_command());
        }

        cmd
    }

    /// Get all command paths from this node
    pub fn command_paths(&self) -> Vec<Vec<String>> {
        let mut paths = Vec::new();

        for child in &self.children {
            let mut child_paths = child.command_paths();
            for path in child_paths.iter_mut() {
                path.insert(0, self.name.clone());
            }
            paths.extend(child_paths);
        }

        if paths.is_empty() {
            // This is a leaf node
            paths.push(vec![self.name.clone()]);
        }

        paths
    }
}

impl CommandTreeBuilder {
    /// Create a new command tree builder
    pub fn new() -> Self {
        Self {
            roots: Vec::new(),
        }
    }

    /// Add a root command
    pub fn add_root(mut self, node: TreeNode) -> Self {
        self.roots.push(node);
        self
    }

    /// Add a root command with handler
    pub fn add_root_with_handler<F>(
        mut self,
        name: impl Into<String>,
        about: impl Into<String>,
        handler: F,
    ) -> Self
    where
        F: Fn(&VerbArgs) -> Result<()> + Send + Sync + 'static,
    {
        let node = TreeNode::new(name, about).with_handler(handler);
        self.roots.push(node);
        self
    }

    /// Add a root command with children
    pub fn add_root_with_children<I>(
        mut self,
        name: impl Into<String>,
        about: impl Into<String>,
        children: I,
    ) -> Self
    where
        I: IntoIterator<Item = TreeNode>,
    {
        let node = TreeNode::new(name, about).add_children(children);
        self.roots.push(node);
        self
    }

    /// Build the command tree
    pub fn build(self) -> CommandTree {
        CommandTree {
            roots: self.roots,
        }
    }
}

impl Default for CommandTree {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CommandTreeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for building common command patterns
pub mod patterns {
    use super::*;

    /// Create a simple noun-verb pattern
    pub fn noun_verb_pattern(
        noun_name: impl Into<String>,
        about: impl Into<String>,
        verbs: Vec<(String, String, Box<dyn Fn(&VerbArgs) -> Result<()> + Send + Sync>)>,
    ) -> TreeNode {
        let mut node = TreeNode::new(noun_name, about);

        for (verb_name, verb_about, handler) in verbs {
            let verb_node = TreeNode::new(verb_name, verb_about).with_handler(handler);
            node = node.add_child(verb_node);
        }

        node
    }

    /// Create a nested command pattern (noun with sub-nouns)
    pub fn nested_pattern(
        noun_name: impl Into<String>,
        about: impl Into<String>,
        sub_nouns: Vec<TreeNode>,
    ) -> TreeNode {
        TreeNode::new(noun_name, about).add_children(sub_nouns)
    }

    /// Create a group pattern (multiple related commands)
    pub fn group_pattern(
        group_name: impl Into<String>,
        about: impl Into<String>,
        commands: Vec<TreeNode>,
    ) -> TreeNode {
        TreeNode::new(group_name, about).add_children(commands)
    }
}
