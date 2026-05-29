use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use chrono::Utc;

/// Represents a missing closure in the Truex Escrow Admission system.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MissingClosure {
    pub law_id: String,
    pub description: String,
    pub expected_elements: Vec<String>,
    pub missing_elements: Vec<String>,
}

/// Represents the specific semantic rule that failed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FailedRule {
    pub rule_id: String,
    pub description: String,
    pub context: String,
}

/// Represents a durable refusal (⊥) from the Escrow Admission layer.
/// This captures the exact missing closure, the specific rule that failed,
/// and the generating replay fixture to prove exactly why the transaction was denied.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurableRefusal {
    /// The unique identifier for this refusal.
    pub refusal_id: String,
    /// The timestamp when the refusal occurred.
    pub timestamp: String,
    /// The exact missing closure that caused the refusal.
    pub missing_closure: MissingClosure,
    /// The specific rule that failed.
    pub failed_rule: FailedRule,
    /// The generating replay fixture, captured as a JSON value to allow replay
    /// without creating cyclical dependencies on the truex-core crate.
    pub generating_fixture: serde_json::Value,
    /// A cryptographic hash ensuring the integrity of the refusal event.
    pub signature: Option<String>,
}

impl DurableRefusal {
    /// Creates a new durable refusal with the specified components.
    pub fn new(
        missing_closure: MissingClosure,
        failed_rule: FailedRule,
        generating_fixture: serde_json::Value,
    ) -> Self {
        let mut refusal = Self {
            refusal_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now().to_rfc3339(),
            missing_closure,
            failed_rule,
            generating_fixture,
            signature: None,
        };
        
        refusal.seal();
        refusal
    }

    /// Generates a cryptographic seal over the durable refusal payload
    /// to prevent tampering and ensure non-repudiation.
    pub fn seal(&mut self) {
        let mut hasher = Sha256::new();
        hasher.update(self.refusal_id.as_bytes());
        hasher.update(b"|");
        hasher.update(self.timestamp.as_bytes());
        hasher.update(b"|");
        hasher.update(self.missing_closure.law_id.as_bytes());
        hasher.update(b"|");
        hasher.update(self.failed_rule.rule_id.as_bytes());
        
        // Include fixture hash to ensure the exact payload that caused failure is tied to the signature
        let fixture_str = serde_json::to_string(&self.generating_fixture).unwrap_or_default();
        hasher.update(b"|");
        hasher.update(fixture_str.as_bytes());

        let result = hasher.finalize();
        self.signature = Some(hex::encode(result));
    }
}
