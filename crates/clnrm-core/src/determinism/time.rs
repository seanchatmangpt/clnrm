//! Time utilities for deterministic testing
//!
//! Provides a frozen clock that advances only when explicitly told to,
//! enabling fully reproducible timestamp-dependent test scenarios.

use chrono::{DateTime, Duration, Utc};

/// Parse RFC3339 timestamp string (legacy helper retained for compatibility)
///
/// # Arguments
/// * `timestamp_str` - RFC3339 formatted timestamp string
///
/// # Returns
/// * Parsed `DateTime<Utc>` or `chrono::ParseError`
pub fn parse_rfc3339(timestamp_str: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    DateTime::parse_from_rfc3339(timestamp_str).map(|dt| dt.with_timezone(&Utc))
}

/// Format timestamp as RFC3339 string (legacy helper retained for compatibility)
///
/// # Arguments
/// * `timestamp` - DateTime to format
///
/// # Returns
/// * RFC3339 formatted string
pub fn format_rfc3339(timestamp: &DateTime<Utc>) -> String {
    timestamp.to_rfc3339()
}

/// A deterministic clock frozen at a fixed point in time.
///
/// `FrozenClock` allows test code to control what "now" means without
/// touching real system time.  Call [`FrozenClock::advance`] to move the
/// clock forward (or backward) by a number of milliseconds.
///
/// # Example
/// ```no_run
/// use clnrm_core::determinism::time::FrozenClock;
///
/// let mut clock = FrozenClock::parse("2025-01-01T00:00:00Z").unwrap(); // OK: doc example
/// assert_eq!(clock.unix_ms() % 1000, 0);
/// clock.advance(500);
/// ```
#[derive(Debug, Clone)]
pub struct FrozenClock {
    /// The base timestamp the clock was created with
    pub frozen_at: DateTime<Utc>,
    /// Accumulated offset from calls to `advance`, in milliseconds
    pub offset_ms: i64,
}

impl FrozenClock {
    /// Create a new `FrozenClock` frozen at `timestamp`.
    ///
    /// The `offset_ms` starts at zero; call [`FrozenClock::advance`] to move it.
    pub fn new(timestamp: DateTime<Utc>) -> Self {
        Self {
            frozen_at: timestamp,
            offset_ms: 0,
        }
    }

    /// Adjust the clock by `ms` milliseconds (may be negative to go back in time).
    pub fn advance(&mut self, ms: i64) {
        self.offset_ms += ms;
    }

    /// Return the current logical time (`frozen_at + offset_ms`).
    pub fn now(&self) -> DateTime<Utc> {
        self.frozen_at + Duration::milliseconds(self.offset_ms)
    }

    /// Return the current logical time as milliseconds since the UNIX epoch.
    pub fn unix_ms(&self) -> i64 {
        self.now().timestamp_millis()
    }

    /// Parse an RFC 3339 string and create a `FrozenClock` at that instant.
    ///
    /// # Errors
    /// Returns `chrono::ParseError` if the string is not a valid RFC 3339 timestamp.
    pub fn parse(s: &str) -> Result<Self, chrono::ParseError> {
        let dt = DateTime::parse_from_rfc3339(s).map(|d| d.with_timezone(&Utc))?;
        Ok(Self::new(dt))
    }

    /// Render the current logical time as an RFC 3339 string.
    pub fn to_rfc3339(&self) -> String {
        self.now().to_rfc3339()
    }
}
