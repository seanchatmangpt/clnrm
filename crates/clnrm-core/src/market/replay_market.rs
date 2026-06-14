use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FixtureListing {
    pub fixture_id: String,
    pub price: f64,
    pub encrypted_payload: Vec<u8>,
    pub seller: String,
    pub created_at_ms: u64,
    pub access_count: u32,
}

pub struct ReplayMarket {
    pub listings: HashMap<String, FixtureListing>,
    pub balances: HashMap<String, f64>,
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
            balances: HashMap::new(),
        }
    }

    pub fn list_fixture(&mut self, listing: FixtureListing) {
        self.listings.insert(listing.fixture_id.clone(), listing);
    }

    /// Purchase access to a fixture. On success, transfers payment to the seller's balance
    /// and returns the XOR-decrypted payload (key = SHA-256 of fixture_id, cycling).
    pub fn buy_fixture_access(
        &mut self,
        buyer: &str,
        fixture_id: &str,
        payment: f64,
    ) -> Result<Vec<u8>, String> {
        let listing = self
            .listings
            .get_mut(fixture_id)
            .ok_or_else(|| "Fixture not found".to_string())?;

        if payment < listing.price {
            return Err("Insufficient payment".to_string());
        }

        // Transfer payment to seller balance
        let seller = listing.seller.clone();
        *self.balances.entry(seller.clone()).or_insert(0.0) += payment;

        // Track access count
        listing.access_count += 1;

        // Derive XOR key from fixture_id using SHA-256
        let key: [u8; 32] = Sha256::digest(fixture_id.as_bytes()).into();

        // XOR-decrypt the payload using the cycling key
        let decrypted: Vec<u8> = listing
            .encrypted_payload
            .iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ key[i % 32])
            .collect();

        // Silence unused variable warning
        let _ = buyer;

        Ok(decrypted)
    }

    /// Drain and return the seller's accumulated balance.
    pub fn withdraw_balance(&mut self, seller: &str) -> f64 {
        self.balances.remove(seller).unwrap_or(0.0)
    }

    /// Read the seller's current balance without draining it.
    pub fn seller_balance(&self, seller: &str) -> f64 {
        *self.balances.get(seller).unwrap_or(&0.0)
    }
}
