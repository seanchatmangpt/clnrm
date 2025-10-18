//! Time utilities for deterministic testing
//!
//! Provides frozen clock and timestamp utilities.

use chrono::{DateTime, Utc};

/// Parse RFC3339 timestamp string
///
/// # Arguments
/// * `timestamp_str` - RFC3339 formatted timestamp string
///
/// # Returns
/// * Parsed DateTime<Utc> or error
pub fn parse_rfc3339(timestamp_str: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    DateTime::parse_from_rfc3339(timestamp_str).map(|dt| dt.with_timezone(&Utc))
}

/// Format timestamp as RFC3339 string
///
/// # Arguments
/// * `timestamp` - DateTime to format
///
/// # Returns
/// * RFC3339 formatted string
pub fn format_rfc3339(timestamp: &DateTime<Utc>) -> String {
    timestamp.to_rfc3339()
}
