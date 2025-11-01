/// Weaver Process Manager
///
/// Manages the lifecycle of Weaver `registry live-check` processes.
/// This includes:
/// - Binary detection and startup
/// - Port discovery (OTLP gRPC + Admin HTTP)
/// - Health check validation
/// - Graceful shutdown (SIGHUP signal)
/// - Conformance report collection
/// - Zombie process prevention
/// - RAII cleanup (Drop trait)
use crate::error::{CleanroomError, Result};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use std::{env, fs, thread};
use tracing::{debug, info, warn};

#[cfg(unix)]
use nix::sys::signal::{kill, Signal};
#[cfg(unix)]
use nix::unistd::Pid;

/// Discovered Weaver ports
#[derive(Debug, Clone, Copy)]
pub struct WeaverPorts {
    /// OTLP gRPC listener port
    pub otlp_grpc: u16,
    /// Admin HTTP server port
    pub admin_http: u16,
}

/// Weaver process manager
pub struct WeaverProcessManager {
    /// Weaver child process
    process: Option<Child>,
    /// Discovered OTLP gRPC port
    otlp_port: Option<u16>,
    /// Discovered admin HTTP port
    admin_port: Option<u16>,
    /// Path to Weaver registry
    registry_path: PathBuf,
    /// Inactivity timeout in seconds
    inactivity_timeout: u64,
    /// Output directory for validation reports
    output_dir: PathBuf,
    /// Process startup timestamp
    started_at: Option<Instant>,
}

impl WeaverProcessManager {
    /// Create a new Weaver process manager
    ///
    /// # Arguments
    /// * `registry_path` - Path to Weaver registry directory
    /// * `inactivity_timeout` - Seconds before Weaver auto-stops
    /// * `output_dir` - Directory for validation reports
    pub fn new(
        registry_path: PathBuf,
        inactivity_timeout: u64,
        output_dir: PathBuf,
    ) -> Result<Self> {
        Ok(Self {
            process: None,
            otlp_port: None,
            admin_port: None,
            registry_path,
            inactivity_timeout,
            output_dir,
            started_at: None,
        })
    }

    /// Start Weaver process and discover ports
    ///
    /// This method:
    /// 1. Finds Weaver binary
    /// 2. Discovers available ports
    /// 3. Spawns Weaver process
    /// 4. Waits for health check to pass
    /// 5. Returns discovered ports
    ///
    /// # Returns
    /// `WeaverPorts` with OTLP and admin ports
    ///
    /// # Errors
    /// - Weaver binary not found
    /// - Port discovery failed
    /// - Process spawn failed
    /// - Health check timeout
    pub async fn start(&mut self) -> Result<WeaverPorts> {
        info!("🚀 Starting Weaver process manager");

        // Cleanup any orphaned processes
        Self::cleanup_orphaned_processes()?;

        // Find Weaver binary
        let weaver_binary = Self::find_weaver_binary()?;
        info!("📍 Found Weaver binary at: {:?}", weaver_binary);

        // Discover available ports
        let otlp_port = Self::find_available_otlp_port()?;
        let admin_port = Self::find_available_admin_port()?;
        info!(
            "📡 Discovered ports: OTLP={}, Admin={}",
            otlp_port, admin_port
        );

        // Create output directory
        fs::create_dir_all(&self.output_dir).map_err(|e| {
            CleanroomError::internal_error(format!("Failed to create output directory: {}", e))
        })?;

        // Build command
        let mut cmd = Command::new(&weaver_binary);
        cmd.args([
            "registry",
            "live-check",
            "--registry",
            &self.registry_path.display().to_string(),
            "--otlp-grpc-port",
            &otlp_port.to_string(),
            "--admin-port",
            &admin_port.to_string(),
            "--format",
            "json",
            "--output",
            &self.output_dir.display().to_string(),
            "--inactivity-timeout",
            &format!("{}s", self.inactivity_timeout),
            "--no-stream", // Batch mode for cleaner logs
            "--future",    // Latest validation rules
        ]);

        // Spawn process with stdout/stderr piped
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        debug!("🔧 Spawning Weaver: {:?}", cmd);

        let child = cmd.spawn().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to spawn Weaver process: {}", e))
        })?;

        let pid = child.id();
        info!("✅ Weaver process started (PID: {})", pid);

        // Store process and ports
        self.process = Some(child);
        self.otlp_port = Some(otlp_port);
        self.admin_port = Some(admin_port);
        self.started_at = Some(Instant::now());

        // Wait for health check
        self.health_check_with_timeout(Duration::from_secs(30))
            .await?;

        info!(
            "🎉 Weaver is ready (startup took {}ms)",
            self.started_at.unwrap().elapsed().as_millis()
        );

        Ok(WeaverPorts {
            otlp_grpc: otlp_port,
            admin_http: admin_port,
        })
    }

    /// Check Weaver health via HTTP admin endpoint
    ///
    /// Performs exponential backoff polling of the admin endpoint.
    ///
    /// # Returns
    /// `Ok(true)` if health check passes
    ///
    /// # Errors
    /// - Health check timeout
    /// - Process crashed before ready
    pub async fn health_check(&self) -> Result<bool> {
        let admin_port = self.admin_port.ok_or_else(|| {
            CleanroomError::internal_error("Admin port not set (Weaver not started)")
        })?;

        Self::perform_health_check(admin_port).await
    }

    /// Perform health check with timeout and exponential backoff
    async fn health_check_with_timeout(&mut self, max_timeout: Duration) -> Result<()> {
        const INITIAL_DELAY: Duration = Duration::from_millis(100);
        const MAX_DELAY: Duration = Duration::from_millis(1000);

        let start = Instant::now();
        let mut delay = INITIAL_DELAY;

        let admin_port = self
            .admin_port
            .ok_or_else(|| CleanroomError::internal_error("Admin port not set"))?;

        loop {
            // Check timeout
            if start.elapsed() > max_timeout {
                return Err(CleanroomError::timeout_error(format!(
                    "Weaver health check timed out after {}s",
                    max_timeout.as_secs()
                )));
            }

            // Check if process crashed
            if let Some(ref mut process) = self.process {
                if let Ok(Some(status)) = process.try_wait() {
                    // Process exited - capture stderr
                    let stderr = self.read_stderr()?;
                    return Err(CleanroomError::internal_error(format!(
                        "Weaver crashed with status {}: {}",
                        status, stderr
                    )));
                }
            }

            // Attempt health check
            match Self::perform_health_check(admin_port).await {
                Ok(true) => {
                    info!(
                        "✅ Weaver health check passed ({}ms)",
                        start.elapsed().as_millis()
                    );
                    return Ok(());
                }
                Ok(false) => {
                    debug!("Health check returned false, retrying...");
                }
                Err(e) => {
                    debug!("Health check error: {}, retrying...", e);
                }
            }

            // Exponential backoff
            tokio::time::sleep(delay).await;
            delay = std::cmp::min(delay * 2, MAX_DELAY);
        }
    }

    /// Perform a single health check
    async fn perform_health_check(admin_port: u16) -> Result<bool> {
        let url = format!("http://localhost:{}/health", admin_port);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .map_err(|e| {
                CleanroomError::internal_error(format!("Failed to create HTTP client: {}", e))
            })?;

        match client.get(&url).send().await {
            Ok(response) if response.status().is_success() => Ok(true),
            Ok(response) => {
                debug!("Health check returned status: {}", response.status());
                Ok(false)
            }
            Err(e) => {
                debug!("Health check request failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Stop Weaver gracefully with SIGHUP
    ///
    /// Sends SIGHUP signal (Unix) or kills process (Windows),
    /// then waits for process to exit with timeout.
    ///
    /// # Returns
    /// Ok(()) if process stopped successfully
    ///
    /// # Errors
    /// - Process not running
    /// - Signal delivery failed
    /// - Shutdown timeout
    pub async fn stop(&mut self) -> Result<()> {
        info!("🛑 Stopping Weaver process");

        let mut process = self
            .process
            .take()
            .ok_or_else(|| CleanroomError::internal_error("Weaver process not running"))?;

        // Send graceful shutdown signal
        #[cfg(unix)]
        {
            let pid = Pid::from_raw(process.id() as i32);
            match kill(pid, Signal::SIGHUP) {
                Ok(()) => {
                    debug!("✅ Sent SIGHUP to Weaver (PID: {})", process.id());
                }
                Err(nix::errno::Errno::ESRCH) => {
                    // Process already exited
                    debug!("Process already exited");
                    return Ok(());
                }
                Err(e) => {
                    return Err(CleanroomError::internal_error(format!(
                        "Failed to send SIGHUP: {}",
                        e
                    )));
                }
            }
        }

        #[cfg(not(unix))]
        {
            // Windows: kill directly
            process.kill().map_err(|e| {
                CleanroomError::internal_error(format!("Failed to kill Weaver: {}", e))
            })?;
        }

        // Wait for process to exit with timeout
        self.wait_with_timeout(&mut process, Duration::from_secs(10))
            .await?;

        info!("✅ Weaver stopped successfully");
        Ok(())
    }

    /// Wait for process to exit with timeout
    async fn wait_with_timeout(&self, process: &mut Child, timeout: Duration) -> Result<()> {
        let start = Instant::now();
        let check_interval = Duration::from_millis(100);

        loop {
            match process.try_wait() {
                Ok(Some(status)) => {
                    debug!("Weaver exited with status: {}", status);
                    return Ok(());
                }
                Ok(None) => {
                    // Still running
                    if start.elapsed() > timeout {
                        warn!("Weaver shutdown timed out, force killing");
                        return Self::force_kill_process(process);
                    }
                    tokio::time::sleep(check_interval).await;
                }
                Err(e) => {
                    return Err(CleanroomError::internal_error(format!(
                        "Failed to wait for Weaver: {}",
                        e
                    )));
                }
            }
        }
    }

    /// Collect conformance report from Weaver
    ///
    /// Reads the JSON report file generated by Weaver.
    ///
    /// # Returns
    /// Raw JSON report string
    ///
    /// # Errors
    /// - Report file not found
    /// - File read error
    pub fn collect_report(&self) -> Result<String> {
        let report_path = self.output_dir.join("live_check.json");

        if !report_path.exists() {
            warn!("Validation report not found at {:?}", report_path);
            return Err(CleanroomError::validation_error(
                "Weaver report file not found",
            ));
        }

        fs::read_to_string(&report_path).map_err(|e| {
            CleanroomError::internal_error(format!("Failed to read validation report: {}", e))
        })
    }

    /// Force kill Weaver process
    ///
    /// Used as last resort if graceful shutdown fails.
    pub fn force_kill(&mut self) -> Result<()> {
        if let Some(mut process) = self.process.take() {
            Self::force_kill_process(&mut process)?;
        }
        Ok(())
    }

    /// Force kill a process (static method to avoid borrowing issues)
    fn force_kill_process(process: &mut Child) -> Result<()> {
        warn!("Force killing Weaver process (PID: {})", process.id());
        process.kill().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to force kill Weaver: {}", e))
        })?;
        let _ = process.wait(); // Reap zombie
        Ok(())
    }

    /// Read stderr from Weaver process
    fn read_stderr(&mut self) -> Result<String> {
        if let Some(ref mut process) = self.process {
            if let Some(ref mut stderr) = process.stderr {
                let mut stderr_content = String::new();
                let reader = BufReader::new(stderr);
                for line in reader.lines().flatten() {
                    stderr_content.push_str(&line);
                    stderr_content.push('\n');
                }
                return Ok(stderr_content);
            }
        }
        Ok(String::new())
    }

    /// Find Weaver binary in PATH or standard locations
    ///
    /// Search order:
    /// 1. `weaver` in PATH
    /// 2. `~/.local/bin/weaver`
    /// 3. `./vendors/weaver/weaver` (project-local)
    ///
    /// # Returns
    /// Path to Weaver binary
    ///
    /// # Errors
    /// Binary not found in any location
    fn find_weaver_binary() -> Result<PathBuf> {
        // Check PATH
        if let Ok(path) = which::which("weaver") {
            return Ok(path);
        }

        // Check ~/.local/bin/weaver
        if let Ok(home) = env::var("HOME") {
            let local_weaver = PathBuf::from(home).join(".local/bin/weaver");
            if local_weaver.exists() {
                return Ok(local_weaver);
            }
        }

        // Check project-local vendors/weaver/weaver
        let project_weaver = PathBuf::from("vendors/weaver/weaver");
        if project_weaver.exists() {
            return Ok(project_weaver);
        }

        Err(CleanroomError::validation_error(
            "Weaver binary not found. Install with: cargo install weaver",
        ))
    }

    /// Find available OTLP gRPC port with multi-tier fallback
    ///
    /// Tries 3 tiers:
    /// - Tier 1: 4317-4327 (standard OTLP range)
    /// - Tier 2: 5317-5327 (fallback)
    /// - Tier 3: 6317-6337 (extended)
    ///
    /// Total capacity: 40 concurrent processes
    fn find_available_otlp_port() -> Result<u16> {
        // Tier 1: Standard OTLP range
        if let Ok(port) = Self::try_port_range(4317, 4327) {
            return Ok(port);
        }

        // Tier 2: Fallback range
        warn!("Primary OTLP range exhausted, trying fallback");
        if let Ok(port) = Self::try_port_range(5317, 5327) {
            return Ok(port);
        }

        // Tier 3: Extended range
        warn!("Secondary OTLP range exhausted, trying extended");
        if let Ok(port) = Self::try_port_range(6317, 6337) {
            return Ok(port);
        }

        Err(CleanroomError::validation_error(
            "All OTLP port ranges exhausted (40 ports). Reduce parallelism.",
        ))
    }

    /// Find available admin HTTP port with multi-tier fallback
    ///
    /// Tries 3 tiers:
    /// - Tier 1: 8080-8089
    /// - Tier 2: 9080-9089
    /// - Tier 3: 10080-10099
    fn find_available_admin_port() -> Result<u16> {
        // Tier 1: Standard admin range
        if let Ok(port) = Self::try_port_range(8080, 8089) {
            return Ok(port);
        }

        // Tier 2: Fallback range
        warn!("Primary admin range exhausted, trying fallback");
        if let Ok(port) = Self::try_port_range(9080, 9089) {
            return Ok(port);
        }

        // Tier 3: Extended range
        warn!("Secondary admin range exhausted, trying extended");
        if let Ok(port) = Self::try_port_range(10080, 10099) {
            return Ok(port);
        }

        Err(CleanroomError::validation_error(
            "All admin port ranges exhausted (40 ports). Reduce parallelism.",
        ))
    }

    /// Try to find available port in range
    fn try_port_range(start: u16, end: u16) -> Result<u16> {
        for port in start..=end {
            if Self::is_port_available(port) {
                return Ok(port);
            }
        }
        Err(CleanroomError::validation_error("Port range exhausted"))
    }

    /// Check if port is available by attempting to bind
    fn is_port_available(port: u16) -> bool {
        std::net::TcpListener::bind(("127.0.0.1", port)).is_ok()
    }

    /// Cleanup orphaned Weaver processes from previous runs
    fn cleanup_orphaned_processes() -> Result<()> {
        #[cfg(unix)]
        {
            // Use pgrep to find weaver processes
            let output = Command::new("pgrep")
                .args(["-f", "weaver registry live-check"])
                .output();

            if let Ok(output) = output {
                if output.status.success() && !output.stdout.is_empty() {
                    let pids = String::from_utf8_lossy(&output.stdout);
                    warn!("Found orphaned Weaver processes, cleaning up: {}", pids);

                    // Kill each process
                    for pid_str in pids.lines() {
                        if let Ok(pid) = pid_str.trim().parse::<i32>() {
                            let _ = kill(Pid::from_raw(pid), Signal::SIGKILL);
                        }
                    }

                    // Wait for cleanup
                    thread::sleep(Duration::from_millis(500));
                }
            }
        }

        Ok(())
    }

    /// Get process PID if running
    pub fn pid(&self) -> Option<u32> {
        self.process.as_ref().map(|p| p.id())
    }

    /// Get discovered OTLP port
    pub fn otlp_port(&self) -> Option<u16> {
        self.otlp_port
    }

    /// Get discovered admin port
    pub fn admin_port(&self) -> Option<u16> {
        self.admin_port
    }

    /// Get startup duration
    pub fn uptime(&self) -> Option<Duration> {
        self.started_at.map(|start| start.elapsed())
    }
}

impl Drop for WeaverProcessManager {
    fn drop(&mut self) {
        // Ensure cleanup even on panic
        if self.process.is_some() {
            debug!("Cleaning up Weaver process in Drop");
            let _ = self.force_kill();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_availability_check() {
        // Bind a port to make it unavailable
        let _listener = std::net::TcpListener::bind(("127.0.0.1", 9999)).unwrap();

        // Should report as unavailable
        assert!(!WeaverProcessManager::is_port_available(9999));

        // Should report as available
        assert!(WeaverProcessManager::is_port_available(9998));
    }

    #[test]
    fn test_port_range_discovery() {
        // Should find an available port in a large range
        let result = WeaverProcessManager::try_port_range(50000, 50100);
        assert!(result.is_ok());
    }

    #[test]
    fn test_weaver_ports_structure() {
        let ports = WeaverPorts {
            otlp_grpc: 4317,
            admin_http: 8080,
        };

        assert_eq!(ports.otlp_grpc, 4317);
        assert_eq!(ports.admin_http, 8080);
    }

    #[test]
    fn test_manager_creation() {
        let manager = WeaverProcessManager::new(
            PathBuf::from("registry/"),
            120,
            PathBuf::from("validation_output/"),
        );

        assert!(manager.is_ok());
        let manager = manager.unwrap();
        assert_eq!(manager.otlp_port, None);
        assert_eq!(manager.admin_port, None);
        assert_eq!(manager.pid(), None);
    }

    #[test]
    fn test_output_directory_path() {
        let output_dir = PathBuf::from("validation_output");
        let report_path = output_dir.join("live_check.json");

        assert_eq!(
            report_path.to_string_lossy(),
            "validation_output/live_check.json"
        );
    }
}
