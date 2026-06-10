use std::collections::HashMap;

/// Unique identifier for a Zero-Knowledge Proof task
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TaskId(pub [u8; 32]);

/// Unique identifier for a Prover node in the network
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProverId(pub [u8; 32]);

/// Represents a complex transaction that requires a Zero-Knowledge Proof.
#[derive(Debug, Clone)]
pub struct ZkTask {
    pub task_id: TaskId,
    pub payload_hash: [u8; 32],
    pub max_fee: u64,
    pub max_compute_time_ms: u64,
    pub created_at: u64,
    pub required_stake: u64,
}

/// Represents a bid from a Prover to generate a Zero-Knowledge Proof for a specific task.
#[derive(Debug, Clone)]
pub struct ProverBid {
    pub prover_id: ProverId,
    pub task_id: TaskId,
    pub fee: u64,
    pub estimated_compute_time_ms: u64,
    pub stake_provided: u64,
    pub submitted_at: u64,
}

/// Errors that can occur within the ZkMarket.
#[derive(Debug, PartialEq, Eq)]
pub enum MarketError {
    TaskNotFound,
    BidExceedsMaxFee,
    BidExceedsMaxComputeTime,
    InsufficientStake,
    TaskAlreadyAssigned,
    NoValidBids,
}

/// Represents a successfully assigned task to a prover.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskAssignment {
    pub task_id: TaskId,
    pub prover_id: ProverId,
    pub fee: u64,
    pub estimated_compute_time_ms: u64,
    pub assigned_at: u64,
}

/// Specialized orderbook for the ZkMarket.
pub struct ZkMarket {
    pub tasks: HashMap<TaskId, ZkTask>,
    pub bids: HashMap<TaskId, Vec<ProverBid>>,
    pub assignments: HashMap<TaskId, TaskAssignment>,
    pub fee_weight: f64,
    pub time_weight: f64,
}

impl Default for ZkMarket {
    fn default() -> Self {
        Self::new()
    }
}

impl ZkMarket {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            bids: HashMap::new(),
            assignments: HashMap::new(),
            fee_weight: 0.6,
            time_weight: 0.4,
        }
    }

    pub fn submit_task(&mut self, task: ZkTask) {
        self.tasks.insert(task.task_id.clone(), task);
    }

    pub fn submit_bid(&mut self, bid: ProverBid) -> Result<(), MarketError> {
        let task = self
            .tasks
            .get(&bid.task_id)
            .ok_or(MarketError::TaskNotFound)?;

        if self.assignments.contains_key(&bid.task_id) {
            return Err(MarketError::TaskAlreadyAssigned);
        }

        if bid.fee > task.max_fee {
            return Err(MarketError::BidExceedsMaxFee);
        }

        if bid.estimated_compute_time_ms > task.max_compute_time_ms {
            return Err(MarketError::BidExceedsMaxComputeTime);
        }

        if bid.stake_provided < task.required_stake {
            return Err(MarketError::InsufficientStake);
        }

        self.bids
            .entry(bid.task_id.clone())
            .or_insert_with(Vec::new)
            .push(bid);

        Ok(())
    }

    pub fn resolve_task(
        &mut self,
        task_id: &TaskId,
        current_time_ms: u64,
    ) -> Result<TaskAssignment, MarketError> {
        let task = self.tasks.get(task_id).ok_or(MarketError::TaskNotFound)?;

        if self.assignments.contains_key(task_id) {
            return Err(MarketError::TaskAlreadyAssigned);
        }

        let bids = self.bids.get(task_id).ok_or(MarketError::NoValidBids)?;
        if bids.is_empty() {
            return Err(MarketError::NoValidBids);
        }

        let mut best_bid: Option<&ProverBid> = None;
        let mut lowest_score = f64::MAX;

        for bid in bids {
            let normalized_fee = if task.max_fee > 0 {
                bid.fee as f64 / task.max_fee as f64
            } else {
                0.0
            };

            let normalized_time = if task.max_compute_time_ms > 0 {
                bid.estimated_compute_time_ms as f64 / task.max_compute_time_ms as f64
            } else {
                0.0
            };

            let score = self.fee_weight * normalized_fee + self.time_weight * normalized_time;

            if score < lowest_score {
                lowest_score = score;
                best_bid = Some(bid);
            } else if (score - lowest_score).abs() < f64::EPSILON {
                if let Some(best) = best_bid {
                    if bid.fee < best.fee {
                        best_bid = Some(bid);
                    }
                }
            }
        }

        let best_bid = best_bid.ok_or(MarketError::NoValidBids)?;

        let assignment = TaskAssignment {
            task_id: task_id.clone(),
            prover_id: best_bid.prover_id.clone(),
            fee: best_bid.fee,
            estimated_compute_time_ms: best_bid.estimated_compute_time_ms,
            assigned_at: current_time_ms,
        };

        self.assignments.insert(task_id.clone(), assignment.clone());

        Ok(assignment)
    }
}
