use crate::truex::marketplace::ConsequenceListing;

// ── General-purpose pricing engine ──────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PricingEngine {
    pub amm_weight: f64,
    pub oracle_weight: f64,
    pub derivatives_weight: f64,
}

impl Default for PricingEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PricingEngine {
    pub fn new() -> Self {
        let w = 1.0 / 3.0;
        Self {
            amm_weight: w,
            oracle_weight: w,
            derivatives_weight: w,
        }
    }

    pub fn with_weights(amm: f64, oracle: f64, derivatives: f64) -> Result<Self, String> {
        let sum = amm + oracle + derivatives;
        if (sum - 1.0).abs() > 1e-9 {
            return Err(format!("Weights must sum to 1.0, got {:.10}", sum));
        }
        Ok(Self {
            amm_weight: amm,
            oracle_weight: oracle,
            derivatives_weight: derivatives,
        })
    }

    pub fn compute_price(&self, amm_price: f64, oracle_price: f64, derivatives_price: f64) -> f64 {
        amm_price * self.amm_weight
            + oracle_price * self.oracle_weight
            + derivatives_price * self.derivatives_weight
    }

    pub fn fair_value_range(&self, price: f64, volatility: f64) -> (f64, f64) {
        (price * (1.0 - volatility), price * (1.0 + volatility))
    }

    pub fn mark_to_market(&self, position_size: f64, entry_price: f64, current_price: f64) -> f64 {
        (current_price - entry_price) * position_size
    }

    pub fn liquidation_price(&self, entry: f64, leverage: f64, is_long: bool) -> f64 {
        if leverage == 0.0 {
            return entry;
        }
        if is_long {
            entry * (1.0 - 1.0 / leverage)
        } else {
            entry * (1.0 + 1.0 / leverage)
        }
    }
}

// ── Truex N-dimensional vector pricing ──────────────────────────────────────

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

    pub fn calculate_price(&self, listing: &ConsequenceListing, vector: &PricingVector) -> f64 {
        let effective_ontology = vector.ontology_fit.max(listing.pricing.ontology_fit);
        let effective_receipt = vector
            .receipt_strength
            .max(listing.pricing.receipt_strength as f64);

        let ontology_value = effective_ontology * self.ontology_weight;
        let procedure_value = vector.procedure_completeness * self.procedure_weight;
        let receipt_value = effective_receipt * self.receipt_weight;
        let replay_value = vector.replay_depth * self.replay_weight;
        let compute_penalty = vector.compute_cost * self.compute_cost_weight;
        let trust_bonus = listing.pricing.counterparty_trust_reduction * 50.0;

        let total_value = self.base_price
            + ontology_value
            + procedure_value
            + receipt_value
            + replay_value
            + trust_bonus
            - compute_penalty;

        if total_value < 0.0 {
            0.0
        } else {
            total_value
        }
    }
}
