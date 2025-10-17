//! File event debouncing for watch mode
//!
//! Prevents excessive test runs when multiple file save events occur in rapid succession.
//! Uses a time-based window to batch events and trigger only once per window.
//!
//! Core Team Compliance:
//! - ✅ Sync functions for computation
//! - ✅ Proper error handling (no panics)
//! - ✅ No unwrap() or expect() calls
//! - ✅ Performance-optimized for <200ms debounce window

use std::time::{Duration, Instant};
use tracing::debug;

/// File event debouncer
///
/// Batches file change events within a time window to prevent
/// excessive test runs on rapid file saves (e.g., auto-save, formatters).
///
/// # Example
///
/// ```
/// use clnrm_core::watch::debouncer::FileDebouncer;
/// use std::time::Duration;
///
/// let mut debouncer = FileDebouncer::new(Duration::from_millis(200));
///
/// // Record multiple events
/// debouncer.record_event();
/// debouncer.record_event();
/// debouncer.record_event();
///
/// // Check if we should trigger (after delay)
/// if debouncer.should_trigger() {
///     debouncer.reset();
///     // Run tests once for all events
/// }
/// ```
#[derive(Debug)]
pub struct FileDebouncer {
    /// Debounce window duration
    window: Duration,
    /// Time of last event
    last_event: Option<Instant>,
    /// Number of events in current window
    event_count: usize,
}

impl FileDebouncer {
    /// Create new debouncer with specified window duration
    ///
    /// # Arguments
    ///
    /// * `window` - Time window for batching events (typically 200-300ms)
    ///
    /// # Example
    ///
    /// ```
    /// use clnrm_core::watch::debouncer::FileDebouncer;
    /// use std::time::Duration;
    ///
    /// let debouncer = FileDebouncer::new(Duration::from_millis(200));
    /// ```
    pub fn new(window: Duration) -> Self {
        Self {
            window,
            last_event: None,
            event_count: 0,
        }
    }

    /// Record a file change event
    ///
    /// Updates the debouncer state to track the most recent event.
    /// Multiple calls within the debounce window will be batched.
    pub fn record_event(&mut self) {
        let now = Instant::now();

        if let Some(last) = self.last_event {
            let elapsed = now.duration_since(last);
            if elapsed < self.window {
                // Within debounce window - increment counter
                self.event_count += 1;
                debug!(
                    "Debouncer: Event {} within window ({:.0}ms since last)",
                    self.event_count,
                    elapsed.as_millis()
                );
            } else {
                // Outside window - reset counter
                debug!(
                    "Debouncer: New window started ({:.0}ms since last)",
                    elapsed.as_millis()
                );
                self.event_count = 1;
            }
        } else {
            // First event
            self.event_count = 1;
            debug!("Debouncer: First event recorded");
        }

        self.last_event = Some(now);
    }

    /// Check if debounced events should trigger action
    ///
    /// Returns true if:
    /// - There are pending events
    /// - The debounce window has elapsed since the last event
    ///
    /// # Returns
    ///
    /// `true` if action should be triggered, `false` otherwise
    pub fn should_trigger(&self) -> bool {
        if let Some(last) = self.last_event {
            if self.event_count > 0 {
                let elapsed = Instant::now().duration_since(last);
                let should_trigger = elapsed >= self.window;

                if should_trigger {
                    debug!(
                        "Debouncer: Triggering with {} event(s) after {:.0}ms",
                        self.event_count,
                        elapsed.as_millis()
                    );
                }

                return should_trigger;
            }
        }
        false
    }

    /// Reset debouncer state after triggering action
    ///
    /// Clears event count and last event timestamp.
    /// Call this after processing debounced events.
    pub fn reset(&mut self) {
        debug!("Debouncer: Reset (processed {} event(s))", self.event_count);
        self.last_event = None;
        self.event_count = 0;
    }

    /// Get current event count in window
    ///
    /// Returns the number of events batched in the current debounce window.
    pub fn event_count(&self) -> usize {
        self.event_count
    }

    /// Get time since last event
    ///
    /// Returns None if no events have been recorded.
    pub fn time_since_last_event(&self) -> Option<Duration> {
        self.last_event
            .map(|last| Instant::now().duration_since(last))
    }
}

impl Default for FileDebouncer {
    /// Create debouncer with default 200ms window
    fn default() -> Self {
        Self::new(Duration::from_millis(200))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_debouncer_creation() {
        // Arrange & Act
        let debouncer = FileDebouncer::new(Duration::from_millis(200));

        // Assert
        assert_eq!(debouncer.event_count, 0);
        assert!(debouncer.last_event.is_none());
    }

    #[test]
    fn test_debouncer_default() {
        // Arrange & Act
        let debouncer = FileDebouncer::default();

        // Assert
        assert_eq!(debouncer.event_count, 0);
        assert_eq!(debouncer.window, Duration::from_millis(200));
    }

    #[test]
    fn test_record_single_event() {
        // Arrange
        let mut debouncer = FileDebouncer::new(Duration::from_millis(100));

        // Act
        debouncer.record_event();

        // Assert
        assert_eq!(debouncer.event_count(), 1);
        assert!(debouncer.last_event.is_some());
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
        assert_eq!(debouncer.event_count(), 3);
    }

    #[test]
    fn test_should_not_trigger_immediately() {
        // Arrange
        let mut debouncer = FileDebouncer::new(Duration::from_millis(100));

        // Act
        debouncer.record_event();

        // Assert - should not trigger immediately
        assert!(!debouncer.should_trigger());
    }

    #[test]
    fn test_should_trigger_after_window() {
        // Arrange
        let mut debouncer = FileDebouncer::new(Duration::from_millis(50));

        // Act
        debouncer.record_event();
        thread::sleep(Duration::from_millis(60));

        // Assert
        assert!(debouncer.should_trigger());
    }

    #[test]
    fn test_reset_clears_state() {
        // Arrange
        let mut debouncer = FileDebouncer::new(Duration::from_millis(100));
        debouncer.record_event();
        debouncer.record_event();

        // Act
        debouncer.reset();

        // Assert
        assert_eq!(debouncer.event_count(), 0);
        assert!(debouncer.last_event.is_none());
    }

    #[test]
    fn test_time_since_last_event_none_when_no_events() {
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
        thread::sleep(Duration::from_millis(10));

        // Assert
        let elapsed = debouncer.time_since_last_event();
        assert!(elapsed.is_some());
        assert!(elapsed.unwrap() >= Duration::from_millis(10));
    }

    #[test]
    fn test_multiple_windows() {
        // Arrange
        let mut debouncer = FileDebouncer::new(Duration::from_millis(50));

        // Act - First window
        debouncer.record_event();
        debouncer.record_event();
        assert_eq!(debouncer.event_count(), 2);

        // Wait for window to expire and start new window
        thread::sleep(Duration::from_millis(60));
        debouncer.record_event();

        // Assert - New window should reset count
        assert_eq!(debouncer.event_count(), 1);
    }

    #[test]
    fn test_should_not_trigger_without_events() {
        // Arrange
        let debouncer = FileDebouncer::new(Duration::from_millis(100));

        // Act & Assert
        assert!(!debouncer.should_trigger());
    }

    #[test]
    fn test_rapid_events_batching() {
        // Arrange
        let mut debouncer = FileDebouncer::new(Duration::from_millis(100));

        // Act - Simulate rapid saves (e.g., auto-save, formatter)
        for _ in 0..10 {
            debouncer.record_event();
            thread::sleep(Duration::from_millis(5));
        }

        // Assert - All events batched
        assert_eq!(debouncer.event_count(), 10);
        assert!(!debouncer.should_trigger()); // Still within window

        // Wait for window to expire
        thread::sleep(Duration::from_millis(100));
        assert!(debouncer.should_trigger()); // Now should trigger
    }
}
