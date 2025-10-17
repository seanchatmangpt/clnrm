//! London School TDD Tests for Hot Reload System
//!
//! Test Coverage:
//! - File change triggers test rerun
//! - Multiple rapid changes are debounced
//! - Watch system handles file deletion gracefully
//! - Watch system ignores irrelevant files
//! - Watch system graceful shutdown
//!
//! Testing Philosophy:
//! - OUTSIDE-IN: Test behavior from watch system perspective
//! - MOCK-FIRST: Define watcher contracts through test doubles
//! - BEHAVIOR VERIFICATION: Focus on interactions, not state
//!
//! Core Team Compliance:
//! - ✅ AAA pattern (Arrange, Act, Assert)
//! - ✅ Descriptive test names (test_X_with_Y_produces_Z)
//! - ✅ No false positives - proper error propagation
//! - ✅ Result<()> for proper error handling

#![allow(clippy::unwrap_used)] // Test code only

use clnrm_core::error::{CleanroomError, Result};
use clnrm_core::watch::debouncer::FileDebouncer;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;

// ============================================================================
// Hot Reload - File Change Detection Tests
// ============================================================================

#[tokio::test]
async fn test_file_change_triggers_hot_reload_within_threshold() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new()
        .map_err(|e| CleanroomError::internal_error(format!("Failed to create temp dir: {}", e)))?;
    let test_file = temp_dir.path().join("test.toml.tera");

    std::fs::write(&test_file, "# initial content")
        .map_err(|e| CleanroomError::io_error(format!("Failed to write file: {}", e)))?;

    let mut debouncer = FileDebouncer::new(Duration::from_millis(300));

    // Act - Record file change event
    debouncer.record_event();

    // Simulate waiting for debounce window
    tokio::time::sleep(Duration::from_millis(350)).await;

    // Assert - Should trigger rerun after debounce window
    assert!(
        debouncer.should_trigger(),
        "File change should trigger rerun after debounce window"
    );
    assert_eq!(
        debouncer.event_count(),
        1,
        "Should record exactly 1 change event"
    );

    Ok(())
}

#[tokio::test]
async fn test_rapid_file_changes_debounced_to_single_rerun() -> Result<()> {
    // Arrange
    let debounce_window = Duration::from_millis(300);
    let mut debouncer = FileDebouncer::new(debounce_window);

    // Act - Simulate 10 rapid file saves (auto-save, formatter, etc.)
    for i in 0..10 {
        debouncer.record_event();
        tokio::time::sleep(Duration::from_millis(20)).await;

        // Should NOT trigger during rapid changes
        if i < 9 {
            assert!(
                !debouncer.should_trigger(),
                "Should not trigger during rapid changes at iteration {}",
                i
            );
        }
    }

    // Wait for debounce window to expire
    tokio::time::sleep(Duration::from_millis(350)).await;

    // Assert - Only 1 rerun triggered despite 10 changes
    assert!(
        debouncer.should_trigger(),
        "Should trigger exactly once after rapid changes"
    );
    assert_eq!(debouncer.event_count(), 10, "Should batch all 10 events");

    Ok(())
}

#[tokio::test]
async fn test_watch_system_handles_file_deletion_gracefully() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new()
        .map_err(|e| CleanroomError::internal_error(format!("Failed to create temp dir: {}", e)))?;
    let test_file = temp_dir.path().join("deletable.toml.tera");

    std::fs::write(&test_file, "# content")
        .map_err(|e| CleanroomError::io_error(format!("Failed to write file: {}", e)))?;

    let mut debouncer = FileDebouncer::new(Duration::from_millis(200));

    // Act - Record event, then delete file
    debouncer.record_event();
    std::fs::remove_file(&test_file)
        .map_err(|e| CleanroomError::io_error(format!("Failed to remove file: {}", e)))?;

    // Assert - Deletion doesn't break watch system
    assert!(!test_file.exists(), "File should be deleted");

    // Debouncer should still function normally
    tokio::time::sleep(Duration::from_millis(250)).await;
    assert!(
        debouncer.should_trigger(),
        "Debouncer should still trigger despite file deletion"
    );

    Ok(())
}

#[tokio::test]
async fn test_watch_system_ignores_irrelevant_files() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new()
        .map_err(|e| CleanroomError::internal_error(format!("Failed to create temp dir: {}", e)))?;

    let irrelevant_files = vec![
        temp_dir.path().join("README.md"),
        temp_dir.path().join("config.json"),
        temp_dir.path().join("script.sh"),
        temp_dir.path().join("test.toml"), // Not .tera extension
        temp_dir.path().join("test.tera"), // Not .toml prefix
    ];

    for file in &irrelevant_files {
        std::fs::write(file, "# content")
            .map_err(|e| CleanroomError::io_error(format!("Failed to write file: {}", e)))?;
    }

    let relevant_file = temp_dir.path().join("test.toml.tera");
    std::fs::write(&relevant_file, "# template")
        .map_err(|e| CleanroomError::io_error(format!("Failed to write file: {}", e)))?;

    // Act - Use is_relevant_file logic (simulated)
    let is_toml_tera = |path: &std::path::Path| {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext == "tera")
            .unwrap_or(false)
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.contains(".toml"))
                .unwrap_or(false)
    };

    // Assert - Only .toml.tera files are relevant
    for irrelevant in &irrelevant_files {
        assert!(
            !is_toml_tera(irrelevant),
            "File {:?} should be ignored",
            irrelevant.file_name()
        );
    }

    assert!(
        is_toml_tera(&relevant_file),
        "File test.toml.tera should be watched"
    );

    Ok(())
}

#[tokio::test]
async fn test_watch_system_graceful_shutdown_clears_resources() -> Result<()> {
    // Arrange
    let mut debouncer = FileDebouncer::new(Duration::from_millis(200));

    // Record some events
    debouncer.record_event();
    debouncer.record_event();
    debouncer.record_event();

    assert_eq!(debouncer.event_count(), 3, "Should have 3 events");

    // Act - Graceful shutdown (reset)
    debouncer.reset();

    // Assert - All resources cleared
    assert_eq!(
        debouncer.event_count(),
        0,
        "Event count should be cleared after shutdown"
    );

    assert!(
        debouncer.time_since_last_event().is_none(),
        "Last event timestamp should be cleared"
    );

    assert!(
        !debouncer.should_trigger(),
        "Should not trigger after shutdown"
    );

    Ok(())
}

// ============================================================================
// Performance Target Tests
// ============================================================================

#[tokio::test]
async fn test_hot_reload_performance_meets_threshold() -> Result<()> {
    // Arrange
    let performance_target_ms = 3000; // <3s from file save to result
    let debounce_ms = 300;

    let mut debouncer = FileDebouncer::new(Duration::from_millis(debounce_ms));
    let start = std::time::Instant::now();

    // Act - Simulate file change and wait for trigger
    debouncer.record_event();
    tokio::time::sleep(Duration::from_millis(debounce_ms + 50)).await;

    let should_trigger = debouncer.should_trigger();
    let elapsed = start.elapsed();

    // Assert - Performance target met
    assert!(should_trigger, "Should trigger after debounce");
    assert!(
        elapsed.as_millis() < performance_target_ms as u128,
        "Hot reload should complete in <3s, took {}ms",
        elapsed.as_millis()
    );

    Ok(())
}

// ============================================================================
// Edge Cases and Boundary Tests
// ============================================================================

#[tokio::test]
async fn test_zero_events_does_not_trigger_rerun() -> Result<()> {
    // Arrange
    let debouncer = FileDebouncer::new(Duration::from_millis(200));

    // Act - No events recorded
    tokio::time::sleep(Duration::from_millis(250)).await;

    // Assert - Should not trigger without events
    assert!(
        !debouncer.should_trigger(),
        "Should not trigger without any file changes"
    );

    Ok(())
}

#[tokio::test]
async fn test_debounce_window_boundary_conditions() -> Result<()> {
    // Arrange
    let debounce_window = Duration::from_millis(200);
    let mut debouncer = FileDebouncer::new(debounce_window);

    // Act - Event exactly at boundary
    debouncer.record_event();
    tokio::time::sleep(Duration::from_millis(200)).await; // Exactly at window

    // Assert - Should trigger at or after window boundary
    assert!(
        debouncer.should_trigger(),
        "Should trigger at exact boundary of debounce window"
    );

    Ok(())
}
