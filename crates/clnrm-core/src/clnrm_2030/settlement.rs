use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EscrowLock {
    pub buyer_id: String,
    pub amount: f64,
    pub locked_at: u64,
}

pub struct SettlementEngine {
    pub locked_funds: HashMap<String, EscrowLock>,
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
        }
    }

    pub fn lock_funds(&mut self, transaction_id: &str, buyer_id: &str, amount: f64, timestamp: u64) {
        self.locked_funds.insert(transaction_id.to_string(), EscrowLock {
            buyer_id: buyer_id.to_string(),
            amount,
            locked_at: timestamp,
        });
    }

    pub fn execute_settlement(&mut self, transaction_id: &str) -> Result<f64, &'static str> {
        if let Some(lock) = self.locked_funds.remove(transaction_id) {
            // Transfer funds to treasury or seller (omitted for brevity)
            Ok(lock.amount)
        } else {
            Err("No locked funds found for transaction")
        }
    }
}
