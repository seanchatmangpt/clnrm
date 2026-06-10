use serde::{Deserialize, Serialize};

/// Represents the value vector pricing model from Section 6.2.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValueVectorPricing {
    /// Fit within the public ontology
    pub ontology_fit: f64,
    /// Execution latency
    pub latency_ms: u64,
    /// Cryptographic receipt strength
    pub receipt_strength: u32,
    /// Reduction in counterparty trust
    pub counterparty_trust_reduction: f64,
}

impl ValueVectorPricing {
    /// Create a new ValueVectorPricing model
    pub fn new(
        ontology_fit: f64,
        latency_ms: u64,
        receipt_strength: u32,
        counterparty_trust_reduction: f64,
    ) -> Self {
        Self {
            ontology_fit,
            latency_ms,
            receipt_strength,
            counterparty_trust_reduction,
        }
    }

    /// Calculates an aggregate score representing the price or value of the consequence.
    pub fn calculate_aggregate_value(&self) -> f64 {
        // A simplistic fallback weighting for the n-dimensional model
        self.ontology_fit * 100.0
            + self.counterparty_trust_reduction * 50.0
            + (self.receipt_strength as f64) * 10.0
            - (self.latency_ms as f64) * 0.1
    }
}

/// N-Dimensional ConsequenceListing from Section 6.2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsequenceListing {
    /// Unique identifier for the listing
    pub id: String,
    /// Description of the admitted consequence
    pub description: String,
    /// N-Dimensional pricing model
    pub pricing: ValueVectorPricing,
    /// Status of the listing in the marketplace
    pub is_active: bool,
}

impl ConsequenceListing {
    /// Create a new ConsequenceListing
    pub fn new(id: String, description: String, pricing: ValueVectorPricing) -> Self {
        Self {
            id,
            description,
            pricing,
            is_active: true,
        }
    }
}
