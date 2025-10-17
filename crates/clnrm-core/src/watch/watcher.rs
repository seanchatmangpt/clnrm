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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::Duration;

    // =================================================================
    // LONDON SCHOOL TDD: Mock-based tests for FileWatcher trait
    // =================================================================

    /// Mock file watcher for testing
    ///
    /// Allows verification of watcher interactions without file system I/O.
    /// This follows London School TDD - testing object collaborations.
    #[derive(Default)]
    struct MockFileWatcher {
        start_called: std::sync::Arc<std::sync::atomic::AtomicBool>,
        stop_called: std::sync::Arc<std::sync::atomic::AtomicBool>,
    }

    impl MockFileWatcher {
        fn new() -> Self {
            Self::default()
        }

        fn was_started(&self) -> bool {
            self.start_called.load(std::sync::atomic::Ordering::SeqCst)
        }

        fn was_stopped(&self) -> bool {
            self.stop_called.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    impl FileWatcher for MockFileWatcher {
        fn start(&self) -> Result<()> {
            self.start_called
                .store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }

        fn stop(&self) -> Result<()> {
            self.stop_called
                .store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn test_mock_watcher_starts() -> Result<()> {
        // Arrange
        let watcher = MockFileWatcher::new();

        // Act
        watcher.start()?;

        // Assert - Verify interaction occurred
        assert!(watcher.was_started(), "Watcher should have been started");
        Ok(())
    }

    #[test]
    fn test_mock_watcher_stops() -> Result<()> {
        // Arrange
        let watcher = MockFileWatcher::new();

        // Act
        watcher.stop()?;

        // Assert - Verify interaction occurred
        assert!(watcher.was_stopped(), "Watcher should have been stopped");
        Ok(())
    }

    #[test]
    fn test_mock_watcher_lifecycle() -> Result<()> {
        // Arrange
        let watcher = MockFileWatcher::new();

        // Act - Start then stop
        watcher.start()?;
        watcher.stop()?;

        // Assert - Verify both interactions
        assert!(watcher.was_started(), "Watcher should have been started");
        assert!(watcher.was_stopped(), "Watcher should have been stopped");
        Ok(())
    }

    // =================================================================
    // WatchConfig tests
    // =================================================================

    #[test]
    fn test_watch_config_creation() {
        // Arrange & Act
        let config = WatchConfig::new(vec![PathBuf::from("tests/")], 300, true);

        // Assert
        assert_eq!(config.paths.len(), 1);
        assert_eq!(config.debounce_ms, 300);
        assert!(config.clear_screen);
    }

    #[test]
    fn test_watch_config_with_cli_config() {
        // Arrange
        let cli_config = CliConfig {
            parallel: true,
            jobs: 4,
            ..Default::default()
        };

        // Act
        let config =
            WatchConfig::new(vec![PathBuf::from("tests/")], 300, false).with_cli_config(cli_config);

        // Assert
        assert!(config.cli_config.parallel);
        assert_eq!(config.cli_config.jobs, 4);
    }

    // =================================================================
    // WatchEvent tests
    // =================================================================

    #[test]
    fn test_watch_event_creation() {
        // Arrange & Act
        let event = WatchEvent {
            path: PathBuf::from("test.toml.tera"),
            kind: WatchEventKind::Modify,
        };

        // Assert
        assert_eq!(event.path, PathBuf::from("test.toml.tera"));
        assert_eq!(event.kind, WatchEventKind::Modify);
    }

    #[test]
    fn test_watch_event_kinds() {
        // Assert - Verify all event kinds
        assert_eq!(WatchEventKind::Create, WatchEventKind::Create);
        assert_eq!(WatchEventKind::Modify, WatchEventKind::Modify);
        assert_eq!(WatchEventKind::Delete, WatchEventKind::Delete);
        assert_eq!(WatchEventKind::Other, WatchEventKind::Other);

        assert_ne!(WatchEventKind::Create, WatchEventKind::Modify);
    }

    // =================================================================
    // NotifyWatcher integration tests
    // =================================================================

    #[tokio::test]
    async fn test_notify_watcher_rejects_nonexistent_path() {
        // Arrange
        let (tx, _rx) = mpsc::channel(100);
        let paths = vec![PathBuf::from("/nonexistent/path/that/does/not/exist")];

        // Act
        let result = NotifyWatcher::new(paths, tx);

        // Assert
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("non-existent"));
    }

    #[tokio::test]
    async fn test_notify_watcher_creates_successfully_with_valid_path() -> Result<()> {
        // Arrange
        let temp_dir = tempfile::tempdir().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
        })?;
        let (tx, _rx) = mpsc::channel(100);

        // Act
        let result = NotifyWatcher::new(vec![temp_dir.path().to_path_buf()], tx);

        // Assert
        assert!(result.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn test_notify_watcher_detects_file_creation() -> Result<()> {
        // Arrange
        let temp_dir = tempfile::tempdir().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
        })?;
        let (tx, mut rx) = mpsc::channel(100);

        let _watcher = NotifyWatcher::new(vec![temp_dir.path().to_path_buf()], tx)?;

        // Wait for watcher to initialize
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Act - Create a file
        let test_file = temp_dir.path().join("test.toml.tera");
        fs::write(&test_file, "# test")
            .map_err(|e| CleanroomError::internal_error(format!("Failed to write file: {}", e)))?;

        // Wait for event
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Assert - Should receive create event
        let mut received_event = false;
        while let Ok(event) = rx.try_recv() {
            if event.path == test_file {
                received_event = true;
                break;
            }
        }

        assert!(received_event, "Should have received file creation event");
        Ok(())
    }

    #[tokio::test]
    async fn test_notify_watcher_detects_file_modification() -> Result<()> {
        // Arrange
        let temp_dir = tempfile::tempdir().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
        })?;
        let test_file = temp_dir.path().join("test.toml.tera");

        // Create file before starting watcher
        fs::write(&test_file, "# initial")
            .map_err(|e| CleanroomError::internal_error(format!("Failed to write file: {}", e)))?;

        let (tx, mut rx) = mpsc::channel(100);
        let _watcher = NotifyWatcher::new(vec![temp_dir.path().to_path_buf()], tx)?;

        // Wait for watcher to initialize
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Act - Modify the file
        fs::write(&test_file, "# modified")
            .map_err(|e| CleanroomError::internal_error(format!("Failed to write file: {}", e)))?;

        // Wait for event
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Assert - Should receive modify event
        let mut received_event = false;
        while let Ok(event) = rx.try_recv() {
            if event.path == test_file && event.kind == WatchEventKind::Modify {
                received_event = true;
                break;
            }
        }

        assert!(
            received_event,
            "Should have received file modification event"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_notify_watcher_watches_multiple_paths() -> Result<()> {
        // Arrange
        let temp_dir1 = tempfile::tempdir().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
        })?;
        let temp_dir2 = tempfile::tempdir().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
        })?;

        let (tx, mut rx) = mpsc::channel(100);
        let _watcher = NotifyWatcher::new(
            vec![
                temp_dir1.path().to_path_buf(),
                temp_dir2.path().to_path_buf(),
            ],
            tx,
        )?;

        // Wait for watcher to initialize
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Act - Create files in both directories
        let file1 = temp_dir1.path().join("test1.toml.tera");
        let file2 = temp_dir2.path().join("test2.toml.tera");

        fs::write(&file1, "# test1")
            .map_err(|e| CleanroomError::internal_error(format!("Failed to write file: {}", e)))?;
        fs::write(&file2, "# test2")
            .map_err(|e| CleanroomError::internal_error(format!("Failed to write file: {}", e)))?;

        // Wait for events
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Assert - Should receive events from both paths
        let mut found_file1 = false;
        let mut found_file2 = false;

        while let Ok(event) = rx.try_recv() {
            if event.path == file1 {
                found_file1 = true;
            }
            if event.path == file2 {
                found_file2 = true;
            }
        }

        assert!(found_file1, "Should detect changes in first directory");
        assert!(found_file2, "Should detect changes in second directory");
        Ok(())
    }
}
