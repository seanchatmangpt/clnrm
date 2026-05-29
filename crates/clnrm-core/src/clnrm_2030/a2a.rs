use std::collections::{HashMap, BinaryHeap};
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AgentId(pub [u8; 32]);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskType {
    Compute,
    Verification,
    Search,
    Arbitrage,
}

#[derive(Debug, Clone)]
pub struct AgentTask {
    pub task_id: [u8; 32],
    pub requester: AgentId,
    pub task_type: TaskType,
    pub max_price: u64,
    pub min_reputation: f64,
}

#[derive(Debug, Clone)]
pub struct AgentBid {
    pub bidder: AgentId,
    pub price: u64,
    pub reputation: f64,
}

// Order bids by lowest price, then highest reputation
impl Ord for AgentBid {
    fn cmp(&self, other: &Self) -> Ordering {
        other.price.cmp(&self.price)
            .then_with(|| self.reputation.partial_cmp(&other.reputation).unwrap_or(Ordering::Equal))
    }
}

impl PartialOrd for AgentBid {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for AgentBid {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price && self.reputation == other.reputation && self.bidder == other.bidder
    }
}
impl Eq for AgentBid {}

pub struct AgentOrderbook {
    tasks: HashMap<[u8; 32], AgentTask>,
    bids: HashMap<[u8; 32], BinaryHeap<AgentBid>>,
}

impl Default for AgentOrderbook {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentOrderbook {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            bids: HashMap::new(),
        }
    }

    pub fn post_task(&mut self, task: AgentTask) {
        let task_id = task.task_id;
        self.tasks.insert(task_id, task);
        self.bids.insert(task_id, BinaryHeap::new());
    }

    pub fn submit_bid(&mut self, task_id: [u8; 32], bid: AgentBid) -> Result<(), &'static str> {
        let task = self.tasks.get(&task_id).ok_or("Task not found")?;
        
        if bid.price > task.max_price {
            return Err("Bid price exceeds max task price");
        }
        if bid.reputation < task.min_reputation {
            return Err("Bidder reputation below task minimum");
        }
        
        if let Some(heap) = self.bids.get_mut(&task_id) {
            heap.push(bid);
        }
        Ok(())
    }

    pub fn match_task(&mut self, task_id: [u8; 32]) -> Option<(AgentTask, AgentBid)> {
        let task = self.tasks.remove(&task_id)?;
        let mut heap = self.bids.remove(&task_id)?;
        
        if let Some(winning_bid) = heap.pop() {
            Some((task, winning_bid))
        } else {
            None
        }
    }
}