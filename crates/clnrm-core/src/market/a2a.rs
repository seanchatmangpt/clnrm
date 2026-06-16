use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

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
        other.price.cmp(&self.price).then_with(|| {
            self.reputation
                .partial_cmp(&other.reputation)
                .unwrap_or(Ordering::Equal)
        })
    }
}

impl PartialOrd for AgentBid {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for AgentBid {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price
            && self.reputation == other.reputation
            && self.bidder == other.bidder
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

        heap.pop().map(|winning_bid| (task, winning_bid))
    }
}

// ─── Proper Order Book ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum OrderSide {
    Bid,
    Ask,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: u64,
    pub side: OrderSide,
    pub price: f64,
    pub quantity: f64,
    pub timestamp: u64,
    pub agent_id: [u8; 32],
}

#[derive(Debug, Clone)]
pub struct Trade {
    pub bid_order_id: u64,
    pub ask_order_id: u64,
    pub price: f64,
    pub quantity: f64,
}

/// Wrapper for bids in max-heap (highest price wins).
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct BidOrder {
    price: f64,
    quantity: f64,
    timestamp: u64,
    id: u64,
    agent_id: [u8; 32],
}

impl PartialEq for BidOrder {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for BidOrder {}

impl PartialOrd for BidOrder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BidOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        // Max-heap: higher price first; tie-break by earlier timestamp (lower)
        self.price
            .partial_cmp(&other.price)
            .unwrap_or(Ordering::Equal)
            .then_with(|| other.timestamp.cmp(&self.timestamp))
    }
}

/// Wrapper for asks in min-heap (lowest price wins). We invert ordering.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct AskOrder {
    price: f64,
    quantity: f64,
    timestamp: u64,
    id: u64,
    agent_id: [u8; 32],
}

impl PartialEq for AskOrder {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for AskOrder {}

impl PartialOrd for AskOrder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AskOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        // Min-heap: lower price first (invert comparison); tie-break by earlier timestamp
        other
            .price
            .partial_cmp(&self.price)
            .unwrap_or(Ordering::Equal)
            .then_with(|| other.timestamp.cmp(&self.timestamp))
    }
}

pub struct OrderBook {
    bids: BinaryHeap<BidOrder>,
    asks: BinaryHeap<AskOrder>,
    next_order_id: u64,
    trades: Vec<Trade>,
}

impl Default for OrderBook {
    fn default() -> Self {
        Self::new()
    }
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: BinaryHeap::new(),
            asks: BinaryHeap::new(),
            next_order_id: 1,
            trades: Vec::new(),
        }
    }

    /// Place an order and return its assigned ID.
    pub fn place_order(
        &mut self,
        side: OrderSide,
        price: f64,
        quantity: f64,
        agent_id: [u8; 32],
        timestamp: u64,
    ) -> u64 {
        let id = self.next_order_id;
        self.next_order_id += 1;

        match side {
            OrderSide::Bid => self.bids.push(BidOrder {
                price,
                quantity,
                timestamp,
                id,
                agent_id,
            }),
            OrderSide::Ask => self.asks.push(AskOrder {
                price,
                quantity,
                timestamp,
                id,
                agent_id,
            }),
        }

        id
    }

    /// Match orders with price-time priority. Returns new trades from this round.
    pub fn match_orders(&mut self) -> Vec<Trade> {
        let mut new_trades = Vec::new();

        loop {
            // Peek at best bid and best ask
            let best_bid_price = match self.bids.peek() {
                Some(b) => b.price,
                None => break,
            };
            let best_ask_price = match self.asks.peek() {
                Some(a) => a.price,
                None => break,
            };

            if best_bid_price < best_ask_price {
                break; // No match possible
            }

            // Pop both
            let mut bid = self.bids.pop().unwrap(); // OK: Safe unwrap - bids is non-empty (checked via peek above)
            let mut ask = self.asks.pop().unwrap(); // OK: Safe unwrap - asks is non-empty (checked via peek above)

            // Fill at ask price (price-time priority)
            let fill_qty = bid.quantity.min(ask.quantity);
            let fill_price = ask.price;

            new_trades.push(Trade {
                bid_order_id: bid.id,
                ask_order_id: ask.id,
                price: fill_price,
                quantity: fill_qty,
            });

            bid.quantity -= fill_qty;
            ask.quantity -= fill_qty;

            // Re-insert partial fills
            if bid.quantity > 0.0 {
                self.bids.push(bid);
            }
            if ask.quantity > 0.0 {
                self.asks.push(ask);
            }
        }

        self.trades.extend(new_trades.clone());
        new_trades
    }

    pub fn get_best_bid(&self) -> Option<f64> {
        self.bids.peek().map(|b| b.price)
    }

    pub fn get_best_ask(&self) -> Option<f64> {
        self.asks.peek().map(|a| a.price)
    }

    pub fn get_trades(&self) -> &Vec<Trade> {
        &self.trades
    }
}
