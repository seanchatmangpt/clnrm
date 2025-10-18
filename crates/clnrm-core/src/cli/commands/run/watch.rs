//! Watch mode implementation
//!
//! Handles file watching and automatic test re-execution when files change.

use crate::cli::types::CliConfig;
use crate::error::{CleanroomError, Result};
use notify::{event::EventKind, Event, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;
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
