use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt;

/// Errors that can occur within the Sybil resistance mechanism.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SybilError {
    ValidatorExists,
    ValidatorNotFound,
    InvalidStakeAmount,
    ValidatorAlreadySlashed,
    NoActiveStake,
    SelectionFailed,
}

impl fmt::Display for SybilError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SybilError::ValidatorExists => write!(f, "Validator already exists"),
            SybilError::ValidatorNotFound => write!(f, "Validator not found"),
            SybilError::InvalidStakeAmount => write!(f, "Invalid stake amount"),
            SybilError::ValidatorAlreadySlashed => write!(f, "Validator is already slashed"),
            SybilError::NoActiveStake => write!(f, "No active stake available for selection"),
            SybilError::SelectionFailed => write!(f, "Failed to select a validator"),
        }
    }
}

impl std::error::Error for SybilError {}

/// A unique identifier for a validator.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValidatorId(pub String);

/// Represents a validator in the Proof-of-Stake system.
#[derive(Debug, Clone)]
pub struct Validator {
    pub id: ValidatorId,
    pub stake: u64,
    pub is_slashed: bool,
}

/// Registry for managing validators, their stakes, and slashing conditions.
#[derive(Debug, Default)]
pub struct SybilRegistry {
    validators: HashMap<ValidatorId, Validator>,
    total_stake: u64,
    slashed_stake: u64,
}

impl SybilRegistry {
    /// Creates a new, empty SybilRegistry.
    pub fn new() -> Self {
        Self {
            validators: HashMap::new(),
            total_stake: 0,
            slashed_stake: 0,
        }
    }

    /// Registers a new validator with the given initial stake.
    pub fn register_validator(&mut self, id: ValidatorId, stake: u64) -> Result<(), SybilError> {
        if self.validators.contains_key(&id) {
            return Err(SybilError::ValidatorExists);
        }
        if stake == 0 {
            return Err(SybilError::InvalidStakeAmount);
        }
        let validator = Validator {
            id: id.clone(),
            stake,
            is_slashed: false,
        };
        self.validators.insert(id, validator);
        self.total_stake += stake;
        Ok(())
    }

    /// Removes a validator from the registry, returning their current stake.
    pub fn unregister_validator(&mut self, id: &ValidatorId) -> Result<u64, SybilError> {
        if let Some(validator) = self.validators.remove(id) {
            if !validator.is_slashed {
                self.total_stake -= validator.stake;
            }
            Ok(validator.stake)
        } else {
            Err(SybilError::ValidatorNotFound)
        }
    }

    /// Gets the current stake of a validator.
    pub fn get_stake(&self, id: &ValidatorId) -> Option<u64> {
        self.validators.get(id).map(|v| v.stake)
    }

    /// Checks if a validator has been completely slashed.
    pub fn is_slashed(&self, id: &ValidatorId) -> bool {
        self.validators
            .get(id)
            .map(|v| v.is_slashed)
            .unwrap_or(false)
    }

    /// Slashes a specific amount from a validator's stake due to malicious behavior.
    pub fn slash_validator(&mut self, id: &ValidatorId, amount: u64) -> Result<(), SybilError> {
        if amount == 0 {
            return Err(SybilError::InvalidStakeAmount);
        }
        if let Some(validator) = self.validators.get_mut(id) {
            if validator.is_slashed {
                return Err(SybilError::ValidatorAlreadySlashed);
            }
            let slash_amount = std::cmp::min(validator.stake, amount);
            validator.stake -= slash_amount;
            self.total_stake -= slash_amount;
            self.slashed_stake += slash_amount;

            if validator.stake == 0 {
                validator.is_slashed = true;
            }

            Ok(())
        } else {
            Err(SybilError::ValidatorNotFound)
        }
    }

    /// Completely slashes a validator, removing all their stake.
    pub fn slash_validator_completely(&mut self, id: &ValidatorId) -> Result<(), SybilError> {
        if let Some(validator) = self.validators.get_mut(id) {
            if validator.is_slashed {
                return Err(SybilError::ValidatorAlreadySlashed);
            }

            self.total_stake -= validator.stake;
            self.slashed_stake += validator.stake;
            validator.stake = 0;
            validator.is_slashed = true;

            Ok(())
        } else {
            Err(SybilError::ValidatorNotFound)
        }
    }

    /// Selects a validator proportionally based on their stake weight.
    /// Uses a deterministic PRNG seeded by the provided block hash.
    pub fn select_validator(&self, block_hash: &[u8]) -> Result<ValidatorId, SybilError> {
        if self.total_stake == 0 {
            return Err(SybilError::NoActiveStake);
        }

        let mut hasher = Sha256::new();
        hasher.update(block_hash);
        let result = hasher.finalize();

        let mut seed = [0u8; 32];
        seed.copy_from_slice(&result[..]);

        let mut rng = StdRng::from_seed(seed);
        let target = rng.random_range(0..self.total_stake);

        let mut current: u64 = 0;

        let mut sorted_validators: Vec<_> = self
            .validators
            .values()
            .filter(|v| !v.is_slashed && v.stake > 0)
            .collect();

        // Sort by ID to ensure deterministic selection across nodes given the same PRNG seed
        sorted_validators.sort_by(|a, b| a.id.0.cmp(&b.id.0));

        for validator in sorted_validators {
            current += validator.stake;
            if current > target {
                return Ok(validator.id.clone());
            }
        }

        Err(SybilError::SelectionFailed)
    }

    /// Returns the total active stake across all non-slashed validators.
    pub fn total_active_stake(&self) -> u64 {
        self.total_stake
    }

    /// Returns the total amount of stake that has been slashed.
    pub fn total_slashed_stake(&self) -> u64 {
        self.slashed_stake
    }
}
