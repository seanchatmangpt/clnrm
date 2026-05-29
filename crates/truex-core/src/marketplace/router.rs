use super::listing::{Listing, ValueVector};
use uuid::Uuid;
use anyhow::{Result, Context};

#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintType {
    LessThanOrEqual,
    GreaterThanOrEqual,
    Equal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DimensionConstraint {
    pub dimension_index: usize,
    pub constraint_type: ConstraintType,
    pub value: f64,
}

impl DimensionConstraint {
    pub fn new(dimension_index: usize, constraint_type: ConstraintType, value: f64) -> Self {
        Self {
            dimension_index,
            constraint_type,
            value,
        }
    }

    pub fn is_satisfied(&self, vector: &ValueVector) -> bool {
        if self.dimension_index >= vector.dimensions.len() {
            return false;
        }
        let val = vector.dimensions[self.dimension_index];
        if !val.is_finite() || !self.value.is_finite() {
            return false;
        }
        match self.constraint_type {
            ConstraintType::LessThanOrEqual => val <= self.value,
            ConstraintType::GreaterThanOrEqual => val >= self.value,
            ConstraintType::Equal => (val - self.value).abs() < 1e-9,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouteRequest {
    pub preference_weights: ValueVector,
    pub constraints: Vec<DimensionConstraint>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouteResponse {
    pub listing_id: Uuid,
    pub score: f64,
    pub matching_vector: ValueVector,
}

pub struct PriceRoutingEngine;

impl PriceRoutingEngine {
    pub fn route(
        listings: &[Listing],
        request: &RouteRequest,
    ) -> Result<Vec<RouteResponse>> {
        let mut responses = Vec::new();

        for listing in listings {
            if !listing.active {
                continue;
            }

            // Verify all constraints are satisfied
            let mut satisfied = true;
            for constraint in &request.constraints {
                if !constraint.is_satisfied(&listing.value_vector) {
                    satisfied = false;
                    break;
                }
            }

            if !satisfied {
                continue;
            }

            // Calculate the score (utility score) using dot product
            let score = listing
                .value_vector
                .dot(&request.preference_weights)
                .context("Failed to compute dot product for routing utility score")?;

            responses.push(RouteResponse {
                listing_id: listing.id,
                score,
                matching_vector: listing.value_vector.clone(),
            });
        }

        // Sort by score descending
        responses.sort_by(|a, b| {
            b.score.total_cmp(&a.score)
        });

        Ok(responses)
    }
}
