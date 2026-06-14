//! Watch mode implementation
//!
//! Handles file watching and automatic test re-execution when files change.

use crate::cli::types::CliConfig;
use crate::error::{CleanroomError, Result};
use notify::{event::EventKind, Event, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{error, info, warn};

use super::run_tests;

/// Watch test files and rerun on changes
pub async fn watch_and_run(paths: &[PathBuf], config: &CliConfig) -> Result<()> {
    info!("Watch mode enabled - monitoring test files for changes");
    info!("Press Ctrl+C to stop watching");

    let mut watch_config = config.clone();
    watch_config.watch = false;

    // Box::pin for recursion
    if let Err(e) = Box::pin(run_tests(paths, &watch_config)).await {
        warn!("Initial test run failed: {}", e);
    }

    let (tx, rx) = channel();
    let mut watcher =
        notify::recommended_watcher(move |res: std::result::Result<Event, notify::Error>| {
            if let Ok(event) = res {
                if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                    let _ = tx.send(event);
                }
            }
        })
        .map_err(|e| {
            CleanroomError::internal_error("Failed to create file watcher")
                .with_context("Watch mode initialization failed")
                .with_source(e.to_string())
        })?;

    for path in paths {
        watcher
            .watch(path.as_ref(), RecursiveMode::Recursive)
            .map_err(|e| {
                CleanroomError::internal_error("Failed to watch path")
                    .with_context(format!("Path: {}", path.display()))
                    .with_source(e.to_string())
            })?;
        info!("Watching: {}", path.display());
    }

    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(event) => {
                info!("File change detected: {:?}", event.paths);
                info!("Rerunning tests...");

                tokio::time::sleep(Duration::from_millis(100)).await;

                // Box::pin for recursion
                if let Err(e) = Box::pin(run_tests(paths, &watch_config)).await {
                    error!("Test run failed: {}", e);
                } else {
                    info!("All tests passed!");
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                continue;
            }
            Err(e) => {
                return Err(CleanroomError::internal_error("File watcher error")
                    .with_context("Watch mode encountered an error")
                    .with_source(e.to_string()));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Extended watch types and helpers
// ---------------------------------------------------------------------------

/// Configuration for a configurable watch-and-run loop.
pub struct WatchConfig {
    /// Paths to watch recursively.
    pub paths: Vec<PathBuf>,
    /// Milliseconds to wait after a change before running the command.
    pub debounce_ms: u64,
    /// Command to run; first element is the program, rest are arguments.
    pub command: Vec<String>,
    /// Substrings: if a changed path contains any of these it is ignored.
    pub ignore_patterns: Vec<String>,
}

/// The kind of filesystem change that triggered a `WatchEvent`.
#[derive(Debug, Clone)]
pub enum WatchEventKind {
    Created,
    Modified,
    Deleted,
}

/// A normalized filesystem event.
#[derive(Debug, Clone)]
pub struct WatchEvent {
    /// The file or directory that changed.
    pub path: PathBuf,
    /// What happened to it.
    pub kind: WatchEventKind,
    /// Unix timestamp in milliseconds when the event was observed.
    pub timestamp_ms: u64,
}

/// Run a configurable watch loop.
///
/// Watches `config.paths`, debounces by sleeping `config.debounce_ms` ms after
/// each event, then executes `config.command`.  Loops until a channel error.
pub fn watch_and_run_config(config: WatchConfig) -> crate::error::Result<()> {
    let (tx, rx) = channel::<notify::Event>();

    let mut watcher = notify::recommended_watcher(
        move |res: std::result::Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
    )
    .map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create watcher: {}", e))
    })?;

    for path in &config.paths {
        watcher
            .watch(path.as_ref(), RecursiveMode::Recursive)
            .map_err(|e| {
                CleanroomError::internal_error(format!(
                    "Failed to watch '{}': {}",
                    path.display(),
                    e
                ))
            })?;
        info!("Watching: {}", path.display());
    }

    loop {
        let event = match rx.recv() {
            Ok(e) => e,
            Err(_) => break,
        };

        // Determine the changed path
        let changed_path = match event.paths.first() {
            Some(p) => p.clone(),
            None => continue,
        };

        // Apply ignore patterns
        if should_ignore(&changed_path, &config.ignore_patterns) {
            continue;
        }

        // Debounce
        std::thread::sleep(Duration::from_millis(config.debounce_ms));

        // Drain any accumulated events
        while rx.try_recv().is_ok() {}

        // Run the configured command
        if let Some((program, args)) = config.command.split_first() {
            match std::process::Command::new(program).args(args).status() {
                Ok(status) => {
                    if status.success() {
                        info!("Command exited successfully");
                    } else {
                        warn!("Command exited with status: {}", status);
                    }
                }
                Err(e) => {
                    error!("Failed to run command '{}': {}", program, e);
                }
            }
        }
    }

    Ok(())
}

/// Convert a `notify::Event` into a `WatchEvent`, or `None` for unrecognised kinds.
pub fn parse_notify_event(event: &notify::Event) -> Option<WatchEvent> {
    let kind = match &event.kind {
        EventKind::Create(_) => WatchEventKind::Created,
        EventKind::Modify(_) => WatchEventKind::Modified,
        EventKind::Remove(_) => WatchEventKind::Deleted,
        _ => return None,
    };

    let path = event.paths.first()?.clone();

    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    Some(WatchEvent {
        path,
        kind,
        timestamp_ms,
    })
}

/// Return `true` if `path` should be ignored based on `patterns`.
pub fn should_ignore(path: &Path, patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy();
    patterns.iter().any(|p| path_str.contains(p.as_str()))
}
