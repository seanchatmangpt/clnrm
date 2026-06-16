use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

/// A cryptographically bound receipt for a TrueX settlement.
#[derive(Debug, Clone)]
pub struct SettlementReceipt {
    /// Unique receipt identifier (UUID v4).
    pub receipt_id: String,
    /// Identifier of the transaction being settled.
    pub transaction_id: String,
    /// Participant IDs involved in the settlement.
    pub parties: Vec<String>,
    /// Settlement amount (must be positive).
    pub amount: f64,
    /// ISO 4217 currency code, e.g. "USD".
    pub currency: String,
    /// Unix timestamp in milliseconds when the receipt was created.
    pub timestamp_ms: u64,
    /// SHA-256 digest over the canonical fields.
    pub digest: [u8; 32],
    /// HMAC-SHA256 of the digest, providing authenticity.
    pub signature: Vec<u8>,
    /// Arbitrary key/value metadata attached to the receipt.
    pub attributes: HashMap<String, String>,
}

impl SettlementReceipt {
    /// Creates a new `SettlementReceipt`, computing the digest and signing it
    /// with an empty secret.  Call [`SettlementReceipt::sign`] separately when
    /// a real secret is available, or re-sign via the builder pattern.
    pub fn new(transaction_id: &str, parties: Vec<String>, amount: f64, currency: &str) -> Self {
        let receipt_id = uuid::Uuid::new_v4().to_string();
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let digest = Self::compute_digest(transaction_id, &parties, amount, timestamp_ms);
        // Sign with an empty secret by default; callers can re-sign with a real key.
        let signature = Self::sign(&digest, b"");

        Self {
            receipt_id,
            transaction_id: transaction_id.to_string(),
            parties,
            amount,
            currency: currency.to_string(),
            timestamp_ms,
            digest,
            signature,
            attributes: HashMap::new(),
        }
    }

    /// Computes the SHA-256 digest of the canonical field representation:
    /// `SHA-256(transaction_id || parties_sorted_joined || amount_le_bytes || timestamp_le_bytes)`.
    pub fn compute_digest(
        transaction_id: &str,
        parties: &[String],
        amount: f64,
        timestamp_ms: u64,
    ) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(transaction_id.as_bytes());

        let mut sorted_parties = parties.to_vec();
        sorted_parties.sort();
        hasher.update(sorted_parties.join(",").as_bytes());

        hasher.update(amount.to_le_bytes());
        hasher.update(timestamp_ms.to_le_bytes());

        hasher.finalize().into()
    }

    /// Produces an HMAC-SHA256 over `digest` using `secret`.
    pub fn sign(digest: &[u8; 32], secret: &[u8]) -> Vec<u8> {
        let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts keys of any length"); // OK: any key len
        mac.update(digest);
        mac.finalize().into_bytes().to_vec()
    }

    /// Returns `true` when the stored signature matches a freshly computed
    /// HMAC of the stored digest using `secret`.
    pub fn verify(&self, secret: &[u8]) -> bool {
        let expected = Self::sign(&self.digest, secret);
        self.signature == expected
    }

    /// Builder-style method to attach a metadata attribute.
    pub fn with_attribute(mut self, key: &str, value: &str) -> Self {
        self.attributes.insert(key.to_string(), value.to_string());
        self
    }

    /// Serializes the receipt to a JSON string.
    pub fn to_json(&self) -> String {
        let digest_hex = hex::encode(self.digest);
        let signature_hex = hex::encode(&self.signature);

        serde_json::json!({
            "receipt_id":      self.receipt_id,
            "transaction_id":  self.transaction_id,
            "parties":         self.parties,
            "amount":          self.amount,
            "currency":        self.currency,
            "timestamp_ms":    self.timestamp_ms,
            "digest":          digest_hex,
            "signature":       signature_hex,
            "attributes":      self.attributes,
        })
        .to_string()
    }

    /// Returns `true` when the receipt is internally consistent:
    /// * `amount` is positive
    /// * `parties` is non-empty
    /// * stored `digest` matches a freshly recomputed digest
    pub fn is_valid(&self) -> bool {
        if self.amount <= 0.0 || self.parties.is_empty() {
            return false;
        }
        let expected = Self::compute_digest(
            &self.transaction_id,
            &self.parties,
            self.amount,
            self.timestamp_ms,
        );
        self.digest == expected
    }
}

// ---------------------------------------------------------------------------
// Legacy types – retained for backward compatibility with admission.rs and
// market modules that still reference TruexReceipt / Verdict.
// ---------------------------------------------------------------------------

/// Verdict of the truex execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Verdict {
    /// The procedure passed successfully.
    Passed,
    /// The procedure failed.
    Failed,
    /// The procedure was skipped.
    Skipped,
}

/// TruexReceipt is the definitive struct per the PRD.
/// It contains cryptographic hashes and metadata to ensure the integrity
/// and non-repudiation of pipeline executions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TruexReceipt {
    /// Hash of the input to the procedure.
    pub input_hash: String,
    /// Hash of the output generated by the procedure.
    pub output_hash: String,
    /// Hash of the closure (environment and dependencies).
    pub closure_hash: String,
    /// Hash of the executed procedure (code/binary).
    pub procedure_hash: String,
    /// Post-Quantum Cryptography seal for future-proofing signatures.
    pub pqc_seal: String,
    /// Hash of the previous receipt to form a verifiable chain.
    pub previous_receipt_hash: String,
    /// Identity of the actor performing the execution.
    pub actor_id: String,
    /// Transport mechanism or protocol used.
    pub transport: String,
    /// Unique identifier for the execution session.
    pub session_id: String,
    /// Pointer for replayability of the execution.
    pub replay_pointer: String,
    /// The final verdict of the execution.
    pub verdict: Verdict,
}

impl TruexReceipt {
    /// Creates a new `TruexReceipt`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        input_hash: String,
        output_hash: String,
        closure_hash: String,
        procedure_hash: String,
        pqc_seal: String,
        previous_receipt_hash: String,
        actor_id: String,
        transport: String,
        session_id: String,
        replay_pointer: String,
        verdict: Verdict,
    ) -> Self {
        Self {
            input_hash,
            output_hash,
            closure_hash,
            procedure_hash,
            pqc_seal,
            previous_receipt_hash,
            actor_id,
            transport,
            session_id,
            replay_pointer,
            verdict,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receipt_creation_and_validity() {
        let parties = vec!["alice".to_string(), "bob".to_string()];
        let receipt = SettlementReceipt::new("tx-001", parties, 42.0, "USD");
        assert!(receipt.is_valid());
        assert_eq!(receipt.currency, "USD");
        assert_eq!(receipt.amount, 42.0);
    }

    #[test]
    fn test_verify_with_correct_secret() {
        let parties = vec!["agent-a".to_string()];
        let mut receipt = SettlementReceipt::new("tx-002", parties, 100.0, "EUR");
        let secret = b"super-secret";
        receipt.signature = SettlementReceipt::sign(&receipt.digest, secret);
        assert!(receipt.verify(secret));
        assert!(!receipt.verify(b"wrong-secret"));
    }

    #[test]
    fn test_with_attribute() {
        let receipt = SettlementReceipt::new("tx-003", vec!["x".to_string()], 1.0, "GBP")
            .with_attribute("network", "mainnet")
            .with_attribute("priority", "high");
        assert_eq!(receipt.attributes["network"], "mainnet");
        assert_eq!(receipt.attributes["priority"], "high");
    }

    #[test]
    fn test_invalid_receipt_zero_amount() {
        let receipt = SettlementReceipt::new("tx-004", vec!["x".to_string()], 0.0, "USD");
        assert!(!receipt.is_valid());
    }

    #[test]
    fn test_to_json_contains_receipt_id() {
        let receipt = SettlementReceipt::new("tx-005", vec!["x".to_string()], 5.0, "USD");
        let json = receipt.to_json();
        assert!(json.contains("receipt_id"));
        assert!(json.contains("tx-005"));
    }
}
