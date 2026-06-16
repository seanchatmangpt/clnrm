use crate::market::amm::NDimensionalAMM;
use crate::market::router::InterDimensionalRouter;

pub struct ArbitrageLoopConfig {
    pub min_profit_threshold: f64,
    pub max_trade_size: f64,
    pub scan_interval_ms: u64,
    pub dimensions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ArbitrageOpportunity {
    pub path: Vec<String>,
    pub estimated_profit: f64,
    pub trade_size: f64,
    pub detected_at_ms: u64,
}

#[derive(Debug, Default, Clone)]
pub struct ArbitrageLoopStats {
    pub scans_run: u64,
    pub opportunities_found: u64,
    pub trades_executed: u64,
    pub total_profit: f64,
    pub last_scan_ms: u64,
}

pub struct ArbitrageLoop {
    pub config: ArbitrageLoopConfig,
    pub router: InterDimensionalRouter,
    pub stats: ArbitrageLoopStats,
}

impl ArbitrageLoop {
    pub fn new(config: ArbitrageLoopConfig, router: InterDimensionalRouter) -> Self {
        Self {
            config,
            router,
            stats: ArbitrageLoopStats::default(),
        }
    }

    /// Scans all dimension pairs for arbitrage opportunities.
    ///
    /// For each ordered pair (a, b), it checks whether a simulated swap a→b then b→a
    /// yields a combined price product > 1.0 + min_profit_threshold/100.
    /// Returns all opportunities found and updates scan stats.
    pub fn scan(
        &mut self,
        amm: &mut NDimensionalAMM,
        current_time_ms: u64,
    ) -> Vec<ArbitrageOpportunity> {
        self.stats.scans_run += 1;
        self.stats.last_scan_ms = current_time_ms;

        let dims = self.config.dimensions.clone();
        let trade_size = self.config.max_trade_size;
        let threshold = 1.0 + self.config.min_profit_threshold / 100.0;

        let mut opportunities = Vec::new();

        for i in 0..dims.len() {
            for j in 0..dims.len() {
                if i == j {
                    continue;
                }
                let a = &dims[i];
                let b = &dims[j];

                // Price of a→b: how much b we get for `trade_size` a
                let price_ab = Self::price_quote(amm, a, b, trade_size);
                // Price of b→a: how much a we get for `price_ab` b
                let price_ba = Self::price_quote(amm, b, a, price_ab);

                if price_ab <= 0.0 || price_ba <= 0.0 {
                    continue;
                }

                // Round-trip product: price_ba / trade_size
                let round_trip = price_ba / trade_size;

                if round_trip > threshold {
                    let estimated_profit = (round_trip - 1.0) * trade_size;
                    opportunities.push(ArbitrageOpportunity {
                        path: vec![a.clone(), b.clone(), a.clone()],
                        estimated_profit,
                        trade_size,
                        detected_at_ms: current_time_ms,
                    });
                    self.stats.opportunities_found += 1;
                }
            }
        }

        opportunities
    }

    /// Execute a two-leg arbitrage: swap path[0]→path[1] then path[1]→path[0].
    /// Returns actual profit (final_amount - trade_size).
    pub fn execute_opportunity(
        &mut self,
        amm: &mut NDimensionalAMM,
        opp: &ArbitrageOpportunity,
    ) -> Result<f64, String> {
        if opp.path.len() < 3 {
            return Err("Arbitrage path must have at least 3 nodes (a→b→a)".into());
        }

        let a = &opp.path[0];
        let b = &opp.path[1];
        let trade_size = opp.trade_size;

        // First leg: a → b
        let received_b = amm.swap(a, trade_size, b)?;
        // Second leg: b → a
        let received_a = amm.swap(b, received_b, a)?;

        let profit = received_a - trade_size;

        self.stats.trades_executed += 1;
        self.stats.total_profit += profit;

        Ok(profit)
    }

    pub fn stats(&self) -> &ArbitrageLoopStats {
        &self.stats
    }

    /// Non-destructive price quote: simulates a swap without modifying AMM state.
    /// Returns amount of output_token received for `amount` of input_token.
    fn price_quote(amm: &NDimensionalAMM, input: &str, output: &str, amount: f64) -> f64 {
        let reserves = amm.reserves();
        let input_reserve = match reserves.get(input) {
            Some(&r) => r,
            None => return 0.0,
        };
        let output_reserve = match reserves.get(output) {
            Some(&r) => r,
            None => return 0.0,
        };
        if input_reserve <= 0.0 || output_reserve <= 0.0 || amount <= 0.0 {
            return 0.0;
        }
        // Constant-product formula: out = (in_reserve * out_reserve) / (in_reserve + amount) - ... nope
        // out = out_reserve - (in_reserve * out_reserve) / (in_reserve + amount)
        let new_input_reserve = input_reserve + amount;
        output_reserve - (input_reserve * output_reserve) / new_input_reserve
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_balanced_amm() -> NDimensionalAMM {
        let mut amm = NDimensionalAMM::new();
        let mut amounts = HashMap::new();
        amounts.insert("A".to_string(), 1_000.0);
        amounts.insert("B".to_string(), 1_000.0);
        amounts.insert("C".to_string(), 1_000.0);
        amm.add_liquidity(&amounts).unwrap();
        amm
    }

    fn default_config(dims: Vec<&str>) -> ArbitrageLoopConfig {
        ArbitrageLoopConfig {
            min_profit_threshold: 0.1,
            max_trade_size: 10.0,
            scan_interval_ms: 1000,
            dimensions: dims.into_iter().map(String::from).collect(),
        }
    }

    #[test]
    fn test_scan_increments_counter() {
        let mut lp = ArbitrageLoop::new(
            default_config(vec!["A", "B"]),
            InterDimensionalRouter::new(),
        );
        let mut amm = make_balanced_amm();
        lp.scan(&mut amm, 1000);
        assert_eq!(lp.stats().scans_run, 1);
        assert_eq!(lp.stats().last_scan_ms, 1000);
    }

    #[test]
    fn test_no_opportunity_in_balanced_amm() {
        // In a balanced AMM, round-trip should always be < 1 (price impact)
        let mut lp = ArbitrageLoop::new(
            default_config(vec!["A", "B"]),
            InterDimensionalRouter::new(),
        );
        let mut amm = make_balanced_amm();
        let opps = lp.scan(&mut amm, 1000);
        assert!(
            opps.is_empty(),
            "Balanced AMM should have no arb opportunities"
        );
    }

    #[test]
    fn test_execute_opportunity_updates_stats() {
        let mut lp = ArbitrageLoop::new(
            default_config(vec!["A", "B"]),
            InterDimensionalRouter::new(),
        );
        let mut amm = make_balanced_amm();
        let opp = ArbitrageOpportunity {
            path: vec!["A".to_string(), "B".to_string(), "A".to_string()],
            estimated_profit: 0.5,
            trade_size: 10.0,
            detected_at_ms: 1000,
        };
        let _ = lp.execute_opportunity(&mut amm, &opp);
        assert_eq!(lp.stats().trades_executed, 1);
    }
}
