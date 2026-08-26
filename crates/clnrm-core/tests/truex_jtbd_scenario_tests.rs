use clnrm_core::config::{parse_toml_config, TestConfig};
use clnrm_core::market::amm::{AssetType, NDimensionalAMM};
use clnrm_core::pqc::hash::custom_hash;
use clnrm_core::truex::admission::{AdmissionKernel, GenerativeConstitution, ValidatedOntology};
use std::collections::HashMap;

#[tokio::test]
async fn test_jtbd_truex_amm_end_to_end() {
    // 1. Load the TOML configuration
    let config_content = include_str!("../../../examples/truex-amm-validation.clnrm.toml");
    let config: TestConfig = parse_toml_config(config_content).expect("Failed to parse TOML");

    // Validate config overrides loaded correctly
    assert!(config.truex.is_some(), "Truex config should be loaded");
    assert!(config.market.is_some(), "Market config should be loaded");
    assert_eq!(config.market.unwrap().target_inflation.unwrap(), 0.05);

    // 2. JTBD Phase 1: N-Dimensional AMM Liquidity Provisioning
    let mut amm = NDimensionalAMM::new();
    amm.register_asset("Token", AssetType::Token);
    amm.register_asset("ComputeContract", AssetType::ComputeContract);
    amm.register_asset("OracleDataPoint", AssetType::OracleDataPoint);

    let mut initial_liquidity = HashMap::new();
    initial_liquidity.insert("Token".to_string(), 1000.0);
    initial_liquidity.insert("ComputeContract".to_string(), 500.0);
    initial_liquidity.insert("OracleDataPoint".to_string(), 250.0);

    amm.add_liquidity(&initial_liquidity)
        .expect("Failed to add liquidity");

    // Verify N-Dimensional Invariant pricing mechanism is mathematically stable
    let k_initial = amm.invariant();
    assert!(k_initial > 0.0, "Invariant should be positive");

    // JTBD Phase 2: Compute execution requires a swap
    // Swap Tokens for ComputeContract execution
    let output = amm
        .swap("Token", 10.0, "ComputeContract")
        .expect("Failed to swap");
    assert!(output > 0.0, "Should receive ComputeContract capacity");

    // Invariant should remain constant (within floating point precision)
    let k_after_swap = amm.invariant();
    assert!(
        (k_initial - k_after_swap).abs() < 1e-4,
        "K invariant deviated"
    );

    // 3. JTBD Phase 3: Admission Kernel Validation (R ⊢ A = μ(O*))

    // Constitution 'R' is cryptographically established
    let receipt_hash = custom_hash(b"test_genesis_constitution");
    let constitution = GenerativeConstitution::new(receipt_hash);
    let mu = AdmissionKernel::new(constitution);

    // The Execution trace 'O*' is bundled into a validated ontology
    let trace_digest = custom_hash(b"CONSEQUENCE_GENERATED_TRACE");
    let ontology = ValidatedOntology::new(
        "law_001_execution".to_string(),
        1,
        "compute_consequence".to_string(),
        trace_digest,
    );

    // Actuate 'A' via Admission Kernel 'μ'
    let receipt = mu.evaluate(&ontology).expect("Admission failed");

    // 4. Verify End-to-End Cryptographic Consequence
    assert_eq!(receipt.actor_id, "admission_kernel");
    assert!(
        receipt.pqc_seal.starts_with("z:"),
        "PQC seal should be valid lattice output"
    );
    assert!(!receipt.closure_hash.is_empty());
}
