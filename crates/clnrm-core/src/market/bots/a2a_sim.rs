use crate::market::a2a::{AgentBid, AgentId, AgentOrderbook, AgentTask, TaskType};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::sync::Arc;
use tokio::sync::RwLock;

pub async fn run_a2a_simulation() {
    let orderbook = Arc::new(RwLock::new(AgentOrderbook::new()));
    let task_ids = Arc::new(RwLock::new(Vec::<[u8; 32]>::new()));
    let mut handles = vec![];

    // Spawn 100 virtual AI agents concurrently
    for _ in 0..100 {
        let ob_clone = Arc::clone(&orderbook);
        let task_ids_clone = Arc::clone(&task_ids);

        let handle = tokio::spawn(async move {
            let mut rng = StdRng::from_os_rng();
            // Each virtual agent has a unique 32-byte ID
            let agent_id_bytes: [u8; 32] = rand::random();
            let agent_id = AgentId(agent_id_bytes);

            // Each agent performs 100 random actions in the high-frequency network
            for _ in 0..100 {
                let action = rng.random_range(0..100);

                if action < 30 {
                    // 30% chance to generate and post a new AgentTask
                    let task_id: [u8; 32] = rand::random();
                    let task_type = match rng.random_range(0..4) {
                        0 => TaskType::Compute,
                        1 => TaskType::Verification,
                        2 => TaskType::Search,
                        _ => TaskType::Arbitrage,
                    };

                    let task = AgentTask {
                        task_id,
                        requester: agent_id.clone(),
                        task_type,
                        max_price: rng.random_range(100..1000),
                        min_reputation: rng.random_range(0.0..10.0),
                    };

                    {
                        let mut ob = ob_clone.write().await;
                        ob.post_task(task);
                    }

                    {
                        let mut t_ids = task_ids_clone.write().await;
                        t_ids.push(task_id);
                    }
                } else if action < 80 {
                    // 50% chance to generate an AgentBid for an existing task
                    let t_ids = task_ids_clone.read().await;
                    if !t_ids.is_empty() {
                        let idx = rng.random_range(0..t_ids.len());
                        let target_task_id = t_ids[idx];

                        let bid = AgentBid {
                            bidder: agent_id.clone(),
                            price: rng.random_range(50..950), // Competitive bidding
                            reputation: rng.random_range(5.0..15.0),
                        };

                        // Drop read lock before acquiring write lock
                        drop(t_ids);

                        let mut ob = ob_clone.write().await;
                        let _ = ob.submit_bid(target_task_id, bid);
                    }
                } else {
                    // 20% chance to trigger task matching
                    let target_task_id = {
                        let t_ids = task_ids_clone.read().await;
                        if t_ids.is_empty() {
                            None
                        } else {
                            let idx = rng.random_range(0..t_ids.len());
                            Some(t_ids[idx])
                        }
                    };

                    if let Some(id) = target_task_id {
                        let mut ob = ob_clone.write().await;
                        let _ = ob.match_task(id);
                    }
                }

                // Process ticks without artificial simulation latency
                tokio::time::sleep(std::time::Duration::from_millis(rng.random_range(1..10))).await;
            }
        });
        handles.push(handle);
    }

    // Await all AI agents
    for handle in handles {
        let _ = handle.await;
    }
}
