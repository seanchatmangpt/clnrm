//! File watching implementation using notify crate
//!
//! This module provides the file watching abstraction and concrete implementation
//! following London School TDD principles.
//!
//! # London TDD Approach
//!
//! - `FileWatcher` trait defines the contract for file watching
//! - `MockFileWatcher` in tests verifies interactions
//! - `NotifyWatcher` is the production implementation
//! - Tests focus on behavior and interactions, not implementation details
//!
//! # Core Team Compliance
//!
//! - ✅ Proper error handling with CleanroomError
//! - ✅ No unwrap() or expect() calls
//! - ✅ Sync trait methods (dyn compatible)
//! - ✅ Async operations handled via channels

use crate::cli::types::CliConfig;
use crate::error::{CleanroomError, Result};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcherTrait};
use std::path::PathBuf;
use std::sync::mpsc as std_mpsc;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

/// File system event
#[derive(Debug, Clone)]
pub struct WatchEvent {
    /// Path to the file that changed
    pub path: PathBuf,
    /// Type of change (create, modify, delete)
    pub kind: WatchEventKind,
}

/// Type of file system change
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchEventKind {
    /// File was created
    Create,
    /// File was modified
    Modify,
    /// File was deleted
    Delete,
    /// Other event type
    Other,
}

/// Configuration for file watching
#[derive(Debug, Clone)]
pub struct WatchConfig {
    /// Paths to watch (files or directories)
    pub paths: Vec<PathBuf>,
    /// Debounce delay in milliseconds
    pub debounce_ms: u64,
    /// Whether to clear screen between test runs
    pub clear_screen: bool,
    /// CLI configuration for test execution
    pub cli_config: CliConfig,
    /// Optional filter pattern for scenario selection (substring match on path)
    pub filter_pattern: Option<String>,
    /// Optional timebox limit in milliseconds per scenario
    pub timebox_ms: Option<u64>,
}

impl WatchConfig {
    /// Create new watch configuration
    ///
    /// # Arguments
    ///
    /// * `paths` - Paths to watch for changes
    /// * `debounce_ms` - Milliseconds to wait before triggering (typically 200-500ms)
    /// * `clear_screen` - Whether to clear terminal between runs
    ///
    /// # Example
    ///
    /// ```
    /// use clnrm_core::watch::WatchConfig;
    /// use std::path::PathBuf;
    ///
    /// let config = WatchConfig::new(
    ///     vec![PathBuf::from("tests/")],
    ///     300,
    ///     true
    /// );
    /// ```
    pub fn new(paths: Vec<PathBuf>, debounce_ms: u64, clear_screen: bool) -> Self {
        Self {
            paths,
            debounce_ms,
            clear_screen,
            cli_config: CliConfig::default(),
            filter_pattern: None,
            timebox_ms: None,
        }
    }

    /// Add CLI configuration for test execution
    ///
    /// # Arguments
    ///
    /// * `cli_config` - CLI configuration to use when running tests
    ///
    /// # Example
    ///
    /// ```
    /// use clnrm_core::watch::WatchConfig;
    /// use clnrm_core::cli::types::CliConfig;
    /// use std::path::PathBuf;
    ///
    /// let config = WatchConfig::new(
    ///     vec![PathBuf::from("tests/")],
    ///     300,
    ///     false
    /// ).with_cli_config(CliConfig::default());
    /// ```
    pub fn with_cli_config(mut self, cli_config: CliConfig) -> Self {
        self.cli_config = cli_config;
        self
    }

    /// Add filter pattern for scenario selection
    ///
    /// Only scenarios whose paths contain this substring will be executed.
    ///
    /// # Arguments
    ///
    /// * `pattern` - Substring to match against scenario file paths
    ///
    /// # Example
    ///
    /// ```
    /// use clnrm_core::watch::WatchConfig;
    /// use std::path::PathBuf;
    ///
    /// let config = WatchConfig::new(
    ///     vec![PathBuf::from("tests/")],
    ///     300,
    ///     false
    /// ).with_filter_pattern("otel".to_string());
    /// ```
    pub fn with_filter_pattern(mut self, pattern: String) -> Self {
        self.filter_pattern = Some(pattern);
        self
    }

    /// Add timebox limit for scenario execution
    ///
    /// Scenarios that exceed this time limit will be terminated.
    ///
    /// # Arguments
    ///
    /// * `timebox_ms` - Maximum execution time in milliseconds
    ///
    /// # Example
    ///
    /// ```
    /// use clnrm_core::watch::WatchConfig;
    /// use std::path::PathBuf;
    ///
    /// let config = WatchConfig::new(
    ///     vec![PathBuf::from("tests/")],
    ///     300,
    ///     false
    /// ).with_timebox(5000);
    /// ```
    pub fn with_timebox(mut self, timebox_ms: u64) -> Self {
        self.timebox_ms = Some(timebox_ms);
        self
    }

    /// Check if a filter pattern is set
    pub fn has_filter_pattern(&self) -> bool {
        self.filter_pattern.is_some()
    }

    /// Check if a timebox is set
    pub fn has_timebox(&self) -> bool {
        self.timebox_ms.is_some()
    }
}

/// File watcher trait for testability
///
/// This trait allows mocking file watching behavior in tests,
/// following London School TDD principles of defining contracts
/// through interfaces.
pub trait FileWatcher: Send + Sync {
    /// Start watching for file changes
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    fn start(&self) -> Result<()>;

    /// Stop watching for file changes
    fn stop(&self) -> Result<()>;
}

/// Production file watcher using notify crate
///
/// Watches file system for changes and sends events to a channel.
/// Uses `notify::RecommendedWatcher` which selects the best backend
/// for the current platform (inotify on Linux, FSEvents on macOS, etc).
#[derive(Debug)]
pub struct NotifyWatcher {
    /// Paths being watched
    _paths: Vec<PathBuf>,
    /// Internal watcher instance (kept alive)
    _watcher: RecommendedWatcher,
}

impl NotifyWatcher {
    /// Create new notify-based file watcher
    ///
    /// # Arguments
    ///
    /// * `paths` - Paths to watch (files or directories)
    /// * `tx` - Channel sender for watch events
    ///
    /// # Returns
    ///
    /// Result containing the watcher or an error
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Watcher creation fails
    /// - Path watching fails
    /// - Path does not exist
    ///
    /// # Example
    ///
    /// ```no_run
    /// use clnrm_core::watch::NotifyWatcher;
    /// use std::path::PathBuf;
    /// use tokio::sync::mpsc;
    ///
    /// # async fn example() -> clnrm_core::error::Result<()> {
    /// let (tx, mut rx) = mpsc::channel(100);
    /// let watcher = NotifyWatcher::new(
    ///     vec![PathBuf::from("tests/")],
    ///     tx
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(paths: Vec<PathBuf>, tx: mpsc::Sender<WatchEvent>) -> Result<Self> {
        info!("Creating file watcher for {} path(s)", paths.len());

        // Create standard library channel for notify crate
        // (notify uses std::sync::mpsc, not tokio::sync::mpsc)
        let (std_tx, std_rx) = std_mpsc::channel::<notify::Result<Event>>();

        // Create watcher with event handler
        let mut watcher = notify::recommended_watcher(std_tx).map_err(|e| {
            CleanroomError::internal_error(format!("Failed to create file watcher: {}", e))
        })?;

        // Watch all specified paths
        for path in &paths {
            if !path.exists() {
                return Err(CleanroomError::validation_error(format!(
                    "Cannot watch non-existent path: {}",
                    path.display()
                ))
                .with_context("Path must exist before watching"));
            }

            info!("Watching path: {}", path.display());
            watcher.watch(path, RecursiveMode::Recursive).map_err(|e| {
                CleanroomError::internal_error(format!(
                    "Failed to watch path {}: {}",
                    path.display(),
                    e
                ))
            })?;
        }

        // Spawn background task to bridge std::mpsc to tokio::mpsc
        tokio::spawn(async move {
            while let Ok(res) = std_rx.recv() {
                match res {
                    Ok(event) => {
                        debug!("File system event: {:?}", event);

                        // Convert notify event to our WatchEvent
                        for path in event.paths {
                            let kind = match event.kind {
                                notify::EventKind::Create(_) => WatchEventKind::Create,
                                notify::EventKind::Modify(_) => WatchEventKind::Modify,
                                notify::EventKind::Remove(_) => WatchEventKind::Delete,
                                _ => WatchEventKind::Other,
                            };

                            let watch_event = WatchEvent { path, kind };

                            // Send to tokio channel
                            if let Err(e) = tx.send(watch_event).await {
                                error!("Failed to send watch event: {}", e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Watch error: {}", e);
                    }
                }
            }
            debug!("File watcher event loop terminated");
        });

        Ok(Self {
            _paths: paths,
            _watcher: watcher,
        })
    }
}

impl FileWatcher for NotifyWatcher {
    fn start(&self) -> Result<()> {
        // Watcher starts automatically when created
        debug!("Watcher already running");
        Ok(())
    }

    fn stop(&self) -> Result<()> {
        // Watcher stops when dropped
        debug!("Watcher will stop on drop");
        Ok(())
    }
}
