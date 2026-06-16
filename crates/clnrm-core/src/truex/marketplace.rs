use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::{CleanroomError, Result};

/// Represents the value vector pricing model from Section 6.2.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValueVectorPricing {
    /// Fit within the public ontology
    pub ontology_fit: f64,
    /// Execution latency
    pub latency_ms: u64,
    /// Cryptographic receipt strength
    pub receipt_strength: u32,
    /// Reduction in counterparty trust
    pub counterparty_trust_reduction: f64,
}

impl ValueVectorPricing {
    /// Create a new ValueVectorPricing model
    pub fn new(
        ontology_fit: f64,
        latency_ms: u64,
        receipt_strength: u32,
        counterparty_trust_reduction: f64,
    ) -> Self {
        Self {
            ontology_fit,
            latency_ms,
            receipt_strength,
            counterparty_trust_reduction,
        }
    }

    /// Calculates an aggregate score representing the price or value of the consequence.
    pub fn calculate_aggregate_value(&self) -> f64 {
        // A simplistic fallback weighting for the n-dimensional model
        self.ontology_fit * 100.0
            + self.counterparty_trust_reduction * 50.0
            + (self.receipt_strength as f64) * 10.0
            - (self.latency_ms as f64) * 0.1
    }
}

/// N-Dimensional ConsequenceListing from Section 6.2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsequenceListing {
    /// Unique identifier for the listing
    pub id: String,
    /// Description of the admitted consequence
    pub description: String,
    /// N-Dimensional pricing model
    pub pricing: ValueVectorPricing,
    /// Status of the listing in the marketplace
    pub is_active: bool,
}

impl ConsequenceListing {
    /// Create a new ConsequenceListing
    pub fn new(id: String, description: String, pricing: ValueVectorPricing) -> Self {
        Self {
            id,
            description,
            pricing,
            is_active: true,
        }
    }
}

// ── Marketplace core types ────────────────────────────────────────────────────

/// Opaque seller identity.
pub type SellerId = String;

/// Opaque buyer identity.
pub type BuyerId = String;

/// Opaque listing identifier (UUID-based).
pub type ListingId = String;

/// Opaque dispute identifier.
pub type DisputeId = String;

/// Token amount (integer units; decimals handled by caller).
pub type TokenAmount = u64;

/// Category of a market item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ItemCategory {
    ConsequenceListing,
    OntologyPack,
    AdmissionReceipt,
    DataAsset,
    ExecutionSlot,
    Other(String),
}

/// A market item offered for sale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketItem {
    pub name: String,
    pub description: String,
    pub category: ItemCategory,
    /// Arbitrary key-value metadata.
    pub metadata: HashMap<String, String>,
}

/// Price description for a listing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Price {
    pub amount: TokenAmount,
    pub currency: String,
}

impl Price {
    pub fn new(amount: TokenAmount, currency: impl Into<String>) -> Self {
        Self {
            amount,
            currency: currency.into(),
        }
    }
}

/// Status of a marketplace listing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ListingStatus {
    Active,
    Sold,
    Cancelled,
    Disputed,
}

/// A full marketplace listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Listing {
    pub id: ListingId,
    pub seller: SellerId,
    pub item: MarketItem,
    pub price: Price,
    pub status: ListingStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Filters for marketplace search.
#[derive(Debug, Clone, Default)]
pub struct SearchFilters {
    pub category: Option<ItemCategory>,
    pub max_price: Option<TokenAmount>,
    pub min_price: Option<TokenAmount>,
    pub currency: Option<String>,
    pub seller: Option<SellerId>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

/// Receipt of a successful purchase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseReceipt {
    pub listing_id: ListingId,
    pub buyer: BuyerId,
    pub seller: SellerId,
    pub price_paid: Price,
    pub purchased_at: DateTime<Utc>,
}

/// Ruling on a dispute.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DisputeRuling {
    /// Refund the buyer.
    BuyerWins,
    /// Release funds to the seller.
    SellerWins,
    /// Split the funds proportionally.
    Split { buyer_pct: u8, seller_pct: u8 },
}

/// A buyer-initiated dispute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dispute {
    pub id: DisputeId,
    pub listing_id: ListingId,
    pub buyer: BuyerId,
    pub reason: String,
    pub status: DisputeStatus,
    pub ruling: Option<DisputeRuling>,
    pub opened_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DisputeStatus {
    Open,
    Resolved,
}

/// The TrueX marketplace: create listings, search, purchase, dispute, and resolve.
#[derive(Debug, Default)]
pub struct Marketplace {
    listings: HashMap<ListingId, Listing>,
    disputes: HashMap<DisputeId, Dispute>,
    /// Simulated per-user balances for fund validation.
    balances: HashMap<String, TokenAmount>,
}

impl Marketplace {
    /// Create a new empty marketplace.
    pub fn new() -> Self {
        Self::default()
    }

    /// Deposit funds into a user's balance (for testing / bootstrapping).
    pub fn deposit(&mut self, user_id: &str, amount: TokenAmount) {
        *self.balances.entry(user_id.to_string()).or_insert(0) += amount;
    }

    /// Create a new listing.
    ///
    /// Returns the new `ListingId` on success.
    pub fn create_listing(
        &mut self,
        seller: SellerId,
        item: MarketItem,
        price: Price,
    ) -> Result<ListingId> {
        if seller.is_empty() {
            return Err(CleanroomError::validation_error(
                "Seller ID cannot be empty",
            ));
        }
        if item.name.is_empty() {
            return Err(CleanroomError::validation_error(
                "Item name cannot be empty",
            ));
        }
        if price.amount == 0 {
            return Err(CleanroomError::validation_error(
                "Price must be greater than zero",
            ));
        }

        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let listing = Listing {
            id: id.clone(),
            seller,
            item,
            price,
            status: ListingStatus::Active,
            created_at: now,
            updated_at: now,
        };
        self.listings.insert(id.clone(), listing);
        Ok(id)
    }

    /// Search listings matching `query` (case-insensitive substring in name or description)
    /// and all provided filters.
    pub fn search(&self, query: &str, filters: &SearchFilters) -> Vec<&Listing> {
        let query_lower = query.to_lowercase();
        self.listings
            .values()
            .filter(|l| {
                // Only active listings
                if l.status != ListingStatus::Active {
                    return false;
                }
                // Text search
                if !query_lower.is_empty()
                    && !l.item.name.to_lowercase().contains(&query_lower)
                    && !l.item.description.to_lowercase().contains(&query_lower)
                {
                    return false;
                }
                // Category filter
                if let Some(cat) = &filters.category {
                    if &l.item.category != cat {
                        return false;
                    }
                }
                // Price filters
                if let Some(max) = filters.max_price {
                    if l.price.amount > max {
                        return false;
                    }
                }
                if let Some(min) = filters.min_price {
                    if l.price.amount < min {
                        return false;
                    }
                }
                // Currency filter
                if let Some(ref cur) = filters.currency {
                    if &l.price.currency != cur {
                        return false;
                    }
                }
                // Seller filter
                if let Some(ref seller) = filters.seller {
                    if &l.seller != seller {
                        return false;
                    }
                }
                // Date range
                if let Some(after) = filters.created_after {
                    if l.created_at < after {
                        return false;
                    }
                }
                if let Some(before) = filters.created_before {
                    if l.created_at > before {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    /// Purchase a listing.
    ///
    /// Validates that:
    /// - The listing exists and is active
    /// - The buyer has sufficient funds
    ///
    /// On success, transfers funds from buyer to seller and marks listing as Sold.
    pub fn purchase(&mut self, buyer: BuyerId, listing_id: ListingId) -> Result<PurchaseReceipt> {
        let listing = self.listings.get(&listing_id).ok_or_else(|| {
            CleanroomError::validation_error(format!("Listing not found: {}", listing_id))
        })?;

        if listing.status != ListingStatus::Active {
            return Err(CleanroomError::validation_error(format!(
                "Listing {} is not active (status: {:?})",
                listing_id, listing.status
            )));
        }

        let price = listing.price.clone();
        let seller = listing.seller.clone();

        // Check buyer balance
        let buyer_balance = self.balances.get(&buyer).copied().unwrap_or(0);
        if buyer_balance < price.amount {
            return Err(CleanroomError::validation_error(format!(
                "Insufficient funds: buyer has {} but listing costs {}",
                buyer_balance, price.amount
            )));
        }

        // Transfer funds
        *self.balances.entry(buyer.clone()).or_insert(0) -= price.amount;
        *self.balances.entry(seller.clone()).or_insert(0) += price.amount;

        // Mark listing as Sold
        if let Some(listing) = self.listings.get_mut(&listing_id) {
            listing.status = ListingStatus::Sold;
            listing.updated_at = Utc::now();
        }

        Ok(PurchaseReceipt {
            listing_id,
            buyer,
            seller,
            price_paid: price,
            purchased_at: Utc::now(),
        })
    }

    /// Open a dispute for a listing.
    ///
    /// The buyer must provide a non-empty reason.
    /// Returns the new `DisputeId`.
    pub fn dispute(&mut self, buyer: BuyerId, listing_id: ListingId, reason: String) -> DisputeId {
        let id = Uuid::new_v4().to_string();
        let dispute = Dispute {
            id: id.clone(),
            listing_id: listing_id.clone(),
            buyer,
            reason,
            status: DisputeStatus::Open,
            ruling: None,
            opened_at: Utc::now(),
            resolved_at: None,
        };
        // Mark listing as disputed
        if let Some(listing) = self.listings.get_mut(&listing_id) {
            listing.status = ListingStatus::Disputed;
            listing.updated_at = Utc::now();
        }
        self.disputes.insert(id.clone(), dispute);
        id
    }

    /// Resolve a dispute with a ruling.
    pub fn resolve_dispute(&mut self, dispute_id: DisputeId, ruling: DisputeRuling) -> Result<()> {
        let dispute = self.disputes.get_mut(&dispute_id).ok_or_else(|| {
            CleanroomError::validation_error(format!("Dispute not found: {}", dispute_id))
        })?;

        if dispute.status != DisputeStatus::Open {
            return Err(CleanroomError::validation_error(format!(
                "Dispute {} is already resolved",
                dispute_id
            )));
        }

        dispute.status = DisputeStatus::Resolved;
        dispute.ruling = Some(ruling.clone());
        dispute.resolved_at = Some(Utc::now());

        // Apply the ruling: adjust balances between buyer and seller
        let listing_id = dispute.listing_id.clone();
        let buyer = dispute.buyer.clone();

        if let Some(listing) = self.listings.get(&listing_id) {
            let seller = listing.seller.clone();
            let price = listing.price.amount;

            match ruling {
                DisputeRuling::BuyerWins => {
                    // Refund buyer from seller
                    let seller_bal = self.balances.entry(seller).or_insert(0);
                    let refund = price.min(*seller_bal);
                    *seller_bal -= refund;
                    *self.balances.entry(buyer).or_insert(0) += refund;
                }
                DisputeRuling::SellerWins => {
                    // No fund transfer; seller keeps the funds
                }
                DisputeRuling::Split {
                    buyer_pct,
                    seller_pct: _,
                } => {
                    let buyer_share = price * (buyer_pct as u64) / 100;
                    let seller_bal = self.balances.entry(seller).or_insert(0);
                    let refund = buyer_share.min(*seller_bal);
                    *seller_bal -= refund;
                    *self.balances.entry(buyer).or_insert(0) += refund;
                }
            }

            // Restore listing status
            if let Some(l) = self.listings.get_mut(&listing_id) {
                l.status = ListingStatus::Sold; // Consider resolved disputes as sold
                l.updated_at = Utc::now();
            }
        }

        Ok(())
    }

    /// Get a listing by ID.
    pub fn get_listing(&self, id: &str) -> Option<&Listing> {
        self.listings.get(id)
    }

    /// Get a dispute by ID.
    pub fn get_dispute(&self, id: &str) -> Option<&Dispute> {
        self.disputes.get(id)
    }

    /// Get the balance of a user.
    pub fn balance_of(&self, user_id: &str) -> TokenAmount {
        self.balances.get(user_id).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests_marketplace {
    use super::*;

    fn make_item(name: &str) -> MarketItem {
        MarketItem {
            name: name.to_string(),
            description: format!("Description of {}", name),
            category: ItemCategory::DataAsset,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_create_and_search_listing() {
        let mut market = Marketplace::new();
        let id = market
            .create_listing(
                "seller-1".to_string(),
                make_item("ontology-pack-v1"),
                Price::new(100, "TRX"),
            )
            .unwrap();

        let results = market.search("ontology", &SearchFilters::default());
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, id);
    }

    #[test]
    fn test_purchase_success() {
        let mut market = Marketplace::new();
        market.deposit("buyer-1", 200);

        let id = market
            .create_listing(
                "seller-1".to_string(),
                make_item("item-a"),
                Price::new(50, "TRX"),
            )
            .unwrap();

        let receipt = market.purchase("buyer-1".to_string(), id.clone()).unwrap();
        assert_eq!(receipt.buyer, "buyer-1");
        assert_eq!(receipt.price_paid.amount, 50);
        assert_eq!(market.balance_of("buyer-1"), 150);
        assert_eq!(market.balance_of("seller-1"), 50);
        assert_eq!(market.get_listing(&id).unwrap().status, ListingStatus::Sold);
    }

    #[test]
    fn test_purchase_insufficient_funds() {
        let mut market = Marketplace::new();
        let id = market
            .create_listing(
                "seller-1".to_string(),
                make_item("expensive"),
                Price::new(1000, "TRX"),
            )
            .unwrap();

        let result = market.purchase("buyer-poor".to_string(), id);
        assert!(result.is_err());
    }

    #[test]
    fn test_dispute_and_resolve_buyer_wins() {
        let mut market = Marketplace::new();
        market.deposit("buyer-1", 100);
        market.deposit("seller-1", 0);

        let id = market
            .create_listing(
                "seller-1".to_string(),
                make_item("disputed-item"),
                Price::new(100, "TRX"),
            )
            .unwrap();
        market.purchase("buyer-1".to_string(), id.clone()).unwrap();

        // Seller now has 100
        let dispute_id = market.dispute(
            "buyer-1".to_string(),
            id.clone(),
            "Item not as described".to_string(),
        );

        market
            .resolve_dispute(dispute_id, DisputeRuling::BuyerWins)
            .unwrap();

        assert_eq!(market.balance_of("buyer-1"), 100); // refunded
        assert_eq!(market.balance_of("seller-1"), 0);
    }

    #[test]
    fn test_search_with_filters() {
        let mut market = Marketplace::new();
        market
            .create_listing("s1".to_string(), make_item("cheap"), Price::new(10, "TRX"))
            .unwrap();
        market
            .create_listing(
                "s1".to_string(),
                make_item("expensive"),
                Price::new(1000, "TRX"),
            )
            .unwrap();

        let filters = SearchFilters {
            max_price: Some(500),
            ..Default::default()
        };
        let results = market.search("", &filters);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].item.name, "cheap");
    }
}
