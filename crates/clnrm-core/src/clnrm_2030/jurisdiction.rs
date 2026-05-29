use std::collections::HashSet;

pub struct JurisdictionEnforcer {
    pub allowed_regions: HashSet<String>,
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
}