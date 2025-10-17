//! Custom serde deserializers for configuration types

use serde::{Deserialize, Deserializer};
use std::time::Duration;

/// Deserialize a duration from a human-readable string like "60s", "5m", "1h"
pub fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    parse_duration(&s).map_err(serde::de::Error::custom)
}

/// Parse a duration string like "60s", "5m", "1h" into a Duration
fn parse_duration(s: &str) -> Result<Duration, String> {
    let s = s.trim();
    
    // Find where the number ends and the unit begins
    let split_pos = s
        .chars()
        .position(|c| !c.is_numeric())
        .ok_or_else(|| format!("Invalid duration format: {}", s))?;
    
    let (num_str, unit) = s.split_at(split_pos);
    let num: u64 = num_str
        .parse()
        .map_err(|_| format!("Invalid number in duration: {}", num_str))?;
    
    let duration = match unit {
        "s" | "sec" | "secs" | "second" | "seconds" => Duration::from_secs(num),
        "m" | "min" | "mins" | "minute" | "minutes" => Duration::from_secs(num * 60),
        "h" | "hr" | "hrs" | "hour" | "hours" => Duration::from_secs(num * 3600),
        "d" | "day" | "days" => Duration::from_secs(num * 86400),
        _ => return Err(format!("Unknown duration unit: {}", unit)),
    };
    
    Ok(duration)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("60s").unwrap(), Duration::from_secs(60));
        assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_duration("1h").unwrap(), Duration::from_secs(3600));
        assert_eq!(parse_duration("2d").unwrap(), Duration::from_secs(172800));
    }
}
