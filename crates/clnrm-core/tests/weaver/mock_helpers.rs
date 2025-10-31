//! Mock Helpers for Weaver Tests
//!
//! Provides mock implementations of Weaver processes and utilities for testing
//! WeaverController without requiring actual Weaver installation.

use std::io::Write;
use std::net::TcpListener;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Mock Weaver process that simulates the behavior of `weaver registry live-check`
pub struct MockWeaverProcess {
    /// Port the mock is listening on
    pub otlp_port: u16,
    /// Admin port
    pub admin_port: u16,
    /// Whether the mock should simulate a crash
    pub should_crash: bool,
    /// Number of samples to simulate receiving
    pub sample_count: u32,
    /// Number of violations to report
    pub violation_count: u32,
    /// Handle to the mock process (if running)
    handle: Option<thread::JoinHandle<()>>,
    /// Shutdown signal
    shutdown: Arc<Mutex<bool>>,
}

impl MockWeaverProcess {
    /// Create a new mock Weaver process
    pub fn new(otlp_port: u16, admin_port: u16) -> Self {
        Self {
            otlp_port,
            admin_port,
            should_crash: false,
            sample_count: 100,
            violation_count: 0,
            handle: None,
            shutdown: Arc::new(Mutex::new(false)),
        }
    }

    /// Configure the mock to crash after startup
    pub fn with_crash(mut self) -> Self {
        self.should_crash = true;
        self
    }

    /// Configure the mock to report zero samples
    pub fn with_zero_samples(mut self) -> Self {
        self.sample_count = 0;
        self
    }

    /// Configure the mock to report violations
    pub fn with_violations(mut self, count: u32) -> Self {
        self.violation_count = count;
        self
    }

    /// Start the mock process
    pub fn start(&mut self) -> std::io::Result<()> {
        let shutdown = Arc::clone(&self.shutdown);
        let should_crash = self.should_crash;
        let sample_count = self.sample_count;
        let violation_count = self.violation_count;

        let handle = thread::spawn(move || {
            if should_crash {
                // Simulate immediate crash
                thread::sleep(Duration::from_millis(100));
                return;
            }

            // Simulate normal operation
            loop {
                if *shutdown.lock().unwrap() {
                    break;
                }
                thread::sleep(Duration::from_millis(100));
            }

            // On shutdown, write validation report
            let report = format!(
                r#"{{
    "status": "{}",
    "violations": {},
    "improvements": 0,
    "information": 0,
    "registry_coverage": 0.85,
    "sample_count": {},
    "details": []
}}"#,
                if violation_count > 0 { "failure" } else { "success" },
                violation_count,
                sample_count
            );

            // Write report to output directory
            let _ = std::fs::create_dir_all("./validation_output");
            let _ = std::fs::write("./validation_output/validation_report.json", report);
        });

        self.handle = Some(handle);
        Ok(())
    }

    /// Stop the mock process
    pub fn stop(&mut self) {
        *self.shutdown.lock().unwrap() = true;
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    /// Check if the mock is still running
    pub fn is_running(&self) -> bool {
        if let Some(ref handle) = self.handle {
            !handle.is_finished()
        } else {
            false
        }
    }
}

/// Find an available port in the given range
pub fn find_available_port(start: u16, end: u16) -> Option<u16> {
    for port in start..=end {
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return Some(port);
        }
    }
    None
}

/// Occupy a port to simulate port conflicts
pub struct PortBlocker {
    listener: Option<TcpListener>,
}

impl PortBlocker {
    /// Create a new port blocker that occupies the given port
    pub fn new(port: u16) -> std::io::Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", port))?;
        Ok(Self {
            listener: Some(listener),
        })
    }

    /// Release the port
    pub fn release(&mut self) {
        self.listener = None;
    }
}

/// Create a mock validation report JSON
pub fn create_mock_validation_report(
    status: &str,
    violations: u32,
    sample_count: u32,
) -> String {
    format!(
        r#"{{
    "status": "{}",
    "violations": {},
    "improvements": 2,
    "information": 5,
    "registry_coverage": 0.85,
    "sample_count": {},
    "details": [
        {{
            "level": "violation",
            "metric_name": "test.metric",
            "span_name": null,
            "message": "Test violation message",
            "registry_path": "registry/test.yaml"
        }}
    ]
}}"#,
        status, violations, sample_count
    )
}

/// Write a mock validation report to the output directory
pub fn write_mock_validation_report(
    output_dir: &std::path::Path,
    status: &str,
    violations: u32,
    sample_count: u32,
) -> std::io::Result<()> {
    std::fs::create_dir_all(output_dir)?;
    let report = create_mock_validation_report(status, violations, sample_count);
    let report_path = output_dir.join("validation_report.json");
    std::fs::write(report_path, report)?;
    Ok(())
}

/// Helper to clean up test artifacts
pub fn cleanup_test_artifacts() {
    let _ = std::fs::remove_dir_all("./validation_output");
    let _ = std::fs::remove_dir_all("./test_validation_output");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_available_port() {
        let port = find_available_port(9000, 9100);
        assert!(port.is_some());
    }

    #[test]
    fn test_port_blocker() {
        let port = find_available_port(9000, 9100).unwrap();
        let mut blocker = PortBlocker::new(port).unwrap();

        // Port should be occupied
        assert!(TcpListener::bind(("127.0.0.1", port)).is_err());

        // Release port
        blocker.release();

        // Port should be available again (after a brief delay)
        thread::sleep(Duration::from_millis(100));
        assert!(TcpListener::bind(("127.0.0.1", port)).is_ok());
    }

    #[test]
    fn test_create_mock_validation_report() {
        let report = create_mock_validation_report("success", 0, 100);
        assert!(report.contains("\"status\": \"success\""));
        assert!(report.contains("\"violations\": 0"));
        assert!(report.contains("\"sample_count\": 100"));
    }
}
