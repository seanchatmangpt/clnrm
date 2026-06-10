use crate::pqc::hash::custom_hash;
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
