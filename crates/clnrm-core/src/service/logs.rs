//! Log collection for gVisor containers
//!
//! Provides log capture, streaming, and export capabilities.

use crate::error::{CleanroomError, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Log format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    /// Plain text
    Text,
    /// JSON lines
    Json,
    /// Structured logging
    Structured,
}

/// Log destination
#[derive(Debug, Clone)]
pub enum LogDestination {
    /// Write to file
    File(PathBuf),
    /// Stream to stdout
    Stdout,
    /// Send to OTEL collector
    OtelCollector { endpoint: String },
    /// Discard logs
    Null,
}

/// Log entry
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Timestamp
    pub timestamp: std::time::SystemTime,
    /// Log level (if available)
    pub level: Option<String>,
    /// Log message
    pub message: String,
    /// Source (stdout or stderr)
    pub source: LogSource,
    /// Container ID
    pub container_id: String,
}

/// Log source
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogSource {
    Stdout,
    Stderr,
}

/// Log collector for containers
pub struct LogCollector {
    /// Log format
    format: LogFormat,
    /// Log destination
    destination: LogDestination,
    /// Buffer size
    buffer_size: usize,
    /// Collected logs
    logs: Arc<RwLock<Vec<LogEntry>>>,
}

impl LogCollector {
    /// Create new log collector
    pub fn new(format: LogFormat, destination: LogDestination) -> Self {
        Self {
            format,
            destination,
            buffer_size: 1000,
            logs: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Set buffer size
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Collect log entry
    pub async fn collect(&self, entry: LogEntry) -> Result<()> {
        let mut logs = self.logs.write().await;

        // Add to buffer
        logs.push(entry.clone());

        // Trim buffer if exceeded
        if logs.len() > self.buffer_size {
            logs.drain(0..logs.len() - self.buffer_size);
        }

        // Write to destination
        match &self.destination {
            LogDestination::File(path) => {
                self.write_to_file(path, &entry)?;
            }
            LogDestination::Stdout => {
                self.write_to_stdout(&entry);
            }
            LogDestination::OtelCollector { endpoint } => {
                self.write_to_otel(endpoint, &entry).await?;
            }
            LogDestination::Null => {
                // Discard
            }
        }

        Ok(())
    }

    /// Write log entry to file
    fn write_to_file(&self, path: &PathBuf, entry: &LogEntry) -> Result<()> {
        use std::fs::OpenOptions;
        use std::io::Write;

        let formatted = self.format_entry(entry);

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| {
                CleanroomError::internal_error(format!("Failed to open log file: {}", e))
            })?;

        writeln!(file, "{}", formatted).map_err(|e| {
            CleanroomError::internal_error(format!("Failed to write log entry: {}", e))
        })?;

        Ok(())
    }

    /// Write log entry to stdout
    fn write_to_stdout(&self, entry: &LogEntry) {
        let formatted = self.format_entry(entry);
        println!("{}", formatted);
    }

    /// Write log entry to OTEL collector
    async fn write_to_otel(&self, endpoint: &str, entry: &LogEntry) -> Result<()> {
        let client = reqwest::Client::new();
        
        let log_payload = serde_json::json!({
            "resourceLogs": [{
                "resource": {
                    "attributes": [
                        { "key": "container.id", "value": { "stringValue": entry.container_id } },
                        { "key": "service.name", "value": { "stringValue": "clnrm.container" } }
                    ]
                },
                "scopeLogs": [{
                    "scope": { "name": "clnrm.logs" },
                    "logRecords": [{
                        "timeUnixNano": entry.timestamp.timestamp_nanos_opt().unwrap_or(0) * 1_000_000,
                        "severityText": "INFO",
                        "body": { "stringValue": entry.message }
                    }]
                }]
            }]
        });

        client.post(endpoint)
            .header("Content-Type", "application/json")
            .json(&log_payload)
            .send()
            .await
            .map_err(|e| CleanroomError::execution_error(format!("Failed to export log to OTEL: {}", e)))?;

        Ok(())
    }

    /// Format log entry
    fn format_entry(&self, entry: &LogEntry) -> String {
        match self.format {
            LogFormat::Text => {
                format!(
                    "[{}] [{}] {}",
                    chrono::DateTime::<chrono::Utc>::from(entry.timestamp)
                        .format("%Y-%m-%d %H:%M:%S%.3f"),
                    match entry.source {
                        LogSource::Stdout => "OUT",
                        LogSource::Stderr => "ERR",
                    },
                    entry.message
                )
            }
            LogFormat::Json => {
                serde_json::json!({
                    "timestamp": chrono::DateTime::<chrono::Utc>::from(entry.timestamp).to_rfc3339(),
                    "level": entry.level,
                    "message": entry.message,
                    "source": match entry.source {
                        LogSource::Stdout => "stdout",
                        LogSource::Stderr => "stderr",
                    },
                    "container_id": entry.container_id,
                })
                .to_string()
            }
            LogFormat::Structured => {
                format!(
                    "timestamp={} level={} source={} container_id={} message={}",
                    chrono::DateTime::<chrono::Utc>::from(entry.timestamp).to_rfc3339(),
                    entry.level.as_deref().unwrap_or("INFO"),
                    match entry.source {
                        LogSource::Stdout => "stdout",
                        LogSource::Stderr => "stderr",
                    },
                    entry.container_id,
                    entry.message
                )
            }
        }
    }

    /// Get collected logs
    pub async fn get_logs(&self, limit: Option<usize>) -> Vec<LogEntry> {
        let logs = self.logs.read().await;

        if let Some(limit) = limit {
            let start = logs.len().saturating_sub(limit);
            logs[start..].to_vec()
        } else {
            logs.clone()
        }
    }

    /// Get logs since timestamp
    pub async fn get_logs_since(&self, since: std::time::SystemTime) -> Vec<LogEntry> {
        let logs = self.logs.read().await;
        logs.iter()
            .filter(|e| e.timestamp >= since)
            .cloned()
            .collect()
    }

    /// Clear collected logs
    pub async fn clear(&self) {
        let mut logs = self.logs.write().await;
        logs.clear();
    }

    /// Export logs to file
    pub async fn export(&self, path: PathBuf) -> Result<()> {
        let logs = self.logs.read().await;

        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(&path).map_err(|e| {
            CleanroomError::internal_error(format!("Failed to create export file: {}", e))
        })?;

        for entry in logs.iter() {
            let formatted = self.format_entry(entry);
            writeln!(file, "{}", formatted).map_err(|e| {
                CleanroomError::internal_error(format!("Failed to write log entry: {}", e))
            })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_log_collector_creation() {
        let collector = LogCollector::new(LogFormat::Text, LogDestination::Null);
        assert_eq!(collector.format, LogFormat::Text);
    }

    #[tokio::test]
    async fn test_log_collection() {
        let collector = LogCollector::new(LogFormat::Text, LogDestination::Null);

        let entry = LogEntry {
            timestamp: std::time::SystemTime::now(),
            level: Some("INFO".to_string()),
            message: "Test log message".to_string(),
            source: LogSource::Stdout,
            container_id: "test-container".to_string(),
        };

        collector.collect(entry).await.unwrap();

        let logs = collector.get_logs(None).await;
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].message, "Test log message");
    }

    #[tokio::test]
    async fn test_log_buffer_size() {
        let collector =
            LogCollector::new(LogFormat::Text, LogDestination::Null).with_buffer_size(3);

        for i in 0..5 {
            let entry = LogEntry {
                timestamp: std::time::SystemTime::now(),
                level: Some("INFO".to_string()),
                message: format!("Message {}", i),
                source: LogSource::Stdout,
                container_id: "test-container".to_string(),
            };
            collector.collect(entry).await.unwrap();
        }

        let logs = collector.get_logs(None).await;
        assert_eq!(logs.len(), 3);
        assert_eq!(logs[0].message, "Message 2");
    }
}
