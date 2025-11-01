//! Weaver Controller - Manages Weaver live-check lifecycle and validation reporting
//!
//! This module provides the critical integration between clnrm tests and Weaver's
//! semantic convention validation. It makes Weaver the single source of truth for
//! telemetry correctness.
//!
//! ## Lifecycle
//!
//! 1. Start Weaver live-check listener before tests
//! 2. Tests emit telemetry via OTLP
//! 3. Weaver validates telemetry in real-time
//! 4. Stop Weaver after tests and get validation report
//! 5. Exit with error if violations detected
//!
//! ## Architecture
//!
//! The WeaverController spawns Weaver as a child process and manages its lifecycle:
//! - Uses SIGHUP for graceful shutdown on Unix
//! - Parses JSON validation reports
//! - Provides both streaming and final validation status

use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Coordination metadata from Weaver startup
///
/// This structure contains all information needed to coordinate OTEL initialization
/// with Weaver's runtime state. It's returned by `start_and_coordinate()` and
/// ensures OTEL exports telemetry to Weaver's actual listening port.
#[derive(Debug, Clone)]
pub struct WeaverCoordination {
    /// Process ID of Weaver instance
    pub weaver_pid: u32,
    /// OTLP gRPC port Weaver is listening on
    pub otlp_grpc_port: u16,
    /// Admin/health port for control interface
    pub admin_port: u16,
    /// Timestamp when Weaver became ready
    pub ready_at: Instant,
}

/// Validation status returned by Weaver
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ValidationStatus {
    /// Validation passed with no violations
    Success,
    /// Validation failed with violations detected
    Failure,
}

/// Detailed information about a single validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationDetail {
    /// Severity level (violation, improvement, information)
    pub level: String,
    /// Metric or span name
    pub metric_name: Option<String>,
    /// Span name if applicable
    pub span_name: Option<String>,
    /// Description of the issue
    pub message: String,
    /// Path in registry
    pub registry_path: Option<String>,
}

/// Complete validation report from Weaver
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Overall validation status
    pub status: ValidationStatus,
    /// Number of violations (blocking issues)
    pub violations: u32,
    /// Number of improvements (suggestions)
    pub improvements: u32,
    /// Number of information messages
    pub information: u32,
    /// Percentage of registry coverage (0.0 - 1.0)
    pub registry_coverage: f64,
    /// Number of telemetry samples received (CRITICAL: must be > 0 for valid validation)
    #[serde(default)]
    pub sample_count: u32,
    /// Detailed list of validation issues
    pub details: Vec<ValidationDetail>,
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self {
            status: ValidationStatus::Failure, // Default to failure (no telemetry = failed validation)
            violations: 0,
            improvements: 0,
            information: 0,
            registry_coverage: 0.0,
            sample_count: 0,
            details: Vec::new(),
        }
    }
}

/// Configuration for Weaver live-check
#[derive(Debug, Clone)]
pub struct WeaverConfig {
    /// Path to semantic convention registry
    pub registry_path: PathBuf,
    /// OTLP gRPC port to listen on
    pub otlp_port: u16,
    /// Admin port for control interface
    pub admin_port: u16,
    /// Directory for output reports
    pub output_dir: PathBuf,
    /// Enable streaming output (for real-time feedback)
    pub stream: bool,
}

impl Default for WeaverConfig {
    fn default() -> Self {
        Self {
            registry_path: PathBuf::from("registry"),
            otlp_port: 0,  // 0 = auto-discover available port
            admin_port: 0, // 0 = auto-discover available port
            output_dir: PathBuf::from("./validation_output"),
            stream: false,
        }
    }
}

/// Controller for Weaver live-check process
///
/// Manages the lifecycle of a Weaver validation process, including:
/// - Starting the live-check listener
/// - Monitoring validation status
/// - Stopping and retrieving final report
///
/// # Example
///
/// ```no_run
/// use clnrm_core::telemetry::weaver_controller::{WeaverController, WeaverConfig};
/// use std::path::PathBuf;
///
/// # async fn example() -> clnrm_core::error::Result<()> {
/// let config = WeaverConfig {
///     registry_path: PathBuf::from("registry"),
///     output_dir: PathBuf::from("./validation_output"),
///     ..Default::default()
/// };
///
/// let mut controller = WeaverController::new(config);
///
/// // Start Weaver before tests
/// controller.start_live_check()?;
///
/// // Run tests (they emit telemetry to OTLP)
/// // ...
///
/// // Stop Weaver and get validation results
/// let report = controller.stop_and_report()?;
///
/// if report.violations > 0 {
///     eprintln!("Validation failed with {} violations", report.violations);
/// }
/// # Ok(())
/// # }
/// ```
pub struct WeaverController {
    config: WeaverConfig,
    live_check_process: Option<Child>,
    has_violations: Arc<AtomicBool>,
    monitor_thread: Option<thread::JoinHandle<()>>,
    coordination: Option<WeaverCoordination>,
}

impl WeaverController {
    /// Create a new WeaverController with the given configuration
    pub fn new(config: WeaverConfig) -> Self {
        Self {
            config,
            live_check_process: None,
            has_violations: Arc::new(AtomicBool::new(false)),
            monitor_thread: None,
            coordination: None,
        }
    }

    /// Start Weaver and return coordination info (Weaver-first pattern)
    ///
    /// This is the PRIMARY method for Weaver-first initialization.
    /// It blocks until Weaver is ready and returns coordination metadata
    /// that MUST be used to configure OTEL exporters.
    ///
    /// # Weaver-First Pattern
    ///
    /// ```no_run
    /// use clnrm_core::telemetry::weaver_controller::{WeaverController, WeaverConfig};
    /// use clnrm_core::telemetry::{init_otel, OtelConfig, Export};
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> clnrm_core::error::Result<()> {
    /// let config = WeaverConfig::default();
    /// let mut controller = WeaverController::new(config);
    ///
    /// // Step 1: Start Weaver and get coordination
    /// let coordination = controller.start_and_coordinate()?;
    /// println!("Weaver listening on port {}", coordination.otlp_grpc_port);
    ///
    /// // Step 2: Initialize OTEL with Weaver's actual port
    /// let endpoint = format!("http://localhost:{}", coordination.otlp_grpc_port);
    /// let _otel_guard = init_otel(OtelConfig {
    ///     service_name: "clnrm",
    ///     deployment_env: "testing",
    ///     sample_ratio: 1.0,
    ///     export: Export::OtlpGrpc {
    ///         endpoint: Box::leak(endpoint.into_boxed_str()),
    ///     },
    ///     enable_fmt_layer: false,
    ///     headers: None,
    /// })?;
    ///
    /// // Step 3: Run tests (telemetry goes to Weaver)
    /// // ...
    ///
    /// // Step 4: Flush OTEL before stopping Weaver
    /// drop(_otel_guard);
    /// std::thread::sleep(std::time::Duration::from_millis(500));
    ///
    /// // Step 5: Stop Weaver and get validation report
    /// let report = controller.stop_and_report()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Weaver binary not found
    /// - No available ports in range
    /// - Weaver process fails to start
    /// - Health check timeout
    pub fn start_and_coordinate(&mut self) -> Result<WeaverCoordination> {
        info!("🚀 Starting Weaver with coordination (Weaver-first pattern)");

        // Cleanup any orphaned processes first
        Self::cleanup_old_weaver_processes()?;

        // Find available ports with intelligent fallback
        let otlp_port = Self::find_available_port_with_fallback()?;
        let admin_port = Self::find_available_port(8080, 8090).or_else(|_| {
            warn!("Primary admin port range exhausted, trying fallback");
            Self::find_available_port(9080, 9090)
        })?;

        // Update config with discovered ports
        self.config.otlp_port = otlp_port;
        self.config.admin_port = admin_port;

        info!("📡 Discovered OTLP port: {}", otlp_port);
        info!("🔧 Discovered admin port: {}", admin_port);

        // Ensure output directory exists
        std::fs::create_dir_all(&self.config.output_dir).map_err(|e| {
            CleanroomError::io_error(format!("Failed to create output directory: {}", e))
        })?;

        // Build Weaver command
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

        // Add streaming flag if disabled
        if !self.config.stream {
            cmd.arg("--no-stream");
        }

        debug!("Weaver command: {:?}", cmd);

        // Spawn process with piped output
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

        // If streaming is enabled, spawn a monitor thread
        if self.config.stream {
            let stderr = child
                .stderr
                .take()
                .ok_or_else(|| CleanroomError::internal_error("Failed to capture Weaver stderr"))?;

            let violations_flag = Arc::clone(&self.has_violations);
            let monitor = thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    match line {
                        Ok(line) => {
                            debug!("Weaver: {}", line);
                            // Check for violation indicators
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
            });

            self.monitor_thread = Some(monitor);
        }

        self.live_check_process = Some(child);

        // Wait for Weaver to be ready
        self.wait_for_ready(Duration::from_secs(10))?;

        let ready_at = Instant::now();
        info!("✅ Weaver is ready and coordinated");

        // Create coordination metadata
        let coordination = WeaverCoordination {
            weaver_pid,
            otlp_grpc_port: otlp_port,
            admin_port,
            ready_at,
        };

        // Store coordination for later queries
        self.coordination = Some(coordination.clone());

        Ok(coordination)
    }

    /// Get current coordination state (non-blocking)
    ///
    /// Returns None if Weaver not started via `start_and_coordinate()`,
    /// otherwise returns the coordination metadata.
    pub fn coordination(&self) -> Option<WeaverCoordination> {
        self.coordination.clone()
    }

    /// Find available port with intelligent fallback
    ///
    /// Strategy:
    /// 1. Try primary range (4317-4327) - standard OTLP gRPC ports
    /// 2. Fallback to secondary range (5317-5327) if primary exhausted
    ///
    /// # Errors
    ///
    /// Returns an error if no ports available in any range
    fn find_available_port_with_fallback() -> Result<u16> {
        debug!("Searching for available OTLP port with fallback");

        // Try primary range (standard OTLP gRPC ports)
        if let Ok(port) = Self::find_available_port(4317, 4327) {
            info!("✅ Found available port in primary range: {}", port);
            return Ok(port);
        }

        // Fallback to secondary range
        warn!("Primary OTLP port range (4317-4327) exhausted, trying fallback range");
        Self::find_available_port(5317, 5327).map_err(|_| {
            CleanroomError::validation_error(
                "No available ports in range 4317-4327, 5317-5327. \
                 All ports in use. Stop other OTLP services or use custom port range.",
            )
        })
    }

    /// Wait for Weaver to become ready
    ///
    /// Health check strategy:
    /// 1. Initial delay (1000ms) for process startup
    /// 2. Check process still running (not crashed)
    /// 3. Return success if process is running
    ///
    /// Future enhancement: Add HTTP health check to admin port
    fn wait_for_ready(&mut self, _timeout: Duration) -> Result<()> {
        info!("⏳ Waiting for Weaver to become ready...");
        let start = Instant::now();

        // Initial startup delay
        thread::sleep(Duration::from_millis(1000));

        // Check process state
        if let Some(ref mut process) = self.live_check_process {
            match process.try_wait() {
                Ok(Some(status)) => {
                    return Err(CleanroomError::internal_error(format!(
                        "Weaver exited prematurely with status: {}. \
                         Check Weaver logs in validation_output/ for details.",
                        status
                    )));
                }
                Ok(None) => {
                    // Still running, assume ready
                    let elapsed = start.elapsed();
                    info!("✅ Weaver ready (elapsed: {}ms)", elapsed.as_millis());
                    return Ok(());
                }
                Err(e) => {
                    return Err(CleanroomError::internal_error(format!(
                        "Failed to check Weaver status: {}",
                        e
                    )));
                }
            }
        }

        Err(CleanroomError::timeout_error(
            "Weaver not ready within timeout",
        ))
    }

    /// Find an available port in the specified range
    ///
    /// Attempts to bind to each port in range [start, end] and returns the first
    /// available port. This prevents port conflicts when starting Weaver.
    ///
    /// # Arguments
    ///
    /// * `start` - Start of port range (inclusive)
    /// * `end` - End of port range (inclusive)
    ///
    /// # Errors
    ///
    /// Returns an error if no ports are available in the range
    fn find_available_port(start: u16, end: u16) -> Result<u16> {
        debug!("Searching for available port in range {}-{}", start, end);

        for port in start..=end {
            match TcpListener::bind(("127.0.0.1", port)) {
                Ok(_) => {
                    debug!("Found available port: {}", port);
                    return Ok(port);
                }
                Err(_) => continue,
            }
        }

        Err(CleanroomError::validation_error(format!(
            "No available ports in range {}-{}. All ports are in use.",
            start, end
        )))
    }

    /// Cleanup any orphaned Weaver processes
    ///
    /// Attempts to kill any existing `weaver registry live-check` processes
    /// to prevent port conflicts and resource leaks. This is called before
    /// starting a new Weaver instance.
    fn cleanup_old_weaver_processes() -> Result<()> {
        debug!("Cleaning up orphaned Weaver processes");

        // Try to kill any existing weaver live-check processes
        #[cfg(unix)]
        {
            let _ = Command::new("pkill")
                .args(["-9", "-f", "weaver registry live-check"])
                .output();

            // Give processes time to terminate
            thread::sleep(Duration::from_millis(500));
        }

        #[cfg(not(unix))]
        {
            // On Windows, use taskkill
            let _ = Command::new("taskkill")
                .args(&["/F", "/IM", "weaver.exe"])
                .output();

            thread::sleep(Duration::from_millis(500));
        }

        Ok(())
    }

    /// Start Weaver live-check listener
    ///
    /// Spawns Weaver as a child process and waits for it to be ready.
    /// Includes intelligent port management and process cleanup.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Weaver binary is not found
    /// - Failed to create output directory
    /// - Process failed to start
    /// - Listener failed to become ready
    /// - No available ports in range
    pub fn start_live_check(&mut self) -> Result<()> {
        info!("🔍 Starting Weaver live-check validation");

        // Cleanup any orphaned processes first
        Self::cleanup_old_weaver_processes()?;

        // Find available ports with retry logic
        let otlp_port = Self::find_available_port(4317, 4327).or_else(|_| {
            warn!("Primary OTLP port range exhausted, trying fallback range");
            Self::find_available_port(5317, 5327)
        })?;

        let admin_port = Self::find_available_port(8080, 8090).or_else(|_| {
            warn!("Primary admin port range exhausted, trying fallback range");
            Self::find_available_port(9080, 9090)
        })?;

        // Update config with discovered ports
        self.config.otlp_port = otlp_port;
        self.config.admin_port = admin_port;

        info!("📡 Using OTLP port: {}", otlp_port);
        info!("🔧 Using admin port: {}", admin_port);

        // Ensure output directory exists
        std::fs::create_dir_all(&self.config.output_dir).map_err(|e| {
            CleanroomError::io_error(format!("Failed to create output directory: {}", e))
        })?;

        // Build Weaver command
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

        // Add streaming flag if disabled
        if !self.config.stream {
            cmd.arg("--no-stream");
        }

        debug!("Weaver command: {:?}", cmd);

        // Spawn process with piped output
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

        // If streaming is enabled, spawn a monitor thread
        if self.config.stream {
            let stderr = child
                .stderr
                .take()
                .ok_or_else(|| CleanroomError::internal_error("Failed to capture Weaver stderr"))?;

            let violations_flag = Arc::clone(&self.has_violations);
            let monitor = thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    match line {
                        Ok(line) => {
                            debug!("Weaver: {}", line);
                            // Check for violation indicators
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
            });

            self.monitor_thread = Some(monitor);
        }

        self.live_check_process = Some(child);

        // Wait for listener to be ready with proper health check
        info!("⏳ Waiting for Weaver to initialize...");
        self.wait_for_weaver_ready()?;

        Ok(())
    }

    /// Stop Weaver and retrieve validation report
    ///
    /// Sends SIGHUP to gracefully stop Weaver, then parses the validation report.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Weaver is not running
    /// - Failed to stop the process
    /// - Report file not found or invalid JSON
    pub fn stop_and_report(&mut self) -> Result<ValidationReport> {
        info!("🛑 Stopping Weaver and retrieving validation report");

        let mut process = self
            .live_check_process
            .take()
            .ok_or_else(|| CleanroomError::internal_error("Weaver live-check is not running"))?;

        // Send graceful shutdown signal
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

        // On non-Unix, just kill the process
        #[cfg(not(unix))]
        {
            warn!("Graceful shutdown not supported on this platform, killing process");
            process.kill().map_err(|e| {
                CleanroomError::internal_error(format!("Failed to kill Weaver: {}", e))
            })?;
        }

        // Wait for process to finish (with timeout)
        let output = self.wait_with_timeout(&mut process, Duration::from_secs(10))?;

        // Log output for debugging
        if !output.stdout.is_empty() {
            debug!("Weaver stdout: {}", String::from_utf8_lossy(&output.stdout));
        }
        if !output.stderr.is_empty() {
            debug!("Weaver stderr: {}", String::from_utf8_lossy(&output.stderr));
        }

        // Wait for monitor thread to finish
        if let Some(monitor) = self.monitor_thread.take() {
            let _ = monitor.join();
        }

        // Parse validation report
        let report_path = self.config.output_dir.join("validation_report.json");
        if !report_path.exists() {
            warn!("Validation report not found at {:?}", report_path);
            // Return a default report indicating unknown status
            return Ok(ValidationReport::default());
        }

        let report_json = std::fs::read_to_string(&report_path).map_err(|e| {
            CleanroomError::io_error(format!("Failed to read validation report: {}", e))
        })?;

        let mut report: ValidationReport = serde_json::from_str(&report_json).map_err(|e| {
            CleanroomError::serialization_error(format!("Failed to parse validation report: {}", e))
        })?;

        // CRITICAL: Zero-sample validation (prevents false positives)
        if report.sample_count == 0 {
            error!("🚨 CRITICAL: Weaver received ZERO telemetry samples!");
            error!("   This means validation did not actually test anything.");
            error!("   Possible causes:");
            error!("   - OTEL exporter not configured correctly");
            error!("   - Telemetry sent to wrong port");
            error!("   - Tests failed before emitting telemetry");
            report.status = ValidationStatus::Failure;
        }

        // Log summary
        info!("📊 Validation Report Summary:");
        info!("   Status: {:?}", report.status);
        info!("   Samples Received: {}", report.sample_count);
        info!("   Violations: {}", report.violations);
        info!("   Improvements: {}", report.improvements);
        info!("   Information: {}", report.information);
        info!(
            "   Registry Coverage: {:.1}%",
            report.registry_coverage * 100.0
        );

        if report.violations > 0 {
            error!("❌ Weaver detected {} violations", report.violations);
            for detail in &report.details {
                if detail.level == "violation" {
                    error!("   - {}", detail.message);
                }
            }
        } else {
            info!("✅ No violations detected");
        }

        Ok(report)
    }

    /// Check if validation has detected violations during streaming
    ///
    /// This is only useful when streaming is enabled in the configuration.
    pub fn is_validation_passing(&self) -> bool {
        !self.has_violations.load(Ordering::Relaxed)
    }

    /// Wait for Weaver to become ready by polling the admin health endpoint
    ///
    /// Uses exponential backoff with a maximum timeout. Polls the HTTP health endpoint
    /// at `http://localhost:{admin_port}/health` to verify Weaver is actually listening.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Weaver process exits before becoming ready
    /// - Health check times out (default: 30 seconds)
    /// - HTTP health check fails persistently
    fn wait_for_weaver_ready(&mut self) -> Result<()> {
        const MAX_TIMEOUT: Duration = Duration::from_secs(30);
        const INITIAL_DELAY: Duration = Duration::from_millis(100);
        const MAX_DELAY: Duration = Duration::from_millis(1000);

        let start = Instant::now();
        let mut delay = INITIAL_DELAY;

        // Check if process is still running
        if let Some(ref mut process) = self.live_check_process {
            match process.try_wait() {
                Ok(Some(status)) => {
                    return Err(CleanroomError::internal_error(format!(
                        "Weaver exited prematurely with status: {}. \
                         Check Weaver logs in validation_output/ for details.",
                        status
                    )));
                }
                Ok(None) => {
                    // Process still running, continue with health check
                }
                Err(e) => {
                    return Err(CleanroomError::internal_error(format!(
                        "Failed to check Weaver status: {}",
                        e
                    )));
                }
            }
        }

        let admin_port = self.config.admin_port;
        let health_url = format!("http://localhost:{}/health", admin_port);

        loop {
            // Check timeout
            if start.elapsed() > MAX_TIMEOUT {
                return Err(CleanroomError::timeout_error(format!(
                    "Weaver health check timed out after {}s. \
                     Admin port: {}, Health URL: {}",
                    MAX_TIMEOUT.as_secs(),
                    admin_port,
                    health_url
                )));
            }

            // Attempt HTTP health check using blocking reqwest client
            match Self::check_weaver_health(&health_url) {
                Ok(true) => {
                    let elapsed = start.elapsed();
                    info!(
                        "✅ Weaver health check passed (elapsed: {}ms)",
                        elapsed.as_millis()
                    );
                    return Ok(());
                }
                Ok(false) => {
                    // Health check failed or network error, continue polling
                    debug!("Health check not ready yet, will retry...");
                }
                Err(e) => {
                    // Unexpected error, but continue polling
                    debug!("Health check error: {} (will retry)", e);
                }
            }

            // Wait before next attempt with exponential backoff
            thread::sleep(delay);
            delay = std::cmp::min(delay * 2, MAX_DELAY);

            // Re-check process state periodically
            if let Some(ref mut process) = self.live_check_process {
                if let Ok(Some(status)) = process.try_wait() {
                    return Err(CleanroomError::internal_error(format!(
                        "Weaver exited during health check with status: {}. \
                         Check Weaver logs in validation_output/ for details.",
                        status
                    )));
                }
            }
        }
    }

    /// Check Weaver health endpoint
    ///
    /// Performs a blocking HTTP GET request to the Weaver health endpoint.
    /// Returns `Ok(true)` if health check succeeds, `Ok(false)` if it fails
    /// or network error occurs (allowing polling to continue).
    fn check_weaver_health(url: &str) -> Result<bool> {
        // Use blocking reqwest client for health checks
        let client = match reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
        {
            Ok(client) => client,
            Err(e) => {
                debug!("Failed to create HTTP client: {} (will retry)", e);
                return Ok(false);
            }
        };

        match client.get(url).send() {
            Ok(response) => {
                let status = response.status();
                Ok(status.is_success())
            }
            Err(e) => {
                // Network errors are expected during startup, return false to continue polling
                debug!("Health check request failed: {} (will retry)", e);
                Ok(false)
            }
        }
    }

    /// Get the OTLP port that Weaver is listening on
    ///
    /// This returns the port discovered during `start_live_check()`.
    /// Useful for configuring OTEL exporters to send to the correct endpoint.
    pub fn get_otlp_port(&self) -> u16 {
        self.config.otlp_port
    }

    /// Get the admin port that Weaver is listening on
    ///
    /// This returns the port discovered during `start_live_check()`.
    pub fn get_admin_port(&self) -> u16 {
        self.config.admin_port
    }

    /// Wait for process to finish with timeout
    fn wait_with_timeout(
        &self,
        process: &mut Child,
        timeout: Duration,
    ) -> Result<std::process::Output> {
        let start = std::time::Instant::now();

        loop {
            match process.try_wait() {
                Ok(Some(status)) => {
                    // Process finished, collect any remaining output
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
                    // Still running
                    if start.elapsed() > timeout {
                        // Timeout reached, kill process
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

impl Drop for WeaverController {
    fn drop(&mut self) {
        // Ensure process is killed if still running
        if let Some(mut process) = self.live_check_process.take() {
            debug!("Cleaning up Weaver process in Drop");
            let _ = process.kill();
            let _ = process.wait();
        }

        // Wait for monitor thread
        if let Some(monitor) = self.monitor_thread.take() {
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
        // Default config uses 0 for auto-discovery, not hardcoded ports
        assert_eq!(config.otlp_port, 0); // 0 = auto-discover available port
        assert_eq!(config.admin_port, 0); // 0 = auto-discover available port
        assert_eq!(config.registry_path, PathBuf::from("registry"));
        assert!(!config.stream);
    }

    #[test]
    fn test_validation_report_default() {
        let report = ValidationReport::default();
        // Default status is Failure (no telemetry = failed validation)
        assert_eq!(report.status, ValidationStatus::Failure);
        assert_eq!(report.violations, 0);
        assert_eq!(report.improvements, 0);
        assert_eq!(report.information, 0);
        assert_eq!(report.registry_coverage, 0.0);
        assert!(report.details.is_empty());
    }

    #[test]
    fn test_validation_status_serialization() {
        let success = ValidationStatus::Success;
        let json = serde_json::to_string(&success).unwrap();
        assert_eq!(json, "\"success\"");

        let failure = ValidationStatus::Failure;
        let json = serde_json::to_string(&failure).unwrap();
        assert_eq!(json, "\"failure\"");
    }

    #[test]
    fn test_weaver_controller_creation() {
        let config = WeaverConfig::default();
        let controller = WeaverController::new(config);
        assert!(controller.live_check_process.is_none());
        assert!(!controller.has_violations.load(Ordering::Relaxed));
    }

    #[test]
    fn test_validation_passing_initial_state() {
        let config = WeaverConfig::default();
        let controller = WeaverController::new(config);
        assert!(controller.is_validation_passing());
    }

    // Integration tests are in tests/weaver/ directory
    // See tests/weaver/otel_integration_tests.rs for complete integration test suite

    #[test]
    #[ignore = "Requires Weaver installation"]
    fn test_weaver_controller_lifecycle() {
        // This test requires Weaver to be installed and a registry to exist
        // For comprehensive integration tests, see:
        // - tests/weaver/otel_integration_tests.rs (24 integration tests)
        // - tests/weaver/phase2_coordination/ (coordination pattern tests)
        // - tests/weaver/phase3_otel_integration/ (OTEL contract tests)
        //
        // These integration tests verify end-to-end Weaver coordination,
        // telemetry export, and schema validation.
    }
}
