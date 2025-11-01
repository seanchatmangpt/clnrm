//! Type-Safe Weaver Coordination Pattern
//!
//! This module provides a compile-time enforced state machine for Weaver lifecycle management
//! It prevents incorrect initialization order and ensures proper resource cleanup through the type system.
//!
//! ## Design Philosophy
//!
//! The type-safe state machine uses Rust's type system to enforce correct usage patterns:
//! - `WeaverController<Unstarted>` - Initial state, can only be started
//! - `WeaverController<Running>` - Active state, can access coordination and stop
//! - `WeaverController<Stopped>` - Terminal state, can only inspect results
//!
//! This prevents runtime errors like:
//! - Starting an already-running Weaver instance
//! - Stopping a non-running instance
//! - Accessing coordination before Weaver is ready
//! - Forgetting to flush telemetry before shutdown

use crate::error::{CleanroomError, Result};
use crate::telemetry::weaver_controller::{ValidationReport, ValidationStatus, WeaverCoordination};
use std::io::{BufRead, BufReader};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Configuration for Weaver live-check (shared across all states)
#[derive(Debug, Clone)]
pub struct WeaverConfig {
    pub registry_path: PathBuf,
    pub otlp_port: u16,
    pub admin_port: u16,
    pub output_dir: PathBuf,
    pub stream: bool,
}

impl Default for WeaverConfig {
    fn default() -> Self {
        Self {
            registry_path: PathBuf::from("registry"),
            otlp_port: 0,
            admin_port: 0,
            output_dir: PathBuf::from("./validation_output"),
            stream: false,
        }
    }
}

/// State marker: Weaver not started yet
pub struct Unstarted;

/// State marker: Weaver is running
pub struct Running;

/// State marker: Weaver has stopped
pub struct Stopped;

/// Internal state container
struct WeaverState {
    live_check_process: Option<Child>,
    has_violations: Arc<AtomicBool>,
    monitor_thread: Option<thread::JoinHandle<()>>,
    coordination: Option<WeaverCoordination>,
    validation_report: Option<ValidationReport>,
}

impl WeaverState {
    fn new() -> Self {
        Self {
            live_check_process: None,
            has_violations: Arc::new(AtomicBool::new(false)),
            monitor_thread: None,
            coordination: None,
            validation_report: None,
        }
    }
}

/// Type-safe Weaver controller with compile-time state enforcement
pub struct WeaverController<State = Unstarted> {
    config: WeaverConfig,
    state: PhantomData<State>,
    inner: ManuallyDrop<WeaverState>,
}

impl WeaverController<Unstarted> {
    pub fn new(config: WeaverConfig) -> Self {
        Self {
            config,
            state: PhantomData,
            inner: ManuallyDrop::new(WeaverState::new()),
        }
    }

    pub fn start_and_coordinate(mut self) -> Result<WeaverController<Running>> {
        info!("🚀 Starting Weaver with type-safe coordination");

        Self::cleanup_old_weaver_processes()?;

        let otlp_port = Self::find_available_port_with_fallback()?;
        let admin_port = Self::find_available_port(8080, 8090).or_else(|_| {
            warn!("Primary admin port range exhausted, trying fallback");
            Self::find_available_port(9080, 9090)
        })?;

        self.config.otlp_port = otlp_port;
        self.config.admin_port = admin_port;

        info!("📡 Discovered OTLP port: {}", otlp_port);
        info!("🔧 Discovered admin port: {}", admin_port);

        std::fs::create_dir_all(&self.config.output_dir).map_err(|e| {
            CleanroomError::io_error(format!("Failed to create output directory: {}", e))
        })?;

        let mut cmd = Command::new("weaver");
        cmd.args([
            "registry",
            "live-check",
            "--registry",
            &self.config.registry_path.display().to_string(),
            "--otlp-grpc-port",
            &self.config.otlp_port.to_string(),
            "--admin-port",
            &self.config.admin_port.to_string(),
            "--output",
            &self.config.output_dir.display().to_string(),
            "--format",
            "json",
        ]);

        if !self.config.stream {
            cmd.arg("--no-stream");
        }

        debug!("Weaver command: {:?}", cmd);

        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                CleanroomError::internal_error(format!(
                    "Failed to start Weaver (is it installed?): {}",
                    e
                ))
            })?;

        let weaver_pid = child.id();
        info!("🔍 Weaver process started (PID: {})", weaver_pid);

        let monitor_thread = if self.config.stream {
            let stderr = child
                .stderr
                .take()
                .ok_or_else(|| CleanroomError::internal_error("Failed to capture Weaver stderr"))?;

            let violations_flag = Arc::clone(&self.inner.has_violations);
            Some(thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    match line {
                        Ok(line) => {
                            debug!("Weaver: {}", line);
                            if line.contains("violation") || line.contains("error") {
                                violations_flag.store(true, Ordering::Relaxed);
                            }
                        }
                        Err(e) => {
                            warn!("Error reading Weaver output: {}", e);
                            break;
                        }
                    }
                }
            }))
        } else {
            None
        };

        Self::wait_for_ready(&mut child, Duration::from_secs(10))?;

        let ready_at = Instant::now();
        info!("✅ Weaver is ready and coordinated");

        let coordination = WeaverCoordination {
            weaver_pid,
            otlp_grpc_port: otlp_port,
            admin_port,
            ready_at,
        };

        self.inner.live_check_process = Some(child);
        self.inner.monitor_thread = monitor_thread;
        self.inner.coordination = Some(coordination);

        // Transfer ownership safely
        let inner = unsafe { ManuallyDrop::take(&mut self.inner) };

        Ok(WeaverController {
            config: self.config.clone(),
            state: PhantomData,
            inner: ManuallyDrop::new(inner),
        })
    }

    fn find_available_port_with_fallback() -> Result<u16> {
        debug!("Searching for available OTLP port with fallback");

        if let Ok(port) = Self::find_available_port(4317, 4327) {
            info!("✅ Found available port in primary range: {}", port);
            return Ok(port);
        }

        warn!("Primary OTLP port range (4317-4327) exhausted, trying fallback range");
        Self::find_available_port(5317, 5327).map_err(|_| {
            CleanroomError::validation_error("No available ports in range 4317-4327, 5317-5327")
        })
    }

    fn find_available_port(start: u16, end: u16) -> Result<u16> {
        debug!("Searching for available port in range {}-{}", start, end);

        for port in start..=end {
            if TcpListener::bind(("127.0.0.1", port)).is_ok() {
                debug!("Found available port: {}", port);
                return Ok(port);
            }
        }

        Err(CleanroomError::validation_error(format!(
            "No available ports in range {}-{}",
            start, end
        )))
    }

    fn wait_for_ready(process: &mut Child, _timeout: Duration) -> Result<()> {
        info!("⏳ Waiting for Weaver to become ready...");
        let start = Instant::now();

        thread::sleep(Duration::from_millis(1000));

        match process.try_wait() {
            Ok(Some(status)) => Err(CleanroomError::internal_error(format!(
                "Weaver exited prematurely with status: {}",
                status
            ))),
            Ok(None) => {
                let elapsed = start.elapsed();
                info!("✅ Weaver ready (elapsed: {}ms)", elapsed.as_millis());
                Ok(())
            }
            Err(e) => Err(CleanroomError::internal_error(format!(
                "Failed to check Weaver status: {}",
                e
            ))),
        }
    }

    fn cleanup_old_weaver_processes() -> Result<()> {
        debug!("Cleaning up orphaned Weaver processes");

        #[cfg(unix)]
        {
            let _ = Command::new("pkill")
                .args(["-9", "-f", "weaver registry live-check"])
                .output();
            thread::sleep(Duration::from_millis(500));
        }

        #[cfg(not(unix))]
        {
            let _ = Command::new("taskkill")
                .args(&["/F", "/IM", "weaver.exe"])
                .output();
            thread::sleep(Duration::from_millis(500));
        }

        Ok(())
    }
}

impl WeaverController<Running> {
    pub fn coordination(&self) -> &WeaverCoordination {
        self.inner
            .coordination
            .as_ref()
            .expect("Running state always has coordination")
    }

    pub fn is_validation_passing(&self) -> bool {
        !self.inner.has_violations.load(Ordering::Relaxed)
    }

    pub fn get_otlp_port(&self) -> u16 {
        self.coordination().otlp_grpc_port
    }

    pub fn get_admin_port(&self) -> u16 {
        self.coordination().admin_port
    }

    pub fn stop(mut self) -> Result<WeaverController<Stopped>> {
        info!("🛑 Stopping Weaver and retrieving validation report");

        let mut process = self
            .inner
            .live_check_process
            .take()
            .expect("Running state always has process");

        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;

            let pid = Pid::from_raw(process.id() as i32);
            debug!("Sending SIGHUP to Weaver (PID: {})", pid);

            kill(pid, Signal::SIGHUP).map_err(|e| {
                CleanroomError::internal_error(format!("Failed to send SIGHUP: {}", e))
            })?;
        }

        #[cfg(not(unix))]
        {
            warn!("Graceful shutdown not supported on this platform, killing process");
            process.kill().map_err(|e| {
                CleanroomError::internal_error(format!("Failed to kill Weaver: {}", e))
            })?;
        }

        // FIX: Always wait for process, even on error, to prevent zombie processes
        let output_result = Self::wait_with_timeout(&mut process, Duration::from_secs(10));

        // Ensure process is cleaned up even if wait_with_timeout failed
        if output_result.is_err() {
            warn!("Wait timeout failed, ensuring process cleanup to prevent zombie");
            let _ = process.wait(); // Prevent zombie process
        }

        let output = output_result?;

        if !output.stdout.is_empty() {
            debug!("Weaver stdout: {}", String::from_utf8_lossy(&output.stdout));
        }
        if !output.stderr.is_empty() {
            debug!("Weaver stderr: {}", String::from_utf8_lossy(&output.stderr));
        }

        if let Some(monitor) = self.inner.monitor_thread.take() {
            let _ = monitor.join();
        }

        let report_path = self.config.output_dir.join("validation_report.json");
        let report = if report_path.exists() {
            let report_json = std::fs::read_to_string(&report_path).map_err(|e| {
                CleanroomError::io_error(format!("Failed to read validation report: {}", e))
            })?;

            let mut report: ValidationReport = serde_json::from_str(&report_json).map_err(|e| {
                CleanroomError::serialization_error(format!(
                    "Failed to parse validation report: {}",
                    e
                ))
            })?;

            if report.sample_count == 0 {
                error!("🚨 CRITICAL: Weaver received ZERO telemetry samples!");
                report.status = ValidationStatus::Failure;
            }

            info!("📊 Validation Report Summary:");
            info!("   Status: {:?}", report.status);
            info!("   Samples Received: {}", report.sample_count);
            info!("   Violations: {}", report.violations);

            if report.violations > 0 {
                error!("❌ Weaver detected {} violations", report.violations);
            } else {
                info!("✅ No violations detected");
            }

            report
        } else {
            warn!("Validation report not found at {:?}", report_path);
            ValidationReport::default()
        };

        self.inner.validation_report = Some(report);

        // Transfer ownership safely
        let inner = unsafe { ManuallyDrop::take(&mut self.inner) };

        Ok(WeaverController {
            config: self.config.clone(),
            state: PhantomData,
            inner: ManuallyDrop::new(inner),
        })
    }

    fn wait_with_timeout(process: &mut Child, timeout: Duration) -> Result<std::process::Output> {
        let start = Instant::now();

        loop {
            match process.try_wait() {
                Ok(Some(status)) => {
                    use std::io::Read;

                    let mut stdout = Vec::new();
                    let mut stderr = Vec::new();

                    if let Some(mut out) = process.stdout.take() {
                        let _ = out.read_to_end(&mut stdout);
                    }
                    if let Some(mut err) = process.stderr.take() {
                        let _ = err.read_to_end(&mut stderr);
                    }

                    return Ok(std::process::Output {
                        status,
                        stdout,
                        stderr,
                    });
                }
                Ok(None) => {
                    if start.elapsed() > timeout {
                        warn!("Weaver did not stop gracefully, killing");
                        let _ = process.kill();
                        return Err(CleanroomError::timeout_error(
                            "Weaver did not stop within timeout",
                        ));
                    }
                    thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    return Err(CleanroomError::internal_error(format!(
                        "Failed to check Weaver status: {}",
                        e
                    )));
                }
            }
        }
    }
}

impl WeaverController<Stopped> {
    pub fn report(&self) -> Result<ValidationReport> {
        Ok(self
            .inner
            .validation_report
            .clone()
            .expect("Stopped state always has report"))
    }

    pub fn coordination(&self) -> Option<&WeaverCoordination> {
        self.inner.coordination.as_ref()
    }

    pub fn into_report(mut self) -> ValidationReport {
        let inner = unsafe { ManuallyDrop::take(&mut self.inner) };
        inner
            .validation_report
            .expect("Stopped state always has report")
    }
}

impl<State> Drop for WeaverController<State> {
    fn drop(&mut self) {
        if let Some(mut process) = self.inner.live_check_process.take() {
            if self.inner.coordination.is_some() {
                warn!("⚠️  WeaverController dropped without calling stop()!");
            }
            debug!("Cleaning up Weaver process in Drop");
            let _ = process.kill();
            let _ = process.wait();
        }

        if let Some(monitor) = self.inner.monitor_thread.take() {
            let _ = monitor.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weaver_config_defaults() {
        let config = WeaverConfig::default();
        assert_eq!(config.otlp_port, 0);
        assert_eq!(config.admin_port, 0);
        assert_eq!(config.registry_path, PathBuf::from("registry"));
        assert!(!config.stream);
    }

    #[test]
    fn test_unstarted_state_creation() {
        let config = WeaverConfig::default();
        let controller = WeaverController::new(config);
        assert!(controller.inner.live_check_process.is_none());
    }
}
