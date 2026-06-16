use crate::error::{CleanroomError, Result};
use crate::truex::admission_types::{
    AdmissionKernel, AdmissionOutcome, EscrowPolicy, Graph, PartyPacket,
};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Mutex, RwLock};
use tracing::{error, info};
use uuid::Uuid;

// ── Token-based escrow ────────────────────────────────────────────────────────

/// Opaque party identifier.
pub type PartyId = String;

/// Token amount (integer units).
pub type TokenAmount = u64;

/// Opaque escrow identifier.
pub type EscrowId = String;

/// Slash amount returned from a slashing action.
pub type SlashAmount = TokenAmount;

/// Condition that must be proven to release escrowed funds.
#[derive(Debug, Clone)]
pub enum ReleaseCondition {
    /// Always releasable — no proof required (for tests / unconditional escrow).
    Unconditional,
    /// A cryptographic hash that must appear in the proof.
    HashPreimage { expected_hash: [u8; 32] },
    /// A time-based condition: release only after this timestamp.
    TimeLock { release_after: DateTime<Utc> },
    /// Requires an external oracle approval (represented as a string token).
    OracleApproval { oracle_id: String },
}

/// Proof submitted to satisfy a release condition.
#[derive(Debug, Clone)]
pub enum ConditionProof {
    /// No proof needed (for Unconditional).
    None,
    /// Raw preimage bytes.
    Preimage(Vec<u8>),
    /// Proof-of-time: current timestamp proves time has passed.
    Timestamp(DateTime<Utc>),
    /// Oracle signed approval token.
    OracleToken { oracle_id: String, token: String },
}

/// Reason for slashing an escrow.
#[derive(Debug, Clone)]
pub struct Violation {
    pub description: String,
    /// Fraction to slash (0.0 – 1.0). 1.0 = full slash.
    pub slash_fraction: f64,
}

/// Status of an escrow entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EscrowStatus {
    Active,
    Released,
    Slashed,
    Refunded,
    Expired,
}

/// A single escrow entry.
#[derive(Debug, Clone)]
pub struct EscrowEntry {
    pub id: EscrowId,
    pub from: PartyId,
    pub amount: TokenAmount,
    pub condition: ReleaseCondition,
    pub status: EscrowStatus,
    pub deposited_at: DateTime<Utc>,
    pub expiry: Option<DateTime<Utc>>,
    pub released_amount: TokenAmount,
    pub slashed_amount: TokenAmount,
}

/// A simple escrow ledger: deposit, release, slash, and refund token-gated funds.
#[derive(Default)]
pub struct Escrow {
    entries: Mutex<HashMap<EscrowId, EscrowEntry>>,
    /// Simulated balances for each party.
    balances: Mutex<HashMap<PartyId, TokenAmount>>,
}

impl Escrow {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fund a party's balance (for bootstrapping / testing).
    pub fn fund(&self, party: &str, amount: TokenAmount) -> Result<()> {
        let mut bal = self
            .balances
            .lock()
            .map_err(|_| CleanroomError::internal_error("Lock poisoned"))?;
        *bal.entry(party.to_string()).or_insert(0) += amount;
        Ok(())
    }

    /// Deposit `amount` tokens from `from` into escrow under `condition`.
    ///
    /// Returns the new `EscrowId`.
    /// Deducts the amount from `from`'s balance.
    pub fn deposit(
        &self,
        from: PartyId,
        amount: TokenAmount,
        condition: ReleaseCondition,
    ) -> Result<EscrowId> {
        if from.is_empty() {
            return Err(CleanroomError::validation_error("Party ID cannot be empty"));
        }
        if amount == 0 {
            return Err(CleanroomError::validation_error(
                "Deposit amount must be > 0",
            ));
        }

        // Check and deduct balance
        let mut bal = self
            .balances
            .lock()
            .map_err(|_| CleanroomError::internal_error("Lock poisoned"))?;
        let party_bal = bal.entry(from.clone()).or_insert(0);
        if *party_bal < amount {
            return Err(CleanroomError::validation_error(format!(
                "Insufficient balance: {} has {}, needs {}",
                from, *party_bal, amount
            )));
        }
        *party_bal -= amount;
        drop(bal);

        let id = Uuid::new_v4().to_string();
        let entry = EscrowEntry {
            id: id.clone(),
            from: from.clone(),
            amount,
            condition,
            status: EscrowStatus::Active,
            deposited_at: Utc::now(),
            expiry: None,
            released_amount: 0,
            slashed_amount: 0,
        };

        self.entries
            .lock()
            .map_err(|_| CleanroomError::internal_error("Lock poisoned"))?
            .insert(id.clone(), entry);

        info!(escrow_id = %id, from = %from, amount = %amount, "Escrow deposited.");
        Ok(id)
    }

    /// Deposit with an explicit expiry time.
    pub fn deposit_with_expiry(
        &self,
        from: PartyId,
        amount: TokenAmount,
        condition: ReleaseCondition,
        expiry: DateTime<Utc>,
    ) -> Result<EscrowId> {
        let id = self.deposit(from, amount, condition)?;
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| CleanroomError::internal_error("Lock poisoned"))?;
        if let Some(e) = entries.get_mut(&id) {
            e.expiry = Some(expiry);
        }
        Ok(id)
    }

    /// Release escrowed funds back to the depositing party after verifying the condition.
    ///
    /// Returns the amount released.
    pub fn release(&self, escrow_id: &EscrowId, proof: &ConditionProof) -> Result<TokenAmount> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| CleanroomError::internal_error("Lock poisoned"))?;
        let entry = entries.get_mut(escrow_id).ok_or_else(|| {
            CleanroomError::validation_error(format!("Escrow not found: {}", escrow_id))
        })?;

        if entry.status != EscrowStatus::Active {
            return Err(CleanroomError::validation_error(format!(
                "Escrow {} is not active (status: {:?})",
                escrow_id, entry.status
            )));
        }

        // Check expiry
        if let Some(exp) = entry.expiry {
            if Utc::now() > exp {
                entry.status = EscrowStatus::Expired;
                return Err(CleanroomError::validation_error(format!(
                    "Escrow {} has expired",
                    escrow_id
                )));
            }
        }

        // Verify condition
        let condition_met = verify_condition(&entry.condition, proof);
        if !condition_met {
            return Err(CleanroomError::validation_error(format!(
                "Condition not satisfied for escrow {}",
                escrow_id
            )));
        }

        let amount = entry.amount;
        let from = entry.from.clone();
        entry.released_amount = amount;
        entry.status = EscrowStatus::Released;
        drop(entries);

        // Credit balance
        let mut bal = self
            .balances
            .lock()
            .map_err(|_| CleanroomError::internal_error("Lock poisoned"))?;
        *bal.entry(from.clone()).or_insert(0) += amount;

        info!(escrow_id = %escrow_id, from = %from, amount = %amount, "Escrow released.");
        Ok(amount)
    }

    /// Slash escrowed funds based on a violation.
    ///
    /// The slash fraction in `violation` controls how much is slashed.
    /// Slashed funds are removed from escrow (burned / sent to penalty pool).
    /// Returns the amount slashed.
    pub fn slash(&self, escrow_id: &EscrowId, violation: &Violation) -> Result<SlashAmount> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| CleanroomError::internal_error("Lock poisoned"))?;
        let entry = entries.get_mut(escrow_id).ok_or_else(|| {
            CleanroomError::validation_error(format!("Escrow not found: {}", escrow_id))
        })?;

        if entry.status != EscrowStatus::Active {
            return Err(CleanroomError::validation_error(format!(
                "Escrow {} is not active (status: {:?})",
                escrow_id, entry.status
            )));
        }

        let slash_frac = violation.slash_fraction.clamp(0.0, 1.0);
        let slash_amount = (entry.amount as f64 * slash_frac) as u64;
        let slash_amount = slash_amount.min(entry.amount);

        entry.slashed_amount = slash_amount;
        entry.amount -= slash_amount;
        if entry.amount == 0 {
            entry.status = EscrowStatus::Slashed;
        }
        // If partial slash, escrow remains Active with reduced amount.

        info!(
            escrow_id = %escrow_id,
            slash_amount = %slash_amount,
            reason = %violation.description,
            "Escrow slashed."
        );
        Ok(slash_amount)
    }

    /// Refund escrow: return remaining funds to the depositor if the condition has expired
    /// or is otherwise unmet.
    pub fn refund(&self, escrow_id: &EscrowId) -> Result<()> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| CleanroomError::internal_error("Lock poisoned"))?;
        let entry = entries.get_mut(escrow_id).ok_or_else(|| {
            CleanroomError::validation_error(format!("Escrow not found: {}", escrow_id))
        })?;

        if entry.status != EscrowStatus::Active && entry.status != EscrowStatus::Expired {
            return Err(CleanroomError::validation_error(format!(
                "Escrow {} cannot be refunded (status: {:?})",
                escrow_id, entry.status
            )));
        }

        let amount = entry.amount;
        let from = entry.from.clone();
        entry.status = EscrowStatus::Refunded;
        drop(entries);

        let mut bal = self
            .balances
            .lock()
            .map_err(|_| CleanroomError::internal_error("Lock poisoned"))?;
        *bal.entry(from.clone()).or_insert(0) += amount;

        info!(escrow_id = %escrow_id, from = %from, amount = %amount, "Escrow refunded.");
        Ok(())
    }

    /// Get the current balance of a party.
    pub fn balance_of(&self, party: &str) -> TokenAmount {
        self.balances
            .lock()
            .ok()
            .and_then(|b| b.get(party).copied())
            .unwrap_or(0)
    }

    /// Get an escrow entry by ID.
    pub fn get(&self, escrow_id: &str) -> Option<EscrowEntry> {
        self.entries.lock().ok()?.get(escrow_id).cloned()
    }
}

/// Verify that a proof satisfies a condition.
fn verify_condition(condition: &ReleaseCondition, proof: &ConditionProof) -> bool {
    match (condition, proof) {
        (ReleaseCondition::Unconditional, _) => true,
        (ReleaseCondition::HashPreimage { expected_hash }, ConditionProof::Preimage(data)) => {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(data);
            let hash: [u8; 32] = hasher.finalize().into();
            &hash == expected_hash
        }
        (ReleaseCondition::TimeLock { release_after }, ConditionProof::Timestamp(ts)) => {
            ts >= release_after
        }
        (ReleaseCondition::TimeLock { release_after }, ConditionProof::None) => {
            Utc::now() >= *release_after
        }
        (
            ReleaseCondition::OracleApproval { oracle_id },
            ConditionProof::OracleToken {
                oracle_id: proof_id,
                token,
            },
        ) => oracle_id == proof_id && !token.is_empty(),
        _ => false,
    }
}

#[cfg(test)]
mod escrow_token_tests {
    use super::*;
    use sha2::{Digest, Sha256};

    #[test]
    fn test_deposit_and_release_unconditional() {
        let escrow = Escrow::new();
        escrow.fund("alice", 500).unwrap();

        let id = escrow
            .deposit("alice".to_string(), 100, ReleaseCondition::Unconditional)
            .unwrap();
        assert_eq!(escrow.balance_of("alice"), 400);

        let released = escrow.release(&id, &ConditionProof::None).unwrap();
        assert_eq!(released, 100);
        assert_eq!(escrow.balance_of("alice"), 500);
    }

    #[test]
    fn test_slash_partial() {
        let escrow = Escrow::new();
        escrow.fund("bob", 1000).unwrap();

        let id = escrow
            .deposit("bob".to_string(), 1000, ReleaseCondition::Unconditional)
            .unwrap();

        let slashed = escrow
            .slash(
                &id,
                &Violation {
                    description: "contract breach".to_string(),
                    slash_fraction: 0.25,
                },
            )
            .unwrap();
        assert_eq!(slashed, 250);

        // Remaining can be released
        let released = escrow.release(&id, &ConditionProof::None).unwrap();
        assert_eq!(released, 750);
    }

    #[test]
    fn test_refund() {
        let escrow = Escrow::new();
        escrow.fund("carol", 200).unwrap();

        let id = escrow
            .deposit("carol".to_string(), 200, ReleaseCondition::Unconditional)
            .unwrap();
        escrow.refund(&id).unwrap();
        assert_eq!(escrow.balance_of("carol"), 200);
    }

    #[test]
    fn test_hash_preimage_condition() {
        let escrow = Escrow::new();
        escrow.fund("dave", 100).unwrap();

        let secret = b"my_secret_preimage";
        let mut hasher = Sha256::new();
        hasher.update(secret);
        let hash: [u8; 32] = hasher.finalize().into();

        let id = escrow
            .deposit(
                "dave".to_string(),
                100,
                ReleaseCondition::HashPreimage {
                    expected_hash: hash,
                },
            )
            .unwrap();

        // Wrong preimage fails
        let bad = escrow.release(&id, &ConditionProof::Preimage(b"wrong".to_vec()));
        assert!(bad.is_err());

        // Correct preimage succeeds
        let ok = escrow.release(&id, &ConditionProof::Preimage(secret.to_vec()));
        assert!(ok.is_ok());
    }
}

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

    pub fn slash_stake(
        &self,
        seller_pubkey: String,
        reason: String,
    ) -> std::result::Result<(), String> {
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
    ) -> std::result::Result<String, String> {
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

        let kp = crate::pqc::lattice::generate_keypair([1u8; 32]);
        let msg = b"malicious_action";
        let sig = crate::pqc::lattice::sign(&kp.secret, msg, [2u8; 32]);

        let signature_hex = format!(
            "z:{}-c:{}",
            hex::encode(
                &sig.z
                    .coeffs
                    .iter()
                    .take(32)
                    .map(|x| format!("{:04x}", x))
                    .collect::<String>()
            ),
            hex::encode(
                &sig.c
                    .coeffs
                    .iter()
                    .take(32)
                    .map(|x| format!("{:04x}", x))
                    .collect::<String>()
            )
        );
        let public_key_hex = hex::encode(
            &kp.public
                .t
                .coeffs
                .iter()
                .take(32)
                .map(|x| format!("{:04x}", x))
                .collect::<String>(),
        );

        // Packet with invalid signature details (simulate tamper by modifying payload but keeping original signature)
        let packet = PartyPacket {
            sender: "seller_01".to_string(),
            payload: "tampered_action".to_string(),
            nonce: 42,
            signature_hex: Some(signature_hex),
            public_key_hex: Some(public_key_hex),
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
