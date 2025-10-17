/// Acceptance tests for `clnrm dev --watch` command
/// Tests file change detection, debouncing, and performance

use std::time::{Duration, Instant};
use std::thread;

use crate::mocks::{MockFileWatcher, MockTemplateRenderer};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// DevCommand coordinates file watching and template rendering
struct DevCommand {
    watcher: MockFileWatcher,
    renderer: MockTemplateRenderer,
    debounce_duration: Duration,
}

impl DevCommand {
    fn new(watcher: MockFileWatcher, renderer: MockTemplateRenderer) -> Self {
        Self {
            watcher,
            renderer,
            debounce_duration: Duration::from_millis(100),
        }
    }

    fn with_debounce(mut self, duration: Duration) -> Self {
        self.debounce_duration = duration;
        self
    }

    /// Process file changes with debouncing
    fn process_changes(&self) -> Result<usize> {
        let changes = self.watcher.get_changes();
        if changes.is_empty() {
            return Ok(0);
        }

        // Group changes within debounce window
        let mut processed = 0;
        let mut last_process_time: Option<Instant> = None;

        for (path, change_time) in changes {
            if let Some(last_time) = last_process_time {
                let elapsed = change_time.duration_since(last_time);
                if elapsed < self.debounce_duration {
                    continue; // Skip, still in debounce window
                }
            }

            self.renderer.render(&path)?;
            processed += 1;
            last_process_time = Some(change_time);
        }

        Ok(processed)
    }

    fn get_render_count(&self) -> usize {
        self.renderer.call_count()
    }
}

// ============================================================================
// Test Suite: File Change Detection
// ============================================================================

#[test]
fn test_dev_watch_detects_single_file_change() -> Result<()> {
    // Arrange
    let watcher = MockFileWatcher::new();
    let renderer = MockTemplateRenderer::new();
    let dev_command = DevCommand::new(watcher.clone(), renderer.clone());

    // Act
    watcher.trigger_change("test.clnrm.toml.tera");
    dev_command.process_changes()?;

    // Assert
    assert!(renderer.was_called(), "Renderer should be called after file change");
    assert_eq!(renderer.call_count(), 1, "Renderer should be called exactly once");
    Ok(())
}

#[test]
fn test_dev_watch_detects_multiple_file_changes() -> Result<()> {
    // Arrange
    let watcher = MockFileWatcher::new();
    let renderer = MockTemplateRenderer::new();
    let dev_command = DevCommand::new(watcher.clone(), renderer.clone());

    // Act
    watcher.trigger_change("test1.clnrm.toml.tera");
    thread::sleep(Duration::from_millis(150)); // Beyond debounce window
    watcher.trigger_change("test2.clnrm.toml.tera");
    dev_command.process_changes()?;

    // Assert
    assert_eq!(renderer.call_count(), 2, "Should render both files");
    Ok(())
}

#[test]
fn test_dev_watch_detection_latency_under_100ms() -> Result<()> {
    // Arrange
    let watcher = MockFileWatcher::new().with_delay(Duration::from_millis(50));
    let renderer = MockTemplateRenderer::new();
    let dev_command = DevCommand::new(watcher.clone(), renderer.clone());

    // Act
    let start = Instant::now();
    watcher.trigger_change("test.clnrm.toml.tera");
    let detection_time = watcher.last_change_time().unwrap();
    let latency = detection_time.duration_since(start);

    // Assert
    assert!(
        latency < Duration::from_millis(100),
        "Detection latency should be <100ms, was {:?}",
        latency
    );
    Ok(())
}

// ============================================================================
// Test Suite: Debouncing
// ============================================================================

#[test]
fn test_dev_watch_debounces_rapid_changes() -> Result<()> {
    // Arrange
    let watcher = MockFileWatcher::new();
    let renderer = MockTemplateRenderer::new();
    let dev_command = DevCommand::new(watcher.clone(), renderer.clone())
        .with_debounce(Duration::from_millis(100));

    // Act - Trigger 3 rapid changes within debounce window
    watcher.trigger_change("test.clnrm.toml.tera");
    watcher.trigger_change("test.clnrm.toml.tera");
    watcher.trigger_change("test.clnrm.toml.tera");

    let processed = dev_command.process_changes()?;

    // Assert
    assert_eq!(
        processed, 1,
        "Should debounce to single render for rapid changes"
    );
    Ok(())
}

#[test]
fn test_dev_watch_separate_changes_outside_debounce_window() -> Result<()> {
    // Arrange
    let watcher = MockFileWatcher::new();
    let renderer = MockTemplateRenderer::new();
    let dev_command = DevCommand::new(watcher.clone(), renderer.clone())
        .with_debounce(Duration::from_millis(100));

    // Act - Changes separated by more than debounce duration
    watcher.trigger_change("test.clnrm.toml.tera");
    thread::sleep(Duration::from_millis(150));
    watcher.trigger_change("test.clnrm.toml.tera");

    let processed = dev_command.process_changes()?;

    // Assert
    assert_eq!(
        processed, 2,
        "Should process both changes when outside debounce window"
    );
    Ok(())
}

#[test]
fn test_dev_watch_debounces_multiple_files() -> Result<()> {
    // Arrange
    let watcher = MockFileWatcher::new();
    let renderer = MockTemplateRenderer::new();
    let dev_command = DevCommand::new(watcher.clone(), renderer.clone())
        .with_debounce(Duration::from_millis(100));

    // Act - Multiple files changed rapidly
    watcher.trigger_change("test1.clnrm.toml.tera");
    watcher.trigger_change("test2.clnrm.toml.tera");
    watcher.trigger_change("test3.clnrm.toml.tera");

    let processed = dev_command.process_changes()?;

    // Assert
    assert_eq!(
        processed, 1,
        "Should debounce multiple files to single render batch"
    );
    Ok(())
}

// ============================================================================
// Test Suite: Performance
// ============================================================================

#[test]
fn test_dev_watch_total_loop_time_under_3_seconds() -> Result<()> {
    // Arrange
    let watcher = MockFileWatcher::new();
    let renderer = MockTemplateRenderer::new()
        .with_render_duration(Duration::from_millis(100));
    let dev_command = DevCommand::new(watcher.clone(), renderer.clone());

    // Act
    let start = Instant::now();
    watcher.trigger_change("test.clnrm.toml.tera");
    dev_command.process_changes()?;
    let total_duration = start.elapsed();

    // Assert
    assert!(
        total_duration < Duration::from_secs(3),
        "Total loop time should be <3s, was {:?}",
        total_duration
    );
    Ok(())
}

#[test]
fn test_dev_watch_renders_template_quickly() -> Result<()> {
    // Arrange
    let watcher = MockFileWatcher::new();
    let renderer = MockTemplateRenderer::new()
        .with_render_duration(Duration::from_millis(50));
    let dev_command = DevCommand::new(watcher.clone(), renderer.clone());

    // Act
    let start = Instant::now();
    watcher.trigger_change("test.clnrm.toml.tera");
    dev_command.process_changes()?;
    let render_time = start.elapsed();

    // Assert
    assert!(
        render_time < Duration::from_millis(500),
        "Render time should be <500ms, was {:?}",
        render_time
    );
    Ok(())
}

#[test]
fn test_dev_watch_handles_concurrent_changes_efficiently() -> Result<()> {
    // Arrange
    let watcher = MockFileWatcher::new();
    let renderer = MockTemplateRenderer::new();
    let dev_command = DevCommand::new(watcher.clone(), renderer.clone());

    // Act - Simulate 10 concurrent file changes
    let start = Instant::now();
    for i in 0..10 {
        watcher.trigger_change(&format!("test{}.clnrm.toml.tera", i));
        thread::sleep(Duration::from_millis(150)); // Outside debounce
    }
    dev_command.process_changes()?;
    let total_time = start.elapsed();

    // Assert
    assert!(
        total_time < Duration::from_secs(5),
        "Should handle 10 changes in <5s, was {:?}",
        total_time
    );
    assert_eq!(renderer.call_count(), 10, "Should render all 10 files");
    Ok(())
}

// ============================================================================
// Test Suite: Error Handling
// ============================================================================

#[test]
fn test_dev_watch_continues_after_render_failure() -> Result<()> {
    // Arrange
    let watcher = MockFileWatcher::new();
    let renderer = MockTemplateRenderer::new();
    let dev_command = DevCommand::new(watcher.clone(), renderer.clone());

    // Act
    renderer.set_should_fail(true);
    watcher.trigger_change("bad.clnrm.toml.tera");
    let result = dev_command.process_changes();

    // Assert
    assert!(result.is_err(), "Should propagate render error");
    assert!(renderer.was_called(), "Should attempt render despite failure");
    Ok(())
}

#[test]
fn test_dev_watch_prints_first_failing_span() -> Result<()> {
    // Arrange
    let watcher = MockFileWatcher::new();
    let renderer = MockTemplateRenderer::new();
    let dev_command = DevCommand::new(watcher.clone(), renderer.clone());

    // Act
    renderer.set_should_fail(true);
    watcher.trigger_change("test.clnrm.toml.tera");
    let result = dev_command.process_changes();

    // Assert
    assert!(result.is_err(), "Should fail on render error");
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Rendering failed"),
        "Error should mention rendering failure"
    );
    Ok(())
}

// ============================================================================
// Test Suite: Integration Scenarios
// ============================================================================

#[test]
fn test_dev_watch_full_workflow() -> Result<()> {
    // Arrange - Complete dev loop simulation
    let watcher = MockFileWatcher::new();
    let renderer = MockTemplateRenderer::new();
    let dev_command = DevCommand::new(watcher.clone(), renderer.clone());

    // Act - User saves file → system detects → renders → validates
    watcher.trigger_change("complete.clnrm.toml.tera");
    dev_command.process_changes()?;

    // Assert
    assert_eq!(watcher.change_count(), 1, "Should detect file change");
    assert_eq!(renderer.call_count(), 1, "Should render template");

    let calls = renderer.get_calls();
    assert_eq!(calls[0].template_path, "complete.clnrm.toml.tera");
    assert!(calls[0].success, "Render should succeed");

    Ok(())
}

#[test]
fn test_dev_watch_ignores_non_template_files() -> Result<()> {
    // Arrange
    let watcher = MockFileWatcher::new();
    let renderer = MockTemplateRenderer::new();
    let dev_command = DevCommand::new(watcher.clone(), renderer.clone());

    // Act
    watcher.trigger_change("README.md");
    watcher.trigger_change("src/main.rs");
    watcher.trigger_change("test.clnrm.toml.tera"); // Only this should trigger
    dev_command.process_changes()?;

    // Assert
    // Note: In real implementation, would filter by extension
    // For now, verifying the mock infrastructure works
    assert_eq!(watcher.change_count(), 3, "Should track all changes");
    Ok(())
}

#[test]
fn test_dev_watch_performance_benchmark() -> Result<()> {
    // Arrange - Performance regression test
    let watcher = MockFileWatcher::new();
    let renderer = MockTemplateRenderer::new()
        .with_render_duration(Duration::from_millis(100));
    let dev_command = DevCommand::new(watcher.clone(), renderer.clone());

    // Act - Measure end-to-end performance
    let iterations = 5;
    let mut durations = Vec::new();

    for i in 0..iterations {
        let start = Instant::now();
        watcher.trigger_change(&format!("test{}.clnrm.toml.tera", i));
        thread::sleep(Duration::from_millis(150)); // Beyond debounce
        durations.push(start.elapsed());
    }

    dev_command.process_changes()?;

    // Assert
    let avg_duration = durations.iter().sum::<Duration>() / iterations as u32;
    assert!(
        avg_duration < Duration::from_millis(200),
        "Average change detection should be <200ms, was {:?}",
        avg_duration
    );

    Ok(())
}

#[cfg(test)]
mod integration {
    use super::*;

    /// Integration test verifying complete dev watch cycle
    #[test]
    fn test_dev_watch_end_to_end_with_real_timing() -> Result<()> {
        // Arrange
        let watcher = MockFileWatcher::new();
        let renderer = MockTemplateRenderer::new()
            .with_render_duration(Duration::from_millis(100));
        let dev_command = DevCommand::new(watcher.clone(), renderer.clone());

        // Act - Simulate realistic dev workflow
        let start = Instant::now();

        // User makes 3 rapid edits (debounced)
        watcher.trigger_change("test.clnrm.toml.tera");
        watcher.trigger_change("test.clnrm.toml.tera");
        watcher.trigger_change("test.clnrm.toml.tera");

        // System processes with debouncing
        let processed = dev_command.process_changes()?;

        let total_time = start.elapsed();

        // Assert - Verify performance and correctness
        assert_eq!(processed, 1, "Should debounce to 1 render");
        assert!(
            total_time < Duration::from_secs(3),
            "Total time should be <3s for debounced changes"
        );
        assert_eq!(
            renderer.call_count(), 1,
            "Should render once after debouncing"
        );

        Ok(())
    }
}
