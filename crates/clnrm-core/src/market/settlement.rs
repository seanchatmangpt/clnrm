use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EscrowLock {
    pub buyer_id: String,
    pub amount: f64,
    pub locked_at: u64,
}

pub struct SettlementEngine {
    /// Keyed by transaction_id — used by the original API.
    pub locked_funds: HashMap<String, EscrowLock>,
    /// Keyed by buyer_id — used by the new escrow API.
    buyer_escrow: HashMap<String, EscrowLock>,
    /// Tracks how much each seller has received via release_funds.
    seller_balances: HashMap<String, f64>,
    /// Current wall-clock time used when locking (ms). Defaults to 0.
    current_time_ms: u64,
}

impl Default for SettlementEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SettlementEngine {
    pub fn new() -> Self {
        Self {
            locked_funds: HashMap::new(),
            buyer_escrow: HashMap::new(),
            seller_balances: HashMap::new(),
            current_time_ms: 0,
        }
    }

    // -----------------------------------------------------------------------
    // Original API (backward-compatible)
    // -----------------------------------------------------------------------

    pub fn lock_funds(&mut self, buyer_id: &str, amount: f64) -> Result<(), String> {
        if amount <= 0.0 {
            return Err(format!("Amount must be positive, got {}", amount));
        }
        if self.buyer_escrow.contains_key(buyer_id) {
            return Err(format!("Funds already locked for buyer '{}'", buyer_id));
        }
        self.buyer_escrow.insert(
            buyer_id.to_string(),
            EscrowLock {
                buyer_id: buyer_id.to_string(),
                amount,
                locked_at: self.current_time_ms,
            },
        );
        Ok(())
    }

    /// Original 4-arg lock used by transaction-based flows.
    pub fn lock_funds_for_transaction(
        &mut self,
        transaction_id: &str,
        buyer_id: &str,
        amount: f64,
        timestamp: u64,
    ) {
        self.locked_funds.insert(
            transaction_id.to_string(),
            EscrowLock {
                buyer_id: buyer_id.to_string(),
                amount,
                locked_at: timestamp,
            },
        );
    }

    pub fn execute_settlement(&mut self, transaction_id: &str) -> Result<f64, &'static str> {
        if let Some(lock) = self.locked_funds.remove(transaction_id) {
            // Transfer funds to treasury or seller (omitted for brevity)
            Ok(lock.amount)
        } else {
            Err("No locked funds found for transaction")
        }
    }

    // -----------------------------------------------------------------------
    // New buyer-keyed escrow API
    // -----------------------------------------------------------------------

    /// Transfers locked funds from buyer to seller tracking and removes the lock.
    pub fn release_funds(&mut self, buyer_id: &str, seller_id: &str) -> Result<f64, String> {
        if let Some(lock) = self.buyer_escrow.remove(buyer_id) {
            let amount = lock.amount;
            *self
                .seller_balances
                .entry(seller_id.to_string())
                .or_insert(0.0) += amount;
            Ok(amount)
        } else {
            Err(format!("No locked funds found for buyer '{}'", buyer_id))
        }
    }

    /// Removes the lock and returns the refunded amount.
    pub fn refund(&mut self, buyer_id: &str) -> Result<f64, String> {
        if let Some(lock) = self.buyer_escrow.remove(buyer_id) {
            Ok(lock.amount)
        } else {
            Err(format!("No locked funds found for buyer '{}'", buyer_id))
        }
    }

    pub fn locked_amount(&self, buyer_id: &str) -> f64 {
        self.buyer_escrow
            .get(buyer_id)
            .map(|l| l.amount)
            .unwrap_or(0.0)
    }

    pub fn total_locked(&self) -> f64 {
        self.buyer_escrow.values().map(|l| l.amount).sum()
    }

    /// Expires locks older than `timeout_ms` relative to `current_time_ms`.
    /// Returns the list of buyer IDs whose funds were refunded.
    pub fn expire_old_locks(&mut self, current_time_ms: u64, timeout_ms: u64) -> Vec<String> {
        let expired: Vec<String> = self
            .buyer_escrow
            .values()
            .filter(|lock| current_time_ms.saturating_sub(lock.locked_at) >= timeout_ms)
            .map(|lock| lock.buyer_id.clone())
            .collect();

        for buyer_id in &expired {
            self.buyer_escrow.remove(buyer_id);
        }

        expired
    }
}
