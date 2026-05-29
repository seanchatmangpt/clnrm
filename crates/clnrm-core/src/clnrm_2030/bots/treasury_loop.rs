use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

use crate::clnrm_2030::treasury::AlgorithmicTreasury;
use crate::clnrm_2030::oracle::DecentralizedOracle;

pub struct TreasuryLoop {
    treasury: Arc<RwLock<AlgorithmicTreasury>>,
    oracle: Arc<RwLock<DecentralizedOracle>>,
    inflation_stream_id: String,
    interval_seconds: u64,
}

impl TreasuryLoop {
    pub fn new(
        treasury: Arc<RwLock<AlgorithmicTreasury>>,
        oracle: Arc<RwLock<DecentralizedOracle>>,
        inflation_stream_id: String,
        interval_seconds: u64,
    ) -> Self {
        Self {
            treasury,
            oracle,
            inflation_stream_id,
            interval_seconds,
        }
    }

    pub async fn run(&self) {
        let mut tick_interval = interval(Duration::from_secs(self.interval_seconds));
        
        loop {
            tick_interval.tick().await;
            
            let oracle_read = self.oracle.read().await;
            let actual_inflation = match oracle_read.aggregate(&self.inflation_stream_id) {
                Some(rate) => rate,
                None => {
                    // Fallback or skip tick if no data
                    continue;
                }
            };
            
            // Release read lock before writing to treasury to avoid potential deadlocks
            drop(oracle_read);

            let mut treasury_write = self.treasury.write().await;
            treasury_write.tick(actual_inflation, self.interval_seconds as f64);
        }
    }
}
