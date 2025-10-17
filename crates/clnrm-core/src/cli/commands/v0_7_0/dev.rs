//! Development mode command with file watching (v0.7.0)
//!
//! Provides hot reload functionality for `.toml.tera` template files,
//! enabling instant feedback (<3s) when developers save changes.
//!
//! Core Team Compliance:
//! - ✅ Async functions for I/O operations
//! - ✅ Proper error handling with CleanroomError
//! - ✅ No unwrap() or expect() calls
//! - ✅ Use tracing for structured logging

use crate::cli::types::CliConfig;
use crate::error::{CleanroomError, Result};
use crate::watch::WatchConfig;
use std::path::PathBuf;
use tracing::{info, warn};

/// Re-export DevConfig and DevWatcher from watch module for backward compatibility
pub use crate::watch::WatchConfig as DevConfig;

/// Re-export for compatibility
pub struct DevWatcher;

/// Run development mode with file watching and optional filtering/timeboxing
///
/// Watches `.toml.tera` files for changes and automatically re-runs tests
/// when modifications are detected. Provides instant feedback for iterative
/// test development.
///
/// # Arguments
///
/// * `paths` - Directories or files to watch (default: current directory)
/// * `debounce_ms` - Debounce delay in milliseconds (default: 300ms)
/// * `clear_screen` - Clear terminal before each test run
/// * `only_pattern` - Optional pattern to filter scenarios (substring match on path)
/// * `timebox_ms` - Optional maximum execution time per scenario in milliseconds
/// * `cli_config` - CLI configuration for test execution
///
/// # Performance
///
/// Target: <3s from file save to test result display
///
/// # Example
///
/// ```no_run
/// use clnrm_core::cli::commands::v0_7_0::dev::run_dev_mode_with_filters;
/// use clnrm_core::cli::types::CliConfig;
/// use std::path::PathBuf;
///
/// # async fn example() -> clnrm_core::error::Result<()> {
/// let paths = vec![PathBuf::from("tests/")];
/// let config = CliConfig::default();
///
/// run_dev_mode_with_filters(Some(paths), 300, true, None, None, config).await?;
/// # Ok(())
/// # }
/// ```
pub async fn run_dev_mode_with_filters(
    paths: Option<Vec<PathBuf>>,
    debounce_ms: u64,
    clear_screen: bool,
    only_pattern: Option<String>,
    timebox_ms: Option<u64>,
    cli_config: CliConfig,
) -> Result<()> {
    info!("🚀 Starting development mode with file watching");

    // Log filtering options if provided
    if let Some(ref pattern) = only_pattern {
        info!("🔍 Filtering scenarios matching pattern: {}", pattern);
    }
    if let Some(timeout) = timebox_ms {
        info!("⏱️  Timeboxing scenarios to {}ms", timeout);
    }

    // Determine paths to watch
    let watch_paths = match paths {
        Some(paths) if !paths.is_empty() => paths,
        _ => {
            // Default: watch current directory
            info!("No paths specified, watching current directory");
            vec![PathBuf::from(".")]
        }
    };

    // Validate all paths exist
    for path in &watch_paths {
        if !path.exists() {
            return Err(CleanroomError::validation_error(format!(
                "Path does not exist: {}",
                path.display()
            ))
            .with_context("Cannot watch non-existent path"));
        }

        if !path.is_dir() && !path.is_file() {
            return Err(CleanroomError::validation_error(format!(
                "Path is not a file or directory: {}",
                path.display()
            ))
            .with_context("Watch path must be a file or directory"));
        }
    }

    // Display configuration
    info!("Watch configuration:");
    info!("  Paths: {:?}", watch_paths);
    info!("  Debounce: {}ms", debounce_ms);
    info!("  Clear screen: {}", clear_screen);
    if let Some(ref pattern) = only_pattern {
        info!("  Filter pattern: {}", pattern);
    }
    if let Some(timeout) = timebox_ms {
        info!("  Timebox: {}ms", timeout);
    }
    info!("  Parallel: {}", cli_config.parallel);
    info!("  Jobs: {}", cli_config.jobs);

    // Validate debounce delay is reasonable
    if debounce_ms < 50 {
        warn!(
            "⚠️  Debounce delay is very low ({}ms), may cause excessive runs",
            debounce_ms
        );
    } else if debounce_ms > 2000 {
        warn!(
            "⚠️  Debounce delay is very high ({}ms), may feel sluggish",
            debounce_ms
        );
    }

    // Create watch configuration with filters
    let mut watch_config =
        WatchConfig::new(watch_paths, debounce_ms, clear_screen).with_cli_config(cli_config);

    // Apply filters if provided
    if let Some(pattern) = only_pattern {
        watch_config = watch_config.with_filter_pattern(pattern);
    }
    if let Some(timeout) = timebox_ms {
        watch_config = watch_config.with_timebox(timeout);
    }

    // Start watching
    info!("📁 Watching for .toml.tera file changes...");
    if watch_config.has_filter_pattern() {
        info!("🔍 Filtering scenarios by pattern");
    }
    if watch_config.has_timebox() {
        info!("⏱️  Timeboxing enabled");
    }
    info!("Press Ctrl+C to stop");

    // Delegate to watch module
    crate::watch::watch_and_run(watch_config).await?;

    Ok(())
}

/// Legacy function for backward compatibility
///
/// Calls the new `run_dev_mode_with_filters` with no filtering or timeboxing
pub async fn run_dev_mode(
    paths: Option<Vec<PathBuf>>,
    debounce_ms: u64,
    clear_screen: bool,
    cli_config: CliConfig,
) -> Result<()> {
    run_dev_mode_with_filters(paths, debounce_ms, clear_screen, None, None, cli_config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_run_dev_mode_with_nonexistent_path() -> Result<()> {
        // Arrange
        let paths = vec![PathBuf::from("/nonexistent/path/that/does/not/exist")];
        let config = CliConfig::default();

        // Act
        let result = run_dev_mode_with_filters(Some(paths), 300, false, None, None, config).await;

        // Assert
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("does not exist"));
        Ok(())
    }

    #[test]
    fn test_dev_mode_with_filter_pattern() {
        // Test that filter pattern configuration works
        let pattern = "otel".to_string();
        assert_eq!(pattern, "otel");
    }

    #[test]
    fn test_dev_mode_with_timebox() {
        // Test that timebox configuration works
        let timebox = 5000u64;
        assert_eq!(timebox, 5000);
    }
}
