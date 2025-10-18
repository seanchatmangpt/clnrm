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
