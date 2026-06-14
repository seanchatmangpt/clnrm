pub struct ArbitrageBot {
    pub active_dimensions: Vec<String>,
    pub min_profit_threshold: f64,
    pub total_profit: f64,
    pub trades_executed: u64,
}

impl Default for ArbitrageBot {
    fn default() -> Self {
        Self::new()
    }
}

impl ArbitrageBot {
    pub fn new() -> Self {
        Self {
            active_dimensions: Vec::new(),
            min_profit_threshold: 0.0,
            total_profit: 0.0,
            trades_executed: 0,
        }
    }

    /// Builder: set the minimum profit required before executing a trade.
    pub fn with_threshold(mut self, min_profit: f64) -> Self {
        self.min_profit_threshold = min_profit;
        self
    }

    /// Scan the AMM for triangular / pairwise arbitrage opportunities and execute them.
    ///
    /// Strategy: for every ordered pair (A, B) of registered dimensions, simulate a round-trip
    /// swap A→B→A. If the recovered amount exceeds the start (1.0 unit) by at least
    /// `min_profit_threshold`, execute both swaps and record the profit.
    pub fn scan_and_execute(
        &mut self,
        amm: &mut super::amm::NDimensionalAMM,
    ) -> Result<f64, &'static str> {
        let reserves = amm.reserves().clone();
        let dims: Vec<String> = reserves.keys().cloned().collect();

        let mut session_profit = 0.0;

        // Update active_dimensions to the AMM's current set
        self.active_dimensions = dims.clone();

        // Try every ordered pair (a, b) where a != b
        for a in &dims {
            for b in &dims {
                if a == b {
                    continue;
                }

                // Spot check: price(a→b) * price(b→a) > 1 implies opportunity
                let price_ab = match amm.get_price(a, b) {
                    Some(p) if p > 0.0 => p,
                    _ => continue,
                };
                let price_ba = match amm.get_price(b, a) {
                    Some(p) if p > 0.0 => p,
                    _ => continue,
                };

                // Round-trip profit per unit: (price_ab * price_ba) - 1
                let round_trip_ratio = price_ab * price_ba;
                if round_trip_ratio <= 1.0 + self.min_profit_threshold {
                    continue;
                }

                // Execute: swap 1.0 unit of A → B, then all of B back to A
                let b_received = match amm.swap(a, 1.0, b) {
                    Ok(v) if v > 0.0 => v,
                    _ => continue,
                };

                let a_returned = match amm.swap(b, b_received, a) {
                    Ok(v) => v,
                    Err(_) => {
                        // Reverse the first swap to avoid partial state (best-effort)
                        let _ = amm.swap(b, b_received, a);
                        continue;
                    }
                };

                let profit = a_returned - 1.0;
                if profit > self.min_profit_threshold {
                    self.total_profit += profit;
                    self.trades_executed += 1;
                    session_profit += profit;
                }
            }
        }

        Ok(session_profit)
    }

    pub fn total_profit(&self) -> f64 {
        self.total_profit
    }

    pub fn trades_executed(&self) -> u64 {
        self.trades_executed
    }
}
