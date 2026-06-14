use sha2::{Digest, Sha256};

pub struct GenesisBlock {
    pub timestamp: u64,
    pub initial_supply: f64,
    pub validator_set: Vec<String>,
    pub merkle_root: [u8; 32],
    pub chain_id: u64,
}

impl GenesisBlock {
    pub fn mint_genesis() -> Self {
        Self {
            timestamp: 0,
            initial_supply: 1_000_000_000.0,
            validator_set: Vec::new(),
            merkle_root: [0u8; 32],
            chain_id: 1,
        }
    }

    pub fn with_validators(validators: Vec<String>) -> Self {
        let mut block = Self {
            timestamp: 0,
            initial_supply: 1_000_000_000.0,
            validator_set: validators,
            merkle_root: [0u8; 32],
            chain_id: 1,
        };
        block.compute_merkle_root();
        block
    }

    /// SHA-256 over sorted validator addresses concatenated.
    pub fn compute_merkle_root(&mut self) {
        let mut sorted = self.validator_set.clone();
        sorted.sort();
        let mut hasher = Sha256::new();
        for addr in &sorted {
            hasher.update(addr.as_bytes());
        }
        let result = hasher.finalize();
        self.merkle_root.copy_from_slice(&result);
    }

    /// Returns true if initial_supply > 0 and there is at least 1 validator.
    pub fn validate(&self) -> bool {
        self.initial_supply > 0.0 && !self.validator_set.is_empty()
    }

    /// SHA-256 of timestamp (8 bytes LE) + initial_supply (8 bytes LE) + merkle_root (32 bytes).
    pub fn genesis_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.timestamp.to_le_bytes());
        hasher.update(self.initial_supply.to_le_bytes());
        hasher.update(self.merkle_root);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}
