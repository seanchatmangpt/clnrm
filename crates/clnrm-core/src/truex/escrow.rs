use crate::truex::admission_types::{
    AdmissionKernel, AdmissionOutcome, EscrowPolicy, Graph, PartyPacket,
};
use std::sync::RwLock;
use tracing::{error, info};

/// Escrow Clearinghouse for stake release.
pub struct EscrowClearinghouse {
    policy: EscrowPolicy,
    slashed_stakes: RwLock<Vec<(String, String)>>,
}

impl EscrowClearinghouse {
    pub fn new(policy: EscrowPolicy) -> Self {
        Self {
            policy,
            slashed_stakes: RwLock::new(Vec::new()),
        }
    }

    pub fn slash_stake(&self, seller_pubkey: String, reason: String) -> Result<(), String> {
        info!(seller = %seller_pubkey, %reason, "Slashed seller stake.");
        if let Ok(mut slashed) = self.slashed_stakes.write() {
            slashed.push((seller_pubkey, reason));
            Ok(())
        } else {
            Err("Failed to acquire write lock for slashed_stakes".to_string())
        }
    }

    pub fn get_slashed_stakes(&self) -> Vec<(String, String)> {
        self.slashed_stakes
            .read()
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    /// Execute settlement: verify PQC seal, check MTP, execute stake release.
    pub fn settle(
        &self,
        graph: &Graph,
        packets: &[PartyPacket],
        evaluation_timestamp: i64,
        past_timestamps: &[i64],
    ) -> Result<String, String> {
        info!(policy_id = %self.policy.policy_id, "Executing settlement.");

        // 1. Evaluate admission
        match AdmissionKernel::evaluate(
            graph,
            packets,
            &self.policy,
            evaluation_timestamp,
            past_timestamps,
        ) {
            AdmissionOutcome::Admit(receipt) => {
                info!(consensus_hash = %receipt.consensus_hash, "Stake release authorized.");
                Ok(receipt.consensus_hash)
            }
            AdmissionOutcome::Refuse(refusal) => {
                error!(policy_id = %refusal.policy_id, "Settlement refused.");

                // Automate slashing for cryptographic deviations (invalid signatures)
                for sender in &refusal.diagnostics.invalid_signatures {
                    let _ = self.slash_stake(
                        sender.clone(),
                        format!(
                            "Cryptographic signature verification failed: {:?}",
                            refusal.reasons
                        ),
                    );
                }

                Err(format!("Settlement refused: {:?}", refusal.reasons))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::truex::admission_types::{
        ConstraintOperator, Record, RecordConstraint, TimeLockConstraint,
    };

    #[test]
    fn test_escrow_settlement() {
        let policy = EscrowPolicy {
            policy_id: "test-policy".to_string(),
            epoch: 1,
            required_senders: vec![],
            verify_signatures: false,
            record_constraints: vec![RecordConstraint {
                entity: "Asset".into(),
                attribute: "Status".into(),
                operator: ConstraintOperator::Equals,
                expected_value: "Locked".into(),
            }],
            time_lock: None,
        };
        let clearinghouse = EscrowClearinghouse::new(policy);
        let graph = Graph {
            records: vec![Record {
                entity: "Asset".into(),
                attribute: "Status".into(),
                value: "Locked".into(),
            }],
        };

        let result = clearinghouse.settle(&graph, &[], 100, &[]);
        assert!(result.is_ok());
        assert_eq!(clearinghouse.get_slashed_stakes().len(), 0);
    }

    #[test]
    fn test_escrow_slashing_on_invalid_signature() {
        let policy = EscrowPolicy {
            policy_id: "sig-policy".to_string(),
            epoch: 1,
            required_senders: vec!["seller_01".to_string()],
            verify_signatures: true,
            record_constraints: vec![],
            time_lock: None,
        };
        let clearinghouse = EscrowClearinghouse::new(policy);
        let graph = Graph { records: vec![] };

        // Packet with missing or invalid signature details
        let packet = PartyPacket {
            sender: "seller_01".to_string(),
            payload: "malicious_action".to_string(),
            nonce: 42,
            signature_hex: Some("00".repeat(1024)), // Dummy signature bytes
            public_key_hex: Some("00".repeat(1024)), // Dummy pubkey bytes
        };

        let result = clearinghouse.settle(&graph, &[packet], 100, &[]);
        assert!(result.is_err());

        let slashed = clearinghouse.get_slashed_stakes();
        assert_eq!(slashed.len(), 1);
        assert_eq!(slashed[0].0, "seller_01");
        assert!(slashed[0]
            .1
            .contains("Cryptographic signature verification failed"));
    }

    #[test]
    fn test_escrow_manual_slashing_on_constraint_deviation() {
        let policy = EscrowPolicy {
            policy_id: "constraint-policy".to_string(),
            epoch: 1,
            required_senders: vec![],
            verify_signatures: false,
            record_constraints: vec![RecordConstraint {
                entity: "Asset".into(),
                attribute: "Status".into(),
                operator: ConstraintOperator::Equals,
                expected_value: "Locked".into(),
            }],
            time_lock: None,
        };
        let clearinghouse = EscrowClearinghouse::new(policy);

        // Deviation: Asset status is Unlocked instead of Locked
        let graph = Graph {
            records: vec![Record {
                entity: "Asset".into(),
                attribute: "Status".into(),
                value: "Unlocked".into(),
            }],
        };

        let result = clearinghouse.settle(&graph, &[], 100, &[]);
        assert!(result.is_err());

        // Manual slash due to contract deviation
        let slash_result = clearinghouse.slash_stake(
            "seller_02".to_string(),
            "Contract deviation: Asset status is Unlocked".to_string(),
        );
        assert!(slash_result.is_ok());

        let slashed = clearinghouse.get_slashed_stakes();
        assert_eq!(slashed.len(), 1);
        assert_eq!(slashed[0].0, "seller_02");
        assert_eq!(slashed[0].1, "Contract deviation: Asset status is Unlocked");
    }

    #[test]
    fn test_escrow_manual_slashing_on_timelock_deviation() {
        let policy = EscrowPolicy {
            policy_id: "timelock-policy".to_string(),
            epoch: 1,
            required_senders: vec![],
            verify_signatures: false,
            record_constraints: vec![],
            time_lock: Some(TimeLockConstraint {
                min_timestamp: Some(200),
                max_timestamp: Some(300),
            }),
        };
        let clearinghouse = EscrowClearinghouse::new(policy);
        let graph = Graph { records: vec![] };

        // Evaluation timestamp (100) is before min_timestamp (200)
        let result = clearinghouse.settle(&graph, &[], 100, &[]);
        assert!(result.is_err());

        // Slash seller for submitting before the timelock window
        let slash_result = clearinghouse.slash_stake(
            "seller_03".to_string(),
            "Contract deviation: Submitted before timelock window".to_string(),
        );
        assert!(slash_result.is_ok());

        let slashed = clearinghouse.get_slashed_stakes();
        assert_eq!(slashed.len(), 1);
        assert_eq!(slashed[0].0, "seller_03");
        assert_eq!(
            slashed[0].1,
            "Contract deviation: Submitted before timelock window"
        );
    }
}
