use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(pub String);

#[derive(Debug, Clone)]
pub struct ComputeContract {
    pub provider: NodeId,
    pub cpu_cores: u32,
    pub memory_mb: u64,
    pub bandwidth_mbps: u32,
    pub price_per_second: u64,
    pub available: bool,
}

// --- Task-based compute market ---

#[derive(Debug, Clone)]
pub struct ComputeTask {
    pub id: String,
    pub description: String,
    pub cpu_units: u64,
    pub memory_mb: u64,
    pub deadline_ms: u64,
    pub reward: f64,
    pub submitter: String,
}

#[derive(Debug, Clone)]
pub struct ComputeBid {
    pub task_id: String,
    pub bidder: String,
    pub price: f64,
    pub estimated_time_ms: u64,
}

#[derive(Debug, Clone)]
pub struct ComputeResult {
    pub task_id: String,
    pub output: Vec<u8>,
    pub execution_time_ms: u64,
    pub proof_hash: [u8; 32],
}

impl ComputeResult {
    /// Compute the expected proof_hash for a given output using SHA-256.
    pub fn compute_proof_hash(output: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(output);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Open,
    AwaitingResult { winner: String },
    Completed,
    NotFound,
}

pub struct ComputeMarket {
    // Legacy spot-price market
    inventory: HashMap<NodeId, ComputeContract>,
    base_demand_multiplier: f64,

    // Task-based auction market
    pub tasks: HashMap<String, ComputeTask>,
    pub bids: HashMap<String, Vec<ComputeBid>>,
    pub results: HashMap<String, ComputeResult>,
    pub balance: HashMap<String, f64>,
}

impl Default for ComputeMarket {
    fn default() -> Self {
        Self::new()
    }
}

impl ComputeMarket {
    pub fn new() -> Self {
        Self {
            inventory: HashMap::new(),
            base_demand_multiplier: 1.0,
            tasks: HashMap::new(),
            bids: HashMap::new(),
            results: HashMap::new(),
            balance: HashMap::new(),
        }
    }

    // ---- Legacy spot-price market ----

    pub fn list_compute(&mut self, contract: ComputeContract) {
        self.inventory.insert(contract.provider.clone(), contract);
    }

    pub fn update_network_utilization(&mut self, utilization_percentage: f64) {
        // Dynamic pricing model based on DAO utilization
        if utilization_percentage > 0.8 {
            self.base_demand_multiplier = 1.0 + (utilization_percentage - 0.8) * 5.0;
        // Surge pricing
        } else {
            self.base_demand_multiplier = 1.0;
        }
    }

    pub fn get_spot_price(&self, provider: &NodeId) -> Option<u64> {
        let contract = self.inventory.get(provider)?;
        Some((contract.price_per_second as f64 * self.base_demand_multiplier) as u64)
    }

    pub fn lease(&mut self, provider: &NodeId) -> Result<ComputeContract, &'static str> {
        let contract = self
            .inventory
            .get_mut(provider)
            .ok_or("Provider not found")?;
        if !contract.available {
            return Err("Compute resources currently leased");
        }
        contract.available = false;
        Ok(contract.clone())
    }

    // ---- Task-based auction market ----

    /// Deposit funds into a participant's balance.
    pub fn deposit(&mut self, account: &str, amount: f64) {
        *self.balance.entry(account.to_string()).or_insert(0.0) += amount;
    }

    /// Submit a new compute task to the market. The submitter's reward is escrowed.
    pub fn submit_task(&mut self, task: ComputeTask) -> Result<(), &'static str> {
        let submitter_balance = self.balance.entry(task.submitter.clone()).or_insert(0.0);
        if *submitter_balance < task.reward {
            return Err("Insufficient balance to escrow reward");
        }
        *submitter_balance -= task.reward;
        self.bids.entry(task.id.clone()).or_default();
        self.tasks.insert(task.id.clone(), task);
        Ok(())
    }

    /// Place a bid on an open task.
    pub fn place_bid(&mut self, bid: ComputeBid) -> Result<(), &'static str> {
        if !self.tasks.contains_key(&bid.task_id) {
            return Err("Task not found");
        }
        if self.results.contains_key(&bid.task_id) {
            return Err("Task already completed");
        }
        self.bids.entry(bid.task_id.clone()).or_default().push(bid);
        Ok(())
    }

    /// Select the winning bid (lowest price). Returns the winning bidder's name.
    pub fn select_winner(&self, task_id: &str) -> Result<String, &'static str> {
        if !self.tasks.contains_key(task_id) {
            return Err("Task not found");
        }
        let bids = self.bids.get(task_id).ok_or("No bids found")?;
        if bids.is_empty() {
            return Err("No bids for this task");
        }
        let winner = bids
            .iter()
            .min_by(|a, b| {
                a.price
                    .partial_cmp(&b.price)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or("No bids found")?;
        Ok(winner.bidder.clone())
    }

    /// Submit a result for a task. Verifies proof_hash is non-zero and equals SHA-256(output).
    pub fn submit_result(&mut self, result: ComputeResult) -> Result<(), &'static str> {
        if !self.tasks.contains_key(&result.task_id) {
            return Err("Task not found");
        }
        if result.proof_hash == [0u8; 32] {
            return Err("Proof hash must be non-zero");
        }
        let expected_hash = ComputeResult::compute_proof_hash(&result.output);
        if result.proof_hash != expected_hash {
            return Err("Proof hash does not match SHA-256 of output");
        }
        self.results.insert(result.task_id.clone(), result);
        Ok(())
    }

    /// Claim the reward for a completed task. Transfers escrowed reward to the winner's balance.
    pub fn claim_reward(&mut self, task_id: &str, claimant: &str) -> Result<f64, &'static str> {
        if !self.results.contains_key(task_id) {
            return Err("No result submitted for task");
        }
        let winner = self.select_winner(task_id)?;
        if winner != claimant {
            return Err("Claimant is not the winning bidder");
        }
        let task = self.tasks.get(task_id).ok_or("Task not found")?;
        let reward = task.reward;
        *self.balance.entry(claimant.to_string()).or_insert(0.0) += reward;
        Ok(reward)
    }

    /// Returns the current status of a task.
    pub fn get_task_status(&self, task_id: &str) -> TaskStatus {
        if !self.tasks.contains_key(task_id) {
            return TaskStatus::NotFound;
        }
        if self.results.contains_key(task_id) {
            return TaskStatus::Completed;
        }
        match self.select_winner(task_id) {
            Ok(winner) => TaskStatus::AwaitingResult { winner },
            Err(_) => TaskStatus::Open,
        }
    }
}
