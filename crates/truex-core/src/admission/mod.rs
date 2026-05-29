use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use sha2::{Sha256, Digest};
use clnrm_core::pqc::lattice::{self, Poly, PublicKey, Signature};

/// Represents a single RDF-like projected record node/triple in the consequence closure graph.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Record {
    pub entity: String,
    pub attribute: String,
    pub value: String,
}

/// Represents a deterministic CONSTRUCT closure graph.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Graph {
    pub records: Vec<Record>,
}

/// A Post-Quantum-signed counterparty packet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PartyPacket {
    pub sender: String,
    pub payload: String,
    #[serde(default)]
    pub nonce: u64,
    /// Hex-encoded lattice signature bytes (contains z and c polynomials).
    pub signature_hex: Option<String>,
    /// Hex-encoded public key bytes (contains a and t polynomials).
    pub public_key_hex: Option<String>,
}

/// Helper methods for serializing and deserializing Post-Quantum lattice keys and signatures.
impl PartyPacket {
    /// Deserializes a Poly from a 512-byte hex string (64 coefficients * 8 bytes = 512 bytes = 1024 hex chars).
    pub fn hex_to_poly(hex_str: &str) -> Result<Poly, String> {
        let bytes = hex::decode(hex_str).map_err(|e| format!("Hex decoding failed: {}", e))?;
        if bytes.len() != lattice::N * 8 {
            return Err(format!(
                "Invalid poly byte length: expected {}, got {}",
                lattice::N * 8,
                bytes.len()
            ));
        }
        let mut coeffs = [0i64; lattice::N];
        for i in 0..lattice::N {
            let chunk = &bytes[i * 8..(i + 1) * 8];
            coeffs[i] = i64::from_be_bytes(chunk.try_into().unwrap());
        }
        Ok(Poly { coeffs })
    }

    /// Serializes a Poly to a hex string.
    pub fn poly_to_hex(p: &Poly) -> String {
        let mut bytes = Vec::with_capacity(lattice::N * 8);
        for &coeff in &p.coeffs {
            bytes.extend_from_slice(&coeff.to_be_bytes());
        }
        hex::encode(bytes)
    }

    /// Deserializes a PublicKey from a 1024-byte hex string (2 polynomials * 512 bytes = 1024 bytes = 2048 hex chars).
    pub fn hex_to_public_key(hex_str: &str) -> Result<PublicKey, String> {
        let bytes = hex::decode(hex_str).map_err(|e| format!("Hex decoding failed: {}", e))?;
        if bytes.len() != lattice::N * 16 {
            return Err(format!(
                "Invalid public key byte length: expected {}, got {}",
                lattice::N * 16,
                bytes.len()
            ));
        }
        let a = Self::hex_to_poly(&hex::encode(&bytes[0..lattice::N * 8]))?;
        let t = Self::hex_to_poly(&hex::encode(&bytes[lattice::N * 8..]))?;
        Ok(PublicKey { a, t })
    }

    /// Serializes a PublicKey to a hex string.
    pub fn public_key_to_hex(pk: &PublicKey) -> String {
        let mut a_hex = Self::poly_to_hex(&pk.a);
        let t_hex = Self::poly_to_hex(&pk.t);
        a_hex.push_str(&t_hex);
        a_hex
    }

    /// Deserializes a Signature from a 1024-byte hex string (2 polynomials * 512 bytes = 1024 bytes = 2048 hex chars).
    pub fn hex_to_signature(hex_str: &str) -> Result<Signature, String> {
        let bytes = hex::decode(hex_str).map_err(|e| format!("Hex decoding failed: {}", e))?;
        if bytes.len() != lattice::N * 16 {
            return Err(format!(
                "Invalid signature byte length: expected {}, got {}",
                lattice::N * 16,
                bytes.len()
            ));
        }
        let z = Self::hex_to_poly(&hex::encode(&bytes[0..lattice::N * 8]))?;
        let c = Self::hex_to_poly(&hex::encode(&bytes[lattice::N * 8..]))?;
        Ok(Signature { z, c })
    }

    /// Serializes a Signature to a hex string.
    pub fn signature_to_hex(sig: &Signature) -> String {
        let mut z_hex = Self::poly_to_hex(&sig.z);
        let c_hex = Self::poly_to_hex(&sig.c);
        z_hex.push_str(&c_hex);
        z_hex
    }

    /// Verifies the cryptographic signature of the party packet using its payload and nonce.
    pub fn verify_signature(&self) -> Result<bool, String> {
        let pk_hex = match &self.public_key_hex {
            Some(pk) => pk,
            None => return Err("Public key is missing".to_string()),
        };
        let sig_hex = match &self.signature_hex {
            Some(sig) => sig,
            None => return Err("Signature is missing".to_string()),
        };

        let pk = Self::hex_to_public_key(pk_hex)?;
        let sig = Self::hex_to_signature(sig_hex)?;
        
        let mut message = self.payload.as_bytes().to_vec();
        message.extend_from_slice(&self.nonce.to_be_bytes());

        Ok(lattice::verify(&pk, &message, &sig))
    }
}

/// Evaluation operators for checking graph attributes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConstraintOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    Contains,
}

/// Constraints to apply to projected records during policy evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecordConstraint {
    pub entity: String,
    pub attribute: String,
    pub operator: ConstraintOperator,
    pub expected_value: String,
}

/// Constraint on the timestamp when admission evaluation occurs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimeLockConstraint {
    pub min_timestamp: Option<i64>,
    pub max_timestamp: Option<i64>,
}

/// Escrow policy containing rules for admission of consequence closures.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EscrowPolicy {
    pub policy_id: String,
    #[serde(default)]
    pub epoch: u64,
    pub required_senders: Vec<String>,
    pub verify_signatures: bool,
    pub record_constraints: Vec<RecordConstraint>,
    pub time_lock: Option<TimeLockConstraint>,
}

/// Result of admitting a record closure into execution trust state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdmissionReceipt {
    pub policy_id: String,
    /// Cryptographic SHA-256 hash verifying the determinism of input graph and packets.
    pub consensus_hash: String,
    pub timestamp: i64,
}

/// Reason/diagnostics details on refusal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RefusalDiagnostics {
    pub missing_senders: Vec<String>,
    pub invalid_signatures: Vec<String>,
    pub failed_constraints: Vec<String>,
    pub time_lock_failed: Option<String>,
}

/// Evidence detailing why the admission was refused.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RefusalReceipt {
    pub policy_id: String,
    pub reasons: Vec<String>,
    pub diagnostics: RefusalDiagnostics,
    pub timestamp: i64,
}

/// Outcome of admission evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AdmissionOutcome {
    Admit(AdmissionReceipt),
    Refuse(RefusalReceipt),
}

pub struct AdmissionKernel;

impl AdmissionKernel {
    /// Calculate the Median-Time-Past (MTP) from a sequence of past timestamps.
    pub fn calculate_median_time_past(past_timestamps: &[i64]) -> Option<i64> {
        if past_timestamps.is_empty() {
            return None;
        }
        let mut sorted = past_timestamps.to_vec();
        sorted.sort_unstable();
        let len = sorted.len();
        if len % 2 == 1 {
            Some(sorted[len / 2])
        } else {
            Some((sorted[len / 2 - 1] + sorted[len / 2]) / 2)
        }
    }

    /// Evaluate projected records and party packets against an escrow policy at the current time.
    pub fn evaluate(
        projected_graph: &Graph,
        packets: &[PartyPacket],
        policy: &EscrowPolicy,
        evaluation_timestamp: i64,
        past_timestamps: &[i64],
    ) -> AdmissionOutcome {
        let mut reasons = Vec::new();
        let mut missing_senders = Vec::new();
        let mut invalid_signatures = Vec::new();
        let mut failed_constraints = Vec::new();
        let mut time_lock_failed = None;

        // 1. Time Lock Constraint check using Median-Time-Past (MTP)
        if let Some(ref lock) = policy.time_lock {
            let reference_time = Self::calculate_median_time_past(past_timestamps)
                .unwrap_or(evaluation_timestamp);

            if let Some(min_t) = lock.min_timestamp {
                if reference_time < min_t {
                    let err = format!(
                        "Time constraint violation: Median-Time-Past {} is before minimum allowed {}",
                        reference_time, min_t
                    );
                    reasons.push(err.clone());
                    time_lock_failed = Some(err);
                }
            }
            if let Some(max_t) = lock.max_timestamp {
                if reference_time > max_t {
                    let err = format!(
                        "Time constraint violation: Median-Time-Past {} is after maximum allowed {}",
                        reference_time, max_t
                    );
                    reasons.push(err.clone());
                    time_lock_failed = Some(err);
                }
            }
        }

        // 2. Required Senders check
        let submitted_senders: HashSet<&str> = packets.iter().map(|p| p.sender.as_str()).collect();
        for req_sender in &policy.required_senders {
            if !submitted_senders.contains(req_sender.as_str()) {
                let err = format!("Required packet sender '{}' is missing", req_sender);
                reasons.push(err.clone());
                missing_senders.push(req_sender.clone());
            }
        }

        // 3. Cryptographic packet signature verification
        if policy.verify_signatures {
            for packet in packets {
                match packet.verify_signature() {
                    Ok(true) => {}
                    Ok(false) => {
                        let err = format!(
                            "Cryptographic validation failed: signature of sender '{}' is invalid",
                            packet.sender
                        );
                        reasons.push(err.clone());
                        invalid_signatures.push(packet.sender.clone());
                    }
                    Err(e) => {
                        let err = format!(
                            "Cryptographic error for sender '{}': {}",
                            packet.sender, e
                        );
                        reasons.push(err.clone());
                        invalid_signatures.push(packet.sender.clone());
                    }
                }
            }
        }

        // 4. Graph record constraints verification
        for constraint in &policy.record_constraints {
            // Find any record matching entity and attribute
            let matching_records: Vec<&Record> = projected_graph
                .records
                .iter()
                .filter(|r| r.entity == constraint.entity && r.attribute == constraint.attribute)
                .collect();

            if matching_records.is_empty() {
                let err = format!(
                    "Record constraint failed: no record found with entity '{}' and attribute '{}'",
                    constraint.entity, constraint.attribute
                );
                reasons.push(err.clone());
                failed_constraints.push(err);
                continue;
            }

            for record in matching_records {
                let pass = match constraint.operator {
                    ConstraintOperator::Equals => record.value == constraint.expected_value,
                    ConstraintOperator::NotEquals => record.value != constraint.expected_value,
                    ConstraintOperator::Contains => record.value.contains(&constraint.expected_value),
                    ConstraintOperator::GreaterThan => {
                        if let (Ok(rec_val), Ok(exp_val)) = (
                            record.value.parse::<f64>(),
                            constraint.expected_value.parse::<f64>(),
                        ) {
                            rec_val > exp_val
                        } else {
                            // Lexicographical comparison if parsing fails
                            record.value > constraint.expected_value
                        }
                    }
                    ConstraintOperator::LessThan => {
                        if let (Ok(rec_val), Ok(exp_val)) = (
                            record.value.parse::<f64>(),
                            constraint.expected_value.parse::<f64>(),
                        ) {
                            rec_val < exp_val
                        } else {
                            // Lexicographical comparison if parsing fails
                            record.value < constraint.expected_value
                        }
                    }
                };

                if !pass {
                    let err = format!(
                        "Record constraint violation: entity '{}' attribute '{}' has value '{}', which fails the {:?} comparison with '{}'",
                        constraint.entity, constraint.attribute, record.value, constraint.operator, constraint.expected_value
                    );
                    reasons.push(err.clone());
                    failed_constraints.push(err);
                }
            }
        }

        // Output final verdict based on evaluation results
        if reasons.is_empty() {
            // Calculate deterministic consensus hash
            let consensus_hash = Self::calculate_consensus_hash(
                projected_graph,
                packets,
                &policy.policy_id,
                policy.epoch,
            );
            AdmissionOutcome::Admit(AdmissionReceipt {
                policy_id: policy.policy_id.clone(),
                consensus_hash,
                timestamp: evaluation_timestamp,
            })
        } else {
            AdmissionOutcome::Refuse(RefusalReceipt {
                policy_id: policy.policy_id.clone(),
                reasons,
                diagnostics: RefusalDiagnostics {
                    missing_senders,
                    invalid_signatures,
                    failed_constraints,
                    time_lock_failed,
                },
                timestamp: evaluation_timestamp,
            })
        }
    }

    /// Calculate a deterministic hash representing the exact input graph state, counterparty packet parameters, policy_id, and epoch.
    pub fn calculate_consensus_hash(
        projected_graph: &Graph,
        packets: &[PartyPacket],
        policy_id: &str,
        epoch: u64,
    ) -> String {
        // Sort records canonicalizing the graph representation
        let mut sorted_records = projected_graph.records.clone();
        sorted_records.sort();

        // Sort packets by sender canonicalizing the inputs
        let mut sorted_packets = packets.to_vec();
        sorted_packets.sort_by(|a, b| a.sender.cmp(&b.sender));

        let mut hasher = Sha256::new();
        hasher.update(b"TRUEX-CANONICAL-V2");
        hasher.update(policy_id.as_bytes());
        hasher.update(&epoch.to_be_bytes());
        
        if let Ok(records_json) = serde_json::to_string(&sorted_records) {
            hasher.update(records_json.as_bytes());
        }
        if let Ok(packets_json) = serde_json::to_string(&sorted_packets) {
            hasher.update(packets_json.as_bytes());
        }
        
        hex::encode(hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clnrm_core::pqc::lattice;

    #[test]
    fn test_admission_success_basic() {
        let graph = Graph {
            records: vec![
                Record {
                    entity: "Escrow-001".to_string(),
                    attribute: "status".to_string(),
                    value: "funded".to_string(),
                },
                Record {
                    entity: "Escrow-001".to_string(),
                    attribute: "amount".to_string(),
                    value: "5000".to_string(),
                },
            ],
        };

        let packets = vec![
            PartyPacket {
                sender: "Alice".to_string(),
                payload: "Agree escrow-001".to_string(),
                nonce: 0,
                signature_hex: None,
                public_key_hex: None,
            },
            PartyPacket {
                sender: "Bob".to_string(),
                payload: "Agree escrow-001".to_string(),
                nonce: 0,
                signature_hex: None,
                public_key_hex: None,
            },
        ];

        let policy = EscrowPolicy {
            policy_id: "EscrowFundedAndAgreed".to_string(),
            epoch: 0,
            required_senders: vec!["Alice".to_string(), "Bob".to_string()],
            verify_signatures: false,
            record_constraints: vec![
                RecordConstraint {
                    entity: "Escrow-001".to_string(),
                    attribute: "status".to_string(),
                    operator: ConstraintOperator::Equals,
                    expected_value: "funded".to_string(),
                },
                RecordConstraint {
                    entity: "Escrow-001".to_string(),
                    attribute: "amount".to_string(),
                    operator: ConstraintOperator::GreaterThan,
                    expected_value: "1000".to_string(),
                },
            ],
            time_lock: Some(TimeLockConstraint {
                min_timestamp: Some(1000),
                max_timestamp: Some(2000),
            }),
        };

        // Evaluate within valid time window
        let outcome = AdmissionKernel::evaluate(&graph, &packets, &policy, 1500, &[]);
        
        match outcome {
            AdmissionOutcome::Admit(receipt) => {
                assert_eq!(receipt.policy_id, "EscrowFundedAndAgreed");
                assert_eq!(receipt.timestamp, 1500);
                assert!(!receipt.consensus_hash.is_empty());
            }
            AdmissionOutcome::Refuse(refusal) => {
                panic!("Expected Admit, got Refuse with reasons: {:?}", refusal.reasons);
            }
        }
    }

    #[test]
    fn test_admission_refuse_missing_senders() {
        let graph = Graph {
            records: vec![],
        };

        // Only Alice submitted, Bob is missing
        let packets = vec![
            PartyPacket {
                sender: "Alice".to_string(),
                payload: "Agree".to_string(),
                nonce: 0,
                signature_hex: None,
                public_key_hex: None,
            },
        ];

        let policy = EscrowPolicy {
            policy_id: "EscrowAgreed".to_string(),
            epoch: 0,
            required_senders: vec!["Alice".to_string(), "Bob".to_string()],
            verify_signatures: false,
            record_constraints: vec![],
            time_lock: None,
        };

        let outcome = AdmissionKernel::evaluate(&graph, &packets, &policy, 100, &[]);
        
        match outcome {
            AdmissionOutcome::Admit(_) => {
                panic!("Expected Refuse, got Admit");
            }
            AdmissionOutcome::Refuse(refusal) => {
                assert_eq!(refusal.policy_id, "EscrowAgreed");
                assert_eq!(refusal.diagnostics.missing_senders, vec!["Bob".to_string()]);
                assert!(refusal.reasons[0].contains("Required packet sender 'Bob' is missing"));
            }
        }
    }

    #[test]
    fn test_admission_refuse_constraint_failed() {
        let graph = Graph {
            records: vec![
                Record {
                    entity: "Escrow-001".to_string(),
                    attribute: "amount".to_string(),
                    value: "500".to_string(), // less than expected 1000
                },
            ],
        };

        let packets = vec![];
        let policy = EscrowPolicy {
            policy_id: "MinAmountPolicy".to_string(),
            epoch: 0,
            required_senders: vec![],
            verify_signatures: false,
            record_constraints: vec![
                RecordConstraint {
                    entity: "Escrow-001".to_string(),
                    attribute: "amount".to_string(),
                    operator: ConstraintOperator::GreaterThan,
                    expected_value: "1000".to_string(),
                },
            ],
            time_lock: None,
        };

        let outcome = AdmissionKernel::evaluate(&graph, &packets, &policy, 100, &[]);
        
        match outcome {
            AdmissionOutcome::Admit(_) => {
                panic!("Expected Refuse, got Admit");
            }
            AdmissionOutcome::Refuse(refusal) => {
                assert_eq!(refusal.diagnostics.failed_constraints.len(), 1);
                assert!(refusal.reasons[0].contains("fails the GreaterThan comparison with '1000'"));
            }
        }
    }

    #[test]
    fn test_admission_refuse_time_lock() {
        let graph = Graph { records: vec![] };
        let packets = vec![];
        let policy = EscrowPolicy {
            policy_id: "TimeLocked".to_string(),
            epoch: 0,
            required_senders: vec![],
            verify_signatures: false,
            record_constraints: vec![],
            time_lock: Some(TimeLockConstraint {
                min_timestamp: Some(1000),
                max_timestamp: Some(2000),
            }),
        };

        // Evaluate too early
        let outcome_early = AdmissionKernel::evaluate(&graph, &packets, &policy, 500, &[]);
        match outcome_early {
            AdmissionOutcome::Admit(_) => panic!("Expected Refuse"),
            AdmissionOutcome::Refuse(refusal) => {
                assert!(refusal.diagnostics.time_lock_failed.is_some());
                assert!(refusal.reasons[0].contains("before minimum allowed"));
            }
        }

        // Evaluate too late
        let outcome_late = AdmissionKernel::evaluate(&graph, &packets, &policy, 2500, &[]);
        match outcome_late {
            AdmissionOutcome::Admit(_) => panic!("Expected Refuse"),
            AdmissionOutcome::Refuse(refusal) => {
                assert!(refusal.diagnostics.time_lock_failed.is_some());
                assert!(refusal.reasons[0].contains("after maximum allowed"));
            }
        }
    }

    #[test]
    fn test_pqc_lattice_signature_verification() {
        // Generate a post-quantum lattice key pair
        let seed = [42u8; 32];
        let kp = lattice::generate_keypair(seed);
        
        let payload = "This is a post-quantum payload".to_string();
        let nonce = 98765u64;
        let mut message = payload.as_bytes().to_vec();
        message.extend_from_slice(&nonce.to_be_bytes());

        let sig_seed = [77u8; 32];
        let sig = lattice::sign(&kp.secret, &message, sig_seed);

        let pk_hex = PartyPacket::public_key_to_hex(&kp.public);
        let sig_hex = PartyPacket::signature_to_hex(&sig);

        // Construct valid packet
        let mut packet = PartyPacket {
            sender: "Bob".to_string(),
            payload: payload.clone(),
            nonce,
            signature_hex: Some(sig_hex.clone()),
            public_key_hex: Some(pk_hex.clone()),
        };

        // Assert self signature check works
        assert!(packet.verify_signature().unwrap());

        // Assert policy verification works
        let graph = Graph { records: vec![] };
        let policy = EscrowPolicy {
            policy_id: "PqcVerified".to_string(),
            epoch: 0,
            required_senders: vec!["Bob".to_string()],
            verify_signatures: true,
            record_constraints: vec![],
            time_lock: None,
        };

        let outcome = AdmissionKernel::evaluate(&graph, &[packet.clone()], &policy, 100, &[]);
        assert!(matches!(outcome, AdmissionOutcome::Admit(_)));

        // Tamper with payload
        packet.payload = "Tampered payload".to_string();
        assert!(!packet.verify_signature().unwrap());

        let outcome_failed = AdmissionKernel::evaluate(&graph, &[packet], &policy, 100, &[]);
        match outcome_failed {
            AdmissionOutcome::Admit(_) => panic!("Expected Refuse due to invalid signature"),
            AdmissionOutcome::Refuse(refusal) => {
                assert_eq!(refusal.diagnostics.invalid_signatures, vec!["Bob".to_string()]);
            }
        }
    }

    #[test]
    fn test_admission_mtp_checking_and_epoch_consensus_hash() {
        let graph = Graph { records: vec![] };
        let packets = vec![];
        
        // Policy with a time lock and an epoch
        let policy = EscrowPolicy {
            policy_id: "MtpEpochTest".to_string(),
            epoch: 5,
            required_senders: vec![],
            verify_signatures: false,
            record_constraints: vec![],
            time_lock: Some(TimeLockConstraint {
                min_timestamp: Some(1000),
                max_timestamp: Some(2000),
            }),
        };

        // If we pass past_timestamps where the median is 1500, even if the evaluation_timestamp is 500 (too early) or 2500 (too late), it should pass!
        // Sorted: [900, 1200, 1500, 1800, 2100], median = 1500
        let past_timestamps = vec![900, 1800, 1500, 2100, 1200];
        
        let outcome = AdmissionKernel::evaluate(&graph, &packets, &policy, 500, &past_timestamps);
        match outcome {
            AdmissionOutcome::Admit(receipt) => {
                assert_eq!(receipt.policy_id, "MtpEpochTest");
                assert_eq!(receipt.timestamp, 500); // receipt timestamp matches the evaluation timestamp, but logic checked MTP
                
                // Let's verify consensus hash includes epoch and policy_id.
                // If we compute the hash with a different epoch, it should be different.
                let hash_epoch_5 = receipt.consensus_hash;
                let hash_epoch_6 = AdmissionKernel::calculate_consensus_hash(&graph, &packets, &policy.policy_id, 6);
                assert_ne!(hash_epoch_5, hash_epoch_6);

                let hash_diff_policy = AdmissionKernel::calculate_consensus_hash(&graph, &packets, "OtherPolicy", 5);
                assert_ne!(hash_epoch_5, hash_diff_policy);
            }
            AdmissionOutcome::Refuse(refusal) => {
                panic!("Expected Admit, got Refuse: {:?}", refusal.reasons);
            }
        }

        // Test with MTP that violates time-lock
        // Sorted: [800, 900, 950], median = 900 (before 1000)
        let past_violating = vec![950, 800, 900];
        let outcome_fail = AdmissionKernel::evaluate(&graph, &packets, &policy, 1500, &past_violating);
        match outcome_fail {
            AdmissionOutcome::Admit(_) => panic!("Expected Refuse due to MTP violation"),
            AdmissionOutcome::Refuse(refusal) => {
                assert!(refusal.reasons[0].contains("Median-Time-Past 900 is before minimum allowed 1000"));
            }
        }
    }
}
