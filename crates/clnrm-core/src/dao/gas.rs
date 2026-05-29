use std::fmt;

/// Defines the types of operations that consume gas in the VM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GasOp {
    /// Basic execution step.
    Step,
    /// Memory allocation.
    MemoryAllocation { bytes: u64 },
    /// Storage read operation.
    StorageRead,
    /// Storage write operation.
    StorageWrite,
    /// Cryptographic hash operation.
    CryptoHash { bytes: u64 },
    /// Cross-contract call.
    Call,
}

impl GasOp {
    /// Returns the gas cost associated with the operation.
    pub fn cost(&self) -> u64 {
        match self {
            GasOp::Step => 1,
            GasOp::MemoryAllocation { bytes } => 10 + (bytes * 2), // Base cost + per byte cost
            GasOp::StorageRead => 100,
            GasOp::StorageWrite => 500,
            GasOp::CryptoHash { bytes } => 50 + (bytes * 5),
            GasOp::Call => 1000,
        }
    }
}

/// Errors that can occur during gas metering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GasError {
    /// Raised when the requested gas exceeds the available gas.
    OutOfGas { limit: u64, requested: u64, available: u64 },
    /// Raised when an arithmetic overflow occurs during gas calculation.
    MathOverflow,
}

impl fmt::Display for GasError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GasError::OutOfGas { limit, requested, available } => {
                write!(
                    f,
                    "Out of gas: limit was {}, requested {}, but only {} available",
                    limit, requested, available
                )
            }
            GasError::MathOverflow => write!(f, "Gas calculation math overflow"),
        }
    }
}

impl std::error::Error for GasError {}

/// Tracks gas usage and limits for VM execution.
#[derive(Debug, Clone)]
pub struct GasMeter {
    limit: u64,
    consumed: u64,
}

impl GasMeter {
    /// Creates a new `GasMeter` with the specified gas limit.
    pub fn new(limit: u64) -> Self {
        Self { limit, consumed: 0 }
    }

    /// Consumes gas based on the operation type.
    /// Returns `Ok(())` if successful, or an `OutOfGas` error.
    pub fn consume(&mut self, op: GasOp) -> Result<(), GasError> {
        let cost = op.cost();
        self.consume_raw(cost)
    }

    /// Consumes a raw amount of gas.
    pub fn consume_raw(&mut self, cost: u64) -> Result<(), GasError> {
        let new_consumed = self.consumed.checked_add(cost).ok_or(GasError::MathOverflow)?;

        if new_consumed > self.limit {
            let available = self.limit.saturating_sub(self.consumed);
            return Err(GasError::OutOfGas {
                limit: self.limit,
                requested: cost,
                available,
            });
        }

        self.consumed = new_consumed;
        Ok(())
    }

    /// Returns the total gas limit.
    pub fn limit(&self) -> u64 {
        self.limit
    }

    /// Returns the currently consumed gas.
    pub fn consumed(&self) -> u64 {
        self.consumed
    }

    /// Returns the remaining available gas.
    pub fn remaining(&self) -> u64 {
        self.limit.saturating_sub(self.consumed)
    }

    /// Refund gas (e.g., if a memory allocation was smaller than expected).
    pub fn refund(&mut self, amount: u64) {
        self.consumed = self.consumed.checked_sub(amount).unwrap_or(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_consumption() {
        let mut meter = GasMeter::new(1000);
        
        assert_eq!(meter.remaining(), 1000);
        assert!(meter.consume(GasOp::Step).is_ok());
        assert_eq!(meter.consumed(), 1);
        
        assert!(meter.consume(GasOp::StorageWrite).is_ok());
        assert_eq!(meter.consumed(), 501);
        assert_eq!(meter.remaining(), 499);
    }

    #[test]
    fn test_out_of_gas() {
        let mut meter = GasMeter::new(50);
        
        let result = meter.consume(GasOp::StorageRead);
        assert!(result.is_err());
        if let Err(GasError::OutOfGas { limit, requested, available }) = result {
            assert_eq!(limit, 50);
            assert_eq!(requested, 100);
            assert_eq!(available, 50);
        } else {
            panic!("Expected OutOfGas error");
        }
    }

    #[test]
    fn test_math_overflow() {
        let mut meter = GasMeter::new(u64::MAX);
        meter.consumed = u64::MAX - 10;
        
        let result = meter.consume_raw(20);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), GasError::OutOfGas { limit: u64::MAX, requested: 20, available: 10 });
        
        let mut meter2 = GasMeter::new(u64::MAX);
        meter2.consumed = u64::MAX;
        let result2 = meter2.consume_raw(1);
        assert!(result2.is_err());
        assert_eq!(result2.unwrap_err(), GasError::MathOverflow);
    }
    
    #[test]
    fn test_refund() {
        let mut meter = GasMeter::new(100);
        meter.consume_raw(50).unwrap();
        assert_eq!(meter.consumed(), 50);
        
        meter.refund(20);
        assert_eq!(meter.consumed(), 30);
        
        meter.refund(100);
        assert_eq!(meter.consumed(), 0);
    }
}
