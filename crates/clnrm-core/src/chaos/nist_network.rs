use crate::chaos::nist_core::{AttackResult, NistAttackVector};
use crate::cleanroom::CleanroomEnvironment;
use async_trait::async_trait;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// ─── Network Latency Injector ──────────────────────────────────────────────

/// Simulates network latency by sleeping the calling thread.
pub struct NetworkLatencyInjector;

impl NetworkLatencyInjector {
    /// Inject synthetic network latency.
    ///
    /// Sleeps the calling thread for `target_ms` plus a random jitter up to
    /// `jitter_ms` milliseconds, simulating an unreliable network link.
    pub fn inject_latency(target_ms: u64, jitter_ms: u64) {
        let jitter = if jitter_ms > 0 {
            let random_bytes: [u8; 8] = rand::random();
            u64::from_ne_bytes(random_bytes) % (jitter_ms + 1)
        } else {
            0
        };
        let total_ms = target_ms + jitter;
        std::thread::sleep(Duration::from_millis(total_ms));
    }
}

// ─── Network Partition Injector ────────────────────────────────────────────

/// Simulates a network partition by marking services as unreachable.
pub struct NetworkPartitionInjector {
    /// Shared state of currently partitioned services.
    partitioned: Arc<Mutex<HashSet<String>>>,
}

impl NetworkPartitionInjector {
    /// Create a new injector with an empty partition set.
    pub fn new() -> Self {
        Self {
            partitioned: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Mark `services` as unreachable for `duration`, then clear the partition.
    pub fn partition(&self, services: &[&str], duration: Duration) {
        {
            let mut set = self.partitioned.lock().expect("partitioned lock");
            for svc in services {
                tracing::info!(
                    service = *svc,
                    duration_ms = duration.as_millis() as u64,
                    "chaos.network.partition" = true,
                    "Network partition injected"
                );
                set.insert(svc.to_string());
            }
        }

        std::thread::sleep(duration);

        {
            let mut set = self.partitioned.lock().expect("partitioned lock");
            for svc in services {
                set.remove(*svc);
            }
        }
    }

    /// Returns `true` if the given service is currently partitioned.
    pub fn is_partitioned(&self, service: &str) -> bool {
        self.partitioned
            .lock()
            .expect("partitioned lock")
            .contains(service)
    }
}

impl Default for NetworkPartitionInjector {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Packet Loss Simulator ─────────────────────────────────────────────────

/// Decides probabilistically whether an individual packet should be dropped.
pub struct PacketLossSimulator;

impl PacketLossSimulator {
    /// Return `true` with probability `loss_rate` (0.0 = no loss, 1.0 = all dropped).
    pub fn should_drop(loss_rate: f64) -> bool {
        let r: f64 = rand::random();
        r < loss_rate
    }
}

// ─── Bandwidth Limiter ─────────────────────────────────────────────────────

/// Computes the sleep duration required to honour a bandwidth cap.
pub struct BandwidthLimiter;

impl BandwidthLimiter {
    /// Return the [`Duration`] the caller must sleep to send `bytes` at `bandwidth_bps`.
    ///
    /// Formula: `sleep = bytes / bandwidth_bps` (in seconds).
    /// Returns [`Duration::ZERO`] when `bandwidth_bps` is zero to avoid division by zero.
    pub fn throttle_bytes(bytes: usize, bandwidth_bps: u64) -> Duration {
        if bandwidth_bps == 0 {
            return Duration::ZERO;
        }
        let seconds = bytes as f64 / bandwidth_bps as f64;
        Duration::from_secs_f64(seconds)
    }
}

/// Simulates a network egress attack attempting to connect to external
/// unauthorized IPs (like 8.8.8.8) or perform DNS tunneling.
pub struct NetworkEgressAttack {
    target_ip: String,
    container_name: String,
}

impl NetworkEgressAttack {
    /// Creates a new NetworkEgressAttack targeting the specified IP.
    pub fn new(target_ip: &str, container_name: &str) -> Self {
        Self {
            target_ip: target_ip.to_string(),
            container_name: container_name.to_string(),
        }
    }
}

#[async_trait]
impl NistAttackVector for NetworkEgressAttack {
    async fn execute(
        &self,
        env: &CleanroomEnvironment,
    ) -> Result<AttackResult, crate::error::CleanroomError> {
        let command = vec![
            "ping".to_string(),
            "-c".to_string(),
            "1".to_string(),
            "-W".to_string(),
            "2".to_string(),
            self.target_ip.clone(),
        ];

        match env
            .execute_in_container(&self.container_name, &command, None, None)
            .await
        {
            Ok(result) => {
                // If the command fails, the network policies successfully blocked the egress.
                if result.exit_code != 0
                    || result.stderr.contains("Network is unreachable")
                    || result.stderr.contains("Operation not permitted")
                    || result.stdout.contains("100% packet loss")
                    || result.stderr.contains("100% packet loss")
                {
                    Ok(AttackResult::Blocked)
                } else {
                    // The egress succeeded, so the adversary won.
                    Ok(AttackResult::Success)
                }
            }
            Err(_) => {
                // Execution failed at the environment level, which acts as a block.
                Ok(AttackResult::Blocked)
            }
        }
    }
}
