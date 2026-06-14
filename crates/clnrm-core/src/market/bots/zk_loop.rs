use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct ZkProofRequest {
    pub circuit_id: String,
    pub public_inputs: Vec<i64>,
    pub witness: Vec<i64>,
    pub priority: u8,
}

#[derive(Debug, Clone)]
pub struct ZkProofResponse {
    pub circuit_id: String,
    pub proof: Vec<u8>,
    pub verification_key: Vec<u8>,
    pub generation_time_ms: u64,
}

#[derive(Debug, Default, Clone)]
pub struct ZkProofStats {
    pub proofs_generated: u64,
    pub total_time_ms: u64,
    pub failed_proofs: u64,
}

pub struct ZkLoop {
    pub queue: VecDeque<ZkProofRequest>,
    pub stats: ZkProofStats,
    pub batch_size: usize,
}

impl ZkLoop {
    pub fn new(batch_size: usize) -> Self {
        Self {
            queue: VecDeque::new(),
            stats: ZkProofStats::default(),
            batch_size,
        }
    }

    pub fn enqueue(&mut self, request: ZkProofRequest) {
        self.queue.push_back(request);
    }

    pub fn queue_depth(&self) -> usize {
        self.queue.len()
    }

    /// Process up to `batch_size` items. For each request, generate a deterministic proof
    /// as SHA-256(circuit_id bytes || public_inputs as little-endian i64 bytes).
    /// The verification_key is SHA-256(proof || circuit_id bytes).
    pub fn process_batch(&mut self) -> Vec<ZkProofResponse> {
        let count = self.batch_size.min(self.queue.len());
        let mut responses = Vec::with_capacity(count);

        for _ in 0..count {
            let req = match self.queue.pop_front() {
                Some(r) => r,
                None => break,
            };

            let start = Instant::now();

            // Build proof: SHA-256(circuit_id || public_inputs)
            let mut hasher = Sha256::new();
            hasher.update(req.circuit_id.as_bytes());
            for &input in &req.public_inputs {
                hasher.update(input.to_le_bytes());
            }
            let proof: Vec<u8> = hasher.finalize().to_vec();

            // Build verification key: SHA-256(proof || circuit_id)
            let mut vk_hasher = Sha256::new();
            vk_hasher.update(&proof);
            vk_hasher.update(req.circuit_id.as_bytes());
            let verification_key: Vec<u8> = vk_hasher.finalize().to_vec();

            let elapsed_ms = start.elapsed().as_millis() as u64;

            self.stats.proofs_generated += 1;
            self.stats.total_time_ms += elapsed_ms;

            responses.push(ZkProofResponse {
                circuit_id: req.circuit_id,
                proof,
                verification_key,
                generation_time_ms: elapsed_ms,
            });
        }

        responses
    }

    pub fn stats(&self) -> &ZkProofStats {
        &self.stats
    }

    /// Average proof generation time in milliseconds. Returns 0.0 if no proofs generated yet.
    pub fn average_proof_time_ms(&self) -> f64 {
        if self.stats.proofs_generated == 0 {
            return 0.0;
        }
        self.stats.total_time_ms as f64 / self.stats.proofs_generated as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_request(id: &str) -> ZkProofRequest {
        ZkProofRequest {
            circuit_id: id.to_string(),
            public_inputs: vec![1, 2, 3],
            witness: vec![10, 20],
            priority: 1,
        }
    }

    #[test]
    fn test_enqueue_and_depth() {
        let mut zk = ZkLoop::new(10);
        assert_eq!(zk.queue_depth(), 0);
        zk.enqueue(make_request("circuit-a"));
        zk.enqueue(make_request("circuit-b"));
        assert_eq!(zk.queue_depth(), 2);
    }

    #[test]
    fn test_process_batch_produces_responses() {
        let mut zk = ZkLoop::new(5);
        for i in 0..3 {
            zk.enqueue(make_request(&format!("circuit-{i}")));
        }
        let responses = zk.process_batch();
        assert_eq!(responses.len(), 3);
        assert_eq!(zk.queue_depth(), 0);
        assert_eq!(zk.stats().proofs_generated, 3);
    }

    #[test]
    fn test_proof_is_deterministic() {
        let req = make_request("deterministic-circuit");

        let mut zk1 = ZkLoop::new(1);
        zk1.enqueue(req.clone());
        let r1 = zk1.process_batch();

        let mut zk2 = ZkLoop::new(1);
        zk2.enqueue(req);
        let r2 = zk2.process_batch();

        assert_eq!(r1[0].proof, r2[0].proof);
        assert_eq!(r1[0].verification_key, r2[0].verification_key);
    }

    #[test]
    fn test_batch_size_respected() {
        let mut zk = ZkLoop::new(2);
        for i in 0..5 {
            zk.enqueue(make_request(&format!("c{i}")));
        }
        let responses = zk.process_batch();
        assert_eq!(responses.len(), 2);
        assert_eq!(zk.queue_depth(), 3);
    }

    #[test]
    fn test_average_proof_time_no_proofs() {
        let zk = ZkLoop::new(10);
        assert_eq!(zk.average_proof_time_ms(), 0.0);
    }
}
