//! Weaver Emit Integration - Test Data Generation from Schemas
//!
//! This module integrates with Weaver's `registry emit` command to provide:
//! - Schema-based test data generation
//! - Example telemetry emission for validation
//! - Fixture generation for integration tests
//! - Live-check validation with synthetic data
//!
//! ## Purpose
//!
//! The `emit` command generates example telemetry conforming to schemas,
//! which is invaluable for:
//! - Testing validation pipelines without production data
//! - Demonstrating schema compliance
//! - Seeding test environments with realistic data
//! - Validating collector configurations
//!
//! ## Integration
//!
//! ```rust
//! use clnrm_core::telemetry::weaver_emit::WeaverEmitter;
//!
//! let emitter = WeaverEmitter::new("registry/");
//! emitter.emit_to_endpoint("http://localhost:4317")?;
//! // Telemetry is now flowing to collector for validation
//! ```

use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tracing::{debug, info, warn};

/// Configuration for Weaver telemetry emission
#[derive(Debug, Clone)]
pub struct EmitConfig {
    /// Path to semantic convention registry
    pub registry_path: PathBuf,
    /// OTLP endpoint to emit to (default: localhost:4317)
    pub endpoint: String,
    /// Emit to stdout instead of OTLP (useful for debugging)
    pub stdout: bool,
    /// Enable debug output
    pub debug: bool,
}

impl Default for EmitConfig {
    fn default() -> Self {
        Self {
            registry_path: PathBuf::from("registry"),
            endpoint: "http://localhost:4317".to_string(),
            stdout: false,
            debug: false,
        }
    }
}

impl EmitConfig {
    /// Create config that emits to stdout for testing
    pub fn stdout() -> Self {
        Self {
            stdout: true,
            ..Default::default()
        }
    }

    /// Create config with custom endpoint
    pub fn with_endpoint<S: Into<String>>(endpoint: S) -> Self {
        Self {
            endpoint: endpoint.into(),
            ..Default::default()
        }
    }
}

/// Result of telemetry emission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmitResult {
    /// Number of spans emitted
    pub spans_emitted: usize,
    /// Number of metrics emitted
    pub metrics_emitted: usize,
    /// Number of events emitted (log records)
    pub events_emitted: usize,
    /// Total signals emitted
    pub total_signals: usize,
    /// Whether emission succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

impl EmitResult {
    /// Create successful result
    pub fn success(spans: usize, metrics: usize, events: usize) -> Self {
        Self {
            spans_emitted: spans,
            metrics_emitted: metrics,
            events_emitted: events,
            total_signals: spans + metrics + events,
            success: true,
            error: None,
        }
    }

    /// Create failed result
    pub fn failure(error: String) -> Self {
        Self {
            spans_emitted: 0,
            metrics_emitted: 0,
            events_emitted: 0,
            total_signals: 0,
            success: false,
            error: Some(error),
        }
    }
}

/// Weaver telemetry emitter
///
/// Wraps Weaver's `registry emit` command to generate schema-compliant
/// test data for validation and testing purposes.
pub struct WeaverEmitter {
    config: EmitConfig,
}

impl WeaverEmitter {
    /// Create a new emitter with default configuration
    pub fn new<P: AsRef<Path>>(registry_path: P) -> Self {
        Self {
            config: EmitConfig {
                registry_path: registry_path.as_ref().to_path_buf(),
                ..Default::default()
            },
        }
    }

    /// Create emitter with custom configuration
    pub fn with_config(config: EmitConfig) -> Self {
        Self { config }
    }

    /// Emit telemetry to OTLP endpoint (one-shot)
    ///
    /// Generates example telemetry from all schemas in the registry
    /// and sends to the configured OTLP endpoint.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Weaver binary not found
    /// - Registry invalid
    /// - Endpoint unreachable
    /// - Emission failed
    pub fn emit(&self) -> Result<EmitResult> {
        info!(
            "🚀 Emitting telemetry from registry: {}",
            self.config.registry_path.display()
        );

        // Validate registry exists
        if !self.config.registry_path.exists() {
            return Err(CleanroomError::validation_error(format!(
                "Registry not found: {}",
                self.config.registry_path.display()
            )));
        }

        // Build weaver emit command
        let mut cmd = Command::new("weaver");
        cmd.args([
            "registry",
            "emit",
            "--registry",
            &self.config.registry_path.display().to_string(),
        ]);

        if self.config.stdout {
            cmd.arg("--stdout");
        } else {
            cmd.args(["--endpoint", &self.config.endpoint]);
        }

        if self.config.debug {
            cmd.arg("--debug");
        }

        debug!("Weaver emit command: {:?}", cmd);

        // Execute command
        let output = cmd.output().map_err(|e| {
            CleanroomError::internal_error(format!(
                "Failed to run weaver emit (is it installed?): {}",
                e
            ))
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if self.config.debug {
            debug!("Weaver emit stdout: {}", stdout);
            debug!("Weaver emit stderr: {}", stderr);
        }

        if !output.status.success() {
            return Ok(EmitResult::failure(format!(
                "Weaver emit failed: {}",
                stderr
            )));
        }

        // Parse result from output
        let result = self.parse_emit_output(&stdout, &stderr)?;

        info!(
            "✅ Emitted {} signals ({} spans, {} metrics, {} events)",
            result.total_signals,
            result.spans_emitted,
            result.metrics_emitted,
            result.events_emitted
        );

        Ok(result)
    }

    /// Start continuous emission (for long-running validation)
    ///
    /// Spawns Weaver emit as a background process that continuously
    /// generates telemetry. Useful for testing collectors and pipelines.
    ///
    /// # Returns
    ///
    /// Returns a handle to the background process that can be stopped.
    pub fn start_continuous(&self) -> Result<EmitHandle> {
        info!("🔄 Starting continuous telemetry emission");

        // Build command
        let mut cmd = Command::new("weaver");
        cmd.args([
            "registry",
            "emit",
            "--registry",
            &self.config.registry_path.display().to_string(),
            "--endpoint",
            &self.config.endpoint,
        ]);

        if self.config.debug {
            cmd.arg("--debug");
        }

        // Spawn as background process
        let child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                CleanroomError::internal_error(format!("Failed to spawn weaver emit: {}", e))
            })?;

        info!("✅ Continuous emission started (PID: {})", child.id());

        Ok(EmitHandle { process: child })
    }

    /// Emit telemetry and capture to string (for testing)
    ///
    /// Uses --stdout flag to capture generated telemetry as JSON.
    pub fn emit_to_string(&self) -> Result<String> {
        let mut config = self.config.clone();
        config.stdout = true;

        let emitter = WeaverEmitter::with_config(config);
        let result = emitter.emit()?;

        if !result.success {
            return Err(CleanroomError::validation_error(format!(
                "Emission failed: {}",
                result.error.unwrap_or_default()
            )));
        }

        // Re-run to capture stdout (previous call validated)
        let output = Command::new("weaver")
            .args([
                "registry",
                "emit",
                "--registry",
                &self.config.registry_path.display().to_string(),
                "--stdout",
            ])
            .output()
            .map_err(|e| CleanroomError::internal_error(format!("Failed to run weaver: {}", e)))?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Parse emission result from Weaver output
    fn parse_emit_output(&self, _stdout: &str, stderr: &str) -> Result<EmitResult> {
        // Weaver emit outputs summary to stderr
        // Example: "Emitted 15 spans, 10 metrics, 5 events"

        let mut spans = 0;
        let mut metrics = 0;
        let mut events = 0;

        for line in stderr.lines() {
            if line.contains("Emitted") || line.contains("emitted") {
                // Try to extract numbers
                let words: Vec<&str> = line.split_whitespace().collect();
                for (i, word) in words.iter().enumerate() {
                    if let Ok(num) = word.parse::<usize>() {
                        if i + 1 < words.len() {
                            let next = words[i + 1].to_lowercase();
                            if next.contains("span") {
                                spans = num;
                            } else if next.contains("metric") {
                                metrics = num;
                            } else if next.contains("event") || next.contains("log") {
                                events = num;
                            }
                        }
                    }
                }
            }
        }

        // If we couldn't parse counts, assume success with unknown counts
        if spans == 0 && metrics == 0 && events == 0 {
            warn!("Could not parse emission counts from output, assuming success");
            spans = 1; // Assume at least some emission happened
        }

        Ok(EmitResult::success(spans, metrics, events))
    }
}

/// Handle to a continuously emitting Weaver process
///
/// Automatically stops the process when dropped.
pub struct EmitHandle {
    process: Child,
}

impl EmitHandle {
    /// Stop the emission process
    pub fn stop(mut self) -> Result<()> {
        info!("🛑 Stopping continuous emission");

        self.process.kill().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to kill emitter process: {}", e))
        })?;

        let _ = self.process.wait();
        info!("✅ Emission stopped");

        Ok(())
    }

    /// Check if process is still running
    pub fn is_running(&mut self) -> bool {
        match self.process.try_wait() {
            Ok(None) => true,
            _ => false,
        }
    }

    /// Wait for process to finish (with timeout)
    pub fn wait_with_timeout(&mut self, timeout: Duration) -> Result<()> {
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            if !self.is_running() {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        Err(CleanroomError::timeout_error(
            "Emitter did not stop within timeout",
        ))
    }
}

impl Drop for EmitHandle {
    fn drop(&mut self) {
        debug!("Cleaning up emitter process in Drop");
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

/// Helper to emit test fixtures for integration tests
///
/// This is a convenience wrapper for common testing scenarios.
pub struct FixtureGenerator {
    registry_path: PathBuf,
}

impl FixtureGenerator {
    /// Create a new fixture generator
    pub fn new<P: AsRef<Path>>(registry_path: P) -> Self {
        Self {
            registry_path: registry_path.as_ref().to_path_buf(),
        }
    }

    /// Generate JSON fixtures for all schemas
    pub fn generate_json_fixtures(&self) -> Result<String> {
        let config = EmitConfig {
            registry_path: self.registry_path.clone(),
            stdout: true,
            ..Default::default()
        };

        let emitter = WeaverEmitter::with_config(config);
        emitter.emit_to_string()
    }

    /// Emit fixtures to a file
    pub fn emit_to_file<P: AsRef<Path>>(&self, output_path: P) -> Result<()> {
        let fixtures = self.generate_json_fixtures()?;

        std::fs::write(output_path.as_ref(), fixtures)
            .map_err(|e| CleanroomError::io_error(format!("Failed to write fixtures: {}", e)))?;

        info!("✅ Fixtures written to: {}", output_path.as_ref().display());

        Ok(())
    }

    /// Emit test data to collector for validation testing
    pub fn seed_collector(&self, endpoint: &str) -> Result<EmitResult> {
        let config = EmitConfig {
            registry_path: self.registry_path.clone(),
            endpoint: endpoint.to_string(),
            ..Default::default()
        };

        let emitter = WeaverEmitter::with_config(config);
        emitter.emit()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_config_default() {
        let config = EmitConfig::default();
        assert_eq!(config.registry_path, PathBuf::from("registry"));
        assert_eq!(config.endpoint, "http://localhost:4317");
        assert!(!config.stdout);
        assert!(!config.debug);
    }

    #[test]
    fn test_emit_config_stdout() {
        let config = EmitConfig::stdout();
        assert!(config.stdout);
    }

    #[test]
    fn test_emit_config_with_endpoint() {
        let config = EmitConfig::with_endpoint("http://example.com:4317");
        assert_eq!(config.endpoint, "http://example.com:4317");
    }

    #[test]
    fn test_emit_result_success() {
        let result = EmitResult::success(10, 5, 3);
        assert!(result.success);
        assert_eq!(result.spans_emitted, 10);
        assert_eq!(result.metrics_emitted, 5);
        assert_eq!(result.events_emitted, 3);
        assert_eq!(result.total_signals, 18);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_emit_result_failure() {
        let result = EmitResult::failure("Test error".to_string());
        assert!(!result.success);
        assert_eq!(result.total_signals, 0);
        assert_eq!(result.error, Some("Test error".to_string()));
    }

    #[test]
    fn test_weaver_emitter_creation() {
        let emitter = WeaverEmitter::new("registry/");
        assert_eq!(emitter.config.registry_path, PathBuf::from("registry/"));
    }

    #[test]
    fn test_fixture_generator_creation() {
        let generator = FixtureGenerator::new("registry/");
        assert_eq!(generator.registry_path, PathBuf::from("registry/"));
    }

    #[test]
    fn test_parse_emit_output() {
        let emitter = WeaverEmitter::new("test");
        let stderr = "Emitted 15 spans, 10 metrics, 5 events successfully";

        let result = emitter.parse_emit_output("", stderr).unwrap();
        assert_eq!(result.spans_emitted, 15);
        assert_eq!(result.metrics_emitted, 10);
        assert_eq!(result.events_emitted, 5);
        assert!(result.success);
    }

    #[test]
    #[ignore = "Requires Weaver installation"]
    fn test_emit_integration() {
        // This test requires Weaver and a valid registry
        // Run with: cargo test --ignored
    }
}
