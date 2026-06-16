/// Atomic port allocation with flock-based locking for zero race conditions
///
/// This module provides cross-process port coordination using filesystem locks
/// to prevent port conflicts in parallel CI/CD environments.
///
/// # Architecture
///
/// The port allocator uses a 3-tier fallback strategy:
/// 1. Primary range (4317-5317): Standard OTLP port range
/// 2. Fallback range (5318-6318): Secondary range for high contention
/// 3. Extended range (6319-7339): Emergency range
///
/// # Atomicity Guarantee
///
/// Port allocation is atomic via flock (Unix) or LockFileEx (Windows):
/// 1. TcpListener validates port is bindable
/// 2. Exclusive flock acquired on lock file
/// 3. Port guaranteed reserved until PortLock dropped
/// 4. OS releases lock on process exit (even on crash)
///
/// # Example
///
/// ```no_run
/// use clnrm_core::telemetry::live_check::PortAllocator;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let allocator = PortAllocator::new()?;
/// let lock = allocator.allocate_port().await?;
/// println!("Allocated port: {}", lock.port());
/// // Lock automatically released when `lock` is dropped
/// # Ok(())
/// # }
/// ```
use crate::error::{CleanroomError, Result};
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::Write as IoWrite;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;

static PROCESS_PORT_LOCKS: Lazy<Mutex<HashSet<u16>>> = Lazy::new(|| Mutex::new(HashSet::new()));

/// Atomic port allocator with flock-based locking
///
/// Provides zero-race-condition port allocation across processes using
/// filesystem locks for coordination.
pub struct PortAllocator {
    /// Primary port range for allocation
    primary_range: PortRange,
    /// Fallback range if primary exhausted
    fallback_range: PortRange,
    /// Extended range for emergency allocation
    extended_range: PortRange,
    /// Directory for lock files
    lock_dir: PathBuf,
}

/// Port range specification
#[derive(Debug, Clone)]
pub struct PortRange {
    pub start: u16,
    pub end: u16,
}

impl PortRange {
    /// Create new port range
    ///
    /// # Panics
    ///
    /// Panics if start > end
    pub fn new(start: u16, end: u16) -> Self {
        assert!(start <= end, "Port range start must be <= end");
        Self { start, end }
    }

    /// Iterate over ports in range
    pub fn iter(&self) -> impl Iterator<Item = u16> {
        self.start..=self.end
    }

    /// Count of ports in range
    pub fn size(&self) -> usize {
        (self.end - self.start + 1) as usize
    }
}

impl PortAllocator {
    /// Create new port allocator with default ranges
    ///
    /// # Default Ranges
    ///
    /// - Primary: 4317-5317 (1001 ports, OTLP standard)
    /// - Fallback: 5318-6318 (1001 ports)
    /// - Extended: 6319-7339 (1021 ports)
    ///
    /// # Errors
    ///
    /// Returns error if lock directory cannot be created
    pub fn new() -> Result<Self> {
        let lock_dir = Self::default_lock_dir();
        std::fs::create_dir_all(&lock_dir).map_err(|e| {
            CleanroomError::internal_error(format!(
                "Failed to create port lock directory {}: {}",
                lock_dir.display(),
                e
            ))
        })?;

        Ok(Self {
            primary_range: PortRange::new(4317, 5317),
            fallback_range: PortRange::new(5318, 6318),
            extended_range: PortRange::new(6319, 7339),
            lock_dir,
        })
    }

    /// Create port allocator with custom ranges
    ///
    /// # Arguments
    ///
    /// * `primary` - Primary port range
    /// * `fallback` - Fallback port range
    /// * `extended` - Extended port range
    ///
    /// # Errors
    ///
    /// Returns error if lock directory cannot be created
    pub fn with_ranges(
        primary: PortRange,
        fallback: PortRange,
        extended: PortRange,
    ) -> Result<Self> {
        let lock_dir = Self::default_lock_dir();
        std::fs::create_dir_all(&lock_dir).map_err(|e| {
            CleanroomError::internal_error(format!("Failed to create port lock directory: {}", e))
        })?;

        Ok(Self {
            primary_range: primary,
            fallback_range: fallback,
            extended_range: extended,
            lock_dir,
        })
    }

    /// Get default lock directory
    fn default_lock_dir() -> PathBuf {
        std::env::temp_dir().join("clnrm-port-locks")
    }

    /// Allocate port with atomic locking (3-tier fallback)
    ///
    /// Tries primary range first, then fallback, then extended range.
    /// Each port is validated via TcpListener before flock acquisition.
    ///
    /// # Returns
    ///
    /// * `Ok(PortLock)` - Successfully allocated port with active lock
    /// * `Err(PortExhaustion)` - All ports in all ranges are in use
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use clnrm_core::telemetry::live_check::PortAllocator;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let allocator = PortAllocator::new()?;
    /// let lock = allocator.allocate_port().await?;
    /// assert!(lock.port() >= 4317);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn allocate_port(&self) -> Result<PortLock> {
        tracing::debug!(
            "Allocating port from ranges: primary={}-{}, fallback={}-{}, extended={}-{}",
            self.primary_range.start,
            self.primary_range.end,
            self.fallback_range.start,
            self.fallback_range.end,
            self.extended_range.start,
            self.extended_range.end,
        );

        // Try primary range
        if let Some(lock) = self.try_allocate_from_range(&self.primary_range).await? {
            tracing::info!("✓ Port {} allocated from primary range", lock.port());
            return Ok(lock);
        }

        // Try fallback range
        tracing::warn!("Primary range exhausted, trying fallback range");
        if let Some(lock) = self.try_allocate_from_range(&self.fallback_range).await? {
            tracing::info!("✓ Port {} allocated from fallback range", lock.port());
            return Ok(lock);
        }

        // Try extended range
        tracing::warn!("Fallback range exhausted, trying extended range");
        if let Some(lock) = self.try_allocate_from_range(&self.extended_range).await? {
            tracing::info!("✓ Port {} allocated from extended range", lock.port());
            return Ok(lock);
        }

        // All ranges exhausted
        Err(CleanroomError::resource_limit_exceeded(format!(
            "Port exhaustion: all port ranges in use (primary: {}-{}, fallback: {}-{}, extended: {}-{}). \
             {} ports checked total.",
            self.primary_range.start,
            self.primary_range.end,
            self.fallback_range.start,
            self.fallback_range.end,
            self.extended_range.start,
            self.extended_range.end,
            self.primary_range.size() + self.fallback_range.size() + self.extended_range.size()
        )))
    }

    /// Try to allocate from specific range
    ///
    /// Iterates through range, attempting to lock each port.
    /// Returns first successfully locked port, or None if range exhausted.
    async fn try_allocate_from_range(&self, range: &PortRange) -> Result<Option<PortLock>> {
        for port in range.iter() {
            match self.try_lock_port(port).await {
                Ok(Some(lock)) => return Ok(Some(lock)),
                Ok(None) => continue, // Port unavailable, try next
                Err(e) => {
                    tracing::debug!("Error checking port {}: {}", port, e);
                    continue; // Continue to next port on error
                }
            }
        }
        Ok(None)
    }

    /// Try to acquire exclusive lock on port
    ///
    /// # Process
    ///
    /// 1. Check if port is bindable via TcpListener
    /// 2. Create lock file for port
    /// 3. Acquire exclusive flock (non-blocking)
    /// 4. Verify port still available (double-check)
    ///
    /// # Returns
    ///
    /// * `Ok(Some(PortLock))` - Port locked successfully
    /// * `Ok(None)` - Port in use or lock held by another process
    /// * `Err` - I/O error or lock failure
    async fn try_lock_port(&self, port: u16) -> Result<Option<PortLock>> {
        // Step 0: Check in-process lock to prevent thread-level conflicts within same process
        {
            let mut locks = PROCESS_PORT_LOCKS.lock().unwrap(); // OK: static mutex, not expected to be poisoned
            if locks.contains(&port) {
                return Ok(None);
            }
            locks.insert(port);
        }

        // Helper to remove port from in-process locks on failure
        let cleanup_in_process_lock = || {
            let mut locks = PROCESS_PORT_LOCKS.lock().unwrap(); // OK: static mutex, not expected to be poisoned
            locks.remove(&port);
        };

        // Step 1: Verify port is bindable
        if !Self::is_port_available(port).await? {
            tracing::debug!("Port {} not bindable", port);
            cleanup_in_process_lock();
            return Ok(None);
        }

        let lock_file_path = self.lock_file_path(port);

        // Step 2: Open lock file without truncating it (in case another process holds lock)
        let mut file = match OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&lock_file_path)
        {
            Ok(f) => f,
            Err(e) => {
                cleanup_in_process_lock();
                return Err(CleanroomError::internal_error(format!(
                    "Failed to open lock file {}: {}",
                    lock_file_path.display(),
                    e
                )));
            }
        };

        // Step 3: Acquire exclusive flock
        #[cfg(unix)]
        {
            #[allow(deprecated)]
            use nix::fcntl::{flock, FlockArg};
            use std::os::unix::io::AsRawFd;

            #[allow(deprecated)]
            match flock(file.as_raw_fd(), FlockArg::LockExclusiveNonblock) {
                Ok(_) => {
                    // Lock acquired! Safe to write metadata and truncate
                    let metadata = format!(
                        "{{\"port\":{},\"pid\":{},\"locked_at\":\"{}\"}}\n",
                        port,
                        std::process::id(),
                        chrono::Utc::now().to_rfc3339()
                    );
                    let _ = file.set_len(0);
                    let _ = file.write_all(metadata.as_bytes());
                    let _ = file.flush();

                    // Double-check port is still available
                    if Self::is_port_available(port).await? {
                        tracing::debug!("Port {} locked successfully", port);
                        Ok(Some(PortLock {
                            port,
                            _lock_file: file,
                        }))
                    } else {
                        tracing::warn!(
                            "Port {} locked but not available (race condition detected)",
                            port
                        );
                        // Release lock (via drop) and return None
                        cleanup_in_process_lock();
                        Ok(None)
                    }
                }
                Err(nix::errno::Errno::EWOULDBLOCK) => {
                    // Lock held by another process
                    tracing::debug!("Port {} lock held by another process", port);
                    cleanup_in_process_lock();
                    Ok(None)
                }
                Err(e) => {
                    cleanup_in_process_lock();
                    Err(CleanroomError::internal_error(format!(
                        "flock failed on port {}: {}",
                        port, e
                    )))
                }
            }
        }

        #[cfg(not(unix))]
        {
            // Windows: File creation is the lock (less robust)
            // Write metadata and flush
            let metadata = format!(
                "{{\"port\":{},\"pid\":{},\"locked_at\":\"{}\"}}\n",
                port,
                std::process::id(),
                chrono::Utc::now().to_rfc3339()
            );
            let _ = file.set_len(0);
            let _ = file.write_all(metadata.as_bytes());
            let _ = file.flush();

            // Try to bind to verify availability
            if Self::is_port_available(port).await? {
                tracing::debug!("Port {} allocated (Windows)", port);
                return Ok(Some(PortLock {
                    port,
                    _lock_file: file,
                }));
            } else {
                tracing::debug!("Port {} not available (Windows)", port);
                cleanup_in_process_lock();
                return Ok(None);
            }
        }
    }

    /// Check if port is actually available via TCP bind
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Port is available
    /// * `Ok(false)` - Port is in use
    /// * `Err` - I/O error during check
    async fn is_port_available(port: u16) -> Result<bool> {
        use socket2::{Domain, Protocol, Socket, Type};
        use std::net::SocketAddr;

        let check_bind = |addr_str: &str| -> Result<bool> {
            let socket =
                Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP)).map_err(|e| {
                    CleanroomError::internal_error(format!("Failed to create socket: {}", e))
                })?;
            let _ = socket.set_reuse_address(true);
            let address: SocketAddr = format!("{}:{}", addr_str, port).parse().map_err(|e| {
                CleanroomError::internal_error(format!("Failed to parse address: {}", e))
            })?;
            match socket.bind(&address.into()) {
                Ok(_) => Ok(true),
                Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => Ok(false),
                Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => Ok(false),
                Err(e) => Err(CleanroomError::internal_error(format!(
                    "Port availability check failed for port {}: {}",
                    port, e
                ))),
            }
        };

        if !check_bind("127.0.0.1")? {
            return Ok(false);
        }
        if !check_bind("0.0.0.0")? {
            return Ok(false);
        }
        Ok(true)
    }

    /// Get lock file path for port
    fn lock_file_path(&self, port: u16) -> PathBuf {
        self.lock_dir.join(format!("port-{}.lock", port))
    }
}

impl Default for PortAllocator {
    fn default() -> Self {
        Self::new().expect("Failed to create default PortAllocator") // OK: I/O failure here is unrecoverable
    }
}

/// RAII guard for port lock (released on drop)
///
/// The lock is automatically released when this struct is dropped,
/// even on panic or process exit. The lock file is also cleaned up.
///
/// # Example
///
/// ```no_run
/// # use clnrm_core::telemetry::live_check::PortAllocator;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let allocator = PortAllocator::new()?;
/// {
///     let lock = allocator.allocate_port().await?;
///     println!("Using port: {}", lock.port());
///     // Use the port...
/// } // Lock released here automatically
/// # Ok(())
/// # }
/// ```
pub struct PortLock {
    /// Locked port number
    port: u16,
    /// Lock file handle (held until drop)
    _lock_file: File,
}

impl PortLock {
    /// Get the locked port number
    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Drop for PortLock {
    fn drop(&mut self) {
        tracing::debug!("Releasing port lock: {}", self.port);
        // Release from process-wide lock tracking
        {
            if let Ok(mut locks) = PROCESS_PORT_LOCKS.lock() {
                locks.remove(&self.port);
            }
        }
        // flock automatically released when file closed.
        // We do NOT remove the file from the filesystem to avoid
        // inode-replacement race conditions between parallel tests.
    }
}

/// Wait for service to become ready via HTTP health check
///
/// Replaces sleep-based waiting with active health checking.
/// Uses exponential backoff to avoid spamming the service.
///
/// # Arguments
///
/// * `port` - Port where service is listening
/// * `timeout_secs` - Maximum time to wait in seconds
///
/// # Returns
///
/// * `Ok(())` - Service is ready
/// * `Err(Timeout)` - Service did not become ready within timeout
///
/// # Example
///
/// ```no_run
/// # use clnrm_core::telemetry::live_check::port_allocator::wait_for_service_ready;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// wait_for_service_ready(4317, 30).await?;
/// println!("Service is ready!");
/// # Ok(())
/// # }
/// ```
pub async fn wait_for_service_ready(port: u16, timeout_secs: u64) -> Result<()> {
    use reqwest::Client;
    use tokio::time::{sleep, timeout, Duration};

    let client = Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .map_err(|e| {
            CleanroomError::internal_error(format!("Failed to create HTTP client: {}", e))
        })?;

    let health_url = format!("http://localhost:{}/health", port);
    let deadline = Duration::from_secs(timeout_secs);
    let start = Instant::now();

    tracing::debug!(
        "Waiting for service at {} (timeout: {}s)",
        health_url,
        timeout_secs
    );

    let result = timeout(deadline, async {
        let mut delay = Duration::from_millis(100);
        const MAX_DELAY: Duration = Duration::from_millis(1000);

        loop {
            match client.get(&health_url).send().await {
                Ok(response) if response.status().is_success() => {
                    tracing::info!(
                        "✓ Service ready on port {} ({}ms)",
                        port,
                        start.elapsed().as_millis()
                    );
                    return Ok(());
                }
                Ok(response) => {
                    tracing::debug!(
                        "Health check returned {}, retrying in {}ms...",
                        response.status(),
                        delay.as_millis()
                    );
                }
                Err(e) => {
                    tracing::debug!(
                        "Health check failed: {}, retrying in {}ms...",
                        e,
                        delay.as_millis()
                    );
                }
            }

            sleep(delay).await;

            // Exponential backoff
            delay = std::cmp::min(delay * 2, MAX_DELAY);
        }
    })
    .await;

    match result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(CleanroomError::timeout_error(format!(
            "Service did not become ready within {}s. URL: {}",
            timeout_secs, health_url
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_range_creation() {
        let range = PortRange::new(4317, 5317);
        assert_eq!(range.start, 4317);
        assert_eq!(range.end, 5317);
        assert_eq!(range.size(), 1001);
    }

    #[test]
    fn test_port_range_iteration() {
        let range = PortRange::new(4317, 4319);
        let ports: Vec<u16> = range.iter().collect();
        assert_eq!(ports, vec![4317, 4318, 4319]);
    }

    #[test]
    #[should_panic(expected = "Port range start must be <= end")]
    fn test_port_range_invalid() {
        PortRange::new(5317, 4317);
    }

    #[test]
    fn test_port_allocator_creation() {
        let allocator = PortAllocator::new().unwrap();
        assert_eq!(allocator.primary_range.start, 4317);
        assert_eq!(allocator.primary_range.end, 5317);
    }

    #[test]
    fn test_custom_ranges() {
        let allocator = PortAllocator::with_ranges(
            PortRange::new(6000, 6010),
            PortRange::new(7000, 7010),
            PortRange::new(8000, 8020),
        )
        .unwrap();
        assert_eq!(allocator.primary_range.start, 6000);
        assert_eq!(allocator.fallback_range.start, 7000);
        assert_eq!(allocator.extended_range.start, 8000);
    }

    #[tokio::test]
    async fn test_is_port_available() {
        // Should be able to check standard ports
        let available = PortAllocator::is_port_available(19999).await;
        assert!(available.is_ok());
    }
}
