use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OntologyPackId(pub String);

#[derive(Debug, Clone)]
pub enum UnderlyingAsset {
    /// Hedge against compute price volatility. The spot price of compute at expiry.
    ComputePrice { provider_id: String },
    /// Hedge against the success probability of a specific `OntologyPack` admission.
    /// Evaluates to a scaled probability score (e.g., 10000 = 100%, 0 = 0%).
    OntologyPackAdmission { pack_id: OntologyPackId },
}

#[derive(Debug, Clone)]
pub struct ConsequenceFuture {
    pub future_id: String,
    pub underlying: UnderlyingAsset,
    pub expiration_timestamp: u64,
    pub strike_price: u64,
    pub quantity: u64,
    pub long_party: EntityId,
    pub short_party: EntityId,
    pub margin_posted: HashMap<EntityId, u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionType {
    Call,
    Put,
}

#[derive(Debug, Clone)]
pub struct ConsequenceOption {
    pub option_id: String,
    pub option_type: OptionType,
    pub underlying: UnderlyingAsset,
    pub expiration_timestamp: u64,
    pub strike_price: u64,
    pub premium: u64,
    pub quantity: u64,
    pub buyer: EntityId,
    pub writer: EntityId,
}

pub struct DerivativesEngine {
    futures: HashMap<String, ConsequenceFuture>,
    options: HashMap<String, ConsequenceOption>,
    compute_prices: HashMap<String, u64>,
    admission_probabilities: HashMap<OntologyPackId, u64>,
}

impl Default for DerivativesEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DerivativesEngine {
    pub fn new() -> Self {
        Self {
            futures: HashMap::new(),
            options: HashMap::new(),
            compute_prices: HashMap::new(),
            admission_probabilities: HashMap::new(),
        }
    }

    pub fn open_future(&mut self, future: ConsequenceFuture) -> Result<(), &'static str> {
        if self.futures.contains_key(&future.future_id) {
            return Err("Future ID already exists");
        }
        self.futures.insert(future.future_id.clone(), future);
        Ok(())
    }

    pub fn write_option(&mut self, option: ConsequenceOption) -> Result<(), &'static str> {
        if self.options.contains_key(&option.option_id) {
            return Err("Option ID already exists");
        }
        self.options.insert(option.option_id.clone(), option);
        Ok(())
    }

    pub fn update_compute_price(&mut self, provider_id: String, price: u64) {
        self.compute_prices.insert(provider_id, price);
    }

    pub fn update_admission_probability(&mut self, pack_id: OntologyPackId, probability: u64) {
        self.admission_probabilities.insert(pack_id, probability);
    }

    pub fn get_spot_price(&self, underlying: &UnderlyingAsset) -> Option<u64> {
        match underlying {
            UnderlyingAsset::ComputePrice { provider_id } => {
                self.compute_prices.get(provider_id).copied()
            }
            UnderlyingAsset::OntologyPackAdmission { pack_id } => {
                self.admission_probabilities.get(pack_id).copied()
            }
        }
    }

    pub fn settle_future(
        &mut self,
        future_id: &str,
        current_timestamp: u64,
    ) -> Result<HashMap<EntityId, i64>, &'static str> {
        let future = self.futures.get(future_id).ok_or("Future not found")?;
        if current_timestamp < future.expiration_timestamp {
            return Err("Future has not expired yet");
        }

        let spot_price = self
            .get_spot_price(&future.underlying)
            .ok_or("Spot price not available")?;

        let mut settlement = HashMap::new();

        let long_payoff = (spot_price as i64 - future.strike_price as i64) * future.quantity as i64;
        let short_payoff = -long_payoff;

        settlement.insert(future.long_party.clone(), long_payoff);
        settlement.insert(future.short_party.clone(), short_payoff);

        self.futures.remove(future_id);

        Ok(settlement)
    }

    pub fn exercise_option(
        &mut self,
        option_id: &str,
        current_timestamp: u64,
    ) -> Result<HashMap<EntityId, i64>, &'static str> {
        let option = self.options.get(option_id).ok_or("Option not found")?;
        if current_timestamp > option.expiration_timestamp {
            return Err("Option has expired");
        }

        let spot_price = self
            .get_spot_price(&option.underlying)
            .ok_or("Spot price not available")?;

        let mut settlement = HashMap::new();

        let payoff = match option.option_type {
            OptionType::Call => {
                if spot_price > option.strike_price {
                    (spot_price as i64 - option.strike_price as i64) * option.quantity as i64
                } else {
                    0
                }
            }
            OptionType::Put => {
                if spot_price < option.strike_price {
                    (option.strike_price as i64 - spot_price as i64) * option.quantity as i64
                } else {
                    0
                }
            }
        };

        settlement.insert(option.buyer.clone(), payoff);
        settlement.insert(option.writer.clone(), -payoff);

        self.options.remove(option_id);

        Ok(settlement)
    }
}
