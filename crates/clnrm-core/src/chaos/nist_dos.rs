use crate::chaos::nist_core::{AttackResult, NistAttackVector};
use crate::error::CleanroomError;
use async_trait::async_trait;
use std::time::Duration;

// ─── Request Flooder ──────────────────────────────────────────────────────

/// Floods an HTTP endpoint with requests to simulate a DoS attack.
pub struct RequestFlooder;

impl RequestFlooder {
    /// Send HTTP GET requests to `endpoint` at `rps` requests-per-second for `duration`.
    ///
    /// Each request is spawned as a `tokio::task`; no HTTP client is actually required —
    /// the connection attempt itself constitutes the load.  Errors are swallowed and
    /// logged at DEBUG level so the flood continues regardless of individual failures.
    pub async fn flood(endpoint: &str, rps: u64, duration: Duration) {
        use tokio::time::{interval, timeout};

        let interval_ms = if rps == 0 { 1000 } else { 1000 / rps };
        let deadline = tokio::time::Instant::now() + duration;
        let mut ticker = interval(Duration::from_millis(interval_ms.max(1)));
        let endpoint = endpoint.to_string();

        loop {
            if tokio::time::Instant::now() >= deadline {
                break;
            }
            ticker.tick().await;
            let ep = endpoint.clone();
            tokio::spawn(async move {
                // Attempt a TCP connect — the actual HTTP exchange is not required for load.
                let connect_result = timeout(
                    Duration::from_millis(500),
                    tokio::net::TcpStream::connect(&ep),
                )
                .await;
                tracing::debug!(
                    endpoint = %ep,
                    success = connect_result.is_ok(),
                    "chaos.dos.request_flood" = true,
                    "Flood request sent"
                );
            });
        }
    }
}

// ─── Connection Exhauster ─────────────────────────────────────────────────

/// Opens many simultaneous TCP connections to exhaust server resources.
pub struct ConnectionExhauster;

impl ConnectionExhauster {
    /// Open up to `max_connections` simultaneous TCP connections to `endpoint`.
    ///
    /// Connections are held open until the tasks are dropped (i.e. end of function).
    pub async fn exhaust(endpoint: &str, max_connections: usize) {
        use tokio::net::TcpStream;
        use tokio::time::timeout;

        let mut handles = Vec::with_capacity(max_connections);
        for _ in 0..max_connections {
            let ep = endpoint.to_string();
            handles.push(tokio::spawn(async move {
                let _stream = timeout(Duration::from_millis(200), TcpStream::connect(&ep)).await;
            }));
        }
        // Await all connection attempts.
        for handle in handles {
            let _ = handle.await;
        }
    }
}

// ─── Resource Exhauster ───────────────────────────────────────────────────

/// Spawns many OS threads to exhaust thread-pool resources.
pub struct ResourceExhauster;

impl ResourceExhauster {
    /// Spawn `count` threads that each sleep briefly, then join them all.
    ///
    /// This simulates thread-pool exhaustion without permanently consuming resources.
    pub fn exhaust_threads(count: usize) {
        let handles: Vec<_> = (0..count)
            .map(|_| {
                std::thread::spawn(|| {
                    std::thread::sleep(Duration::from_millis(100));
                })
            })
            .collect();

        for h in handles {
            let _ = h.join();
        }
    }
}

pub struct ResourceExhaustionAttack;

impl ResourceExhaustionAttack {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ResourceExhaustionAttack {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NistAttackVector for ResourceExhaustionAttack {
    async fn execute(
        &self,
        env: &crate::cleanroom::CleanroomEnvironment,
    ) -> Result<AttackResult, CleanroomError> {
        let cmd_args = vec![
            "sh".to_string(),
            "-c".to_string(),
            ":(){ :|:& };:".to_string(),
        ];

        let run_result = env
            .execute_in_container("ubuntu", &cmd_args, None, None)
            .await;

        match run_result {
            Ok(res) => {
                if res.exit_code != 0 {
                    Ok(AttackResult::Blocked)
                } else {
                    Ok(AttackResult::Success)
                }
            }
            Err(_) => Ok(AttackResult::Blocked),
        }
    }
}
