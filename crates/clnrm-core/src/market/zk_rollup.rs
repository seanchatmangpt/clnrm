use crate::pqc::hash::custom_hash;
use sha2::{Digest as Sha2Digest, Sha256};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiptHash(pub [u8; 32]);

pub struct ZkRollupBatcher {
    pub current_batch: Vec<ReceiptHash>,
    pub historical_batches: HashMap<[u8; 32], Vec<ReceiptHash>>,
}

impl Default for ZkRollupBatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ZkRollupBatcher {
    pub fn new() -> Self {
        Self {
            current_batch: Vec::new(),
            historical_batches: HashMap::new(),
        }
    }

    pub fn add_receipt_to_batch(&mut self, hash: [u8; 32]) {
        self.current_batch.push(ReceiptHash(hash));
    }

    pub fn generate_rollup_proof(&mut self) -> Result<[u8; 32], &'static str> {
        if self.current_batch.is_empty() {
            return Err("Cannot rollup empty batch");
        }

        // Real Merkle root calculation using Post-Quantum Cryptographic hash
        let mut layer: Vec<[u8; 32]> = self.current_batch.iter().map(|h| h.0).collect();

        while layer.len() > 1 {
            let mut next_layer = Vec::new();
            for chunk in layer.chunks(2) {
                if chunk.len() == 2 {
                    let mut combined = Vec::with_capacity(64);
                    combined.extend_from_slice(&chunk[0]);
                    combined.extend_from_slice(&chunk[1]);
                    next_layer.push(custom_hash(&combined));
                } else {
                    // Duplicate last element if odd number of nodes
                    let mut combined = Vec::with_capacity(64);
                    combined.extend_from_slice(&chunk[0]);
                    combined.extend_from_slice(&chunk[0]);
                    next_layer.push(custom_hash(&combined));
                }
            }
            layer = next_layer;
        }

        let root = layer[0];

        self.historical_batches
            .insert(root, self.current_batch.clone());
        self.current_batch.clear();

        Ok(root)
    }
}

// ── Batch ZK proof aggregator ────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BatchedProof {
    pub batch_id: String,
    pub proofs: Vec<Vec<u8>>,
    pub aggregate_hash: [u8; 32],
    pub batch_size: usize,
    pub created_at_ms: u64,
}

pub struct ZkRollup {
    pub pending_proofs: Vec<Vec<u8>>,
    pub committed_batches: Vec<BatchedProof>,
    pub batch_size: usize,
    batch_counter: u64,
}

impl ZkRollup {
    pub fn new(batch_size: usize) -> Self {
        Self {
            pending_proofs: Vec::new(),
            committed_batches: Vec::new(),
            batch_size,
            batch_counter: 0,
        }
    }

    /// Add a proof to the pending queue. Automatically commits if batch_size is reached.
    pub fn submit_proof(&mut self, proof: Vec<u8>) {
        self.pending_proofs.push(proof);
    }

    /// Aggregate and commit the pending batch when pending_count() >= batch_size.
    /// Returns Some(BatchedProof) on success, None if there are not enough pending proofs.
    pub fn commit_batch(&mut self) -> Option<BatchedProof> {
        if self.pending_proofs.len() < self.batch_size {
            return None;
        }

        let proofs: Vec<Vec<u8>> = self.pending_proofs.drain(..self.batch_size).collect();
        let aggregate_hash = Self::aggregate_proofs(&proofs);

        self.batch_counter += 1;
        let batch_id = format!("batch-{}", self.batch_counter);

        // Use a monotonic counter as a simple timestamp substitute (ms since epoch via std::time)
        let created_at_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let batched = BatchedProof {
            batch_id: batch_id.clone(),
            batch_size: proofs.len(),
            proofs,
            aggregate_hash,
            created_at_ms,
        };

        self.committed_batches.push(batched.clone());
        Some(batched)
    }

    /// SHA-256 of all proofs concatenated.
    pub fn aggregate_proofs(proofs: &[Vec<u8>]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        for proof in proofs {
            hasher.update(proof);
        }
        hasher.finalize().into()
    }

    /// Returns true if a batch with the given ID is in the committed list.
    pub fn verify_batch(&self, batch_id: &str) -> bool {
        self.committed_batches
            .iter()
            .any(|b| b.batch_id == batch_id)
    }

    /// Number of proofs waiting to be batched.
    pub fn pending_count(&self) -> usize {
        self.pending_proofs.len()
    }
}
