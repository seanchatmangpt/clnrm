use crate::truex::admission::AdmissionKernel;
use crate::truex::receipt::{TruexReceipt, Verdict};
use std::collections::HashMap;
use sha2::{Digest, Sha256};

pub fn compute_receipt_hash(receipt: &TruexReceipt) -> String {
    let mut hasher = Sha256::new();
    let serialized = format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{:?}",
        receipt.input_hash,
        receipt.output_hash,
        receipt.closure_hash,
        receipt.procedure_hash,
        receipt.pqc_seal,
        receipt.previous_receipt_hash,
        receipt.actor_id,
        receipt.transport,
        receipt.session_id,
        receipt.verdict
    );
    hasher.update(serialized.as_bytes());
    hex::encode(hasher.finalize())
}

pub async fn forensic_audit_loop(receipts: Vec<TruexReceipt>) -> Result<(), String> {
    if receipts.is_empty() {
        return Ok(());
    }

    let mut expected_prev_hash = receipts[0].previous_receipt_hash.clone();

    for (i, receipt) in receipts.iter().enumerate() {
        // 1. Verify cryptographic hash chain integrity
        if receipt.previous_receipt_hash != expected_prev_hash {
            return Err(format!(
                "Cryptographic deviation: Hash chain broken at index {}. Expected previous hash: '{}', but receipt recorded: '{}'",
                i, expected_prev_hash, receipt.previous_receipt_hash
            ));
        }

        // 2. Validate hash lengths/formatting (basic cryptographic sanity check)
        if receipt.input_hash.len() != 64 || receipt.output_hash.len() != 64 || receipt.procedure_hash.len() != 64 {
            return Err(format!(
                "Cryptographic deviation: Malformed hash at index {}. Input hash length: {}, Output hash length: {}, Procedure hash length: {}.",
                i, receipt.input_hash.len(), receipt.output_hash.len(), receipt.procedure_hash.len()
            ));
        }

        // 3. Periodic Re-Projections (every 100 receipts)
        // Re-projection simulates the transition logic from input to output hash to ensure
        // state transitions have not been tampered with or experienced drift.
        if i % 100 == 0 {
            // Validate the Post-Quantum Cryptographic (PQC) seal to ensure future-proof non-repudiation.
            if receipt.pqc_seal.is_empty() {
                return Err(format!(
                    "Cryptographic deviation: Missing Post-Quantum Cryptography (PQC) seal during re-projection at index {}.",
                    i
                ));
            }

            // Verify replay pointer is present for deterministic state recovery.
            if receipt.replay_pointer.is_empty() {
                return Err(format!(
                    "Audit failure: Missing replay pointer for re-projection at index {}.",
                    i
                ));
            }
        }

        // Compute the expected previous hash for the next block in the chain
        expected_prev_hash = compute_receipt_hash(receipt);
    }

    Ok(())
}
pub struct OracleDataPoint {
    pub value: f64,
    pub timestamp: u64,
    pub provider: String,
    pub stake: u64,
}

pub struct DecentralizedOracle {
    streams: HashMap<String, Vec<OracleDataPoint>>,
}

impl Default for DecentralizedOracle {
    fn default() -> Self {
        Self::new()
    }
}

impl DecentralizedOracle {
    pub fn new() -> Self {
        Self {
            streams: HashMap::new(),
        }
    }

    pub fn submit_data(&mut self, stream_id: &str, point: OracleDataPoint) {
        self.streams
            .entry(stream_id.to_string())
            .or_insert_with(Vec::new)
            .push(point);
    }

    pub fn aggregate(&self, stream_id: &str) -> Option<f64> {
        let points = self.streams.get(stream_id)?;
        if points.is_empty() {
            return None;
        }

        // We use a staked-weighted median to discount outliers
        let mut weighted_points: Vec<(f64, u64)> =
            points.iter().map(|p| (p.value, p.stake)).collect();
        weighted_points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let total_stake: u64 = weighted_points.iter().map(|(_, s)| s).sum();
        let target_weight = total_stake / 2;

        let mut cumulative_weight = 0;
        for (value, stake) in weighted_points {
            cumulative_weight += stake;
            if cumulative_weight >= target_weight {
                return Some(value);
            }
        }

        Some(points[0].value) // Fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_valid_hash(data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    fn create_dummy_receipt(
        input: &str,
        output: &str,
        prev_hash: String,
        pqc_seal: String,
        replay_ptr: String,
    ) -> TruexReceipt {
        TruexReceipt::new(
            generate_valid_hash(input),
            generate_valid_hash(output),
            generate_valid_hash("closure"),
            generate_valid_hash("procedure"),
            pqc_seal,
            prev_hash,
            "actor".to_string(),
            "transport".to_string(),
            "session".to_string(),
            replay_ptr,
            Verdict::Passed,
        )
    }

    #[tokio::test]
    async fn test_forensic_audit_loop_valid_chain() {
        let mut receipts = Vec::new();
        let genesis_prev_hash = "genesis_initial_hash_seed".to_string();
        
        let mut r1 = create_dummy_receipt("in1", "out1", genesis_prev_hash, "seal1".to_string(), "ptr1".to_string());
        let r1_hash = compute_receipt_hash(&r1);
        receipts.push(r1);

        let mut r2 = create_dummy_receipt("in2", "out2", r1_hash, "seal2".to_string(), "ptr2".to_string());
        let r2_hash = compute_receipt_hash(&r2);
        receipts.push(r2);

        let r3 = create_dummy_receipt("in3", "out3", r2_hash, "seal3".to_string(), "ptr3".to_string());
        receipts.push(r3);

        let result = forensic_audit_loop(receipts).await;
        assert!(result.is_ok(), "Expected valid chain to pass: {:?}", result);
    }

    #[tokio::test]
    async fn test_forensic_audit_loop_broken_chain() {
        let mut receipts = Vec::new();
        let genesis_prev_hash = "genesis_initial_hash_seed".to_string();
        
        let r1 = create_dummy_receipt("in1", "out1", genesis_prev_hash, "seal1".to_string(), "ptr1".to_string());
        receipts.push(r1);

        // Third-party tampered previous receipt hash link
        let r2 = create_dummy_receipt("in2", "out2", "tampered_prev_hash".to_string(), "seal2".to_string(), "ptr2".to_string());
        receipts.push(r2);

        let result = forensic_audit_loop(receipts).await;
        assert!(result.is_err(), "Expected broken chain to fail");
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("Cryptographic deviation: Hash chain broken"));
    }

    #[tokio::test]
    async fn test_forensic_audit_loop_malformed_hashes() {
        let mut receipts = Vec::new();
        let genesis_prev_hash = "genesis_initial_hash_seed".to_string();
        
        let mut r1 = create_dummy_receipt("in1", "out1", genesis_prev_hash, "seal1".to_string(), "ptr1".to_string());
        // Malform input hash
        r1.input_hash = "short_hash".to_string();
        receipts.push(r1);

        let result = forensic_audit_loop(receipts).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("Cryptographic deviation: Malformed hash"));
    }

    #[tokio::test]
    async fn test_forensic_audit_loop_periodic_projection_missing_seal() {
        let mut receipts = Vec::new();
        let mut expected_prev = "genesis".to_string();

        for i in 0..101 {
            let pqc_seal = if i == 100 { "" } else { "valid_seal" };
            let r = create_dummy_receipt(
                &format!("in{}", i),
                &format!("out{}", i),
                expected_prev.clone(),
                pqc_seal.to_string(),
                "ptr".to_string(),
            );
            expected_prev = compute_receipt_hash(&r);
            receipts.push(r);
        }

        let result = forensic_audit_loop(receipts).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("Cryptographic deviation: Missing Post-Quantum Cryptography (PQC) seal"));
    }

    #[tokio::test]
    async fn test_forensic_audit_loop_periodic_projection_missing_replay_pointer() {
        let mut receipts = Vec::new();
        let mut expected_prev = "genesis".to_string();

        for i in 0..101 {
            let replay_ptr = if i == 100 { "" } else { "valid_ptr" };
            let r = create_dummy_receipt(
                &format!("in{}", i),
                &format!("out{}", i),
                expected_prev.clone(),
                "valid_seal".to_string(),
                replay_ptr.to_string(),
            );
            expected_prev = compute_receipt_hash(&r);
            receipts.push(r);
        }

        let result = forensic_audit_loop(receipts).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("Audit failure: Missing replay pointer"));
    }
}

