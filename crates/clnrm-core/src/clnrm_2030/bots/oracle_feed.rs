use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use rand::Rng;
use tokio::task::JoinHandle;

use crate::clnrm_2030::oracle::{DecentralizedOracle, OracleDataPoint};

/// Simulator that continuously generates mock real-world metric data
/// and submits it to the DecentralizedOracle every 5 seconds.
pub struct OracleFeedSimulator {
    oracle: Arc<RwLock<DecentralizedOracle>>,
    provider_id: String,
    stake: u64,
}

impl OracleFeedSimulator {
    /// Creates a new OracleFeedSimulator.
    pub fn new(oracle: Arc<RwLock<DecentralizedOracle>>, provider_id: String, stake: u64) -> Self {
        Self {
            oracle,
            provider_id,
            stake,
        }
    }

    /// Spawns a background task that continuously generates and submits mock data.
    pub fn spawn(self) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5));
            loop {
                interval.tick().await;

                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let (compute_utilization, token_price, bandwidth_usage) = {
                    let mut rng = rand::thread_rng();
                    (
                        rng.gen_range(10.0..95.0),
                        rng.gen_range(100.0..120.0),
                        rng.gen_range(50.0..500.0)
                    )
                };

                let compute_point = OracleDataPoint {
                    value: compute_utilization,
                    timestamp,
                    provider: self.provider_id.clone(),
                    stake: self.stake,
                };

                let price_point = OracleDataPoint {
                    value: token_price,
                    timestamp,
                    provider: self.provider_id.clone(),
                    stake: self.stake,
                };

                let bandwidth_point = OracleDataPoint {
                    value: bandwidth_usage,
                    timestamp,
                    provider: self.provider_id.clone(),
                    stake: self.stake,
                };

                // Submit the generated data points to the oracle
                let mut oracle_lock = self.oracle.write().await;
                oracle_lock.submit_data("compute_utilization", compute_point);
                oracle_lock.submit_data("token_price", price_point);
                oracle_lock.submit_data("bandwidth_usage", bandwidth_point);
            }
        })
    }
}
