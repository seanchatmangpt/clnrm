pub struct GenesisBlock {
    pub timestamp: u64,
    pub initial_supply: f64,
}

impl GenesisBlock {
    pub fn mint_genesis() -> Self {
        Self {
            timestamp: 0,
            initial_supply: 1_000_000_000.0,
        }
    }
}
