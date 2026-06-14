use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct DimensionEdge {
    pub target_dimension: String,
    pub cost_weight: f64,
    pub pool_liquidity: f64,
}

pub struct InterDimensionalRouter {
    graph: HashMap<String, Vec<DimensionEdge>>,
}

impl Default for InterDimensionalRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl InterDimensionalRouter {
    pub fn new() -> Self {
        Self {
            graph: HashMap::new(),
        }
    }

    pub fn add_route(&mut self, from: &str, to: &str, cost: f64, liquidity: f64) {
        self.graph
            .entry(from.to_string())
            .or_default()
            .push(DimensionEdge {
                target_dimension: to.to_string(),
                cost_weight: cost,
                pool_liquidity: liquidity,
            });
    }

    pub fn find_arbitrage_path(&self, start: &str, amount: f64) -> Option<Vec<String>> {
        let mut distances: HashMap<String, f64> = HashMap::new();
        let mut predecessors: HashMap<String, String> = HashMap::new();

        for node in self.graph.keys() {
            distances.insert(node.clone(), f64::INFINITY);
        }
        distances.insert(start.to_string(), 0.0);

        let vertices = self.graph.keys().len();
        if vertices == 0 {
            return None;
        }

        for _ in 0..vertices {
            for (u, edges) in &self.graph {
                for edge in edges {
                    let u_dist = *distances.get(u).unwrap_or(&f64::INFINITY);
                    if u_dist == f64::INFINITY {
                        continue;
                    }

                    let cost = edge.cost_weight + (amount / edge.pool_liquidity);
                    let v_dist = *distances
                        .get(&edge.target_dimension)
                        .unwrap_or(&f64::INFINITY);

                    if u_dist + cost < v_dist {
                        distances.insert(edge.target_dimension.clone(), u_dist + cost);
                        predecessors.insert(edge.target_dimension.clone(), u.clone());
                    }
                }
            }
        }

        let mut cycle_start = None;
        for (u, edges) in &self.graph {
            for edge in edges {
                let u_dist = *distances.get(u).unwrap_or(&f64::INFINITY);
                if u_dist == f64::INFINITY {
                    continue;
                }
                let cost = edge.cost_weight + (amount / edge.pool_liquidity);
                let v_dist = *distances
                    .get(&edge.target_dimension)
                    .unwrap_or(&f64::INFINITY);

                if u_dist + cost < v_dist {
                    cycle_start = Some(edge.target_dimension.clone());
                    break;
                }
            }
            if cycle_start.is_some() {
                break;
            }
        }

        if let Some(start_node) = cycle_start {
            let mut curr = start_node;
            for _ in 0..vertices {
                if let Some(pred) = predecessors.get(&curr) {
                    curr = pred.clone();
                }
            }

            let mut cycle = vec![curr.clone()];
            let mut next = curr;
            let mut visited = HashSet::new();
            visited.insert(next.clone());

            while let Some(pred) = predecessors.get(&next) {
                let pred = pred.clone();
                if visited.contains(&pred) {
                    cycle.push(pred.clone());
                    cycle.reverse();
                    if let Some(pos) = cycle.iter().position(|x| x == &pred) {
                        return Some(cycle[pos..].to_vec());
                    }
                    return Some(cycle);
                }
                visited.insert(pred.clone());
                cycle.push(pred.clone());
                next = pred;
            }
        }

        None
    }

    /// Execute arbitrage across a path of nodes. Returns net profit (final - initial).
    /// Each hop: new_amount = old_amount * (pool_liquidity / (pool_liquidity + old_amount)) * (1.0 - cost_weight)
    pub fn execute_arbitrage(&self, path: &[String], amount: f64) -> Result<f64, String> {
        if path.len() < 2 {
            return Err("Path must have at least 2 nodes".into());
        }

        let mut current_amount = amount;

        for window in path.windows(2) {
            let from = &window[0];
            let to = &window[1];

            let edge = self
                .graph
                .get(from)
                .and_then(|edges| edges.iter().find(|e| e.target_dimension == *to))
                .ok_or_else(|| format!("No edge from {} to {}", from, to))?;

            current_amount = current_amount
                * (edge.pool_liquidity / (edge.pool_liquidity + current_amount))
                * (1.0 - edge.cost_weight);
        }

        Ok(current_amount - amount)
    }

    /// Calculate net profit from traversing a path, same logic as execute_arbitrage.
    pub fn calculate_profit(&self, path: &[String], amount: f64) -> f64 {
        self.execute_arbitrage(path, amount).unwrap_or(0.0)
    }

    /// Returns true if every pool on the path has liquidity >= min_amount.
    pub fn is_path_liquid(&self, path: &[String], min_amount: f64) -> bool {
        if path.len() < 2 {
            return false;
        }

        for window in path.windows(2) {
            let from = &window[0];
            let to = &window[1];

            let liquid = self
                .graph
                .get(from)
                .and_then(|edges| edges.iter().find(|e| e.target_dimension == *to))
                .map(|e| e.pool_liquidity >= min_amount)
                .unwrap_or(false);

            if !liquid {
                return false;
            }
        }

        true
    }
}
