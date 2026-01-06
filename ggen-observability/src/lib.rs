use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ObservabilityError {
    #[error("Metric not found: {0}")]
    MetricNotFound(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
}

pub type Result<T> = std::result::Result<T, ObservabilityError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}

impl LogLevel {
    pub fn as_str(&self) -> &str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Critical => "CRITICAL",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: u64,
    pub level: LogLevel,
    pub service: String,
    pub message: String,
    pub trace_id: String,
    pub span_id: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub labels: HashMap<String, String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    pub trace_id: String,
    pub spans: Vec<Span>,
    pub start_time: u64,
    pub end_time: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    pub span_id: String,
    pub trace_id: String,
    pub parent_span_id: Option<String>,
    pub operation: String,
    pub service: String,
    pub start_time: u64,
    pub end_time: u64,
    pub duration_ms: u64,
    pub status: SpanStatus,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpanStatus {
    Unset,
    Ok,
    Error,
}

pub struct MetricsCollector {
    metrics: Arc<Vec<Metric>>,
    counters: HashMap<String, Arc<AtomicU64>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Vec::new()),
            counters: HashMap::new(),
        }
    }

    pub fn create_counter(&mut self, name: &str) -> Arc<AtomicU64> {
        let counter = Arc::new(AtomicU64::new(0));
        self.counters.insert(name.to_string(), counter.clone());
        counter
    }

    pub fn increment_counter(&self, name: &str) -> Result<()> {
        self.counters
            .get(name)
            .ok_or(ObservabilityError::MetricNotFound(name.to_string()))?
            .fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    pub fn get_counter(&self, name: &str) -> Result<u64> {
        Ok(self
            .counters
            .get(name)
            .ok_or(ObservabilityError::MetricNotFound(name.to_string()))?
            .load(Ordering::Relaxed))
    }

    pub fn record_metric(&self, metric: Metric) {
        // In a real implementation, this would store metrics in time-series DB
        // For this demo, metrics are logged
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LogCollector {
    logs: Vec<LogEntry>,
    max_logs: usize,
}

impl LogCollector {
    pub fn new(max_logs: usize) -> Self {
        Self {
            logs: Vec::new(),
            max_logs,
        }
    }

    pub fn log(&mut self, entry: LogEntry) {
        if self.logs.len() >= self.max_logs {
            self.logs.remove(0);
        }
        self.logs.push(entry);
    }

    pub fn get_logs(&self) -> &[LogEntry] {
        &self.logs
    }

    pub fn filter_by_level(&self, level: LogLevel) -> Vec<&LogEntry> {
        self.logs.iter().filter(|l| l.level == level).collect()
    }

    pub fn filter_by_service(&self, service: &str) -> Vec<&LogEntry> {
        self.logs.iter().filter(|l| l.service == service).collect()
    }

    pub fn filter_by_trace(&self, trace_id: &str) -> Vec<&LogEntry> {
        self.logs.iter().filter(|l| l.trace_id == trace_id).collect()
    }
}

pub struct TraceCollector {
    traces: Vec<Trace>,
    max_traces: usize,
}

impl TraceCollector {
    pub fn new(max_traces: usize) -> Self {
        Self {
            traces: Vec::new(),
            max_traces,
        }
    }

    pub fn record_trace(&mut self, trace: Trace) {
        if self.traces.len() >= self.max_traces {
            self.traces.remove(0);
        }
        self.traces.push(trace);
    }

    pub fn get_traces(&self) -> &[Trace] {
        &self.traces
    }

    pub fn get_trace(&self, trace_id: &str) -> Option<&Trace> {
        self.traces.iter().find(|t| t.trace_id == trace_id)
    }

    pub fn traces_by_service(&self, service: &str) -> Vec<&Trace> {
        self.traces
            .iter()
            .filter(|t| t.spans.iter().any(|s| s.service == service))
            .collect()
    }

    pub fn slow_traces(&self, threshold_ms: u64) -> Vec<&Trace> {
        self.traces.iter().filter(|t| t.duration_ms > threshold_ms).collect()
    }

    pub fn error_traces(&self) -> Vec<&Trace> {
        self.traces
            .iter()
            .filter(|t| t.spans.iter().any(|s| s.status == SpanStatus::Error))
            .collect()
    }
}

pub struct ObservabilityPipeline {
    pub metrics: MetricsCollector,
    pub logs: LogCollector,
    pub traces: TraceCollector,
}

impl ObservabilityPipeline {
    pub fn new() -> Self {
        Self {
            metrics: MetricsCollector::new(),
            logs: LogCollector::new(10000),
            traces: TraceCollector::new(1000),
        }
    }

    pub fn health_check(&self) -> HealthMetrics {
        HealthMetrics {
            total_logs: self.logs.get_logs().len(),
            total_traces: self.traces.get_traces().len(),
            error_logs: self.logs.filter_by_level(LogLevel::Error).len(),
            error_traces: self.traces.error_traces().len(),
            slow_traces: self.traces.slow_traces(1000).len(),
        }
    }
}

impl Default for ObservabilityPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub total_logs: usize,
    pub total_traces: usize,
    pub error_logs: usize,
    pub error_traces: usize,
    pub slow_traces: usize,
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector() {
        let mut collector = MetricsCollector::new();
        let counter = collector.create_counter("requests");

        assert!(collector.increment_counter("requests").is_ok());
        assert_eq!(collector.get_counter("requests").unwrap(), 1);
    }

    #[test]
    fn test_log_collector() {
        let mut collector = LogCollector::new(100);

        let entry = LogEntry {
            timestamp: now_millis(),
            level: LogLevel::Info,
            service: "api".to_string(),
            message: "Request processed".to_string(),
            trace_id: "trace-1".to_string(),
            span_id: "span-1".to_string(),
            metadata: HashMap::new(),
        };

        collector.log(entry);
        assert_eq!(collector.get_logs().len(), 1);
    }

    #[test]
    fn test_log_filtering() {
        let mut collector = LogCollector::new(100);

        collector.log(LogEntry {
            timestamp: now_millis(),
            level: LogLevel::Info,
            service: "api".to_string(),
            message: "Info message".to_string(),
            trace_id: "trace-1".to_string(),
            span_id: "span-1".to_string(),
            metadata: HashMap::new(),
        });

        collector.log(LogEntry {
            timestamp: now_millis(),
            level: LogLevel::Error,
            service: "api".to_string(),
            message: "Error message".to_string(),
            trace_id: "trace-2".to_string(),
            span_id: "span-2".to_string(),
            metadata: HashMap::new(),
        });

        assert_eq!(collector.filter_by_level(LogLevel::Error).len(), 1);
        assert_eq!(collector.filter_by_service("api").len(), 2);
    }

    #[test]
    fn test_trace_collector() {
        let mut collector = TraceCollector::new(100);

        let trace = Trace {
            trace_id: "trace-1".to_string(),
            spans: vec![],
            start_time: now_millis(),
            end_time: now_millis() + 500,
            duration_ms: 500,
        };

        collector.record_trace(trace);
        assert_eq!(collector.get_traces().len(), 1);
    }

    #[test]
    fn test_observability_pipeline() {
        let pipeline = ObservabilityPipeline::new();
        let health = pipeline.health_check();

        assert_eq!(health.total_logs, 0);
        assert_eq!(health.error_logs, 0);
    }

    #[test]
    fn test_span_tracking() {
        let span = Span {
            span_id: "span-1".to_string(),
            trace_id: "trace-1".to_string(),
            parent_span_id: None,
            operation: "api.request".to_string(),
            service: "api".to_string(),
            start_time: now_millis(),
            end_time: now_millis() + 100,
            duration_ms: 100,
            status: SpanStatus::Ok,
            attributes: HashMap::new(),
        };

        assert_eq!(span.operation, "api.request");
        assert_eq!(span.status, SpanStatus::Ok);
    }
}
