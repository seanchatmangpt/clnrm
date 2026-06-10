use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct PeerId(pub String);

#[derive(Debug, Clone, Default)]
pub struct TrustRecord {
    pub successful_interactions: u64,
    pub failed_interactions: u64,
}

impl TrustRecord {
    pub fn new() -> Self {
        Self {
            successful_interactions: 0,
            failed_interactions: 0,
        }
    }

    pub fn net_score(&self) -> f64 {
        let sat = self.successful_interactions as f64;
        let unsat = self.failed_interactions as f64;
        let score = sat - unsat;
        if score > 0.0 {
            score
        } else {
            0.0
        }
    }
}

pub struct ReputationEngine {
    // source peer -> target peer -> interaction record
    interactions: HashMap<PeerId, HashMap<PeerId, TrustRecord>>,
    pre_trusted_peers: HashSet<PeerId>,
    alpha: f64,
    max_iterations: usize,
    tolerance: f64,
}

impl Default for ReputationEngine {
    fn default() -> Self {
        Self::new(0.15, 100, 1e-6)
    }
}

impl ReputationEngine {
    pub fn new(alpha: f64, max_iterations: usize, tolerance: f64) -> Self {
        Self {
            interactions: HashMap::new(),
            pre_trusted_peers: HashSet::new(),
            alpha,
            max_iterations,
            tolerance,
        }
    }

    pub fn add_pre_trusted_peer(&mut self, peer: PeerId) {
        self.pre_trusted_peers.insert(peer);
    }

    pub fn record_interaction(&mut self, source: PeerId, target: PeerId, success: bool) {
        let targets = self.interactions.entry(source).or_default();
        let record = targets.entry(target).or_default();
        if success {
            record.successful_interactions += 1;
        } else {
            record.failed_interactions += 1;
        }
    }

    pub fn get_all_peers(&self) -> HashSet<PeerId> {
        let mut peers = HashSet::new();
        for (source, targets) in &self.interactions {
            peers.insert(source.clone());
            for target in targets.keys() {
                peers.insert(target.clone());
            }
        }
        for peer in &self.pre_trusted_peers {
            peers.insert(peer.clone());
        }
        peers
    }

    /// Computes the global reputation score for all known peers using an EigenTrust-like algorithm.
    pub fn compute_reputation(&self) -> HashMap<PeerId, f64> {
        let peers: Vec<PeerId> = self.get_all_peers().into_iter().collect();
        let num_peers = peers.len();
        if num_peers == 0 {
            return HashMap::new();
        }

        let mut peer_indices: HashMap<&PeerId, usize> = HashMap::new();
        for (i, peer) in peers.iter().enumerate() {
            peer_indices.insert(peer, i);
        }

        // Build the local trust matrix C
        let mut c_matrix = vec![vec![0.0; num_peers]; num_peers];

        for (i, source_peer) in peers.iter().enumerate() {
            if let Some(targets) = self.interactions.get(source_peer) {
                let mut sum_scores = 0.0;
                for (target_peer, record) in targets {
                    let j = peer_indices[target_peer];
                    let score = record.net_score();
                    c_matrix[i][j] = score;
                    sum_scores += score;
                }

                // Normalize the row
                if sum_scores > 0.0 {
                    for j in 0..num_peers {
                        c_matrix[i][j] /= sum_scores;
                    }
                } else {
                    // If no valid trust, fallback to trusting pre-trusted peers equally
                    self.fallback_to_pre_trusted(&mut c_matrix[i], &peers, num_peers);
                }
            } else {
                // If no interactions, fallback to trusting pre-trusted peers equally
                self.fallback_to_pre_trusted(&mut c_matrix[i], &peers, num_peers);
            }
        }

        // Build the pre-trusted vector P
        let mut p_vector = vec![0.0; num_peers];
        let num_pre_trusted = self.pre_trusted_peers.len() as f64;
        if num_pre_trusted > 0.0 {
            for (i, peer) in peers.iter().enumerate() {
                if self.pre_trusted_peers.contains(peer) {
                    p_vector[i] = 1.0 / num_pre_trusted;
                }
            }
        } else {
            // If no pre-trusted peers, uniform distribution
            for i in 0..num_peers {
                p_vector[i] = 1.0 / (num_peers as f64);
            }
        }

        // Initialize trust vector T to P
        let mut t_vector = p_vector.clone();

        // Power iteration
        for _ in 0..self.max_iterations {
            let mut next_t = vec![0.0; num_peers];

            // t_{k+1} = (1 - alpha) * C^T * t_k + alpha * P
            for i in 0..num_peers {
                let mut sum = 0.0;
                for j in 0..num_peers {
                    sum += c_matrix[j][i] * t_vector[j]; // C^T multiplication
                }
                next_t[i] = (1.0 - self.alpha) * sum + self.alpha * p_vector[i];
            }

            // Check convergence
            let mut diff = 0.0;
            for i in 0..num_peers {
                diff += (next_t[i] - t_vector[i]).abs();
            }

            t_vector = next_t;

            if diff < self.tolerance {
                break;
            }
        }

        // Construct result
        let mut result = HashMap::new();
        for (i, peer) in peers.iter().enumerate() {
            result.insert(peer.clone(), t_vector[i]);
        }

        result
    }

    fn fallback_to_pre_trusted(&self, row: &mut Vec<f64>, peers: &[PeerId], num_peers: usize) {
        let num_pre_trusted = self.pre_trusted_peers.len() as f64;
        if num_pre_trusted > 0.0 {
            for (j, peer) in peers.iter().enumerate() {
                if self.pre_trusted_peers.contains(peer) {
                    row[j] = 1.0 / num_pre_trusted;
                }
            }
        } else {
            for j in 0..num_peers {
                row[j] = 1.0 / (num_peers as f64);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reputation_engine() {
        let mut engine = ReputationEngine::new(0.15, 100, 1e-6);
        let peer_a = PeerId("A".to_string());
        let peer_b = PeerId("B".to_string());
        let peer_c = PeerId("C".to_string());

        engine.add_pre_trusted_peer(peer_a.clone());

        // A trusts B
        engine.record_interaction(peer_a.clone(), peer_b.clone(), true);
        engine.record_interaction(peer_a.clone(), peer_b.clone(), true);

        // B trusts C
        engine.record_interaction(peer_b.clone(), peer_c.clone(), true);

        // C does some bad things to A
        engine.record_interaction(peer_c.clone(), peer_a.clone(), false);

        let reputation = engine.compute_reputation();

        assert!(reputation.contains_key(&peer_a));
        assert!(reputation.contains_key(&peer_b));
        assert!(reputation.contains_key(&peer_c));

        // Output something or just assert sum is 1.0
        let sum: f64 = reputation.values().sum();
        assert!((sum - 1.0).abs() < 1e-5);
    }
}
