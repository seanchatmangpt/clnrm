use rand::Rng;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use crate::market::amm::NDimensionalAMM;

pub struct AutonomousMarketMaker {
    pub amm: Arc<Mutex<NDimensionalAMM>>,
    pub is_running: Arc<AtomicBool>,
}

impl AutonomousMarketMaker {
    pub fn new(amm: Arc<Mutex<NDimensionalAMM>>) -> Self {
        Self {
            amm,
            is_running: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    pub async fn run(&self) {
        let mut rng = rand::thread_rng();

        // Initial liquidity injection if AMM is completely empty
        {
            let mut amm = self.amm.lock().await;
            if amm.reserves().is_empty() {
                let mut initial_amounts = HashMap::new();
                initial_amounts.insert("TOKEN_A".to_string(), 1_000_000.0);
                initial_amounts.insert("TOKEN_B".to_string(), 1_000_000.0);
                initial_amounts.insert("TOKEN_C".to_string(), 1_000_000.0);
                let _ = amm.add_liquidity(&initial_amounts);
            }
        }

        while self.is_running.load(Ordering::SeqCst) {
            // Simulate trading intervals
            let delay_ms = rng.gen_range(10..200);
            sleep(Duration::from_millis(delay_ms)).await;

            let mut amm = self.amm.lock().await;
            let reserves = amm.reserves().clone();

            if reserves.is_empty() {
                continue;
            }

            let action = rng.gen_range(0..100);

            if action < 15 {
                // 15% probability: Provide liquidity proportionally
                let proportion = rng.gen_range(0.01..0.05);
                let mut amounts_to_add = HashMap::new();
                for (token, amount) in reserves.iter() {
                    amounts_to_add.insert(token.clone(), amount * proportion);
                }
                let _ = amm.add_liquidity(&amounts_to_add);
            } else if action < 30 {
                // 15% probability: Remove liquidity
                let proportion = rng.gen_range(0.01..0.05);
                let _ = amm.remove_liquidity(proportion);
            } else {
                // 70% probability: Inject random trading volume (swap)
                let tokens: Vec<String> = reserves.keys().cloned().collect();
                if tokens.len() >= 2 {
                    let idx1 = rng.gen_range(0..tokens.len());
                    let mut idx2 = rng.gen_range(0..tokens.len());
                    while idx1 == idx2 {
                        idx2 = rng.gen_range(0..tokens.len());
                    }

                    // Calculate a reasonable swap amount based on the pool size
                    let max_swap_size = reserves[&tokens[idx1]] * 0.1; // Max 10% of reserve
                    if max_swap_size > 1.0 {
                        let amount_in = rng.gen_range(1.0..max_swap_size);
                        let _ = amm.swap(&tokens[idx1], amount_in, &tokens[idx2]);
                    }
                }
            }
        }
    }
}
