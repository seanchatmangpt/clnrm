use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Result of a NIST attack vector execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttackResult {
    /// The adversary won (attack was successful).
    Success,
    /// The system defended successfully (attack was blocked).
    Blocked,
    /// An error occurred during the execution of the attack.
    Error,
}

/// A generic trait defining an adversarial attack vector.
#[async_trait]
pub trait NistAttackVector: Send + Sync {
    /// Executes the attack vector against the given cleanroom environment.
    async fn execute(
        &self,
        env: &crate::cleanroom::CleanroomEnvironment,
    ) -> Result<AttackResult, crate::error::CleanroomError>;
}

/// Orchestrates the execution of NIST adversarial attack vectors.
pub struct NistAdversarialEngine {
    vectors: Vec<Arc<dyn NistAttackVector>>,
}

impl NistAdversarialEngine {
    /// Creates a new, empty `NistAdversarialEngine`.
    pub fn new() -> Self {
        Self {
            vectors: Vec::new(),
        }
    }

    /// Registers a new attack vector with the engine.
    pub fn add_vector(&mut self, vector: Arc<dyn NistAttackVector>) {
        self.vectors.push(vector);
    }

    /// Executes all registered attack vectors.
    pub async fn execute_all(
        &self,
        env: &crate::cleanroom::CleanroomEnvironment,
    ) -> Result<Vec<AttackResult>, crate::error::CleanroomError> {
        let mut results = Vec::new();
        for vector in &self.vectors {
            let result = vector.execute(env).await?;
            results.push(result);
        }
        Ok(results)
    }
}

impl Default for NistAdversarialEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// CPU stressor — spins threads doing SHA-256 to saturate CPU cores.
pub struct CpuStressor;

impl CpuStressor {
    /// Spawn `target_cores` threads, each running SHA-256 in a tight loop for `duration`.
    pub fn inject(duration: Duration, target_cores: usize) {
        use sha2::{Digest, Sha256};
        use std::thread;

        let mut handles = Vec::with_capacity(target_cores);
        for _ in 0..target_cores {
            let deadline = Instant::now() + duration;
            handles.push(thread::spawn(move || {
                let mut hasher = Sha256::new();
                let data = b"chaos_cpu_stress_payload";
                while Instant::now() < deadline {
                    hasher.update(data);
                    let _ = hasher.finalize_reset();
                }
            }));
        }
        for h in handles {
            let _ = h.join();
        }
    }
}

/// Memory stressor — allocates, writes (to prevent optimization), holds, then releases.
pub struct MemoryStressor;

impl MemoryStressor {
    /// Allocate `target_mb` megabytes, write to every page, hold for `duration`, then release.
    pub fn inject(target_mb: usize, duration: Duration) {
        use std::hint::black_box;

        let byte_count = target_mb * 1024 * 1024;
        let mut buf: Vec<u8> = vec![0xAB; byte_count];

        // Write to every 4096-byte page to prevent OS over-commit optimizations
        for chunk in buf.chunks_mut(4096) {
            chunk[0] = black_box(0xFF);
        }

        std::thread::sleep(duration);

        // Ensure the compiler cannot eliminate the allocation
        let _ = black_box(buf.as_mut_ptr());
    }
}

/// Disk filler — creates a file of the requested size for chaos testing.
pub struct DiskFiller;

impl DiskFiller {
    /// Write a fill file at `path/<uuid>.chaos` of size `fill_mb` MB.
    /// Returns the path of the created file for later cleanup.
    pub fn inject(
        dir: &Path,
        fill_mb: usize,
    ) -> Result<std::path::PathBuf, crate::error::CleanroomError> {
        use std::io::Write;
        use uuid::Uuid;

        let file_path = dir.join(format!("clnrm_disk_fill_{}.chaos", Uuid::new_v4()));
        let mut file = std::fs::File::create(&file_path).map_err(|e| {
            crate::error::CleanroomError::io_error(format!(
                "DiskFiller: failed to create fill file {:?}: {}",
                file_path, e
            ))
        })?;

        let chunk = vec![0u8; 1024 * 1024]; // 1 MB buffer
        for _ in 0..fill_mb {
            file.write_all(&chunk).map_err(|e| {
                crate::error::CleanroomError::io_error(format!("DiskFiller: write failed: {}", e))
            })?;
        }
        file.sync_all().map_err(|e| {
            crate::error::CleanroomError::io_error(format!("DiskFiller: sync failed: {}", e))
        })?;

        Ok(file_path)
    }

    /// Delete the fill file created by `inject`.
    pub fn cleanup(path: &Path) -> Result<(), crate::error::CleanroomError> {
        std::fs::remove_file(path).map_err(|e| {
            crate::error::CleanroomError::io_error(format!(
                "DiskFiller: cleanup failed for {:?}: {}",
                path, e
            ))
        })
    }
}

/// Chaos metrics recorder — emits OTEL span events for injection lifecycles.
pub struct ChaosMetrics;

impl ChaosMetrics {
    /// Emit a tracing span event recording the injection of a chaos scenario.
    pub fn record_injection(scenario_type: &str, start: Instant, end: Instant) {
        let duration_ms = end.duration_since(start).as_millis();
        tracing::info!(
            chaos.scenario = scenario_type,
            chaos.duration_ms = duration_ms as u64,
            "chaos injection completed"
        );
    }
}
