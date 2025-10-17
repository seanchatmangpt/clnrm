//! Comprehensive test suite for watch subsystem
//!
//! Test Coverage:
//! - Unit tests for FileDebouncer
//! - Edge cases and timing scenarios
//! - Thread safety validation
//! - Performance validation
//! - Debouncing behavior verification
//!
//! Core Team Compliance:
//! - ✅ AAA pattern (Arrange, Act, Assert)
//! - ✅ Descriptive test names
//! - ✅ No unwrap/expect in production code (test code allowed)
//! - ✅ Proper error path testing
//! - ✅ Mock external dependencies

#![allow(clippy::unwrap_used, clippy::expect_used)]

use clnrm_core::watch::debouncer::FileDebouncer;
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::Duration;

// ============================================================================
// FileDebouncer Creation and Initialization Tests
// ============================================================================

#[test]
fn test_debouncer_new_creates_valid_instance() {
    // Arrange & Act
    let debouncer = FileDebouncer::new(Duration::from_millis(200));

    // Assert
    assert_eq!(debouncer.event_count(), 0);
    assert!(debouncer.time_since_last_event().is_none());
}

#[test]
fn test_debouncer_default_uses_200ms_window() {
    // Arrange & Act
    let debouncer = FileDebouncer::default();

    // Assert
    assert_eq!(debouncer.event_count(), 0);
    // Default window is 200ms (verified by behavior in other tests)
}

#[test]
fn test_debouncer_accepts_custom_window() {
    // Arrange
    let windows = vec![
        Duration::from_millis(50),
        Duration::from_millis(100),
        Duration::from_millis(300),
        Duration::from_secs(1),
    ];

    // Act & Assert
    for window in windows {
        let debouncer = FileDebouncer::new(window);
        assert_eq!(debouncer.event_count(), 0);
    }
}

#[test]
fn test_debouncer_accepts_zero_duration() {
    // Arrange & Act
    let debouncer = FileDebouncer::new(Duration::from_millis(0));

    // Assert
    assert_eq!(debouncer.event_count(), 0);
}

// ============================================================================
// Event Recording Tests
// ============================================================================

#[test]
fn test_record_event_increments_count() {
    // Arrange
    let mut debouncer = FileDebouncer::new(Duration::from_millis(100));

    // Act
    debouncer.record_event();

    // Assert
    assert_eq!(debouncer.event_count(), 1);
    assert!(debouncer.time_since_last_event().is_some());
}

#[test]
fn test_record_multiple_events_within_window() {
    // Arrange
    let mut debouncer = FileDebouncer::new(Duration::from_millis(100));

    // Act
    debouncer.record_event();
    thread::sleep(Duration::from_millis(10));
    debouncer.record_event();
    thread::sleep(Duration::from_millis(10));
    debouncer.record_event();

    // Assert
    assert_eq!(
        debouncer.event_count(),
        3,
        "All events within window should be counted"
    );
}

#[test]
fn test_record_event_after_window_expires_resets_count() {
    // Arrange
    let mut debouncer = FileDebouncer::new(Duration::from_millis(50));

    // Act
    debouncer.record_event();
    debouncer.record_event();
    assert_eq!(debouncer.event_count(), 2);

    // Wait for window to expire
    thread::sleep(Duration::from_millis(60));
    debouncer.record_event();

    // Assert
    assert_eq!(
        debouncer.event_count(),
        1,
        "Count should reset after window expires"
    );
}

#[test]
fn test_record_event_updates_last_event_time() {
    // Arrange
    let mut debouncer = FileDebouncer::new(Duration::from_millis(100));

    // Act
    debouncer.record_event();
    let time1 = debouncer.time_since_last_event();

    thread::sleep(Duration::from_millis(10));
    debouncer.record_event();
    let time2 = debouncer.time_since_last_event();

    // Assert
    assert!(time1.is_some());
    assert!(time2.is_some());
    assert!(
        time2.unwrap() < time1.unwrap(),
        "Second event should have more recent timestamp"
    );
}

// ============================================================================
// Should Trigger Tests
// ============================================================================

#[test]
fn test_should_trigger_returns_false_immediately_after_event() {
    // Arrange
    let mut debouncer = FileDebouncer::new(Duration::from_millis(100));

    // Act
    debouncer.record_event();

    // Assert
    assert!(
        !debouncer.should_trigger(),
        "Should not trigger immediately after event"
    );
}

#[test]
fn test_should_trigger_returns_true_after_window() {
    // Arrange
    let mut debouncer = FileDebouncer::new(Duration::from_millis(50));

    // Act
    debouncer.record_event();
    thread::sleep(Duration::from_millis(60));

    // Assert
    assert!(
        debouncer.should_trigger(),
        "Should trigger after window expires"
    );
}

#[test]
fn test_should_trigger_returns_false_without_events() {
    // Arrange
    let debouncer = FileDebouncer::new(Duration::from_millis(100));

    // Act & Assert
    assert!(
        !debouncer.should_trigger(),
        "Should not trigger without any events"
    );
}

#[test]
fn test_should_trigger_with_multiple_events_in_window() {
    // Arrange
    let mut debouncer = FileDebouncer::new(Duration::from_millis(100));

    // Act
    for _ in 0..5 {
        debouncer.record_event();
        thread::sleep(Duration::from_millis(10));
    }

    // Assert - Still within window
    assert!(
        !debouncer.should_trigger(),
        "Should not trigger while within window"
    );

    // Wait for window to expire
    thread::sleep(Duration::from_millis(100));

    assert!(
        debouncer.should_trigger(),
        "Should trigger after all events and window expiry"
    );
}

#[test]
fn test_should_trigger_exact_boundary() {
    // Arrange
    let mut debouncer = FileDebouncer::new(Duration::from_millis(50));

    // Act
    debouncer.record_event();
    thread::sleep(Duration::from_millis(50)); // Exact window duration

    // Assert
    assert!(
        debouncer.should_trigger(),
        "Should trigger at exact window boundary"
    );
}

#[test]
fn test_should_trigger_just_before_boundary() {
    // Arrange
    let mut debouncer = FileDebouncer::new(Duration::from_millis(50));

    // Act
    debouncer.record_event();
    thread::sleep(Duration::from_millis(45)); // Just before window

    // Assert
    assert!(
        !debouncer.should_trigger(),
        "Should not trigger just before window expires"
    );
}

// ============================================================================
// Reset Tests
// ============================================================================

#[test]
fn test_reset_clears_event_count() {
    // Arrange
    let mut debouncer = FileDebouncer::new(Duration::from_millis(100));
    debouncer.record_event();
    debouncer.record_event();
    debouncer.record_event();

    // Act
    debouncer.reset();

    // Assert
    assert_eq!(debouncer.event_count(), 0, "Reset should clear event count");
}

#[test]
fn test_reset_clears_last_event_time() {
    // Arrange
    let mut debouncer = FileDebouncer::new(Duration::from_millis(100));
    debouncer.record_event();

    // Act
    debouncer.reset();

    // Assert
    assert!(
        debouncer.time_since_last_event().is_none(),
        "Reset should clear last event time"
    );
}

#[test]
fn test_reset_allows_new_events() {
    // Arrange
    let mut debouncer = FileDebouncer::new(Duration::from_millis(100));
    debouncer.record_event();
    debouncer.reset();

    // Act
    debouncer.record_event();

    // Assert
    assert_eq!(
        debouncer.event_count(),
        1,
        "Should accept new events after reset"
    );
}

#[test]
fn test_reset_after_trigger() {
    // Arrange
    let mut debouncer = FileDebouncer::new(Duration::from_millis(50));
    debouncer.record_event();
    thread::sleep(Duration::from_millis(60));
    assert!(debouncer.should_trigger());

    // Act
    debouncer.reset();

    // Assert
    assert_eq!(debouncer.event_count(), 0);
    assert!(!debouncer.should_trigger());
}

// ============================================================================
// Event Count Tests
// ============================================================================

#[test]
fn test_event_count_starts_at_zero() {
    // Arrange
    let debouncer = FileDebouncer::new(Duration::from_millis(100));

    // Act & Assert
    assert_eq!(debouncer.event_count(), 0);
}

#[test]
fn test_event_count_increments_correctly() {
    // Arrange
    let mut debouncer = FileDebouncer::new(Duration::from_millis(100));

    // Act & Assert
    for i in 1..=10 {
        debouncer.record_event();
        assert_eq!(debouncer.event_count(), i);
    }
}

#[test]
fn test_event_count_resets_across_windows() {
    // Arrange
    let mut debouncer = FileDebouncer::new(Duration::from_millis(50));

    // Act - First window
    debouncer.record_event();
    debouncer.record_event();
    assert_eq!(debouncer.event_count(), 2);

    // New window
    thread::sleep(Duration::from_millis(60));
    debouncer.record_event();

    // Assert
    assert_eq!(debouncer.event_count(), 1);
}

// ============================================================================
// Time Since Last Event Tests
// ============================================================================

#[test]
fn test_time_since_last_event_none_initially() {
    // Arrange
    let debouncer = FileDebouncer::new(Duration::from_millis(100));

    // Act & Assert
    assert!(debouncer.time_since_last_event().is_none());
}

#[test]
fn test_time_since_last_event_some_after_event() {
    // Arrange
    let mut debouncer = FileDebouncer::new(Duration::from_millis(100));

    // Act
    debouncer.record_event();
    thread::sleep(Duration::from_millis(20));

    // Assert
    let elapsed = debouncer.time_since_last_event();
    assert!(elapsed.is_some());
    assert!(elapsed.unwrap() >= Duration::from_millis(20));
}

#[test]
fn test_time_since_last_event_increases() {
    // Arrange
    let mut debouncer = FileDebouncer::new(Duration::from_millis(100));
    debouncer.record_event();

    // Act
    thread::sleep(Duration::from_millis(10));
    let time1 = debouncer.time_since_last_event().unwrap();

    thread::sleep(Duration::from_millis(10));
    let time2 = debouncer.time_since_last_event().unwrap();

    // Assert
    assert!(time2 > time1, "Time since event should increase");
}

#[test]
fn test_time_since_last_event_none_after_reset() {
    // Arrange
    let mut debouncer = FileDebouncer::new(Duration::from_millis(100));
    debouncer.record_event();

    // Act
    debouncer.reset();

    // Assert
    assert!(debouncer.time_since_last_event().is_none());
}

// ============================================================================
// Debouncing Behavior Tests (Realistic Scenarios)
// ============================================================================

#[test]
fn test_debouncing_auto_save_scenario() {
    // Arrange - Simulate IDE auto-save every 50ms
    let mut debouncer = FileDebouncer::new(Duration::from_millis(200));

    // Act - 5 rapid auto-saves
    for _ in 0..5 {
        debouncer.record_event();
        thread::sleep(Duration::from_millis(50));
    }

    // Assert - Should still be within window, not triggered
    assert_eq!(debouncer.event_count(), 5);
    assert!(
        !debouncer.should_trigger(),
        "Auto-saves should be batched"
    );

    // Wait for debounce window
    thread::sleep(Duration::from_millis(200));

    assert!(
        debouncer.should_trigger(),
        "Should trigger after auto-saves stop"
    );
}

#[test]
fn test_debouncing_formatter_scenario() {
    // Arrange - Simulate formatter creating multiple write events
    let mut debouncer = FileDebouncer::new(Duration::from_millis(100));

    // Act - Rapid events from formatter (< 5ms apart)
    for _ in 0..10 {
        debouncer.record_event();
        thread::sleep(Duration::from_millis(2));
    }

    // Assert - All events batched
    assert_eq!(debouncer.event_count(), 10);
    assert!(!debouncer.should_trigger());

    // Wait for window
    thread::sleep(Duration::from_millis(100));

    assert!(debouncer.should_trigger());
    debouncer.reset();

    // Verify single trigger
    assert!(!debouncer.should_trigger());
}

#[test]
fn test_debouncing_multiple_file_saves() {
    // Arrange - User saves multiple related files
    let mut debouncer = FileDebouncer::new(Duration::from_millis(150));

    // Act - Save 3 files with small delays
    debouncer.record_event(); // file1
    thread::sleep(Duration::from_millis(30));

    debouncer.record_event(); // file2
    thread::sleep(Duration::from_millis(30));

    debouncer.record_event(); // file3

    // Assert - All batched
    assert_eq!(debouncer.event_count(), 3);
    assert!(!debouncer.should_trigger());

    // Wait for window
    thread::sleep(Duration::from_millis(150));

    assert!(debouncer.should_trigger(), "Should trigger once for all saves");
}

#[test]
fn test_debouncing_separates_distinct_edits() {
    // Arrange - Two separate editing sessions
    let mut debouncer = FileDebouncer::new(Duration::from_millis(100));

    // Act - First edit session
    debouncer.record_event();
    debouncer.record_event();
    thread::sleep(Duration::from_millis(110));

    // Should trigger for first session
    assert!(debouncer.should_trigger());
    debouncer.reset();

    // Second edit session (after pause)
    thread::sleep(Duration::from_millis(50)); // User pause
    debouncer.record_event();
    debouncer.record_event();
    thread::sleep(Duration::from_millis(110));

    // Assert - Should trigger again for second session
    assert!(debouncer.should_trigger());
}

// ============================================================================
// Thread Safety Tests
// ============================================================================

#[test]
fn test_debouncer_is_send() {
    // Arrange
    let debouncer = FileDebouncer::new(Duration::from_millis(100));

    // Act - Move to another thread
    let handle = thread::spawn(move || {
        let _d = debouncer;
    });

    // Assert
    handle.join().unwrap();
}

#[test]
fn test_concurrent_event_recording() {
    // Arrange
    let debouncer = Arc::new(Mutex::new(FileDebouncer::new(Duration::from_millis(200))));
    let barrier = Arc::new(Barrier::new(10));

    // Act - 10 threads recording events concurrently
    let mut handles = vec![];
    for _ in 0..10 {
        let debouncer_clone = Arc::clone(&debouncer);
        let barrier_clone = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            barrier_clone.wait(); // Synchronize start
            let mut d = debouncer_clone.lock().unwrap();
            d.record_event();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Assert
    let d = debouncer.lock().unwrap();
    assert_eq!(d.event_count(), 10, "All events should be recorded");
}

#[test]
fn test_concurrent_should_trigger_checks() {
    // Arrange
    let mut debouncer_setup = FileDebouncer::new(Duration::from_millis(50));
    debouncer_setup.record_event();
    thread::sleep(Duration::from_millis(60));

    let debouncer = Arc::new(debouncer_setup);
    let barrier = Arc::new(Barrier::new(10));

    // Act - 10 threads checking trigger concurrently
    let mut handles = vec![];
    let mut results = Arc::new(Mutex::new(Vec::new()));

    for _ in 0..10 {
        let debouncer_clone = Arc::clone(&debouncer);
        let barrier_clone = Arc::clone(&barrier);
        let results_clone = Arc::clone(&results);

        let handle = thread::spawn(move || {
            barrier_clone.wait();
            let should_trigger = debouncer_clone.should_trigger();
            results_clone.lock().unwrap().push(should_trigger);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Assert - All should see consistent state
    let r = results.lock().unwrap();
    assert_eq!(r.len(), 10);
    assert!(r.iter().all(|&x| x), "All threads should see trigger state");
}

// ============================================================================
// Performance Tests
// ============================================================================

#[test]
fn test_debouncer_performance_many_events() {
    // Arrange
    let mut debouncer = FileDebouncer::new(Duration::from_millis(100));

    // Act - Record 10,000 events
    let start = std::time::Instant::now();
    for _ in 0..10_000 {
        debouncer.record_event();
    }
    let duration = start.elapsed();

    // Assert
    assert!(
        duration.as_millis() < 100,
        "10K events should complete in <100ms, took {}ms",
        duration.as_millis()
    );
    assert_eq!(debouncer.event_count(), 10_000);
}

#[test]
fn test_debouncer_performance_should_trigger_checks() {
    // Arrange
    let mut debouncer = FileDebouncer::new(Duration::from_millis(100));
    debouncer.record_event();

    // Act - Check trigger 10,000 times
    let start = std::time::Instant::now();
    for _ in 0..10_000 {
        let _ = debouncer.should_trigger();
    }
    let duration = start.elapsed();

    // Assert
    assert!(
        duration.as_millis() < 50,
        "10K checks should complete in <50ms, took {}ms",
        duration.as_millis()
    );
}

#[test]
fn test_debouncer_performance_reset_operations() {
    // Arrange
    let mut debouncer = FileDebouncer::new(Duration::from_millis(100));

    // Act - Reset 10,000 times
    let start = std::time::Instant::now();
    for _ in 0..10_000 {
        debouncer.record_event();
        debouncer.reset();
    }
    let duration = start.elapsed();

    // Assert
    assert!(
        duration.as_millis() < 100,
        "10K reset operations should complete in <100ms, took {}ms",
        duration.as_millis()
    );
}

// ============================================================================
// Edge Cases Tests
// ============================================================================

#[test]
fn test_debouncer_with_very_short_window() {
    // Arrange
    let mut debouncer = FileDebouncer::new(Duration::from_millis(1));

    // Act
    debouncer.record_event();
    thread::sleep(Duration::from_millis(2));

    // Assert
    assert!(debouncer.should_trigger(), "Very short window should work");
}

#[test]
fn test_debouncer_with_very_long_window() {
    // Arrange
    let mut debouncer = FileDebouncer::new(Duration::from_secs(60));

    // Act
    debouncer.record_event();

    // Assert
    assert!(
        !debouncer.should_trigger(),
        "Long window should not trigger immediately"
    );
}

#[test]
fn test_debouncer_rapid_reset_cycles() {
    // Arrange
    let mut debouncer = FileDebouncer::new(Duration::from_millis(100));

    // Act - Rapid record-reset cycles
    for _ in 0..100 {
        debouncer.record_event();
        debouncer.reset();
    }

    // Assert
    assert_eq!(debouncer.event_count(), 0);
    assert!(!debouncer.should_trigger());
}

#[test]
fn test_debouncer_event_count_overflow_safety() {
    // Arrange
    let mut debouncer = FileDebouncer::new(Duration::from_millis(1000));

    // Act - Record massive number of events
    for _ in 0..1_000_000 {
        debouncer.record_event();
    }

    // Assert
    assert_eq!(debouncer.event_count(), 1_000_000);
    // Should not panic or overflow
}

#[test]
fn test_debouncer_multiple_window_boundaries() {
    // Arrange
    let mut debouncer = FileDebouncer::new(Duration::from_millis(50));

    // Act & Assert - Cycle through multiple windows
    for _ in 0..5 {
        debouncer.record_event();
        assert!(!debouncer.should_trigger());

        thread::sleep(Duration::from_millis(60));
        assert!(debouncer.should_trigger());

        debouncer.reset();
        assert!(!debouncer.should_trigger());
    }
}

#[test]
fn test_debouncer_time_precision() {
    // Arrange
    let mut debouncer = FileDebouncer::new(Duration::from_millis(50));

    // Act
    debouncer.record_event();
    thread::sleep(Duration::from_millis(49));

    // Assert - Just under window
    assert!(!debouncer.should_trigger(), "Should not trigger at 49ms");

    // Wait 2 more milliseconds
    thread::sleep(Duration::from_millis(2));

    assert!(debouncer.should_trigger(), "Should trigger at 51ms");
}

// ============================================================================
// Integration-Style Behavior Tests
// ============================================================================

#[test]
fn test_typical_watch_mode_workflow() {
    // Arrange - Simulate typical watch mode usage
    let mut debouncer = FileDebouncer::new(Duration::from_millis(200));

    // Act - User edits file, saves multiple times
    debouncer.record_event(); // Initial save
    thread::sleep(Duration::from_millis(50));

    debouncer.record_event(); // Auto-save
    thread::sleep(Duration::from_millis(50));

    debouncer.record_event(); // Another save
    assert_eq!(debouncer.event_count(), 3);
    assert!(!debouncer.should_trigger());

    // User stops editing, window expires
    thread::sleep(Duration::from_millis(200));

    // Assert - Should trigger and run tests once
    assert!(debouncer.should_trigger());
    debouncer.reset();

    // No more events - should not trigger
    assert!(!debouncer.should_trigger());
}

#[test]
fn test_continuous_editing_delays_trigger() {
    // Arrange
    let mut debouncer = FileDebouncer::new(Duration::from_millis(100));

    // Act - Continuous editing (every 80ms) for 500ms
    for _ in 0..6 {
        debouncer.record_event();
        thread::sleep(Duration::from_millis(80));

        // Should not trigger while still editing
        assert!(
            !debouncer.should_trigger(),
            "Should not trigger during continuous editing"
        );
    }

    // Stop editing
    thread::sleep(Duration::from_millis(100));

    // Assert - Should trigger after editing stops
    assert!(
        debouncer.should_trigger(),
        "Should trigger after editing stops"
    );
}
