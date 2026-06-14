use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct BotMetric {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub timestamp_ms: u64,
}

pub struct BotMetrics {
    pub metrics: Vec<BotMetric>,
    pub bot_name: String,
    pub start_time_ms: u64,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

impl BotMetrics {
    pub fn new(bot_name: &str) -> Self {
        Self {
            metrics: Vec::new(),
            bot_name: bot_name.to_string(),
            start_time_ms: now_ms(),
        }
    }

    /// Record a metric value with the current wall-clock timestamp.
    pub fn record(&mut self, name: &str, value: f64, unit: &str) {
        self.metrics.push(BotMetric {
            name: name.to_string(),
            value,
            unit: unit.to_string(),
            timestamp_ms: now_ms(),
        });
    }

    /// Returns the most recent metric with the given name.
    pub fn latest(&self, name: &str) -> Option<&BotMetric> {
        self.metrics.iter().rev().find(|m| m.name == name)
    }

    /// Returns the average value for all metrics with the given name.
    pub fn average(&self, name: &str) -> Option<f64> {
        let values: Vec<f64> = self
            .metrics
            .iter()
            .filter(|m| m.name == name)
            .map(|m| m.value)
            .collect();
        if values.is_empty() {
            return None;
        }
        Some(values.iter().sum::<f64>() / values.len() as f64)
    }

    /// Returns the minimum value for all metrics with the given name.
    pub fn min(&self, name: &str) -> Option<f64> {
        self.metrics
            .iter()
            .filter(|m| m.name == name)
            .map(|m| m.value)
            .reduce(f64::min)
    }

    /// Returns the maximum value for all metrics with the given name.
    pub fn max(&self, name: &str) -> Option<f64> {
        self.metrics
            .iter()
            .filter(|m| m.name == name)
            .map(|m| m.value)
            .reduce(f64::max)
    }

    /// Returns milliseconds elapsed since this BotMetrics instance was created.
    pub fn since_start_ms(&self) -> u64 {
        now_ms().saturating_sub(self.start_time_ms)
    }

    /// Converts each recorded metric into an OTEL-style attribute map.
    /// Each map contains: "bot", "name", "value", "unit", "timestamp_ms".
    pub fn to_otel_events(&self) -> Vec<HashMap<String, String>> {
        self.metrics
            .iter()
            .map(|m| {
                let mut map = HashMap::new();
                map.insert("bot".to_string(), self.bot_name.clone());
                map.insert("name".to_string(), m.name.clone());
                map.insert("value".to_string(), m.value.to_string());
                map.insert("unit".to_string(), m.unit.clone());
                map.insert("timestamp_ms".to_string(), m.timestamp_ms.to_string());
                map
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_latest() {
        let mut bm = BotMetrics::new("treasury-bot");
        bm.record("error", 5.0, "units");
        bm.record("error", 3.0, "units");
        let latest = bm.latest("error").unwrap();
        assert_eq!(latest.value, 3.0);
    }

    #[test]
    fn test_latest_missing_returns_none() {
        let bm = BotMetrics::new("bot");
        assert!(bm.latest("nonexistent").is_none());
    }

    #[test]
    fn test_average() {
        let mut bm = BotMetrics::new("bot");
        bm.record("latency", 10.0, "ms");
        bm.record("latency", 20.0, "ms");
        bm.record("latency", 30.0, "ms");
        assert_eq!(bm.average("latency").unwrap(), 20.0);
    }

    #[test]
    fn test_min_max() {
        let mut bm = BotMetrics::new("bot");
        bm.record("cpu", 30.0, "%");
        bm.record("cpu", 80.0, "%");
        bm.record("cpu", 50.0, "%");
        assert_eq!(bm.min("cpu").unwrap(), 30.0);
        assert_eq!(bm.max("cpu").unwrap(), 80.0);
    }

    #[test]
    fn test_to_otel_events() {
        let mut bm = BotMetrics::new("arb-bot");
        bm.record("profit", 42.5, "tokens");
        let events = bm.to_otel_events();
        assert_eq!(events.len(), 1);
        let ev = &events[0];
        assert_eq!(ev["bot"], "arb-bot");
        assert_eq!(ev["name"], "profit");
        assert_eq!(ev["unit"], "tokens");
        assert!(ev.contains_key("timestamp_ms"));
        assert!(ev.contains_key("value"));
    }

    #[test]
    fn test_since_start_ms_is_non_negative() {
        let bm = BotMetrics::new("bot");
        // A small sleep is not needed; since_start_ms should be >= 0
        assert!(bm.since_start_ms() < 5000, "should be nearly instant");
    }
}
