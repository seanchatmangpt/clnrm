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

        // Mock Merkle root calculation for the batch
        let mut root = [0u8; 32];
        for (i, hash) in self.current_batch.iter().enumerate() {
            root[i % 32] ^= hash.0[i % 32];
        }

        self.historical_batches.insert(root, self.current_batch.clone());
        self.current_batch.clear();

        Ok(root)
    }
}
