// Type-safe state machine orchestrator for Weaver live-check lifecycle
//
// This orchestrator provides compile-time guarantees about correct usage:
// - Cannot stop before starting
// - Cannot access OTLP port before starting
// - Cannot access report before stopping
//
// The state machine ensures that operations are only available in the correct states,
// preventing runtime errors and making the API self-documenting.

use std::marker::PhantomData;
use std::path::PathBuf;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use crate::error::{CleanroomError, Result};
use crate::telemetry::live_check::WeaverProcessManager;

// ============================================================================
// Configuration Types
// ============================================================================

/// Configuration for live-check orchestration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveCheckConfig {
    /// Enable live-check validation
    pub enabled: bool,
    /// Path to Weaver registry
    pub registry_path: PathBuf,
    /// OTLP gRPC port (None = auto-discover)
    pub otlp_port: Option<u16>,
    /// Admin HTTP port (None = auto-discover)
    pub admin_port: Option<u16>,
    /// Output directory for validation reports
    pub output_dir: PathBuf,
    /// Enable streaming output
    pub stream: bool,
    /// Fail fast on first violation
    pub fail_fast: bool,
}

impl Default for LiveCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            registry_path: PathBuf::from("registry"),
            otlp_port: None,
            admin_port: None,
            output_dir: std::env::temp_dir().join("weaver-validation"),
            stream: false,
            fail_fast: false,
        }
    }
}

impl LiveCheckConfig {
    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Port validation
        if let Some(port) = self.otlp_port {
            if port < 1024 {
                return Err(CleanroomError::config_error(
                    "OTLP port must be >= 1024 or None for auto-discovery",
                ));
            }
        }

        if let Some(port) = self.admin_port {
            if port < 1024 {
                return Err(CleanroomError::config_error(
                    "Admin port must be >= 1024 or None for auto-discovery",
                ));
            }
        }

        if self.otlp_port.is_some()
            && self.admin_port.is_some()
            && self.otlp_port == self.admin_port
        {
            return Err(CleanroomError::config_error(
                "OTLP and admin ports must be different",
            ));
        }

        // Registry path validation
        if self.registry_path.as_os_str().is_empty() {
            return Err(CleanroomError::config_error(
                "Registry path cannot be empty",
            ));
        }

        Ok(())
    }
}

// ============================================================================
// Validation Report Types
// ============================================================================

/// Validation status from Weaver
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationStatus {
    /// Validation passed (no violations)
    Success,
    /// Validation failed (violations detected)
    Failure,
}

/// Validation report from Weaver live-check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Overall status
    pub status: ValidationStatus,
    /// Number of violations
    pub violations: u32,
    /// Number of improvements suggested
    pub improvements: u32,
    /// Number of informational messages
    pub information: u32,
    /// Registry coverage (0.0 - 1.0)
    pub registry_coverage: f64,
    /// Number of telemetry samples received
    pub sample_count: u32,
    /// Detailed violation/improvement messages
    pub details: Vec<ValidationDetail>,
}

/// Detail entry in validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationDetail {
    /// Level: "violation", "improvement", or "information"
    pub level: String,
    /// Metric or span name
    pub metric_name: Option<String>,
    /// Span name (if applicable)
    pub span_name: Option<String>,
    /// Human-readable message
    pub message: String,
    /// Registry file path
    pub registry_path: Option<String>,
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self {
            status: ValidationStatus::Failure,
            violations: 0,
            improvements: 0,
            information: 0,
            registry_coverage: 0.0,
            sample_count: 0,
            details: Vec::new(),
        }
    }
}

// ============================================================================
// State Marker Types
// ============================================================================

/// State marker: Orchestrator created but Weaver not started
pub struct Uninitialized;

/// State marker: Weaver process running and coordinated
pub struct WeaverRunning {
    otlp_port: u16,
    admin_port: u16,
    start_time: Instant,
}

/// State marker: Weaver stopped, conformance report available
pub struct Completed {
    report: ValidationReport,
    runtime_duration_ms: u64,
}

// ============================================================================
// Core Orchestrator
// ============================================================================

/// Type-safe orchestrator for Weaver live-check lifecycle
///
/// This orchestrator uses compile-time state enforcement to prevent incorrect usage.
/// The type parameter `State` ensures that only valid operations are available
/// in each state of the lifecycle.
///
/// # State Transitions
///
/// ```text
/// Uninitialized --start_weaver()--> WeaverRunning --stop_weaver()--> Completed
/// ```
///
/// # Example
///
/// ```no_run
/// use clnrm_core::telemetry::live_check::{LiveCheckOrchestrator, LiveCheckConfig};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = LiveCheckConfig::default();
///
/// // Create orchestrator (Uninitialized state)
/// let orchestrator = LiveCheckOrchestrator::new(config)?;
///
/// // Start Weaver (transition to WeaverRunning state)
/// let running = orchestrator.start_weaver().await?;
///
/// // Can now access OTLP port
/// let port = running.otlp_port();
///
/// // Configure OTEL to send to this port
/// // ... run tests ...
///
/// // Stop Weaver and get report (transition to Completed state)
/// let completed = running.stop_weaver().await?;
///
/// // Check if validation passed
/// if !completed.passed() {
///     eprintln!("Validation failed: {}", completed.summary());
/// }
/// # Ok(())
/// # }
/// ```
pub struct LiveCheckOrchestrator<State = Uninitialized> {
    state: PhantomData<State>,
    // Wrap in Option to allow moving during state transitions
    weaver_manager: Option<WeaverProcessManager>,
    config: Option<LiveCheckConfig>,

    // State-specific data stored in type-erased form
    // (accessed via state marker methods)
    running_state: Option<WeaverRunning>,
    completed_state: Option<Completed>,
}

// ============================================================================
// Uninitialized State
// ============================================================================

impl LiveCheckOrchestrator<Uninitialized> {
    /// Create new orchestrator in uninitialized state
    ///
    /// Validates configuration and creates the Weaver process manager.
    /// Does not start Weaver - call `start_weaver()` to transition to running state.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Configuration validation fails (invalid ports, missing registry, etc.)
    /// - Weaver binary not found in PATH
    /// - Registry path does not exist
    pub fn new(config: LiveCheckConfig) -> Result<Self> {
        // Validate configuration before proceeding
        config.validate()?;

        debug!("Creating LiveCheckOrchestrator with config: {:?}", config);

        // Create process manager (does not start process yet)
        let weaver_manager = WeaverProcessManager::new(
            config.registry_path.clone(),
            30, // 30 second inactivity timeout
            config.output_dir.clone(),
        )?;

        Ok(Self {
            state: PhantomData,
            weaver_manager: Some(weaver_manager),
            config: Some(config),
            running_state: None,
            completed_state: None,
        })
    }

    /// Start Weaver and transition to WeaverRunning state
    ///
    /// This method:
    /// 1. Discovers available ports
    /// 2. Spawns the `weaver registry live-check` process
    /// 3. Waits for readiness
    /// 4. Returns orchestrator in WeaverRunning state
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - No available ports in discovery ranges
    /// - Process spawn fails (Weaver not installed, permissions, etc.)
    /// - Readiness timeout (process exits prematurely)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use clnrm_core::telemetry::live_check::{LiveCheckOrchestrator, LiveCheckConfig};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let orchestrator = LiveCheckOrchestrator::new(LiveCheckConfig::default())?;
    /// let running = orchestrator.start_weaver().await?;
    /// println!("Weaver listening on OTLP port: {}", running.otlp_port());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start_weaver(mut self) -> Result<LiveCheckOrchestrator<WeaverRunning>> {
        info!("Starting Weaver live-check process");

        // Start the Weaver process and get ports
        let weaver_manager = self
            .weaver_manager
            .as_mut()
            .expect("weaver_manager must be Some in Uninitialized state");
        let ports = weaver_manager.start().await?;

        info!(
            "Weaver started: OTLP gRPC port {}, Admin port {}",
            ports.otlp_grpc, ports.admin_http
        );

        // Create running state data
        let running_state = WeaverRunning {
            otlp_port: ports.otlp_grpc,
            admin_port: ports.admin_http,
            start_time: Instant::now(),
        };

        Ok(LiveCheckOrchestrator {
            state: PhantomData,
            weaver_manager: self.weaver_manager.take(),
            config: self.config.take(),
            running_state: Some(running_state),
            completed_state: None,
        })
    }

    /// Try to start Weaver, returning fallback mode on failure
    ///
    /// This provides graceful degradation: if Weaver cannot start, the caller
    /// can fall back to static registry checking instead of failing completely.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use clnrm_core::telemetry::live_check::{LiveCheckOrchestrator, LiveCheckConfig, OrchestrationMode};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let orchestrator = LiveCheckOrchestrator::new(LiveCheckConfig::default())?;
    ///
    /// match orchestrator.start_with_fallback().await? {
    ///     OrchestrationMode::LiveCheck(running) => {
    ///         println!("Live-check enabled");
    ///     }
    ///     OrchestrationMode::RegistryCheckOnly { reason, .. } => {
    ///         println!("Live-check unavailable ({}), using registry check", reason);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start_with_fallback(self) -> Result<OrchestrationMode> {
        let registry_path = self
            .config
            .as_ref()
            .expect("config must be Some in Uninitialized state")
            .registry_path
            .clone();

        match self.start_weaver().await {
            Ok(running) => Ok(OrchestrationMode::LiveCheck(running)),
            Err(e) => {
                warn!("Live-check failed to start: {}", e);
                warn!("Falling back to registry check only");

                Ok(OrchestrationMode::RegistryCheckOnly {
                    registry_path,
                    reason: format!("{}", e),
                })
            }
        }
    }
}

// ============================================================================
// WeaverRunning State
// ============================================================================

impl LiveCheckOrchestrator<WeaverRunning> {
    /// Get OTLP gRPC port for OTEL SDK configuration
    ///
    /// This port should be used as the endpoint for OTLP exporters:
    /// ```text
    /// http://127.0.0.1:{port}
    /// ```
    pub fn otlp_port(&self) -> u16 {
        self.running_state
            .as_ref()
            .expect("running_state must be Some in WeaverRunning state")
            .otlp_port
    }

    /// Get admin port for health checks and control
    pub fn admin_port(&self) -> u16 {
        self.running_state
            .as_ref()
            .expect("running_state must be Some in WeaverRunning state")
            .admin_port
    }

    /// Get formatted OTLP endpoint URL for OTEL SDK
    ///
    /// Returns a complete URL suitable for OTLP gRPC exporter configuration.
    pub fn otlp_endpoint(&self) -> String {
        format!("http://127.0.0.1:{}", self.otlp_port())
    }

    /// Get elapsed time since Weaver started
    pub fn uptime(&self) -> std::time::Duration {
        self.running_state
            .as_ref()
            .expect("running_state must be Some in WeaverRunning state")
            .start_time
            .elapsed()
    }

    /// Perform health check on Weaver process
    ///
    /// Checks if the Weaver process is still alive and responsive.
    ///
    /// # Returns
    ///
    /// - `Ok(true)` - Weaver is healthy
    /// - `Ok(false)` - Weaver is unhealthy (process died)
    /// - `Err(...)` - Health check failed (cannot determine status)
    pub async fn health_check(&self) -> Result<bool> {
        self.weaver_manager
            .as_ref()
            .expect("weaver_manager must be Some in WeaverRunning state")
            .health_check()
            .await
    }

    /// Get process ID of running Weaver
    pub fn pid(&self) -> Option<u32> {
        self.weaver_manager
            .as_ref()
            .expect("weaver_manager must be Some in WeaverRunning state")
            .pid()
    }

    /// Stop Weaver and collect conformance report
    ///
    /// This method:
    /// 1. Sends SIGHUP to Weaver (graceful shutdown)
    /// 2. Waits for process to exit (with timeout)
    /// 3. Parses the conformance report from disk
    /// 4. Returns orchestrator in Completed state
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Graceful shutdown times out (Weaver killed forcefully)
    /// - Conformance report missing or malformed
    /// - Report parsing fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use clnrm_core::telemetry::live_check::{LiveCheckOrchestrator, LiveCheckConfig};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let running = LiveCheckOrchestrator::new(LiveCheckConfig::default())?
    ///     .start_weaver().await?;
    ///
    /// // ... run tests that emit telemetry ...
    ///
    /// let completed = running.stop_weaver().await?;
    ///
    /// if completed.passed() {
    ///     println!("Validation passed!");
    /// } else {
    ///     eprintln!("Validation failed:\n{}", completed.summary());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn stop_weaver(mut self) -> Result<LiveCheckOrchestrator<Completed>> {
        let start_time = self
            .running_state
            .as_ref()
            .expect("running_state must be Some in WeaverRunning state")
            .start_time;

        info!("Stopping Weaver and collecting conformance report");

        // Stop the Weaver process (graceful shutdown)
        let weaver_manager = self
            .weaver_manager
            .as_mut()
            .expect("weaver_manager must be Some in WeaverRunning state");
        weaver_manager.stop().await?;

        // Collect report as string and parse it
        let report_json = weaver_manager.collect_report()?;
        let report = Self::parse_report(&report_json)?;

        let runtime_duration_ms = start_time.elapsed().as_millis() as u64;

        info!(
            "Conformance report collected: {} violations, {} samples ({}ms runtime)",
            report.violations, report.sample_count, runtime_duration_ms
        );

        // CRITICAL: Check for zero samples (no validation occurred)
        if report.sample_count == 0 {
            error!("CRITICAL: Weaver received ZERO telemetry samples!");
            error!("This means OTLP export failed or was misconfigured.");
        }

        let completed_state = Completed {
            report,
            runtime_duration_ms,
        };

        Ok(LiveCheckOrchestrator {
            state: PhantomData,
            weaver_manager: self.weaver_manager.take(),
            config: self.config.take(),
            running_state: None,
            completed_state: Some(completed_state),
        })
    }

    /// Parse report JSON string to ValidationReport
    fn parse_report(json: &str) -> Result<ValidationReport> {
        serde_json::from_str(json).map_err(|e| {
            CleanroomError::internal_error(format!("Failed to parse validation report: {}", e))
        })
    }

    /// Get reference to config (for StopCoordinator)
    pub fn config(&self) -> &LiveCheckConfig {
        self.config
            .as_ref()
            .expect("config must be Some in WeaverRunning state")
    }

    /// Stop Weaver gracefully (for StopCoordinator Phase 2)
    ///
    /// This sends SIGHUP to Weaver and waits for it to exit gracefully.
    /// Returns Ok(()) if Weaver exits within timeout, Err otherwise.
    pub async fn stop_weaver_gracefully(&mut self) -> Result<()> {
        let weaver_manager = self
            .weaver_manager
            .as_mut()
            .expect("weaver_manager must be Some in WeaverRunning state");

        weaver_manager.stop().await
    }

    /// Force kill Weaver (for StopCoordinator Phase 2 timeout)
    ///
    /// This sends SIGKILL to forcefully terminate Weaver.
    pub fn force_kill_weaver(&mut self) -> Result<()> {
        let weaver_manager = self
            .weaver_manager
            .as_mut()
            .expect("weaver_manager must be Some in WeaverRunning state");

        weaver_manager.force_kill()
    }
}

// ============================================================================
// Completed State
// ============================================================================

impl LiveCheckOrchestrator<Completed> {
    /// Get reference to conformance report
    ///
    /// Only available in Completed state (after stopping Weaver).
    pub fn report(&self) -> &ValidationReport {
        &self
            .completed_state
            .as_ref()
            .expect("completed_state must be Some in Completed state")
            .report
    }

    /// Get runtime duration in milliseconds
    ///
    /// Returns the time elapsed between starting and stopping Weaver.
    pub fn runtime_duration_ms(&self) -> u64 {
        self.completed_state
            .as_ref()
            .expect("completed_state must be Some in Completed state")
            .runtime_duration_ms
    }

    /// Consume orchestrator and return conformance report
    pub fn into_report(mut self) -> ValidationReport {
        self.completed_state
            .take()
            .expect("completed_state must be Some in Completed state")
            .report
    }

    /// Check if validation passed
    ///
    /// Returns `true` if and only if:
    /// - Status is Success
    /// - Zero violations
    /// - At least one sample received (telemetry was actually validated)
    pub fn passed(&self) -> bool {
        let report = self.report();
        report.status == ValidationStatus::Success
            && report.violations == 0
            && report.sample_count > 0
    }

    /// Determine exit code for CLI
    ///
    /// Returns:
    /// - `0` - Validation passed
    /// - `1` - Validation failed (violations or zero samples)
    pub fn exit_code(&self) -> i32 {
        if self.passed() {
            0
        } else {
            1
        }
    }

    /// Generate human-readable summary
    ///
    /// Returns a multi-line string summarizing the validation results,
    /// including violations, improvements, coverage, etc.
    pub fn summary(&self) -> String {
        let report = self.report();
        let runtime = self.runtime_duration_ms();

        let mut summary = String::new();

        summary.push_str("╔═══════════════════════════════════════════════════════════════╗\n");
        summary.push_str("║          Weaver Live-Check Validation Report                 ║\n");
        summary.push_str("╚═══════════════════════════════════════════════════════════════╝\n\n");

        summary.push_str(&format!("Status:            {:?}\n", report.status));
        summary.push_str(&format!("Runtime:           {}ms\n", runtime));
        summary.push_str(&format!("Samples Received:  {}\n", report.sample_count));
        summary.push_str(&format!("Violations:        {}\n", report.violations));
        summary.push_str(&format!("Improvements:      {}\n", report.improvements));
        summary.push_str(&format!("Information:       {}\n", report.information));
        summary.push_str(&format!(
            "Registry Coverage: {:.1}%\n\n",
            report.registry_coverage * 100.0
        ));

        if report.sample_count == 0 {
            summary.push_str("⚠️  ERROR: No telemetry samples received!\n\n");
            summary.push_str("Possible causes:\n");
            summary.push_str("  • OTLP exporter not configured\n");
            summary.push_str("  • OTLP endpoint incorrect\n");
            summary.push_str("  • Network connectivity issues\n");
            summary.push_str("  • Telemetry initialization failed\n\n");
        }

        if report.violations > 0 {
            summary.push_str("❌ VIOLATIONS:\n");
            for (i, detail) in report
                .details
                .iter()
                .filter(|d| d.level == "violation")
                .enumerate()
            {
                summary.push_str(&format!(
                    "  {}. {}: {}\n",
                    i + 1,
                    detail
                        .metric_name
                        .as_deref()
                        .or(detail.span_name.as_deref())
                        .unwrap_or("unknown"),
                    detail.message
                ));
            }
            summary.push('\n');
        }

        if report.improvements > 0 {
            summary.push_str(&format!(
                "💡 {} improvements suggested\n",
                report.improvements
            ));
        }

        if self.passed() {
            summary.push_str("\n✅ Validation PASSED\n");
        } else {
            summary.push_str("\n❌ Validation FAILED\n");
        }

        summary
    }
}

// ============================================================================
// Drop Implementation (Cleanup)
// ============================================================================

impl<State> Drop for LiveCheckOrchestrator<State> {
    fn drop(&mut self) {
        // WeaverProcessManager handles cleanup in its own Drop
        debug!("LiveCheckOrchestrator dropped");

        // Warn if dropped in WeaverRunning state without explicit stop
        if self.running_state.is_some() {
            warn!("LiveCheckOrchestrator dropped in Running state!");
            warn!("Call stop_weaver() explicitly for graceful shutdown.");
        }
    }
}

// ============================================================================
// RAII Guard Pattern
// ============================================================================

/// RAII guard for automatic Weaver cleanup
///
/// This guard ensures Weaver is stopped even if the orchestrator is dropped
/// due to panic or early return. Use this when you want automatic cleanup
/// but need to handle errors manually.
///
/// # Example
///
/// ```no_run
/// # use clnrm_core::telemetry::live_check::{LiveCheckOrchestrator, LiveCheckConfig, LiveCheckGuard};
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let orchestrator = LiveCheckOrchestrator::new(LiveCheckConfig::default())?
///     .start_weaver().await?;
///
/// let guard = LiveCheckGuard::new(orchestrator);
///
/// // ... run tests (even if panic occurs, cleanup happens) ...
///
/// // Explicitly take orchestrator to get report
/// let orchestrator = guard.take_orchestrator();
/// let completed = orchestrator.stop_weaver().await?;
/// # Ok(())
/// # }
/// ```
pub struct LiveCheckGuard {
    orchestrator: Option<LiveCheckOrchestrator<WeaverRunning>>,
}

impl LiveCheckGuard {
    /// Create new guard wrapping orchestrator
    pub fn new(orchestrator: LiveCheckOrchestrator<WeaverRunning>) -> Self {
        Self {
            orchestrator: Some(orchestrator),
        }
    }

    /// Get reference to orchestrator
    ///
    /// # Panics
    ///
    /// Panics if orchestrator has been taken via `take_orchestrator()`.
    pub fn orchestrator(&self) -> &LiveCheckOrchestrator<WeaverRunning> {
        self.orchestrator
            .as_ref()
            .expect("orchestrator already taken")
    }

    /// Take orchestrator to transition states
    ///
    /// After calling this, the guard no longer has ownership and will not
    /// perform cleanup on drop.
    ///
    /// # Panics
    ///
    /// Panics if orchestrator has already been taken.
    pub fn take_orchestrator(mut self) -> LiveCheckOrchestrator<WeaverRunning> {
        self.orchestrator
            .take()
            .expect("orchestrator already taken")
    }
}

impl Drop for LiveCheckGuard {
    fn drop(&mut self) {
        if let Some(mut orchestrator) = self.orchestrator.take() {
            warn!("LiveCheckGuard dropped without explicit stop - forcing cleanup");

            // Force cleanup even on panic (best-effort)
            if let Some(weaver_manager) = orchestrator.weaver_manager.as_mut() {
                let _ = weaver_manager.force_kill();
            }
        }
    }
}

// ============================================================================
// Orchestration Mode (Fallback Support)
// ============================================================================

/// Orchestration mode: Live-check or fallback to registry check
///
/// This enum represents the result of `start_with_fallback()`, allowing
/// graceful degradation when Weaver cannot be started.
pub enum OrchestrationMode {
    /// Full live validation with running Weaver
    LiveCheck(LiveCheckOrchestrator<WeaverRunning>),

    /// Fallback: static registry check only (Weaver unavailable)
    RegistryCheckOnly {
        /// Path to registry for static validation
        registry_path: std::path::PathBuf,
        /// Reason why live-check failed
        reason: String,
    },
}

// ============================================================================
// Integration Helpers
// ============================================================================

/// Graceful degradation: Try live-check, fallback to registry check
///
/// This function encapsulates the complete graceful fallback logic:
/// 1. Try to start Weaver live-check
/// 2. If successful, run with live validation
/// 3. If failed, fall back to static registry check
/// 4. Return result indicating which mode was used
///
/// # Example
///
/// ```no_run
/// # use clnrm_core::telemetry::live_check::{run_with_graceful_fallback, LiveCheckConfig};
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = LiveCheckConfig::default();
///
/// let result = run_with_graceful_fallback(&config).await?;
///
/// println!("Validation mode: {:?}", result.mode);
/// println!("Passed: {}", result.passed);
/// # Ok(())
/// # }
/// ```
pub async fn run_with_graceful_fallback(
    config: &LiveCheckConfig,
) -> Result<GracefulFallbackResult> {
    // Try to create and start orchestrator
    let orchestrator = LiveCheckOrchestrator::new(config.clone())?;

    match orchestrator.start_with_fallback().await? {
        OrchestrationMode::LiveCheck(running) => {
            // Live-check available - use it
            info!("Live-check enabled, using Weaver validation");

            // For full integration, caller would run tests here
            // For now, just stop and return report
            let completed = running.stop_weaver().await?;

            Ok(GracefulFallbackResult {
                mode: FallbackMode::LiveCheck,
                passed: completed.passed(),
                report: Some(completed.into_report()),
            })
        }
        OrchestrationMode::RegistryCheckOnly {
            registry_path,
            reason,
        } => {
            // Live-check unavailable - use static registry check
            warn!("Live-check unavailable: {}", reason);
            info!("Falling back to static registry check");

            // Run static registry check
            let passed = run_registry_check(&registry_path)?;

            Ok(GracefulFallbackResult {
                mode: FallbackMode::RegistryCheckOnly { reason },
                passed,
                report: None,
            })
        }
    }
}

/// Result of graceful fallback validation
pub struct GracefulFallbackResult {
    /// Which validation mode was used
    pub mode: FallbackMode,
    /// Whether validation passed
    pub passed: bool,
    /// Full validation report (only available for live-check)
    pub report: Option<ValidationReport>,
}

/// Validation mode used
#[derive(Debug, Clone)]
pub enum FallbackMode {
    /// Full live-check validation
    LiveCheck,
    /// Static registry check (fallback)
    RegistryCheckOnly { reason: String },
}

/// Run static registry check (fallback mode)
///
/// Executes: `weaver registry check -r <path>`
fn run_registry_check(registry_path: &std::path::Path) -> Result<bool> {
    use std::process::Command;

    info!("Running static registry check: {}", registry_path.display());

    let output = Command::new("weaver")
        .args([
            "registry",
            "check",
            "-r",
            &registry_path.display().to_string(),
        ])
        .output()
        .map_err(|e| {
            CleanroomError::internal_error(format!(
                "Failed to run registry check (is Weaver installed?): {}",
                e
            ))
        })?;

    if !output.status.success() {
        error!(
            "Registry check failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Ok(false);
    }

    info!("Registry check passed");
    Ok(true)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrator_creation() {
        let config = LiveCheckConfig::default();
        let orchestrator = LiveCheckOrchestrator::new(config);

        // Should succeed with default config
        assert!(orchestrator.is_ok());
    }

    #[test]
    fn test_state_types_are_distinct() {
        // These should all be different types (no runtime test, just type checking)
        fn _check_uninitialized(_: LiveCheckOrchestrator<Uninitialized>) {}
        fn _check_running(_: LiveCheckOrchestrator<WeaverRunning>) {}
        fn _check_completed(_: LiveCheckOrchestrator<Completed>) {}

        // If this compiles, types are distinct ✓
    }
}
