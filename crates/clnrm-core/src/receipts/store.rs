//! Receipt Store with Hash Chain Validation
//!
//! Content-addressable store for test receipts with hash chain validation.
//! Receipts form a tamper-evident chain where each receipt references the
//! previous receipt's hash, creating an immutable audit trail.

use super::receipt::{ReceiptId, TestReceipt};
use crate::error::{CleanroomError, Result};
use std::collections::HashMap;
use std::sync::RwLock;

/// Content-addressable receipt store with hash chain validation
pub struct ReceiptStore {
    /// In-memory store (receipt ID → receipt)
    /// In production, this could be backed by a database or file system
    store: RwLock<HashMap<ReceiptId, TestReceipt>>,

    /// Hash chain head (most recent receipt)
    chain_head: RwLock<Option<ReceiptId>>,
}

impl ReceiptStore {
    /// Create a new empty receipt store
    pub fn new() -> Self {
        Self {
            store: RwLock::new(HashMap::new()),
            chain_head: RwLock::new(None),
        }
    }

    /// Store a receipt
    ///
    /// The receipt's ID must match its computed hash (verified)
    /// If the receipt has a previous_receipt, it must exist in the store
    pub fn put(&self, receipt: TestReceipt) -> Result<ReceiptId> {
        // Validate receipt integrity
        receipt.validate()?;

        // Verify previous receipt exists (if specified)
        if let Some(prev_id) = &receipt.previous_receipt {
            if !self.contains(prev_id) {
                return Err(CleanroomError::internal_error(&format!(
                    "Previous receipt not found in chain: {}",
                    prev_id
                )));
            }
        }

        // Store receipt
        let id = receipt.id.clone();
        self.store
            .write()
            .map_err(|e| CleanroomError::internal_error(&format!("Lock poisoned: {}", e)))?
            .insert(id.clone(), receipt);

        // Update chain head
        *self
            .chain_head
            .write()
            .map_err(|e| CleanroomError::internal_error(&format!("Lock poisoned: {}", e)))? =
            Some(id.clone());

        Ok(id)
    }

    /// Retrieve a receipt by ID
    pub fn get(&self, id: &ReceiptId) -> Result<TestReceipt> {
        self.store
            .read()
            .map_err(|e| CleanroomError::internal_error(&format!("Lock poisoned: {}", e)))?
            .get(id)
            .cloned()
            .ok_or_else(|| CleanroomError::internal_error(&format!("Receipt not found: {}", id)))
    }

    /// Check if a receipt exists
    pub fn contains(&self, id: &ReceiptId) -> bool {
        self.store
            .read()
            .map(|store| store.contains_key(id))
            .unwrap_or(false)
    }

    /// Get the hash chain head (most recent receipt)
    pub fn get_chain_head(&self) -> Option<ReceiptId> {
        self.chain_head
            .read()
            .ok()
            .and_then(|head| head.as_ref().cloned())
    }

    /// Validate hash chain integrity
    ///
    /// Walks backwards from the chain head to ensure all links are valid
    pub fn validate_chain(&self) -> Result<ChainValidationResult> {
        let head_id = match self.get_chain_head() {
            Some(id) => id,
            None => {
                return Ok(ChainValidationResult {
                    valid: true,
                    length: 0,
                    broken_links: vec![],
                    missing_receipts: vec![],
                })
            }
        };

        let mut current_id = Some(head_id);
        let mut visited = vec![];
        let mut broken_links = vec![];
        let mut missing_receipts = vec![];

        while let Some(id) = current_id {
            // Check for cycles
            if visited.contains(&id) {
                broken_links.push(format!("Cycle detected at receipt: {}", id));
                break;
            }

            visited.push(id.clone());

            // Get receipt
            let receipt = match self.get(&id) {
                Ok(r) => r,
                Err(_) => {
                    missing_receipts.push(id.as_str().to_string());
                    break;
                }
            };

            // Validate receipt
            if let Err(e) = receipt.validate() {
                broken_links.push(format!("Invalid receipt {}: {}", id, e));
                break;
            }

            // Move to previous receipt
            current_id = receipt.previous_receipt;
        }

        Ok(ChainValidationResult {
            valid: broken_links.is_empty() && missing_receipts.is_empty(),
            length: visited.len(),
            broken_links,
            missing_receipts,
        })
    }

    /// Get receipt chain from head backwards (limited by max_depth)
    pub fn get_chain(&self, max_depth: Option<usize>) -> Result<Vec<TestReceipt>> {
        let head_id = match self.get_chain_head() {
            Some(id) => id,
            None => return Ok(vec![]),
        };

        let mut chain = vec![];
        let mut current_id = Some(head_id);
        let max = max_depth.unwrap_or(usize::MAX);

        while let Some(id) = current_id {
            if chain.len() >= max {
                break;
            }

            let receipt = self.get(&id)?;
            current_id = receipt.previous_receipt.clone();
            chain.push(receipt);
        }

        Ok(chain)
    }

    /// List all receipt IDs
    pub fn list(&self) -> Result<Vec<ReceiptId>> {
        Ok(self
            .store
            .read()
            .map_err(|e| CleanroomError::internal_error(&format!("Lock poisoned: {}", e)))?
            .keys()
            .cloned()
            .collect())
    }

    /// Get number of stored receipts
    pub fn len(&self) -> usize {
        self.store.read().map(|store| store.len()).unwrap_or(0)
    }

    /// Check if store is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Delete a receipt (use with caution - breaks hash chain)
    ///
    /// This should only be used for garbage collection or administrative purposes
    /// Deleting a receipt in the middle of a chain breaks chain validation
    pub fn delete(&self, id: &ReceiptId) -> Result<()> {
        self.store
            .write()
            .map_err(|e| CleanroomError::internal_error(&format!("Lock poisoned: {}", e)))?
            .remove(id);
        Ok(())
    }

    /// Clear all receipts (use with caution)
    pub fn clear(&self) -> Result<()> {
        self.store
            .write()
            .map_err(|e| CleanroomError::internal_error(&format!("Lock poisoned: {}", e)))?
            .clear();

        *self
            .chain_head
            .write()
            .map_err(|e| CleanroomError::internal_error(&format!("Lock poisoned: {}", e)))? =
            None;

        Ok(())
    }
}

impl Default for ReceiptStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of hash chain validation
#[derive(Debug, Clone)]
pub struct ChainValidationResult {
    /// Whether the chain is valid
    pub valid: bool,

    /// Number of receipts in the chain
    pub length: usize,

    /// Broken links (validation errors)
    pub broken_links: Vec<String>,

    /// Missing receipts (referenced but not in store)
    pub missing_receipts: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capabilities::{
        CapabilityId, ConstraintSet, EffectSet, LatencyBand, ResourceLimits,
        ScenarioId,
    };
    use crate::environment::sigma::ContentHash;
    use crate::receipts::receipt::{
        HermeticityWitness, ImageDigest, TestReceipt, TimingFootprint,
    };
    use std::time::Duration;

    fn create_test_receipt(
        scenario_name: &str,
        previous: Option<ReceiptId>,
    ) -> TestReceipt {
        let mut image_digests = HashMap::new();
        image_digests.insert(
            "test".to_string(),
            ImageDigest {
                image: "alpine:latest".to_string(),
                digest: format!("sha256:{}", scenario_name),
                platform: Some("linux/amd64".to_string()),
            },
        );

        let receipt = TestReceipt {
            id: ContentHash::from_string("placeholder"),
            scenario_id: ScenarioId(scenario_name.to_string()),
            capabilities: vec![CapabilityId("test".to_string())],
            effects: EffectSet::new(),
            sigma_hash: ContentHash::from_string("test-sigma"),
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
                total_duration: Duration::from_millis(50),
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
            previous_receipt: previous,
            signature: None,
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            metadata: HashMap::new(),
        };

        // Compute actual ID
        let id = receipt.compute_id();
        TestReceipt { id, ..receipt }
    }

    #[test]
    fn test_store_put_and_get() {
        // Arrange
        let store = ReceiptStore::new();
        let receipt = create_test_receipt("test1", None);
        let expected_id = receipt.id.clone();

        // Act
        let stored_id = store.put(receipt.clone()).unwrap();
        let retrieved = store.get(&stored_id).unwrap();

        // Assert
        assert_eq!(stored_id, expected_id);
        assert_eq!(retrieved.scenario_id, receipt.scenario_id);
    }

    #[test]
    fn test_store_contains() {
        // Arrange
        let store = ReceiptStore::new();
        let receipt = create_test_receipt("test", None);
        let id = store.put(receipt).unwrap();

        // Act & Assert
        assert!(store.contains(&id));
        assert!(!store.contains(&ContentHash::from_string("nonexistent")));
    }

    #[test]
    fn test_chain_head_updated() {
        // Arrange
        let store = ReceiptStore::new();
        assert!(store.get_chain_head().is_none());

        // Act
        let receipt1 = create_test_receipt("test1", None);
        let id1 = store.put(receipt1).unwrap();

        // Assert
        assert_eq!(store.get_chain_head(), Some(id1.clone()));

        // Act - add another receipt
        let receipt2 = create_test_receipt("test2", Some(id1));
        let id2 = store.put(receipt2).unwrap();

        // Assert - head updated
        assert_eq!(store.get_chain_head(), Some(id2));
    }

    #[test]
    fn test_chain_validation_empty() {
        // Arrange
        let store = ReceiptStore::new();

        // Act
        let result = store.validate_chain().unwrap();

        // Assert
        assert!(result.valid);
        assert_eq!(result.length, 0);
    }

    #[test]
    fn test_chain_validation_single_receipt() {
        // Arrange
        let store = ReceiptStore::new();
        let receipt = create_test_receipt("test", None);
        store.put(receipt).unwrap();

        // Act
        let result = store.validate_chain().unwrap();

        // Assert
        assert!(result.valid);
        assert_eq!(result.length, 1);
        assert!(result.broken_links.is_empty());
        assert!(result.missing_receipts.is_empty());
    }

    #[test]
    fn test_chain_validation_multiple_receipts() {
        // Arrange
        let store = ReceiptStore::new();

        let receipt1 = create_test_receipt("test1", None);
        let id1 = store.put(receipt1).unwrap();

        let receipt2 = create_test_receipt("test2", Some(id1.clone()));
        let id2 = store.put(receipt2).unwrap();

        let receipt3 = create_test_receipt("test3", Some(id2));
        store.put(receipt3).unwrap();

        // Act
        let result = store.validate_chain().unwrap();

        // Assert
        assert!(result.valid);
        assert_eq!(result.length, 3);
        assert!(result.broken_links.is_empty());
        assert!(result.missing_receipts.is_empty());
    }

    #[test]
    fn test_chain_validation_fails_on_missing_receipt() {
        // Arrange
        let store = ReceiptStore::new();
        let missing_id = ContentHash::from_string("missing");

        let receipt = create_test_receipt("test", Some(missing_id.clone()));

        // Act & Assert - should fail because previous receipt doesn't exist
        assert!(store.put(receipt).is_err());
    }

    #[test]
    fn test_get_chain() {
        // Arrange
        let store = ReceiptStore::new();

        let receipt1 = create_test_receipt("test1", None);
        let id1 = store.put(receipt1).unwrap();

        let receipt2 = create_test_receipt("test2", Some(id1));
        let id2 = store.put(receipt2).unwrap();

        let receipt3 = create_test_receipt("test3", Some(id2));
        store.put(receipt3).unwrap();

        // Act
        let chain = store.get_chain(None).unwrap();

        // Assert
        assert_eq!(chain.len(), 3);
        assert_eq!(chain[0].scenario_id.0, "test3"); // Most recent first
        assert_eq!(chain[1].scenario_id.0, "test2");
        assert_eq!(chain[2].scenario_id.0, "test1");
    }

    #[test]
    fn test_get_chain_with_max_depth() {
        // Arrange
        let store = ReceiptStore::new();

        let receipt1 = create_test_receipt("test1", None);
        let id1 = store.put(receipt1).unwrap();

        let receipt2 = create_test_receipt("test2", Some(id1));
        let id2 = store.put(receipt2).unwrap();

        let receipt3 = create_test_receipt("test3", Some(id2));
        store.put(receipt3).unwrap();

        // Act
        let chain = store.get_chain(Some(2)).unwrap();

        // Assert - only most recent 2
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0].scenario_id.0, "test3");
        assert_eq!(chain[1].scenario_id.0, "test2");
    }
}
