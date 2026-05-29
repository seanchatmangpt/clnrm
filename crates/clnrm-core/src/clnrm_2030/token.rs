use std::collections::HashMap;

/// Unique identifier for an account or entity holding tokens.
pub type AccountId = [u8; 32];

/// Unique identifier for a specific dimension (asset type).
pub type DimensionId = [u8; 32];

/// Represents errors that can occur during token operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenError {
    DimensionNotFound(DimensionId),
    DimensionAlreadyExists(DimensionId),
    AccountNotFound(AccountId),
    InsufficientBalance {
        dimension: DimensionId,
        required: u128,
        actual: u128,
    },
    ExceedsMaxSupply {
        dimension: DimensionId,
        requested: u128,
        remaining: u128,
    },
    InvalidAmountForClass(DimensionId),
}

/// The mathematical properties of a specific token dimension.
/// This single enum natively encodes fungible, non-fungible, and fractional assets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DimensionClass {
    /// A fungible dimension with arbitrary divisibility and total supply.
    Fungible { decimals: u8 },
    /// A non-fungible dimension, indivisible, with a strict maximum supply of 1.
    NonFungible,
    /// A fractionalized dimension, usually derived from a non-fungible or real-world asset.
    /// Represents parts of a whole (e.g., 1,000,000 parts).
    Fractional { parts: u128 },
}

/// Represents a registered dimension in the multi-dimensional space.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dimension {
    pub id: DimensionId,
    pub class: DimensionClass,
    pub current_supply: u128,
    pub max_supply: Option<u128>,
}

/// A mathematical vector representing token balances across different dimensions.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TokenVector {
    pub components: HashMap<DimensionId, u128>,
}

impl TokenVector {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn get(&self, dimension: &DimensionId) -> u128 {
        self.components.get(dimension).copied().unwrap_or(0)
    }

    pub fn set(&mut self, dimension: DimensionId, amount: u128) {
        if amount == 0 {
            self.components.remove(&dimension);
        } else {
            self.components.insert(dimension, amount);
        }
    }

    pub fn add(&mut self, dimension: DimensionId, amount: u128) {
        let current = self.get(&dimension);
        self.set(dimension, current.saturating_add(amount));
    }

    pub fn subtract(&mut self, dimension: DimensionId, amount: u128) {
        let current = self.get(&dimension);
        self.set(dimension, current.saturating_sub(amount));
    }
}

/// Intent to mint assets across multiple dimensions into a specific account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Minting {
    pub to: AccountId,
    pub vector: TokenVector,
}

/// Intent to burn assets across multiple dimensions from a specific account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Burning {
    pub from: AccountId,
    pub vector: TokenVector,
}

/// Intent to transfer assets across multiple dimensions between two accounts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transferring {
    pub from: AccountId,
    pub to: AccountId,
    pub vector: TokenVector,
}

/// The core multi-dimensional token standard manager.
/// Facilitates mapping states for arbitrary token schemas mapped over R^n dimensions.
#[derive(Debug, Clone, Default)]
pub struct NDimensionalToken {
    /// Currently registered dimensions.
    pub dimensions: HashMap<DimensionId, Dimension>,
    /// State of all accounts, mapping an account ID to its token vector.
    pub accounts: HashMap<AccountId, TokenVector>,
}

impl NDimensionalToken {
    pub fn new() -> Self {
        Self {
            dimensions: HashMap::new(),
            accounts: HashMap::new(),
        }
    }

    /// Registers a new dimension into the standard.
    pub fn register_dimension(
        &mut self,
        id: DimensionId,
        class: DimensionClass,
        custom_max_supply: Option<u128>,
    ) -> Result<(), TokenError> {
        if self.dimensions.contains_key(&id) {
            return Err(TokenError::DimensionAlreadyExists(id));
        }

        let max_supply = match class {
            DimensionClass::NonFungible => Some(1),
            DimensionClass::Fractional { parts } => Some(parts),
            DimensionClass::Fungible { .. } => custom_max_supply,
        };

        self.dimensions.insert(
            id,
            Dimension {
                id,
                class,
                current_supply: 0,
                max_supply,
            },
        );

        Ok(())
    }

    /// Applies a multidimensional minting operation, creating new assets across dimensions.
    pub fn apply_mint(&mut self, operation: Minting) -> Result<(), TokenError> {
        // Validate all components before state changes
        for (dim_id, amount) in &operation.vector.components {
            let dim = self.dimensions
                .get(dim_id)
                .ok_or(TokenError::DimensionNotFound(*dim_id))?;

            if let Some(max) = dim.max_supply {
                if dim.current_supply.saturating_add(*amount) > max {
                    return Err(TokenError::ExceedsMaxSupply {
                        dimension: *dim_id,
                        requested: *amount,
                        remaining: max.saturating_sub(dim.current_supply),
                    });
                }
            }
        }

        // Apply mutations
        let account = self.accounts.entry(operation.to).or_insert_with(TokenVector::new);
        for (dim_id, amount) in operation.vector.components {
            let dim = self.dimensions.get_mut(&dim_id).expect("Dimension verified");
            dim.current_supply = dim.current_supply.saturating_add(amount);

            account.add(dim_id, amount);
        }

        Ok(())
    }

    /// Applies a multidimensional burning operation, removing assets across dimensions.
    pub fn apply_burn(&mut self, operation: Burning) -> Result<(), TokenError> {
        let account = self.accounts
            .get(&operation.from)
            .ok_or(TokenError::AccountNotFound(operation.from))?;

        // Validate all components before state changes
        for (dim_id, amount) in &operation.vector.components {
            if !self.dimensions.contains_key(dim_id) {
                return Err(TokenError::DimensionNotFound(*dim_id));
            }
            let current_balance = account.get(dim_id);
            if current_balance < *amount {
                return Err(TokenError::InsufficientBalance {
                    dimension: *dim_id,
                    required: *amount,
                    actual: current_balance,
                });
            }
        }

        // Apply mutations
        let account = self.accounts.get_mut(&operation.from).expect("Account verified");
        for (dim_id, amount) in operation.vector.components {
            let dim = self.dimensions.get_mut(&dim_id).expect("Dimension verified");
            dim.current_supply = dim.current_supply.saturating_sub(amount);

            account.subtract(dim_id, amount);
        }

        Ok(())
    }

    /// Applies a multidimensional transfer operation, atomically shifting vectors.
    pub fn apply_transfer(&mut self, operation: Transferring) -> Result<(), TokenError> {
        // Validation phase
        let from_account = self.accounts
            .get(&operation.from)
            .ok_or(TokenError::AccountNotFound(operation.from))?;

        for (dim_id, amount) in &operation.vector.components {
            if !self.dimensions.contains_key(dim_id) {
                return Err(TokenError::DimensionNotFound(*dim_id));
            }
            let current_balance = from_account.get(dim_id);
            if current_balance < *amount {
                return Err(TokenError::InsufficientBalance {
                    dimension: *dim_id,
                    required: *amount,
                    actual: current_balance,
                });
            }
        }

        // Mutation phase - subtract from sender
        let from_account = self.accounts.get_mut(&operation.from).expect("Account verified");
        for (dim_id, amount) in &operation.vector.components {
            from_account.subtract(*dim_id, *amount);
        }

        // Mutation phase - add to receiver
        let to_account = self.accounts.entry(operation.to).or_insert_with(TokenVector::new);
        for (dim_id, amount) in operation.vector.components {
            to_account.add(dim_id, amount);
        }

        Ok(())
    }
}
