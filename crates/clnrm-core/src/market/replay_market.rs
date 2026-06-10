use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FixtureListing {
    pub fixture_id: String,
    pub price: f64,
    pub encrypted_payload: Vec<u8>,
}

pub struct ReplayMarket {
    pub listings: HashMap<String, FixtureListing>,
}

impl Default for ReplayMarket {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplayMarket {
    pub fn new() -> Self {
        Self {
            listings: HashMap::new(),
        }
    }

    pub fn list_fixture(&mut self, listing: FixtureListing) {
        self.listings.insert(listing.fixture_id.clone(), listing);
    }

    pub fn buy_fixture_access(
        &self,
        fixture_id: &str,
        payment: f64,
    ) -> Result<Vec<u8>, &'static str> {
        let listing = self.listings.get(fixture_id).ok_or("Fixture not found")?;
        if payment >= listing.price {
            // Unseal logic omitted
            Ok(listing.encrypted_payload.clone())
        } else {
            Err("Insufficient payment")
        }
    }
}
