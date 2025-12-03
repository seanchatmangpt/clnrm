//! Content-Addressable Ontology Store
//!
//! Stores Σ* ontologies indexed by content hash for immutability and deduplication.

use super::sigma::{ContentHash, SigmaBase};
use crate::error::{CleanroomError, Result};
use std::collections::HashMap;
use std::sync::RwLock;

/// Content-addressable ontology store
///
/// Stores SigmaBase ontologies indexed by their content hash.
/// Provides immutability guarantees and deduplication.
pub struct OntologyStore {
    /// In-memory store (hash → ontology)
    /// In production, this could be backed by a database or file system
    store: RwLock<HashMap<ContentHash, SigmaBase>>,
}

impl OntologyStore {
    /// Create a new empty ontology store
    pub fn new() -> Self {
        Self {
            store: RwLock::new(HashMap::new()),
        }
    }

    /// Store an ontology
    ///
    /// The ontology's hash must match its computed hash (verified)
    pub fn put(&self, ontology: SigmaBase) -> Result<ContentHash> {
        // Verify hash matches content
        let computed_hash = ontology.compute_hash();
        if ontology.hash != computed_hash {
            return Err(CleanroomError::internal_error(format!(
                "Ontology hash mismatch: declared {}, computed {}",
                ontology.hash, computed_hash
            )));
        }

        // Validate ontology
        ontology.validate()?;

        // Store
        let hash = ontology.hash.clone();
        self.store
            .write()
            .map_err(|e| CleanroomError::internal_error(format!("Lock poisoned: {}", e)))?
            .insert(hash.clone(), ontology);

        Ok(hash)
    }

    /// Retrieve an ontology by hash
    pub fn get(&self, hash: &ContentHash) -> Result<SigmaBase> {
        self.store
            .read()
            .map_err(|e| CleanroomError::internal_error(format!("Lock poisoned: {}", e)))?
            .get(hash)
            .cloned()
            .ok_or_else(|| CleanroomError::internal_error(format!("Ontology not found: {}", hash)))
    }

    /// Check if an ontology exists
    pub fn contains(&self, hash: &ContentHash) -> bool {
        self.store
            .read()
            .map(|store| store.contains_key(hash))
            .unwrap_or(false)
    }

    /// List all ontology hashes
    pub fn list(&self) -> Result<Vec<ContentHash>> {
        Ok(self
            .store
            .read()
            .map_err(|e| CleanroomError::internal_error(format!("Lock poisoned: {}", e)))?
            .keys()
            .cloned()
            .collect())
    }

    /// Get number of stored ontologies
    pub fn len(&self) -> usize {
        self.store.read().map(|store| store.len()).unwrap_or(0)
    }

    /// Check if store is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Delete an ontology (use with caution - breaks immutability)
    ///
    /// This should only be used for garbage collection or administrative purposes
    pub fn delete(&self, hash: &ContentHash) -> Result<()> {
        self.store
            .write()
            .map_err(|e| CleanroomError::internal_error(format!("Lock poisoned: {}", e)))?
            .remove(hash);
        Ok(())
    }

    /// Clear all ontologies (use with caution)
    pub fn clear(&self) -> Result<()> {
        self.store
            .write()
            .map_err(|e| CleanroomError::internal_error(format!("Lock poisoned: {}", e)))?
            .clear();
        Ok(())
    }
}

impl Default for OntologyStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::sigma::{SemVer, TelemetryDef};

    fn create_test_ontology(description: &str) -> SigmaBase {
        let timestamp = "2025-01-01T00:00:00Z".to_string();

        let sigma = SigmaBase {
            version: SemVer::new(1, 0, 0),
            hash: ContentHash::from_string("placeholder"),
            description: description.to_string(),
            services: HashMap::new(),
            networks: HashMap::new(),
            volumes: HashMap::new(),
            volume_mounts: HashMap::new(),
            telemetry: TelemetryDef {
                otel_collector: None,
                weaver: None,
                service_instrumentation: HashMap::new(),
            },
            metadata: HashMap::new(),
            created_at: timestamp,
        };

        // Compute actual hash
        let hash = sigma.compute_hash();

        // Create final version with correct hash
        SigmaBase { hash, ..sigma }
    }

    #[test]
    fn test_store_put_and_get() {
        // Arrange: Create store and ontology
        let store = OntologyStore::new();
        let ontology = create_test_ontology("Test ontology");
        let expected_hash = ontology.hash.clone();

        // Act: Store and retrieve
        let stored_hash = store.put(ontology.clone()).unwrap();
        let retrieved = store.get(&stored_hash).unwrap();

        // Assert: Hash matches and content is identical
        assert_eq!(stored_hash, expected_hash);
        assert_eq!(retrieved.description, ontology.description);
    }

    #[test]
    fn test_store_rejects_invalid_hash() {
        // Arrange: Create ontology with wrong hash
        let store = OntologyStore::new();
        let mut ontology = create_test_ontology("Test");
        ontology.hash = ContentHash::from_string("wrong-hash");

        // Act & Assert: Store rejects invalid hash
        assert!(store.put(ontology).is_err());
    }

    #[test]
    fn test_store_contains() {
        // Arrange: Create and store ontology
        let store = OntologyStore::new();
        let ontology = create_test_ontology("Test");
        let hash = store.put(ontology).unwrap();

        // Act & Assert: Contains returns true
        assert!(store.contains(&hash));
        assert!(!store.contains(&ContentHash::from_string("nonexistent")));
    }

    #[test]
    fn test_store_list() {
        // Arrange: Store multiple ontologies
        let store = OntologyStore::new();
        let ont1 = create_test_ontology("Ontology 1");
        let ont2 = create_test_ontology("Ontology 2");

        let hash1 = store.put(ont1).unwrap();
        let hash2 = store.put(ont2).unwrap();

        // Act: List all hashes
        let hashes = store.list().unwrap();

        // Assert: Both hashes present
        assert_eq!(hashes.len(), 2);
        assert!(hashes.contains(&hash1));
        assert!(hashes.contains(&hash2));
    }

    #[test]
    fn test_store_delete() {
        // Arrange: Store ontology
        let store = OntologyStore::new();
        let ontology = create_test_ontology("Test");
        let hash = store.put(ontology).unwrap();

        assert!(store.contains(&hash));

        // Act: Delete ontology
        store.delete(&hash).unwrap();

        // Assert: No longer in store
        assert!(!store.contains(&hash));
    }

    #[test]
    fn test_store_deduplication() {
        // Arrange: Create two identical ontologies
        let store = OntologyStore::new();
        let ont1 = create_test_ontology("Same content");
        let ont2 = create_test_ontology("Same content");

        // Act: Store both
        let hash1 = store.put(ont1).unwrap();
        let hash2 = store.put(ont2).unwrap();

        // Assert: Same hash (deduplicated)
        assert_eq!(hash1, hash2);
        assert_eq!(store.len(), 1);
    }
}
