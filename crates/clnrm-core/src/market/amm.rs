use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum AssetType {
    Token,
    ComputeContract,
    ProverBid,
    OracleDataPoint,
    SybilRegistry,
}

#[derive(Debug, Clone)]
pub struct NDimensionalAMM {
    reserves: HashMap<String, f64>,
    asset_types: HashMap<String, AssetType>,
}

impl Default for NDimensionalAMM {
    fn default() -> Self {
        Self::new()
    }
}

impl NDimensionalAMM {
    pub fn new() -> Self {
        Self {
            reserves: HashMap::new(),
            asset_types: HashMap::new(),
        }
    }

    pub fn reserves(&self) -> &HashMap<String, f64> {
        &self.reserves
    }

    pub fn asset_types(&self) -> &HashMap<String, AssetType> {
        &self.asset_types
    }

    pub fn register_asset(&mut self, token: &str, asset_type: AssetType) {
        self.asset_types.insert(token.to_string(), asset_type);
    }

    /// Calculates the current invariant (k = product of all reserves).
    pub fn invariant(&self) -> f64 {
        if self.reserves.is_empty() {
            return 0.0;
        }
        self.reserves.values().product()
    }

    /// Adds liquidity proportionally to existing reserves, or initializes them if empty.
    pub fn add_liquidity(&mut self, amounts: &HashMap<String, f64>) -> Result<(), String> {
        if amounts.is_empty() {
            return Err("Cannot add zero liquidity".into());
        }

        if self.reserves.is_empty() {
            for (token, amount) in amounts {
                if *amount <= 0.0 {
                    return Err("Initial amounts must be positive".into());
                }
                self.reserves.insert(token.clone(), *amount);
            }
            return Ok(());
        }

        // Validate all tokens exist in the AMM
        for token in amounts.keys() {
            if !self.reserves.contains_key(token) {
                return Err(format!("Token {} not in AMM", token));
            }
        }

        // Ensure proportional addition
        let first_token = amounts.keys().next().unwrap();
        let ratio = amounts[first_token] / self.reserves[first_token];

        for (token, amount) in amounts {
            let expected_amount = self.reserves[token] * ratio;
            if (amount - expected_amount).abs() > 1e-6 {
                return Err("Unbalanced liquidity addition: does not match reserve ratios".into());
            }
        }

        // Apply state updates
        for (token, amount) in amounts {
            *self.reserves.get_mut(token).unwrap() += amount;
        }

        Ok(())
    }

    /// Removes liquidity proportionally across all dimensions based on a 0.0-1.0 share.
    pub fn remove_liquidity(&mut self, proportion: f64) -> Result<HashMap<String, f64>, String> {
        if proportion <= 0.0 || proportion > 1.0 {
            return Err("Proportion must be strictly between 0 and 1".into());
        }

        let mut removed = HashMap::new();
        for (token, reserve) in self.reserves.iter_mut() {
            let amount = *reserve * proportion;
            *reserve -= amount;
            removed.insert(token.clone(), amount);
        }

        Ok(removed)
    }

    /// Swaps an `input_amount` of `input_token` for `output_token` while preserving the invariant product.
    pub fn swap(
        &mut self,
        input_token: &str,
        input_amount: f64,
        output_token: &str,
    ) -> Result<f64, String> {
        if input_amount <= 0.0 {
            return Err("Swap amount must be positive".into());
        }
        if input_token == output_token {
            return Err("Input and output tokens must be different".into());
        }
        if !self.reserves.contains_key(input_token) || !self.reserves.contains_key(output_token) {
            return Err("One or both tokens not found in the AMM".into());
        }

        let input_reserve = self.reserves[input_token];
        let output_reserve = self.reserves[output_token];

        let new_input_reserve = input_reserve + input_amount;

        // R_out_new = (R_in_old * R_out_old) / R_in_new
        let new_output_reserve = (input_reserve * output_reserve) / new_input_reserve;
        let output_amount = output_reserve - new_output_reserve;

        // Apply state updates
        *self.reserves.get_mut(input_token).unwrap() = new_input_reserve;
        *self.reserves.get_mut(output_token).unwrap() = new_output_reserve;

        Ok(output_amount)
    }

    /// Spot price = ratio of reserves (how much to_token per 1 from_token).
    pub fn get_price(&self, from_token: &str, to_token: &str) -> Option<f64> {
        let from_reserve = self.reserves.get(from_token)?;
        let to_reserve = self.reserves.get(to_token)?;
        if *from_reserve == 0.0 {
            return None;
        }
        Some(to_reserve / from_reserve)
    }
}

impl NDimensionalAMM {
    /// Spot price = ratio of reserves (how much to_token per 1 from_token)
    pub fn get_price(&self, from_token: &str, to_token: &str) -> Option<f64> {
        let from_reserve = self.reserves.get(from_token)?;
        let to_reserve = self.reserves.get(to_token)?;
        if *from_reserve == 0.0 {
            return None;
        }
        Some(to_reserve / from_reserve)
    }

    /// Price impact percentage = (1 - new_price/old_price) * 100
    pub fn calculate_price_impact(
        &self,
        from_token: &str,
        to_token: &str,
        amount_in: f64,
    ) -> Option<f64> {
        let old_price = self.get_price(from_token, to_token)?;
        let from_reserve = *self.reserves.get(from_token)?;
        let to_reserve = *self.reserves.get(to_token)?;

        // Simulate swap to get new reserves
        let new_from_reserve = from_reserve + amount_in;
        if new_from_reserve == 0.0 {
            return None;
        }
        let new_to_reserve = (from_reserve * to_reserve) / new_from_reserve;
        let new_price = new_to_reserve / new_from_reserve;
        Some((1.0 - new_price / old_price) * 100.0)
    }

    /// Swap with fee applied (multiply amount_in by (1-fee_rate)) and slippage check
    pub fn swap_with_fee(
        &mut self,
        input_token: &str,
        input_amount: f64,
        output_token: &str,
        fee_rate: f64,
        max_slippage: f64,
    ) -> Result<f64, String> {
        if input_amount <= 0.0 {
            return Err("Swap amount must be positive".into());
        }
        if input_token == output_token {
            return Err("Input and output tokens must be different".into());
        }
        if !self.reserves.contains_key(input_token) || !self.reserves.contains_key(output_token) {
            return Err("One or both tokens not found in the AMM".into());
        }

        let effective_amount = input_amount * (1.0 - fee_rate);

        // Check price impact before executing swap
        let price_impact = self
            .calculate_price_impact(input_token, output_token, effective_amount)
            .ok_or("Could not calculate price impact")?;

        if price_impact > max_slippage {
            return Err(format!(
                "Price impact {:.4}% exceeds max slippage {:.4}%",
                price_impact, max_slippage
            ));
        }

        self.swap(input_token, effective_amount, output_token)
    }
}

/// A simple constant-product LP pool with fee support
#[derive(Debug, Clone)]
pub struct LpPool {
    pub reserves: Vec<f64>,
    pub fee_rate: f64,
    pub max_slippage: f64,
    pub total_lp_tokens: f64,
}

impl LpPool {
    pub fn new(fee_rate: f64, max_slippage: f64) -> Self {
        Self {
            reserves: Vec::new(),
            fee_rate,
            max_slippage,
            total_lp_tokens: 0.0,
        }
    }

    /// Constant product swap with fee applied
    pub fn swap(&mut self, from_dim: usize, to_dim: usize, amount_in: f64) -> Result<f64, String> {
        if from_dim >= self.reserves.len() || to_dim >= self.reserves.len() {
            return Err("Dimension index out of bounds".into());
        }
        if from_dim == to_dim {
            return Err("Cannot swap a dimension with itself".into());
        }
        if amount_in <= 0.0 {
            return Err("Swap amount must be positive".into());
        }

        let effective_in = amount_in * (1.0 - self.fee_rate);
        let from_reserve = self.reserves[from_dim];
        let to_reserve = self.reserves[to_dim];

        if from_reserve == 0.0 || to_reserve == 0.0 {
            return Err("Pool has zero reserves".into());
        }

        // Check price impact
        let old_price = to_reserve / from_reserve;
        let new_from = from_reserve + effective_in;
        let new_to = (from_reserve * to_reserve) / new_from;
        let new_price = new_to / new_from;
        let impact = (1.0 - new_price / old_price) * 100.0;
        if impact > self.max_slippage {
            return Err(format!(
                "Price impact {:.4}% exceeds max slippage {:.4}%",
                impact, self.max_slippage
            ));
        }

        let amount_out = to_reserve - new_to;
        self.reserves[from_dim] = new_from;
        self.reserves[to_dim] = new_to;

        Ok(amount_out)
    }

    /// Add liquidity and return LP tokens minted (proportional share)
    pub fn add_liquidity(&mut self, reserves: &[f64]) -> Result<f64, String> {
        if reserves.is_empty() {
            return Err("Cannot add empty reserves".into());
        }
        for &r in reserves {
            if r < 0.0 {
                return Err("Reserve amounts must be non-negative".into());
            }
        }

        if self.reserves.is_empty() {
            // Initialize pool
            self.reserves = reserves.to_vec();
            // LP tokens = geometric mean of reserves for initialization
            let product: f64 = reserves.iter().copied().filter(|&r| r > 0.0).product();
            let lp_tokens = if product > 0.0 {
                product.powf(1.0 / reserves.len() as f64)
            } else {
                0.0
            };
            self.total_lp_tokens += lp_tokens;
            return Ok(lp_tokens);
        }

        if reserves.len() != self.reserves.len() {
            return Err("Reserve dimension mismatch".into());
        }

        // Calculate proportional share based on first non-zero reserve
        let first_nonzero = self
            .reserves
            .iter()
            .enumerate()
            .find(|(_, &r)| r > 0.0)
            .map(|(i, _)| i)
            .ok_or("Pool has all-zero reserves")?;

        let ratio = reserves[first_nonzero] / self.reserves[first_nonzero];
        let lp_tokens = self.total_lp_tokens * ratio;

        for (i, &amount) in reserves.iter().enumerate() {
            self.reserves[i] += amount;
        }
        self.total_lp_tokens += lp_tokens;

        Ok(lp_tokens)
    }

    /// Remove liquidity by burning LP tokens, returns withdrawn amounts
    pub fn remove_liquidity(&mut self, lp_tokens: f64) -> Result<Vec<f64>, String> {
        if lp_tokens <= 0.0 {
            return Err("LP token amount must be positive".into());
        }
        if lp_tokens > self.total_lp_tokens {
            return Err("Insufficient LP tokens".into());
        }
        if self.reserves.is_empty() {
            return Err("Pool is empty".into());
        }

        let share = lp_tokens / self.total_lp_tokens;
        let amounts: Vec<f64> = self.reserves.iter().map(|&r| r * share).collect();

        for (i, &amount) in amounts.iter().enumerate() {
            self.reserves[i] -= amount;
        }
        self.total_lp_tokens -= lp_tokens;

        Ok(amounts)
    }

    /// Spot price: how much of to_dim per unit of from_dim
    pub fn get_price(&self, from_dim: usize, to_dim: usize) -> Option<f64> {
        if from_dim >= self.reserves.len() || to_dim >= self.reserves.len() {
            return None;
        }
        let from_reserve = self.reserves[from_dim];
        if from_reserve == 0.0 {
            return None;
        }
        Some(self.reserves[to_dim] / from_reserve)
    }

    /// Price impact percentage for a given swap
    pub fn calculate_price_impact(&self, from_dim: usize, to_dim: usize, amount_in: f64) -> f64 {
        if from_dim >= self.reserves.len() || to_dim >= self.reserves.len() {
            return 0.0;
        }
        let from_reserve = self.reserves[from_dim];
        let to_reserve = self.reserves[to_dim];
        if from_reserve == 0.0 || to_reserve == 0.0 {
            return 0.0;
        }

        let old_price = to_reserve / from_reserve;
        let effective_in = amount_in * (1.0 - self.fee_rate);
        let new_from = from_reserve + effective_in;
        let new_to = (from_reserve * to_reserve) / new_from;
        let new_price = new_to / new_from;

        (1.0 - new_price / old_price) * 100.0
    }
}
