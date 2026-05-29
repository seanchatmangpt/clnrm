use std::collections::HashMap;

/// An N-Dimensional Automated Market Maker using the invariant product surface (x1 * x2 * ... * xn = k).
#[derive(Debug, Clone, Default)]
pub struct NDimensionalAMM {
    reserves: HashMap<String, f64>,
}

impl NDimensionalAMM {
    pub fn new() -> Self {
        Self {
            reserves: HashMap::new(),
        }
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
    pub fn swap(&mut self, input_token: &str, input_amount: f64, output_token: &str) -> Result<f64, String> {
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
}