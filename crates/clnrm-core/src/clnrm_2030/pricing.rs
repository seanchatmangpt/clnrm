use crate::truex::marketplace::ConsequenceListing;

/// The N-dimensional vector specified in the Truex PRD.
/// Dimensions: ontology fit, procedure completeness, receipt strength, replay depth, compute cost.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PricingVector {
    pub ontology_fit: f64,
    pub procedure_completeness: f64,
    pub receipt_strength: f64,
    pub replay_depth: f64,
    pub compute_cost: f64,
}

impl PricingVector {
    pub fn new(
        ontology_fit: f64,
        procedure_completeness: f64,
        receipt_strength: f64,
        replay_depth: f64,
        compute_cost: f64,
    ) -> Self {
        Self {
            ontology_fit,
            procedure_completeness,
            receipt_strength,
            replay_depth,
            compute_cost,
        }
    }
}

/// The mathematical engine that dynamically prices `ConsequenceListing` items
/// based on the N-dimensional vector specified in the Truex PRD.
#[derive(Debug, Clone)]
pub struct ValueVectorPricingEngine {
    pub base_price: f64,
    pub ontology_weight: f64,
    pub procedure_weight: f64,
    pub receipt_weight: f64,
    pub replay_weight: f64,
    pub compute_cost_weight: f64,
}

impl ValueVectorPricingEngine {
    pub fn new(
        base_price: f64,
        ontology_weight: f64,
        procedure_weight: f64,
        receipt_weight: f64,
        replay_weight: f64,
        compute_cost_weight: f64,
    ) -> Self {
        Self {
            base_price,
            ontology_weight,
            procedure_weight,
            receipt_weight,
            replay_weight,
            compute_cost_weight,
        }
    }

    /// Calculates the dynamic price of a ConsequenceListing based on the PRD's N-dimensional vector.
    /// It combines the inherent pricing properties of the listing with the dynamic metrics from the vector.
    pub fn calculate_price(
        &self,
        listing: &ConsequenceListing,
        vector: &PricingVector,
    ) -> f64 {
        // We incorporate both the vector's dimensions and the listing's existing intrinsic values.
        let effective_ontology = vector.ontology_fit.max(listing.pricing.ontology_fit);
        let effective_receipt = vector.receipt_strength.max(listing.pricing.receipt_strength as f64);

        let ontology_value = effective_ontology * self.ontology_weight;
        let procedure_value = vector.procedure_completeness * self.procedure_weight;
        let receipt_value = effective_receipt * self.receipt_weight;
        let replay_value = vector.replay_depth * self.replay_weight;

        // Compute cost represents an overhead/latency factor reducing the listing's value
        let compute_penalty = vector.compute_cost * self.compute_cost_weight;

        // Listing intrinsic trust value bonus
        let trust_bonus = listing.pricing.counterparty_trust_reduction * 50.0;

        let total_value = self.base_price
            + ontology_value
            + procedure_value
            + receipt_value
            + replay_value
            + trust_bonus
            - compute_penalty;

        // Minimum price bounds
        if total_value < 0.0 {
            0.0
        } else {
            total_value
        }
    }
}
