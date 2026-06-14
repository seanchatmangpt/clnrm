use std::collections::{HashMap, HashSet};

pub struct SanctionedEntity {
    pub entity_id: String,
    pub reason: String,
}

pub struct JurisdictionEnforcer {
    pub allowed_regions: HashSet<String>,
    pub sanctioned_entities: HashMap<String, SanctionedEntity>,
}

impl Default for JurisdictionEnforcer {
    fn default() -> Self {
        Self::new()
    }
}

impl JurisdictionEnforcer {
    pub fn new() -> Self {
        Self {
            allowed_regions: HashSet::new(),
            sanctioned_entities: HashMap::new(),
        }
    }

    pub fn add_allowed_region(&mut self, region: &str) {
        self.allowed_regions.insert(region.to_string());
    }

    pub fn enforce(&self, target_region: &str) -> Result<(), &'static str> {
        if self.allowed_regions.contains(target_region) {
            Ok(())
        } else {
            Err("Jurisdictional boundary violated")
        }
    }

    // -----------------------------------------------------------------------
    // Sanctions API
    // -----------------------------------------------------------------------

    pub fn add_sanction(&mut self, entity_id: &str, reason: &str) {
        self.sanctioned_entities.insert(
            entity_id.to_string(),
            SanctionedEntity {
                entity_id: entity_id.to_string(),
                reason: reason.to_string(),
            },
        );
    }

    /// Removes a sanction. Returns true if the entity was previously sanctioned.
    pub fn remove_sanction(&mut self, entity_id: &str) -> bool {
        self.sanctioned_entities.remove(entity_id).is_some()
    }

    pub fn is_sanctioned(&self, entity_id: &str) -> bool {
        self.sanctioned_entities.contains_key(entity_id)
    }

    /// Checks that both parties are not sanctioned AND the region is allowed.
    pub fn check_transaction(
        &self,
        buyer: &str,
        seller: &str,
        region: &str,
    ) -> Result<(), String> {
        if self.is_sanctioned(buyer) {
            return Err(format!("Buyer '{}' is sanctioned", buyer));
        }
        if self.is_sanctioned(seller) {
            return Err(format!("Seller '{}' is sanctioned", seller));
        }
        if !self.allowed_regions.contains(region) {
            return Err(format!("Region '{}' is not an allowed jurisdiction", region));
        }
        Ok(())
    }
}
