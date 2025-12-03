//! Test Receipt Structure (Γₜ)
//!
//! Cryptographically verifiable receipts for test executions.
//! Every test execution emits a receipt that proves:
//! - What was tested (scenario, capabilities)
//! - How it was tested (environment ontology, constraints)
//! - That validation passed (Weaver, timing, hermeticity)
//! - When it was tested (timestamp, hash chain)

use crate::capabilities::{CapabilityId, ConstraintSet, EffectSet, ScenarioId};
use crate::environment::sigma::ContentHash;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Receipt identifier (content-addressable hash)
pub type ReceiptId = ContentHash;

/// Test receipt - cryptographically verifiable proof of test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReceipt {
    /// Receipt ID (SHA-256 hash of receipt content)
    pub id: ReceiptId,

    /// Scenario that was executed
    pub scenario_id: ScenarioId,

    /// Capabilities exercised
    pub capabilities: Vec<CapabilityId>,

    /// Effects produced
    pub effects: EffectSet,

    /// Environment ontology used
    pub sigma_hash: ContentHash,

    /// Container image digests (immutable proof)
    pub image_digests: HashMap<String, ImageDigest>,

    /// Constraints that were enforced
    pub constraints: ConstraintSet,

    /// Weaver validation proof
    pub weaver_proof: Option<WeaverProof>,

    /// Timing footprint (hot/warm/cold paths)
    pub timing_footprint: TimingFootprint,

    /// Hermeticity witness (proof of isolation)
    pub hermeticity_witness: HermeticityWitness,

    /// Previous receipt in hash chain (for ordering)
    pub previous_receipt: Option<ReceiptId>,

    /// Cryptographic signature (optional)
    pub signature: Option<ReceiptSignature>,

    /// Timestamp of execution
    pub timestamp: String, // ISO 8601

    /// Metadata (tags, labels, custom data)
    pub metadata: HashMap<String, String>,
}

/// Container image digest (SHA-256)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageDigest {
    /// Image name (e.g., "postgres:14")
    pub image: String,

    /// SHA-256 digest (e.g., "sha256:abc123...")
    pub digest: String,

    /// Platform (e.g., "linux/amd64")
    pub platform: Option<String>,
}

/// Weaver validation proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaverProof {
    /// Registry path that was validated
    pub registry_path: String,

    /// Schema version
    pub schema_version: String,

    /// Validation result
    pub validation_passed: bool,

    /// Warnings (if any)
    pub warnings: Vec<String>,

    /// OTEL graph proof (span IDs, trace IDs)
    pub otel_graph: OtelGraphProof,

    /// Weaver validation timestamp
    pub validated_at: String, // ISO 8601
}

/// OTEL graph proof (telemetry evidence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelGraphProof {
    /// Trace IDs observed
    pub trace_ids: Vec<String>,

    /// Span count by operation
    pub span_counts: HashMap<String, u64>,

    /// Metric names observed
    pub metrics: Vec<String>,

    /// Log entries count
    pub log_entries: u64,
}

/// Timing footprint (hot/warm/cold path measurements)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingFootprint {
    /// Total execution time
    pub total_duration: Duration,

    /// Hot path timings (sub-millisecond)
    pub hot_paths: Vec<PathTiming>,

    /// Warm path timings (milliseconds)
    pub warm_paths: Vec<PathTiming>,

    /// Cold path timings (seconds)
    pub cold_paths: Vec<PathTiming>,

    /// τ violations (paths that exceeded their latency band)
    pub tau_violations: Vec<TimingViolation>,
}

/// Individual path timing measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathTiming {
    /// Operation name
    pub operation: String,

    /// Duration
    pub duration: Duration,

    /// Expected latency band
    pub expected_band: String, // "hot", "warm", "cold"

    /// Whether timing met constraint
    pub met_constraint: bool,
}

/// Timing constraint violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingViolation {
    /// Operation that violated constraint
    pub operation: String,

    /// Actual duration
    pub actual_duration: Duration,

    /// Expected max duration
    pub expected_max: Duration,

    /// Violation severity (how much over limit)
    pub severity: f64, // actual / expected
}

/// Hermeticity witness (proof of isolation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermeticityWitness {
    /// Network isolation verified
    pub network_isolated: bool,

    /// External connections detected (should be empty for hermetic)
    pub external_connections: Vec<String>,

    /// Filesystem isolation verified
    pub filesystem_isolated: bool,

    /// Non-hermetic paths accessed (if any)
    pub non_hermetic_paths: Vec<String>,

    /// Process isolation verified
    pub process_isolated: bool,

    /// Determinism verified (same inputs → same outputs)
    pub deterministic: bool,

    /// Determinism violations (if any)
    pub determinism_violations: Vec<String>,
}

/// Cryptographic signature (Ed25519)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptSignature {
    /// Public key (hex-encoded)
    pub public_key: String,

    /// Signature (hex-encoded)
    pub signature: String,

    /// Signing algorithm
    pub algorithm: String, // "ed25519"

    /// Signature timestamp
    pub signed_at: String, // ISO 8601
}

impl TestReceipt {
    /// Compute receipt ID (SHA-256 hash of receipt content)
    ///
    /// Excludes the `id` and `signature` fields to avoid circular dependency
    pub fn compute_id(&self) -> ReceiptId {
        use serde::Serialize;
        use sha2::{Digest, Sha256};

        // Create hashable version without id and signature
        #[derive(Serialize)]
        struct TestReceiptForHashing<'a> {
            scenario_id: &'a ScenarioId,
            capabilities: &'a Vec<CapabilityId>,
            effects: &'a EffectSet,
            sigma_hash: &'a ContentHash,
            image_digests: &'a HashMap<String, ImageDigest>,
            constraints: &'a ConstraintSet,
            weaver_proof: &'a Option<WeaverProof>,
            timing_footprint: &'a TimingFootprint,
            hermeticity_witness: &'a HermeticityWitness,
            previous_receipt: &'a Option<ReceiptId>,
            timestamp: &'a str,
            metadata: &'a HashMap<String, String>,
        }

        let hashable = TestReceiptForHashing {
            scenario_id: &self.scenario_id,
            capabilities: &self.capabilities,
            effects: &self.effects,
            sigma_hash: &self.sigma_hash,
            image_digests: &self.image_digests,
            constraints: &self.constraints,
            weaver_proof: &self.weaver_proof,
            timing_footprint: &self.timing_footprint,
            hermeticity_witness: &self.hermeticity_witness,
            previous_receipt: &self.previous_receipt,
            timestamp: &self.timestamp,
            metadata: &self.metadata,
        };

        let serialized =
            serde_json::to_string(&hashable).expect("Failed to serialize TestReceipt for hashing");

        let mut hasher = Sha256::new();
        hasher.update(serialized.as_bytes());
        let result = hasher.finalize();

        ContentHash::from_bytes(&result)
    }

    /// Validate receipt integrity
    pub fn validate(&self) -> crate::error::Result<()> {
        // Verify receipt ID matches computed hash
        let computed_id = self.compute_id();
        if self.id != computed_id {
            return Err(crate::error::CleanroomError::internal_error(format!(
                "Receipt ID mismatch: declared {}, computed {}",
                self.id, computed_id
            )));
        }

        // Verify timing footprint consistency
        if self.timing_footprint.total_duration < Duration::from_secs(0) {
            return Err(crate::error::CleanroomError::internal_error(
                "Invalid timing footprint: negative duration",
            ));
        }

        // Verify hermeticity witness consistency
        if self.constraints.hermetic && !self.hermeticity_witness.external_connections.is_empty() {
            return Err(crate::error::CleanroomError::internal_error(format!(
                "Hermeticity violation: {} external connections detected",
                self.hermeticity_witness.external_connections.len()
            )));
        }

        Ok(())
    }

    /// Verify signature (if present)
    #[cfg(feature = "crypto")]
    pub fn verify_signature(&self) -> crate::error::Result<bool> {
        if let Some(sig) = &self.signature {
            // Decode public key and signature
            let public_key_bytes = hex::decode(&sig.public_key).map_err(|e| {
                crate::error::CleanroomError::internal_error(format!(
                    "Failed to decode public key: {}",
                    e
                ))
            })?;

            let signature_bytes = hex::decode(&sig.signature).map_err(|e| {
                crate::error::CleanroomError::internal_error(format!(
                    "Failed to decode signature: {}",
                    e
                ))
            })?;

            // Verify algorithm
            if sig.algorithm != "ed25519" {
                return Err(crate::error::CleanroomError::internal_error(format!(
                    "Unsupported signature algorithm: {}",
                    sig.algorithm
                )));
            }

            // Compute message hash (receipt ID)
            let message = self.compute_id();

            // Verify signature using Ed25519
            use ed25519_dalek::{Signature, Verifier, VerifyingKey};

            let public_key =
                VerifyingKey::from_bytes(&public_key_bytes.try_into().map_err(|_| {
                    crate::error::CleanroomError::internal_error("Invalid public key length")
                })?)
                .map_err(|e| {
                    crate::error::CleanroomError::internal_error(format!(
                        "Failed to parse public key: {}",
                        e
                    ))
                })?;

            let signature = Signature::from_bytes(&signature_bytes.try_into().map_err(|_| {
                crate::error::CleanroomError::internal_error("Invalid signature length")
            })?);

            public_key
                .verify(message.as_str().as_bytes(), &signature)
                .map_err(|e| {
                    crate::error::CleanroomError::internal_error(format!(
                        "Signature verification failed: {}",
                        e
                    ))
                })?;

            Ok(true)
        } else {
            Ok(false) // No signature present
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capabilities::{LatencyBand, ResourceLimits};

    fn create_test_receipt() -> TestReceipt {
        let mut image_digests = HashMap::new();
        image_digests.insert(
            "postgres".to_string(),
            ImageDigest {
                image: "postgres:14".to_string(),
                digest: "sha256:abc123".to_string(),
                platform: Some("linux/amd64".to_string()),
            },
        );

        TestReceipt {
            id: ContentHash::from_string("placeholder"),
            scenario_id: ScenarioId("test-scenario".to_string()),
            capabilities: vec![CapabilityId("hermetic_execution".to_string())],
            effects: EffectSet::new(),
            sigma_hash: ContentHash::from_string("sigma-hash"),
            image_digests,
            constraints: ConstraintSet {
                hermetic: true,
                latency_band: LatencyBand::Hot {
                    max_duration: Duration::from_millis(1),
                },
                deterministic: true,
                resource_limits: ResourceLimits::default(),
                idempotent: true,
                max_execution_time: Some(Duration::from_secs(60)),
            },
            weaver_proof: None,
            timing_footprint: TimingFootprint {
                total_duration: Duration::from_millis(100),
                hot_paths: vec![],
                warm_paths: vec![],
                cold_paths: vec![],
                tau_violations: vec![],
            },
            hermeticity_witness: HermeticityWitness {
                network_isolated: true,
                external_connections: vec![],
                filesystem_isolated: true,
                non_hermetic_paths: vec![],
                process_isolated: true,
                deterministic: true,
                determinism_violations: vec![],
            },
            previous_receipt: None,
            signature: None,
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_receipt_compute_id() {
        // Arrange
        let receipt = create_test_receipt();

        // Act
        let id = receipt.compute_id();

        // Assert
        assert!(!id.as_str().is_empty());
        assert_eq!(id, receipt.compute_id()); // Deterministic
    }

    #[test]
    fn test_receipt_validation_succeeds() {
        // Arrange
        let mut receipt = create_test_receipt();
        receipt.id = receipt.compute_id();

        // Act & Assert
        assert!(receipt.validate().is_ok());
    }

    #[test]
    fn test_receipt_validation_fails_on_id_mismatch() {
        // Arrange
        let mut receipt = create_test_receipt();
        receipt.id = ContentHash::from_string("wrong-id");

        // Act & Assert
        assert!(receipt.validate().is_err());
    }

    #[test]
    fn test_receipt_validation_fails_on_hermeticity_violation() {
        // Arrange
        let mut receipt = create_test_receipt();
        receipt.id = receipt.compute_id();
        receipt.hermeticity_witness.external_connections = vec!["8.8.8.8:53".to_string()];

        // Act & Assert
        assert!(receipt.validate().is_err());
    }
}
