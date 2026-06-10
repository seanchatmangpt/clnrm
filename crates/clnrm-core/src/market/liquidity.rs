use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct StakeInfo {
    pub amount: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct LiquidityIncentiveEngine {
    pub stakers: HashMap<String, StakeInfo>,
    pub total_staked: u64,
    pub reward_rate_per_second: u64,
    pub accumulated_reward_per_share: u128,
    pub last_update_time: u64,
    pub user_reward_debt: HashMap<String, u128>,
    pub user_accumulated_yield: HashMap<String, u64>,
}

impl LiquidityIncentiveEngine {
    pub fn new(reward_rate_per_second: u64) -> Self {
        Self {
            stakers: HashMap::new(),
            total_staked: 0,
            reward_rate_per_second,
            accumulated_reward_per_share: 0,
            last_update_time: Self::current_timestamp(),
            user_reward_debt: HashMap::new(),
            user_accumulated_yield: HashMap::new(),
        }
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Updates the reward variables for the pool to be up-to-date.
    pub fn update_pool(&mut self) {
        let now = Self::current_timestamp();
        self.update_pool_with_time(now);
    }

    /// Allows updating with a specific time for testing.
    pub fn update_pool_with_time(&mut self, now: u64) {
        if now <= self.last_update_time {
            return;
        }
        if self.total_staked == 0 {
            self.last_update_time = now;
            return;
        }
        let time_elapsed = now - self.last_update_time;
        let token_reward = time_elapsed as u128 * self.reward_rate_per_second as u128;

        // Precision factor to avoid rounding down to zero for small rewards / large stakes.
        let precision_factor: u128 = 1_000_000_000_000;
        self.accumulated_reward_per_share +=
            (token_reward * precision_factor) / self.total_staked as u128;
        self.last_update_time = now;
    }

    pub fn stake_tokens(&mut self, user_id: &str, amount: u64) {
        let now = Self::current_timestamp();
        self.stake_tokens_with_time(user_id, amount, now);
    }

    pub fn stake_tokens_with_time(&mut self, user_id: &str, amount: u64, now: u64) {
        self.update_pool_with_time(now);

        let precision_factor: u128 = 1_000_000_000_000;
        let mut user_amount = 0;

        if let Some(stake) = self.stakers.get(user_id) {
            user_amount = stake.amount;
            let user_debt = *self.user_reward_debt.get(user_id).unwrap_or(&0);

            let pending = ((user_amount as u128 * self.accumulated_reward_per_share)
                / precision_factor)
                .saturating_sub(user_debt);
            let accumulated = self
                .user_accumulated_yield
                .entry(user_id.to_string())
                .or_insert(0);
            *accumulated += pending as u64;
        }

        user_amount += amount;
        self.total_staked += amount;

        self.stakers.insert(
            user_id.to_string(),
            StakeInfo {
                amount: user_amount,
                timestamp: now,
            },
        );

        self.user_reward_debt.insert(
            user_id.to_string(),
            (user_amount as u128 * self.accumulated_reward_per_share) / precision_factor,
        );
    }

    pub fn unstake_tokens(&mut self, user_id: &str, amount: u64) -> Result<(), String> {
        let now = Self::current_timestamp();
        self.unstake_tokens_with_time(user_id, amount, now)
    }

    pub fn unstake_tokens_with_time(
        &mut self,
        user_id: &str,
        amount: u64,
        now: u64,
    ) -> Result<(), String> {
        self.update_pool_with_time(now);

        let precision_factor: u128 = 1_000_000_000_000;
        let stake = self.stakers.get_mut(user_id).ok_or("User has no stake")?;

        if stake.amount < amount {
            return Err("Insufficient staked amount".to_string());
        }

        let user_amount = stake.amount;
        let user_debt = *self.user_reward_debt.get(user_id).unwrap_or(&0);

        let pending = ((user_amount as u128 * self.accumulated_reward_per_share)
            / precision_factor)
            .saturating_sub(user_debt);
        let accumulated = self
            .user_accumulated_yield
            .entry(user_id.to_string())
            .or_insert(0);
        *accumulated += pending as u64;

        stake.amount -= amount;
        self.total_staked -= amount;
        stake.timestamp = now;

        let new_debt =
            (stake.amount as u128 * self.accumulated_reward_per_share) / precision_factor;
        self.user_reward_debt.insert(user_id.to_string(), new_debt);

        Ok(())
    }

    pub fn claim_yield(&mut self, user_id: &str) -> Result<u64, String> {
        let now = Self::current_timestamp();
        self.claim_yield_with_time(user_id, now)
    }

    pub fn claim_yield_with_time(&mut self, user_id: &str, now: u64) -> Result<u64, String> {
        self.update_pool_with_time(now);

        let precision_factor: u128 = 1_000_000_000_000;
        let stake_amount = self.stakers.get(user_id).map(|s| s.amount).unwrap_or(0);

        let mut pending = 0;
        if stake_amount > 0 {
            let user_debt = self.user_reward_debt.get_mut(user_id).unwrap();
            let current_share =
                (stake_amount as u128 * self.accumulated_reward_per_share) / precision_factor;
            pending = current_share.saturating_sub(*user_debt);
            *user_debt = current_share;
        }

        let accumulated_yield = self.user_accumulated_yield.get_mut(user_id);
        let mut total_yield = pending as u64;

        if let Some(acc) = accumulated_yield {
            total_yield += *acc;
            *acc = 0;
        }

        Ok(total_yield)
    }

    pub fn get_stake_amount(&self, user_id: &str) -> u64 {
        self.stakers.get(user_id).map(|s| s.amount).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stake_and_yield_with_time() {
        let mut engine = LiquidityIncentiveEngine::new(10); // 10 tokens per second
        engine.last_update_time = 1000;

        // User1 stakes 100 at time 1000
        engine.stake_tokens_with_time("user1", 100, 1000);

        // 1010: User1 claims
        let yield1 = engine.claim_yield_with_time("user1", 1010).unwrap();
        assert_eq!(yield1, 100);

        // 1010: User2 stakes 100
        engine.stake_tokens_with_time("user2", 100, 1010);

        // 1020: User1 and User2 claim
        let yield1_more = engine.claim_yield_with_time("user1", 1020).unwrap();
        let yield2 = engine.claim_yield_with_time("user2", 1020).unwrap();

        assert_eq!(yield1_more, 50);
        assert_eq!(yield2, 50);

        // 1030: User1 unstakes 50
        engine.unstake_tokens_with_time("user1", 50, 1030).unwrap();

        // 1040: Both claim
        let yield1_final = engine.claim_yield_with_time("user1", 1040).unwrap();
        let yield2_final = engine.claim_yield_with_time("user2", 1040).unwrap();

        assert_eq!(yield1_final, 83);
        assert_eq!(yield2_final, 116);
    }
}
