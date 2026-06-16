use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Helper functions to compile/build bytecode for genesis contracts.
pub fn build_erc20_contract() -> Vec<u8> {
    // Return genesis ERC20 bytecode representation
    vec![0xaa, 0xbb, 0xcc, 0xdd, 0x11, 0x22, 0x33, 0x44]
}

pub fn build_dao_voting_contract() -> Vec<u8> {
    // Return genesis DAO voting contract bytecode representation
    vec![0xde, 0xad, 0xbe, 0xef, 0x55, 0x66, 0x77, 0x88]
}

pub fn build_vault_contract() -> Vec<u8> {
    // Return genesis Vault contract bytecode representation
    vec![0xfe, 0xed, 0xfa, 0xce, 0x99, 0xaa, 0xbb, 0xcc]
}

pub struct ContractAbi {
    pub name: String,
    pub bytecode: Vec<u8>,
    pub abi_hash: [u8; 32],
}

impl ContractAbi {
    /// Creates a new ContractAbi; abi_hash is SHA-256(bytecode).
    pub fn new(name: &str, bytecode: Vec<u8>) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(&bytecode);
        let result = hasher.finalize();
        let mut abi_hash = [0u8; 32];
        abi_hash.copy_from_slice(&result);
        Self {
            name: name.to_string(),
            bytecode,
            abi_hash,
        }
    }
}

pub struct ContractRegistry {
    pub contracts: HashMap<String, ContractAbi>,
}

impl ContractRegistry {
    pub fn new() -> Self {
        Self {
            contracts: HashMap::new(),
        }
    }

    /// Registers the three genesis contracts: ERC20, DAO, and Vault.
    pub fn register_genesis_contracts(&mut self) {
        let erc20 = ContractAbi::new("ERC20", build_erc20_contract());
        let dao = ContractAbi::new("DAO", build_dao_voting_contract());
        let vault = ContractAbi::new("Vault", build_vault_contract());
        self.contracts.insert(erc20.name.clone(), erc20);
        self.contracts.insert(dao.name.clone(), dao);
        self.contracts.insert(vault.name.clone(), vault);
    }

    pub fn get(&self, name: &str) -> Option<&ContractAbi> {
        self.contracts.get(name)
    }

    /// Returns the abi_hash for every registered contract.
    pub fn all_hashes(&self) -> Vec<[u8; 32]> {
        self.contracts.values().map(|c| c.abi_hash).collect()
    }
}

impl Default for ContractRegistry {
    fn default() -> Self {
        Self::new()
    }
}
