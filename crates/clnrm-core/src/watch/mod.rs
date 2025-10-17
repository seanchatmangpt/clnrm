//! File watching subsystem for auto-test execution
//!
//! Provides file system watching capabilities for development mode,
//! automatically re-running tests when `.toml.tera` files change.
//!
//! # Architecture
//!
//! - `FileWatcher` trait: Abstract file watching interface (testable via mocks)
//! - `NotifyWatcher`: Production implementation using `notify` crate
//! - `WatchConfig`: Configuration for watch behavior
//! - `debouncer`: Time-based event batching to prevent excessive runs
//!
//! # London TDD Approach
//!
//! This module follows London School (mockist) TDD:
//! - Tests verify object interactions and collaborations
//! - Mocks define contracts between components
//! - Focus on behavior verification over state testing
//!
//! # Core Team Compliance
//!
//! - ✅ Async functions for I/O operations (file watching)
//! - ✅ Sync trait methods with tokio::task::block_in_place
//! - ✅ Proper error handling (no unwrap/expect)
//! - ✅ Result<T, CleanroomError> for all fallible operations
//! - ✅ Structured logging with tracing
//!
//! # Example
//!
//! ```no_run
//! use clnrm_core::watch::{WatchConfig, watch_and_run};
//! use clnrm_core::cli::types::CliConfig;
//! use std::path::PathBuf;
//!
//! # async fn example() -> clnrm_core::error::Result<()> {
//! let paths = vec![PathBuf::from("tests/")];
//! let config = WatchConfig::new(paths, 300, true)
//!     .with_cli_config(CliConfig::default());
//!
//! watch_and_run(config).await?;
//! # Ok(())
//! # }
//! ```

pub mod debouncer;
pub mod watcher;

pub use debouncer::FileDebouncer;
pub use watcher::{FileWatcher, NotifyWatcher, WatchConfig, WatchEvent};

use crate::error::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Main watch loop that monitors files and runs tests on changes
///
/// This function sets up file watching, processes events with debouncing,
/// and triggers test execution when `.toml.tera` files change.
///
/// # Arguments
///
/// * `config` - Watch configuration including paths and CLI settings
///
/// # Behavior
///
/// 1. Creates file watcher for specified paths
/// 2. Filters events for `.toml.tera` files only
/// 3. Debounces rapid file saves (prevents excessive runs)
/// 4. Executes tests when debounce window expires
/// 5. Clears screen between runs if configured
///
/// # Performance Target
///
/// <3s from file save to test result display
///
/// # Example
///
/// ```no_run
/// use clnrm_core::watch::{WatchConfig, watch_and_run};
/// use std::path::PathBuf;
///
/// # async fn example() -> clnrm_core::error::Result<()> {
/// let config = WatchConfig::new(
///     vec![PathBuf::from("tests/")],
///     300,
///     true
/// );
///
/// watch_and_run(config).await?;
/// # Ok(())
/// # }
/// ```
pub async fn watch_and_run(config: WatchConfig) -> Result<()> {
    info!("Starting watch loop");

    // Create event channel for file notifications
    let (tx, mut rx) = mpsc::channel::<WatchEvent>(100);

    // Create and start watcher
    let watcher = NotifyWatcher::new(config.paths.clone(), tx)?;
    let _watcher_guard = Arc::new(watcher);

    // Create debouncer for event batching
    let debounce_duration = std::time::Duration::from_millis(config.debounce_ms);
    let mut debouncer = FileDebouncer::new(debounce_duration);

    // Run initial tests
    info!("🧪 Running initial tests...");
    run_tests(&config).await?;

    info!("👀 Watching for changes (Press Ctrl+C to stop)...");

    // Main watch loop
    loop {
        tokio::select! {
            // Handle file events
            Some(event) = rx.recv() => {
                debug!("Received watch event: {:?}", event);

                // Filter for .toml.tera files
                if is_relevant_file(&event.path) {
                    info!("📝 Change detected: {}", event.path.display());
                    debouncer.record_event();
                } else {
                    debug!("Ignoring non-template file: {}", event.path.display());
                }
            }

            // Check debouncer periodically
            _ = tokio::time::sleep(std::time::Duration::from_millis(50)) => {
                if debouncer.should_trigger() {
                    let event_count = debouncer.event_count();
                    info!("🔄 Running tests ({} change{})...",
                        event_count,
                        if event_count == 1 { "" } else { "s" }
                    );

                    // Clear screen if configured
                    if config.clear_screen {
                        clear_terminal();
                    }

                    // Run tests and handle errors gracefully
                    match run_tests(&config).await {
                        Ok(_) => {
                            info!("✅ Tests completed");
                        }
                        Err(e) => {
                            error!("❌ Test execution failed: {}", e);
                            // Don't exit on test failure - keep watching
                        }
                    }

                    debouncer.reset();
                    info!("👀 Watching for changes...");
                }
            }
        }
    }
}

/// Execute tests with configured options
///
/// # Arguments
///
/// * `config` - Watch configuration containing CLI settings
///
/// # Returns
///
/// Result indicating success or failure of test execution
async fn run_tests(config: &WatchConfig) -> Result<()> {
    // Use the CLI configuration to run tests
    // This integrates with the existing test runner

    debug!(
        "Executing tests with config: parallel={}, jobs={}",
        config.cli_config.parallel, config.cli_config.jobs
    );

    // Determine test paths to run
    let test_paths = determine_test_paths(&config.paths)?;

    if test_paths.is_empty() {
        warn!("No test files found in watched paths");
        return Ok(());
    }

    info!("Running {} test file(s)", test_paths.len());

    // Execute tests using the run command logic
    crate::cli::commands::run::run_tests(&test_paths, &config.cli_config).await
}

/// Determine which test files to run from watched paths
///
/// Scans watched paths for `.toml.tera` files that represent tests.
///
/// # Arguments
///
/// * `paths` - Paths being watched (files or directories)
///
/// # Returns
///
/// Vector of test file paths to execute
fn determine_test_paths(paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut test_paths = Vec::new();

    for path in paths {
        if path.is_file() {
            // Single file - check if it's a test file
            if is_relevant_file(path) {
                test_paths.push(path.clone());
            }
        } else if path.is_dir() {
            // Directory - find all .toml.tera files
            let walker = walkdir::WalkDir::new(path)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .filter(|e| is_relevant_file(e.path()));

            for entry in walker {
                test_paths.push(entry.path().to_path_buf());
            }
        }
    }

    Ok(test_paths)
}

/// Check if a file is relevant for test watching
///
/// Currently watches `.toml.tera` template files.
///
/// # Arguments
///
/// * `path` - File path to check
///
/// # Returns
///
/// `true` if file should trigger test runs, `false` otherwise
fn is_relevant_file(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "tera")
        .unwrap_or(false)
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.contains(".toml"))
            .unwrap_or(false)
}

/// Clear terminal screen
///
/// Provides clean output between test runs.
fn clear_terminal() {
    // ANSI escape sequence to clear screen and move cursor to top
    print!("\x1B[2J\x1B[1;1H");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CleanroomError;

    #[test]
    fn test_is_relevant_file_matches_toml_tera() {
        // Arrange
        let path = PathBuf::from("test.toml.tera");

        // Act
        let result = is_relevant_file(&path);

        // Assert
        assert!(result, "Should match .toml.tera files");
    }

    #[test]
    fn test_is_relevant_file_rejects_other_extensions() {
        // Arrange
        let test_cases = vec![
            PathBuf::from("test.rs"),
            PathBuf::from("test.toml"),
            PathBuf::from("test.tera"),
            PathBuf::from("test.txt"),
        ];

        for path in test_cases {
            // Act
            let result = is_relevant_file(&path);

            // Assert
            assert!(!result, "Should reject {}", path.display());
        }
    }

    #[test]
    fn test_is_relevant_file_handles_no_extension() {
        // Arrange
        let path = PathBuf::from("test");

        // Act
        let result = is_relevant_file(&path);

        // Assert
        assert!(!result, "Should reject files without extension");
    }

    #[test]
    fn test_determine_test_paths_with_file() -> Result<()> {
        // Arrange
        let temp_dir = tempfile::tempdir().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
        })?;
        let test_file = temp_dir.path().join("test.toml.tera");
        std::fs::write(&test_file, "# test")
            .map_err(|e| CleanroomError::internal_error(format!("Failed to write file: {}", e)))?;

        // Act
        let result = determine_test_paths(std::slice::from_ref(&test_file))?;

        // Assert
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], test_file);
        Ok(())
    }

    #[test]
    fn test_determine_test_paths_with_directory() -> Result<()> {
        // Arrange
        let temp_dir = tempfile::tempdir().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
        })?;

        let test_file1 = temp_dir.path().join("test1.toml.tera");
        let test_file2 = temp_dir.path().join("test2.toml.tera");
        let other_file = temp_dir.path().join("other.txt");

        std::fs::write(&test_file1, "# test1")
            .map_err(|e| CleanroomError::internal_error(format!("Failed to write file: {}", e)))?;
        std::fs::write(&test_file2, "# test2")
            .map_err(|e| CleanroomError::internal_error(format!("Failed to write file: {}", e)))?;
        std::fs::write(&other_file, "# other")
            .map_err(|e| CleanroomError::internal_error(format!("Failed to write file: {}", e)))?;

        // Act
        let result = determine_test_paths(&[temp_dir.path().to_path_buf()])?;

        // Assert
        assert_eq!(result.len(), 2);
        assert!(result.contains(&test_file1));
        assert!(result.contains(&test_file2));
        Ok(())
    }

    #[test]
    fn test_determine_test_paths_empty_directory() -> Result<()> {
        // Arrange
        let temp_dir = tempfile::tempdir().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
        })?;

        // Act
        let result = determine_test_paths(&[temp_dir.path().to_path_buf()])?;

        // Assert
        assert_eq!(result.len(), 0);
        Ok(())
    }
}
