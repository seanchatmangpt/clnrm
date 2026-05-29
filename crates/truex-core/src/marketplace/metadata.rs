use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListingMetadata {
    pub name: String,
    pub description: String,
    pub attributes: HashMap<String, String>,
    pub category: String,
    pub version: String,
}

impl ListingMetadata {
    pub fn new(
        name: String,
        description: String,
        attributes: HashMap<String, String>,
        category: String,
        version: String,
    ) -> Self {
        Self {
            name,
            description,
            attributes,
            category,
            version,
        }
    }

    pub fn compute_hash(&self) -> String {
        let serialized = serde_json::to_string(self).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(serialized.as_bytes());
        hex::encode(hasher.finalize())
    }
}
