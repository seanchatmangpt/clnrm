pub mod listing;
pub mod metadata;
pub mod router;

use listing::Listing;
use metadata::ListingMetadata;
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;
use anyhow::{Result, bail};

#[derive(Default)]
pub struct MarketplaceRegistry {
    listings: RwLock<HashMap<Uuid, Listing>>,
    metadata: RwLock<HashMap<String, ListingMetadata>>,
}

impl MarketplaceRegistry {
    pub fn new() -> Self {
        Self {
            listings: RwLock::new(HashMap::new()),
            metadata: RwLock::new(HashMap::new()),
        }
    }

    /// Registers a new listing. The listing's metadata_hash must match an already published metadata entry,
    /// or we will return an error indicating the metadata was not found.
    pub fn register_listing(&self, listing: Listing) -> Result<()> {
        let metadata_exists = {
            let meta_read = self.metadata.read().map_err(|e| anyhow::anyhow!("Lock error: {:?}", e))?;
            meta_read.contains_key(&listing.metadata_hash)
        };

        if !metadata_exists {
            bail!(
                "Cannot register listing: metadata with hash {} has not been published yet.",
                listing.metadata_hash
            );
        }

        let mut listings_write = self.listings.write().map_err(|e| anyhow::anyhow!("Lock error: {:?}", e))?;
        listings_write.insert(listing.id, listing);
        Ok(())
    }

    /// Publishes metadata and returns its calculated SHA-256 hash.
    pub fn publish_metadata(&self, metadata: ListingMetadata) -> Result<String> {
        let hash = metadata.compute_hash();
        let mut meta_write = self.metadata.write().map_err(|e| anyhow::anyhow!("Lock error: {:?}", e))?;
        meta_write.insert(hash.clone(), metadata);
        Ok(hash)
    }

    /// Retrieves a listing by ID.
    pub fn get_listing(&self, id: &Uuid) -> Result<Option<Listing>> {
        let listings_read = self.listings.read().map_err(|e| anyhow::anyhow!("Lock error: {:?}", e))?;
        Ok(listings_read.get(id).cloned())
    }

    /// Retrieves published metadata by its hash.
    pub fn get_metadata(&self, hash: &str) -> Result<Option<ListingMetadata>> {
        let meta_read = self.metadata.read().map_err(|e| anyhow::anyhow!("Lock error: {:?}", e))?;
        Ok(meta_read.get(hash).cloned())
    }

    /// Updates the status (active/inactive) of a listing.
    pub fn update_listing_status(&self, id: &Uuid, active: bool) -> Result<()> {
        let mut listings_write = self.listings.write().map_err(|e| anyhow::anyhow!("Lock error: {:?}", e))?;
        if let Some(listing) = listings_write.get_mut(id) {
            listing.active = active;
            Ok(())
        } else {
            bail!("Listing with ID {} not found", id);
        }
    }

    /// Returns a list of all currently active listings.
    pub fn list_active_listings(&self) -> Result<Vec<Listing>> {
        let listings_read = self.listings.read().map_err(|e| anyhow::anyhow!("Lock error: {:?}", e))?;
        Ok(listings_read
            .values()
            .filter(|l| l.active)
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use listing::ValueVector;
    use router::{PriceRoutingEngine, RouteRequest, DimensionConstraint, ConstraintType};

    #[test]
    fn test_value_vector_math() {
        let v1 = ValueVector::new(vec![1.0, 2.0, 3.0]);
        let v2 = ValueVector::new(vec![4.0, 5.0, 6.0]);

        // Dot product: 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
        let dot = v1.dot(&v2).unwrap();
        assert_eq!(dot, 32.0);

        // Distance: sqrt((1-4)^2 + (2-5)^2 + (3-6)^2) = sqrt(9 + 9 + 9) = sqrt(27) = 5.1961524...
        let dist = v1.distance(&v2).unwrap();
        assert!((dist - 27.0f64.sqrt()).abs() < 1e-9);

        // Manhattan distance: |1-4| + |2-5| + |3-6| = 3 + 3 + 3 = 9
        let manhattan = v1.manhattan_distance(&v2).unwrap();
        assert_eq!(manhattan, 9.0);

        // L2 Norm of v1: sqrt(1 + 4 + 9) = sqrt(14)
        assert!((v1.norm().unwrap() - 14.0f64.sqrt()).abs() < 1e-9);

        // Cosine similarity
        let cos = v1.cosine_similarity(&v2).unwrap();
        let expected_cos = 32.0 / (14.0f64.sqrt() * 77.0f64.sqrt());
        assert!((cos - expected_cos).abs() < 1e-9);

        // Dimension mismatch error validation
        let v_mismatch = ValueVector::new(vec![1.0, 2.0]);
        assert!(v1.dot(&v_mismatch).is_err());
        assert!(v1.distance(&v_mismatch).is_err());
        assert!(v1.manhattan_distance(&v_mismatch).is_err());
        assert!(v1.cosine_similarity(&v_mismatch).is_err());
    }

    #[test]
    fn test_metadata_publishing_and_listing_registration() {
        let registry = MarketplaceRegistry::new();
        let mut attributes = HashMap::new();
        attributes.insert("region".to_string(), "us-west-2".to_string());
        
        let metadata = ListingMetadata::new(
            "High-CPU Compute Node".to_string(),
            "Optimized for compute heavy operations".to_string(),
            attributes,
            "compute".to_string(),
            "1.0.0".to_string(),
        );

        // Publish metadata
        let metadata_hash = registry.publish_metadata(metadata.clone()).unwrap();
        assert_eq!(metadata_hash, metadata.compute_hash());

        // Get metadata
        let fetched_metadata = registry.get_metadata(&metadata_hash).unwrap().unwrap();
        assert_eq!(fetched_metadata.name, "High-CPU Compute Node");

        // Try to register listing with invalid hash
        let vector = ValueVector::new(vec![0.05, 0.99, 10.0]); // e.g. latency, reliability, price
        let invalid_listing = Listing::new(
            "provider-1".to_string(),
            "invalid-listing".to_string(),
            vector.clone(),
            "non-existent-hash".to_string(),
        );
        let reg_result = registry.register_listing(invalid_listing);
        assert!(reg_result.is_err());

        // Register listing with valid metadata
        let valid_listing = Listing::new(
            "provider-1".to_string(),
            "compute-node-1".to_string(),
            vector.clone(),
            metadata_hash.clone(),
        );
        let listing_id = valid_listing.id;
        registry.register_listing(valid_listing).unwrap();

        // Retrieve and check listing
        let fetched_listing = registry.get_listing(&listing_id).unwrap().unwrap();
        assert_eq!(fetched_listing.name, "compute-node-1");
        assert_eq!(fetched_listing.value_vector, vector);

        // List active listings
        let active = registry.list_active_listings().unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, listing_id);

        // Deactivate listing
        registry.update_listing_status(&listing_id, false).unwrap();
        let active_after = registry.list_active_listings().unwrap();
        assert_eq!(active_after.len(), 0);

        // Try deactivating non-existent listing
        assert!(registry.update_listing_status(&Uuid::new_v4(), false).is_err());
    }

    #[test]
    fn test_price_routing_engine() {
        // Prepare list of compute nodes.
        // Dimension mapping: [Latency (ms), Reliability (0 to 1), Cost (USD)]
        let l1 = Listing::new(
            "provider-a".to_string(),
            "node-fast-expensive".to_string(),
            ValueVector::new(vec![10.0, 0.999, 100.0]),
            "hash-a".to_string(),
        );
        let l2 = Listing::new(
            "provider-b".to_string(),
            "node-slow-cheap".to_string(),
            ValueVector::new(vec![150.0, 0.95, 10.0]),
            "hash-b".to_string(),
        );
        let l3 = Listing::new(
            "provider-c".to_string(),
            "node-balanced".to_string(),
            ValueVector::new(vec![40.0, 0.99, 45.0]),
            "hash-c".to_string(),
        );

        let listings = vec![l1.clone(), l2.clone(), l3.clone()];

        // Request 1: We care about low cost and don't want latency to exceed 50ms.
        // To minimize latency/cost, we use negative weight values for latency and cost.
        // Weights: [Latency: -0.1, Reliability: 10.0, Cost: -1.0]
        // Constraints: Latency <= 50.0
        let request1 = RouteRequest {
            preference_weights: ValueVector::new(vec![-0.1, 10.0, -1.0]),
            constraints: vec![
                DimensionConstraint::new(0, ConstraintType::LessThanOrEqual, 50.0),
            ],
        };

        let routes1 = PriceRoutingEngine::route(&listings, &request1).unwrap();

        // Only node-fast-expensive and node-balanced satisfy Latency <= 50.
        // Scores calculation:
        // l1 (fast-expensive): 10.0*(-0.1) + 0.999*10.0 + 100.0*(-1.0) = -1.0 + 9.99 - 100.0 = -91.01
        // l3 (balanced): 40.0*(-0.1) + 0.99*10.0 + 45.0*(-1.0) = -4.0 + 9.9 - 45.0 = -39.1
        // Since -39.1 > -91.01, node-balanced should be ranked #1
        assert_eq!(routes1.len(), 2);
        assert_eq!(routes1[0].listing_id, l3.id);
        assert_eq!(routes1[1].listing_id, l1.id);

        // Request 2: No constraints, preferences highly weight reliability.
        // Weights: [Latency: 0.0, Reliability: 100.0, Cost: 0.0]
        let request2 = RouteRequest {
            preference_weights: ValueVector::new(vec![0.0, 100.0, 0.0]),
            constraints: vec![],
        };

        let routes2 = PriceRoutingEngine::route(&listings, &request2).unwrap();
        assert_eq!(routes2.len(), 3);
        // Sorted: node-fast-expensive (0.999), node-balanced (0.99), node-slow-cheap (0.95)
        assert_eq!(routes2[0].listing_id, l1.id);
        assert_eq!(routes2[1].listing_id, l3.id);
        assert_eq!(routes2[2].listing_id, l2.id);
    }
}
