use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::{Result, bail};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValueVector {
    pub dimensions: Vec<f64>,
}

impl ValueVector {
    pub fn new(dimensions: Vec<f64>) -> Self {
        Self { dimensions }
    }

    /// Computes the dot product of this value vector with a weights vector.
    pub fn dot(&self, weights: &ValueVector) -> Result<f64> {
        if self.dimensions.len() != weights.dimensions.len() {
            bail!(
                "Dimension mismatch: expected {} dimensions, got {}",
                self.dimensions.len(),
                weights.dimensions.len()
            );
        }
        let mut sum = 0.0;
        for (&a, &b) in self.dimensions.iter().zip(&weights.dimensions) {
            if !a.is_finite() || !b.is_finite() {
                bail!("ValueVector contains non-finite dimensions");
            }
            let prod = a * b;
            if !prod.is_finite() {
                bail!("Float overflow in vector multiplication");
            }
            sum += prod;
            if !sum.is_finite() {
                bail!("Float overflow in vector dot product sum");
            }
        }
        Ok(sum)
    }

    /// Computes the Euclidean distance between this vector and another.
    pub fn distance(&self, target: &ValueVector) -> Result<f64> {
        if self.dimensions.len() != target.dimensions.len() {
            bail!(
                "Dimension mismatch for distance calculation: expected {} dimensions, got {}",
                self.dimensions.len(),
                target.dimensions.len()
            );
        }
        let mut sum_sq = 0.0;
        for (&a, &b) in self.dimensions.iter().zip(&target.dimensions) {
            if !a.is_finite() || !b.is_finite() {
                bail!("ValueVector contains non-finite dimensions");
            }
            let diff = a - b;
            if !diff.is_finite() {
                bail!("Float overflow in distance difference calculation");
            }
            let sq = diff * diff;
            if !sq.is_finite() {
                bail!("Float overflow in distance squared difference calculation");
            }
            sum_sq += sq;
            if !sum_sq.is_finite() {
                bail!("Float overflow in distance sum of squares calculation");
            }
        }
        let dist = sum_sq.sqrt();
        if !dist.is_finite() {
            bail!("Float overflow in distance square root calculation");
        }
        Ok(dist)
    }

    /// Computes Manhattan distance between this vector and another.
    pub fn manhattan_distance(&self, target: &ValueVector) -> Result<f64> {
        if self.dimensions.len() != target.dimensions.len() {
            bail!(
                "Dimension mismatch for Manhattan distance: expected {} dimensions, got {}",
                self.dimensions.len(),
                target.dimensions.len()
            );
        }
        let mut sum_abs = 0.0;
        for (&a, &b) in self.dimensions.iter().zip(&target.dimensions) {
            if !a.is_finite() || !b.is_finite() {
                bail!("ValueVector contains non-finite dimensions");
            }
            let diff = (a - b).abs();
            if !diff.is_finite() {
                bail!("Float overflow in Manhattan distance difference calculation");
            }
            sum_abs += diff;
            if !sum_abs.is_finite() {
                bail!("Float overflow in Manhattan distance sum calculation");
            }
        }
        Ok(sum_abs)
    }

    /// Computes the cosine similarity between this vector and another.
    pub fn cosine_similarity(&self, target: &ValueVector) -> Result<f64> {
        if self.dimensions.len() != target.dimensions.len() {
            bail!(
                "Dimension mismatch for Cosine similarity: expected {} dimensions, got {}",
                self.dimensions.len(),
                target.dimensions.len()
            );
        }
        let dot = self.dot(target)?;
        let norm_a = self.norm()?;
        let norm_b = target.norm()?;
        if norm_a == 0.0 || norm_b == 0.0 {
            bail!("Cannot compute cosine similarity with a zero vector");
        }
        let denom = norm_a * norm_b;
        if !denom.is_finite() || denom == 0.0 {
            bail!("Cannot compute cosine similarity: denominator is non-finite or zero (underflow/overflow)");
        }
        let sim = dot / denom;
        if !sim.is_finite() {
            bail!("Float overflow/NaN in cosine similarity calculation");
        }
        Ok(sim)
    }

    /// Computes the Euclidean norm (L2 norm) of the vector.
    pub fn norm(&self) -> Result<f64> {
        let mut sum_sq = 0.0;
        for &x in &self.dimensions {
            if !x.is_finite() {
                bail!("ValueVector contains non-finite dimension");
            }
            let sq = x * x;
            if !sq.is_finite() {
                bail!("Float overflow in vector norm calculation");
            }
            sum_sq += sq;
            if !sum_sq.is_finite() {
                bail!("Float overflow in vector norm sum calculation");
            }
        }
        let norm_val = sum_sq.sqrt();
        if !norm_val.is_finite() {
            bail!("Float overflow in vector norm square root calculation");
        }
        Ok(norm_val)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Listing {
    pub id: Uuid,
    pub provider: String,
    pub name: String,
    pub value_vector: ValueVector,
    pub metadata_hash: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

impl Listing {
    pub fn new(
        provider: String,
        name: String,
        value_vector: ValueVector,
        metadata_hash: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            provider,
            name,
            value_vector,
            metadata_hash,
            active: true,
            created_at: Utc::now(),
        }
    }
}
