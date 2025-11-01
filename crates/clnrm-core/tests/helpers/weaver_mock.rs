//! Mock Weaver Process for Unit Testing
//!
//! Provides a mock implementation of Weaver for testing the orchestration
//! logic without requiring the actual Weaver binary.

use clnrm_core::error::{CleanroomError, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;

/// Mock Weaver process for testing
pub struct MockWeaverProcess {
    /// Process ID (simulated)
    pid: u32,
    /// OTLP port
    otlp_port: u16,
    /// Admin port
    admin_port: u16,
    /// Is the process running?
    running: Arc<Mutex<bool>>,
    /// Telemetry samples received
    samples: Arc<Mutex<Vec<TelemetrySample>>>,
    /// Shutdown signal sender
    shutdown_tx: broadcast::Sender<()>,
    /// Shutdown signal receiver
    shutdown_rx: broadcast::Receiver<()>,
}

/// Telemetry sample (simplified)
#[derive(Debug, Clone)]
pub struct TelemetrySample {
    pub span_name: String,
    pub attributes: HashMap<String, String>,
    pub timestamp: std::time::Instant,
}

impl MockWeaverProcess {
    /// Create a new mock Weaver process
    pub fn new(otlp_port: u16, admin_port: u16) -> Self {
        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);

        Self {
            pid: std::process::id(),
            otlp_port,
            admin_port,
            running: Arc::new(Mutex::new(false)),
            samples: Arc::new(Mutex::new(Vec::new())),
            shutdown_tx,
            shutdown_rx,
        }
    }

    /// Start the mock process
    pub async fn start(&mut self) -> Result<()> {
        let mut running = self.running.lock().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to lock running state: {}", e))
        })?;

        if *running {
            return Err(CleanroomError::internal_error(
                "Mock Weaver already running",
            ));
        }

        *running = true;

        // Simulate startup delay
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(())
    }

    /// Stop the mock process
    pub async fn stop(&mut self) -> Result<()> {
        let mut running = self.running.lock().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to lock running state: {}", e))
        })?;

        if !*running {
            return Err(CleanroomError::internal_error("Mock Weaver not running"));
        }

        *running = false;

        // Send shutdown signal
        let _ = self.shutdown_tx.send(());

        // Simulate graceful shutdown delay
        tokio::time::sleep(Duration::from_millis(50)).await;

        Ok(())
    }

    /// Check if process is running
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap_or(&mut false)
    }

    /// Get process ID
    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// Get OTLP port
    pub fn otlp_port(&self) -> u16 {
        self.otlp_port
    }

    /// Get admin port
    pub fn admin_port(&self) -> u16 {
        self.admin_port
    }

    /// Simulate receiving a telemetry sample
    pub fn receive_sample(&self, sample: TelemetrySample) -> Result<()> {
        let mut samples = self.samples.lock().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to lock samples: {}", e))
        })?;

        samples.push(sample);
        Ok(())
    }

    /// Get all received samples
    pub fn get_samples(&self) -> Vec<TelemetrySample> {
        self.samples.lock().unwrap().clone()
    }

    /// Get sample count
    pub fn sample_count(&self) -> usize {
        self.samples.lock().unwrap().len()
    }

    /// Simulate health check
    pub async fn health_check(&self) -> Result<bool> {
        if !self.is_running() {
            return Ok(false);
        }

        // Simulate network latency
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(true)
    }

    /// Generate mock validation report
    pub fn generate_report(&self) -> MockValidationReport {
        let samples = self.get_samples();
        let sample_count = samples.len();

        // Simulate validation logic
        let critical_spans = vec![
            "clnrm.test.execute",
            "clnrm.container.start",
            "clnrm.container.stop",
        ];

        let mut found_spans = std::collections::HashSet::new();
        for sample in &samples {
            found_spans.insert(sample.span_name.clone());
        }

        let missing_spans: Vec<String> = critical_spans
            .iter()
            .filter(|s| !found_spans.contains(*s))
            .map(|s| s.to_string())
            .collect();

        let violations = missing_spans.len() as u32;
        let status = if violations == 0 {
            MockValidationStatus::Success
        } else {
            MockValidationStatus::Failure
        };

        MockValidationReport {
            status,
            violations,
            sample_count: sample_count as u32,
            missing_spans,
        }
    }
}

/// Mock validation report
#[derive(Debug, Clone)]
pub struct MockValidationReport {
    pub status: MockValidationStatus,
    pub violations: u32,
    pub sample_count: u32,
    pub missing_spans: Vec<String>,
}

/// Mock validation status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockValidationStatus {
    Success,
    Failure,
}

// ============================================================================
// Mock Weaver Manager (higher-level abstraction)
// ============================================================================

/// Mock implementation of WeaverProcessManager for testing
pub struct MockWeaverManager {
    process: MockWeaverProcess,
    registry_path: std::path::PathBuf,
    output_dir: std::path::PathBuf,
}

impl MockWeaverManager {
    /// Create new mock manager
    pub fn new(
        registry_path: std::path::PathBuf,
        output_dir: std::path::PathBuf,
    ) -> Result<Self> {
        // Allocate mock ports
        let otlp_port = 14317; // Mock port
        let admin_port = 18080; // Mock port

        let process = MockWeaverProcess::new(otlp_port, admin_port);

        Ok(Self {
            process,
            registry_path,
            output_dir,
        })
    }

    /// Start mock Weaver
    pub async fn start(&mut self) -> Result<(u16, u16)> {
        self.process.start().await?;
        Ok((self.process.otlp_port(), self.process.admin_port()))
    }

    /// Stop mock Weaver
    pub async fn stop(&mut self) -> Result<()> {
        self.process.stop().await
    }

    /// Get validation report
    pub fn collect_report(&self) -> Result<MockValidationReport> {
        Ok(self.process.generate_report())
    }

    /// Simulate receiving telemetry
    pub fn simulate_telemetry(&self, span_name: &str, attributes: HashMap<String, String>) {
        let sample = TelemetrySample {
            span_name: span_name.to_string(),
            attributes,
            timestamp: std::time::Instant::now(),
        };
        let _ = self.process.receive_sample(sample);
    }

    /// Get process reference
    pub fn process(&self) -> &MockWeaverProcess {
        &self.process
    }
}

// ============================================================================
// Test Utilities
// ============================================================================

/// Create a mock validation report with specific characteristics
pub fn create_mock_report(
    sample_count: u32,
    violations: u32,
) -> MockValidationReport {
    MockValidationReport {
        status: if violations == 0 {
            MockValidationStatus::Success
        } else {
            MockValidationStatus::Failure
        },
        violations,
        sample_count,
        missing_spans: (0..violations)
            .map(|i| format!("missing.span.{}", i))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_weaver_startup_and_shutdown() -> Result<()> {
        let mut mock = MockWeaverProcess::new(14317, 18080);

        assert!(!mock.is_running());

        mock.start().await?;
        assert!(mock.is_running());
        assert_eq!(mock.otlp_port(), 14317);
        assert_eq!(mock.admin_port(), 18080);

        mock.stop().await?;
        assert!(!mock.is_running());

        Ok(())
    }

    #[tokio::test]
    async fn test_mock_weaver_health_check() -> Result<()> {
        let mut mock = MockWeaverProcess::new(14317, 18080);

        // Not running - health check fails
        assert!(!mock.health_check().await?);

        // Running - health check succeeds
        mock.start().await?;
        assert!(mock.health_check().await?);

        mock.stop().await?;
        Ok(())
    }

    #[test]
    fn test_mock_weaver_telemetry_collection() -> Result<()> {
        let mock = MockWeaverProcess::new(14317, 18080);

        assert_eq!(mock.sample_count(), 0);

        let mut attrs = HashMap::new();
        attrs.insert("test.name".to_string(), "test1".to_string());

        let sample = TelemetrySample {
            span_name: "clnrm.test.execute".to_string(),
            attributes: attrs,
            timestamp: std::time::Instant::now(),
        };

        mock.receive_sample(sample)?;
        assert_eq!(mock.sample_count(), 1);

        let samples = mock.get_samples();
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].span_name, "clnrm.test.execute");

        Ok(())
    }

    #[test]
    fn test_mock_validation_report_generation() -> Result<()> {
        let mock = MockWeaverProcess::new(14317, 18080);

        // No samples - should fail validation
        let report = mock.generate_report();
        assert_eq!(report.status, MockValidationStatus::Failure);
        assert!(report.violations > 0);
        assert_eq!(report.sample_count, 0);

        // Add critical spans
        let critical_spans = vec![
            "clnrm.test.execute",
            "clnrm.container.start",
            "clnrm.container.stop",
        ];

        for span in critical_spans {
            let sample = TelemetrySample {
                span_name: span.to_string(),
                attributes: HashMap::new(),
                timestamp: std::time::Instant::now(),
            };
            mock.receive_sample(sample)?;
        }

        // All critical spans present - should pass
        let report = mock.generate_report();
        assert_eq!(report.status, MockValidationStatus::Success);
        assert_eq!(report.violations, 0);
        assert_eq!(report.sample_count, 3);

        Ok(())
    }
}
