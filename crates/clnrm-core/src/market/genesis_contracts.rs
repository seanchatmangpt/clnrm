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
