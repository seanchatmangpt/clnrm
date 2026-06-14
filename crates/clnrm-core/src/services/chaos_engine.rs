//! Chaos Engineering Service Plugin
//!
//! Revolutionary chaos testing plugin that introduces controlled failures,
//! network partitions, and system degradation to test resilience.

use crate::cleanroom::{HealthStatus, ServiceHandle, ServicePlugin};
use crate::error::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Chaos engineering configuration
#[derive(Debug, Clone)]
pub struct ChaosConfig {
    /// Failure injection rate (0.0 to 1.0)
    pub failure_rate: f64,
    /// Latency injection in milliseconds
    pub latency_ms: u64,
    /// Network partition probability
    pub network_partition_rate: f64,
    /// Memory pressure injection
    pub memory_pressure_mb: u64,
    /// CPU stress injection
    pub cpu_stress_percent: u8,
    /// Chaos scenarios to run
    pub scenarios: Vec<ChaosScenario>,
}

/// Chaos testing scenarios
#[derive(Debug, Clone)]
pub enum ChaosScenario {
    /// Random service failures
    RandomFailures {
        duration_secs: u64,
        failure_rate: f64,
    },
    /// Network latency spikes
    LatencySpikes {
        duration_secs: u64,
        max_latency_ms: u64,
    },
    /// Memory exhaustion
    MemoryExhaustion { duration_secs: u64, target_mb: u64 },
    /// CPU saturation
    CpuSaturation {
        duration_secs: u64,
        target_percent: u8,
    },
    /// Network partition
    NetworkPartition {
        duration_secs: u64,
        affected_services: Vec<String>,
    },
    /// Cascading failures
    CascadingFailures {
        trigger_service: String,
        propagation_delay_ms: u64,
    },
    /// Disk fill scenario
    DiskFill {
        duration_secs: u64,
        fill_mb: u64,
        path: Option<String>,
    },
}

impl Default for ChaosConfig {
    fn default() -> Self {
        Self {
            failure_rate: 0.1,
            latency_ms: 100,
            network_partition_rate: 0.05,
            memory_pressure_mb: 100,
            cpu_stress_percent: 50,
            scenarios: vec![
                ChaosScenario::RandomFailures {
                    duration_secs: 30,
                    failure_rate: 0.2,
                },
                ChaosScenario::LatencySpikes {
                    duration_secs: 60,
                    max_latency_ms: 500,
                },
            ],
        }
    }
}

/// Chaos engineering service plugin
#[derive(Debug)]
pub struct ChaosEnginePlugin {
    name: String,
    config: ChaosConfig,
    active_scenarios: Arc<RwLock<Vec<String>>>,
    metrics: Arc<RwLock<ChaosMetrics>>,
}

/// Chaos testing metrics
#[derive(Debug, Default, Clone)]
pub struct ChaosMetrics {
    /// Total failures injected
    pub failures_injected: u64,
    /// Total latency injected (ms)
    pub latency_injected_ms: u64,
    /// Network partitions created
    pub network_partitions: u64,
    /// Services affected by chaos
    pub affected_services: Vec<String>,
    /// Chaos scenarios executed
    pub scenarios_executed: u64,
    /// Total scenario execution duration (ms)
    pub total_duration_ms: u64,
    /// Memory pressure injected (MB × seconds)
    pub memory_pressure_mb_secs: u64,
    /// CPU stress thread-seconds consumed
    pub cpu_stress_thread_secs: u64,
    /// Disk bytes written by fill scenarios
    pub disk_bytes_written: u64,
    /// Scenario start timestamps (scenario_id → epoch ms)
    pub scenario_start_times_ms: std::collections::HashMap<String, u64>,
    /// Per-scenario duration (scenario_id → ms)
    pub scenario_durations_ms: std::collections::HashMap<String, u64>,
}

impl ChaosEnginePlugin {
    /// Create a new chaos engine plugin
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            config: ChaosConfig::default(),
            active_scenarios: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RwLock::new(ChaosMetrics::default())),
        }
    }

    /// Create with custom configuration
    pub fn with_config(name: &str, config: ChaosConfig) -> Self {
        Self {
            name: name.to_string(),
            config,
            active_scenarios: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RwLock::new(ChaosMetrics::default())),
        }
    }

    /// Set failure injection rate
    pub fn with_failure_rate(mut self, rate: f64) -> Self {
        self.config.failure_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Set latency injection
    pub fn with_latency(mut self, latency_ms: u64) -> Self {
        self.config.latency_ms = latency_ms;
        self
    }

    /// Add chaos scenario
    pub fn with_scenario(mut self, scenario: ChaosScenario) -> Self {
        self.config.scenarios.push(scenario);
        self
    }

    /// Inject random failure
    pub async fn inject_failure(&self, service_name: &str) -> Result<bool> {
        let should_fail = rand::random::<f64>() < self.config.failure_rate;

        if should_fail {
            let mut metrics = self.metrics.write().await;
            metrics.failures_injected += 1;
            metrics.affected_services.push(service_name.to_string());

            tracing::info!(
                service = %service_name,
                "Chaos engine injecting failure"
            );
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Inject latency
    pub async fn inject_latency(&self, service_name: &str) -> Result<u64> {
        let latency = if rand::random::<f64>() < 0.3 {
            self.config.latency_ms + rand::random::<u64>() % 200
        } else {
            0
        };

        if latency > 0 {
            let mut metrics = self.metrics.write().await;
            metrics.latency_injected_ms += latency;

            tracing::info!(
                service = %service_name,
                latency_ms = latency,
                "Chaos engine injecting latency"
            );

            // Simulate latency
            tokio::time::sleep(std::time::Duration::from_millis(latency)).await;
        }

        Ok(latency)
    }

    /// Create network partition
    pub async fn create_network_partition(&self, services: &[String]) -> Result<()> {
        if rand::random::<f64>() < self.config.network_partition_rate {
            let mut metrics = self.metrics.write().await;
            metrics.network_partitions += 1;
            metrics.affected_services.extend(services.iter().cloned());

            tracing::info!(
                services = ?services,
                "Chaos engine creating network partition"
            );
        }
        Ok(())
    }

    /// Run chaos scenario
    pub async fn run_scenario(&self, scenario: &ChaosScenario) -> Result<()> {
        let scenario_id = Uuid::new_v4().to_string();
        let scenario_start = std::time::Instant::now();
        let scenario_start_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        {
            let mut active = self.active_scenarios.write().await;
            active.push(scenario_id.clone());
        }

        {
            let mut metrics = self.metrics.write().await;
            metrics.scenarios_executed += 1;
            metrics
                .scenario_start_times_ms
                .insert(scenario_id.clone(), scenario_start_ms);
        }

        match scenario {
            ChaosScenario::RandomFailures {
                duration_secs,
                failure_rate,
            } => {
                tracing::info!(
                    duration_secs,
                    failure_rate_percent = failure_rate * 100.0,
                    "Chaos engine running random failures scenario"
                );

                // Simulate random failures over duration, updating metrics after each tick
                let mut injected: u64 = 0;
                for _ in 0..*duration_secs {
                    if rand::random::<f64>() < *failure_rate {
                        injected += 1;
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
                {
                    let mut m = self.metrics.write().await;
                    m.failures_injected += injected;
                }
            }
            ChaosScenario::LatencySpikes {
                duration_secs,
                max_latency_ms,
            } => {
                tracing::info!(
                    duration_secs,
                    max_latency_ms,
                    "Chaos engine running latency spikes scenario"
                );

                // Simulate latency spikes, accumulate latency without holding lock during sleep
                let mut total_latency: u64 = 0;
                for _ in 0..*duration_secs {
                    if rand::random::<f64>() < 0.1 {
                        let max_latency = (*max_latency_ms).max(1);
                        let latency = rand::random::<u64>() % max_latency;
                        total_latency += latency;
                        tokio::time::sleep(std::time::Duration::from_millis(latency)).await;
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
                {
                    let mut m = self.metrics.write().await;
                    m.latency_injected_ms += total_latency;
                }
            }
            ChaosScenario::MemoryExhaustion {
                duration_secs,
                target_mb,
            } => {
                tracing::info!(
                    duration_secs,
                    target_mb,
                    "Chaos engine running memory exhaustion scenario"
                );

                let start = std::time::Instant::now();
                let duration_secs_val = *duration_secs;
                let target_mb_val = *target_mb;

                // Actually allocate and touch the memory so the OS cannot optimize it away.
                // Run in a blocking task to avoid starving the tokio executor.
                let hold_task = tokio::task::spawn_blocking(move || {
                    let size = (target_mb_val * 1024 * 1024) as usize;
                    // Allocate and write to every page so it is resident in physical memory
                    let mut memory_pressure = vec![0u8; size];
                    memory_pressure.iter_mut().for_each(|b| *b = 1);
                    // Hold the allocation for the requested duration
                    let target = std::time::Duration::from_secs(duration_secs_val);
                    while start.elapsed() < target {
                        // Touch a few bytes every 100ms to prevent deallocation
                        memory_pressure[0] = memory_pressure[0].wrapping_add(1);
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                    // Return the vec so it is dropped only after the duration
                    memory_pressure
                });

                tokio::time::sleep(std::time::Duration::from_secs(*duration_secs)).await;
                // Cancel the blocking task by dropping (best-effort)
                drop(hold_task);

                // Record memory pressure metrics
                {
                    let mut m = self.metrics.write().await;
                    m.memory_pressure_mb_secs += target_mb * duration_secs;
                }
            }
            ChaosScenario::CpuSaturation {
                duration_secs,
                target_percent,
            } => {
                tracing::info!(
                    duration_secs,
                    target_percent,
                    "Chaos engine running CPU saturation scenario"
                );

                let duration_secs_val = *duration_secs;
                let target_percent_val = *target_percent;

                // Determine how many logical CPUs to stress
                let num_cpus = std::thread::available_parallelism()
                    .map(|n| n.get())
                    .unwrap_or(1);
                // Scale number of threads by target_percent (1–100 → 1..num_cpus)
                let stress_threads = ((num_cpus as f64 * (target_percent_val as f64 / 100.0))
                    .ceil() as usize)
                    .max(1)
                    .min(num_cpus);

                tracing::info!(
                    stress_threads,
                    num_cpus,
                    "Spawning CPU stress threads"
                );

                // Spawn busy-loop threads that actually consume CPU
                let mut thread_handles = Vec::with_capacity(stress_threads);
                for _ in 0..stress_threads {
                    let duration = std::time::Duration::from_secs(duration_secs_val);
                    let handle = std::thread::spawn(move || {
                        let deadline = std::time::Instant::now() + duration;
                        let mut counter: u64 = 0;
                        while std::time::Instant::now() < deadline {
                            // Busy loop — genuinely consumes CPU
                            counter = counter.wrapping_add(1);
                            // Occasionally yield to avoid starving the OS scheduler
                            if counter % 1_000_000 == 0 {
                                std::thread::yield_now();
                            }
                        }
                    });
                    thread_handles.push(handle);
                }

                // Wait for duration in async context while threads burn CPU
                tokio::time::sleep(std::time::Duration::from_secs(*duration_secs)).await;

                // Threads will self-terminate after duration; join for clean shutdown
                for h in thread_handles {
                    let _ = h.join();
                }

                // Record CPU stress metrics (thread-seconds = threads × duration)
                {
                    let mut m = self.metrics.write().await;
                    m.cpu_stress_thread_secs += stress_threads as u64 * duration_secs_val;
                }
            }
            ChaosScenario::NetworkPartition {
                duration_secs,
                affected_services,
            } => {
                tracing::info!(
                    duration_secs,
                    affected_services = ?affected_services,
                    "Chaos engine running network partition scenario"
                );

                {
                    let mut m = self.metrics.write().await;
                    m.network_partitions += 1;
                    m.affected_services.extend(affected_services.iter().cloned());
                }
                tokio::time::sleep(std::time::Duration::from_secs(*duration_secs)).await;
            }
            ChaosScenario::CascadingFailures {
                trigger_service,
                propagation_delay_ms,
            } => {
                tracing::info!(
                    trigger_service = %trigger_service,
                    propagation_delay_ms,
                    "Chaos engine running cascading failures scenario"
                );

                // Simulate cascading failure — primary service fails immediately
                {
                    let mut m = self.metrics.write().await;
                    m.failures_injected += 1;
                    m.affected_services.push(trigger_service.clone());
                }

                tokio::time::sleep(std::time::Duration::from_millis(*propagation_delay_ms)).await;

                // Simulate propagation to downstream services
                let cascade_services = vec!["service_b".to_string(), "service_c".to_string()];
                {
                    let mut m = self.metrics.write().await;
                    m.failures_injected += cascade_services.len() as u64;
                    m.affected_services.extend(cascade_services);
                }
            }
            ChaosScenario::DiskFill {
                duration_secs,
                fill_mb,
                path,
            } => {
                let target_dir = path
                    .as_deref()
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(std::env::temp_dir);

                tracing::info!(
                    duration_secs,
                    fill_mb,
                    target_dir = ?target_dir,
                    "Chaos engine running disk fill scenario"
                );

                // Create the target directory if it doesn't exist
                if !target_dir.exists() {
                    std::fs::create_dir_all(&target_dir).map_err(|e| {
                        crate::error::CleanroomError::internal_error(format!(
                            "Failed to create target directory: {}",
                            e
                        ))
                    })?;
                }

                // Generate a unique temp file path
                let file_path =
                    target_dir.join(format!("clnrm_chaos_disk_fill_{}.tmp", Uuid::new_v4()));

                // Perform disk fill in a blocking task to avoid stalling tokio executor
                let file_path_clone = file_path.clone();
                let fill_mb_val = *fill_mb;
                let bytes_written = tokio::task::spawn_blocking(move || -> std::io::Result<u64> {
                    use std::io::Write;
                    let mut file = std::fs::File::create(&file_path_clone)?;
                    let chunk = vec![0u8; 1024 * 1024]; // 1MB buffer
                    let mut written: u64 = 0;
                    for _ in 0..fill_mb_val {
                        file.write_all(&chunk)?;
                        written += chunk.len() as u64;
                    }
                    file.sync_all()?;
                    Ok(written)
                })
                .await
                .map_err(|e| {
                    crate::error::CleanroomError::internal_error(format!(
                        "Disk fill task panicked: {}",
                        e
                    ))
                })?
                .map_err(|e| {
                    crate::error::CleanroomError::internal_error(format!(
                        "Failed to write disk fill file: {}",
                        e
                    ))
                })?;

                // Record disk bytes written
                {
                    let mut metrics = self.metrics.write().await;
                    metrics.disk_bytes_written += bytes_written;
                }

                // Keep it filled for duration_secs
                tokio::time::sleep(std::time::Duration::from_secs(*duration_secs)).await;

                // Cleanup
                if let Err(e) = tokio::fs::remove_file(&file_path).await {
                    tracing::warn!(error = %e, path = ?file_path, "Failed to remove disk fill temp file");
                }
            }
        }

        // Record scenario duration and remove from active scenarios
        let scenario_duration_ms = scenario_start.elapsed().as_millis() as u64;
        {
            let mut metrics = self.metrics.write().await;
            metrics
                .scenario_durations_ms
                .insert(scenario_id.clone(), scenario_duration_ms);
            metrics.total_duration_ms += scenario_duration_ms;
        }

        {
            let mut active = self.active_scenarios.write().await;
            active.retain(|id| id != &scenario_id);
        }

        tracing::info!(
            scenario_id = %scenario_id,
            duration_ms = scenario_duration_ms,
            "Chaos scenario completed"
        );

        Ok(())
    }

    /// Get chaos metrics
    pub async fn get_metrics(&self) -> ChaosMetrics {
        self.metrics.read().await.clone()
    }
}

impl ServicePlugin for ChaosEnginePlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                tracing::info!("Chaos engine starting");

                // Run initial chaos scenarios
                for scenario in &self.config.scenarios {
                    if let Err(e) = self.run_scenario(scenario).await {
                        tracing::warn!(error = %e, "Chaos scenario failed");
                    }
                }

                let mut metadata = HashMap::new();
                metadata.insert("chaos_engine_version".to_string(), "1.0.0".to_string());
                metadata.insert(
                    "failure_rate".to_string(),
                    self.config.failure_rate.to_string(),
                );
                metadata.insert("latency_ms".to_string(), self.config.latency_ms.to_string());
                metadata.insert(
                    "scenarios_count".to_string(),
                    self.config.scenarios.len().to_string(),
                );
                metadata.insert("service_type".to_string(), "chaos_engine".to_string());
                metadata.insert("status".to_string(), "running".to_string());

                Ok(ServiceHandle {
                    id: Uuid::new_v4().to_string(),
                    service_name: self.name.clone(),
                    metadata,
                })
            })
        })
    }

    fn stop(&self, _handle: ServiceHandle) -> Result<()> {
        tracing::info!("Chaos engine stopping");

        // Stop all active scenarios
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut active = self.active_scenarios.write().await;
                active.clear();
            })
        });

        Ok(())
    }

    fn health_check(&self, handle: &ServiceHandle) -> HealthStatus {
        if handle.metadata.contains_key("chaos_engine_version") {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        }
    }
}
