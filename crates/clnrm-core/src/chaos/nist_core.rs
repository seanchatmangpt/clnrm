use async_trait::async_trait;
use std::sync::Arc;

/// Result of a NIST attack vector execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttackResult {
    /// The adversary won (attack was successful).
    Success,
    /// The system defended successfully (attack was blocked).
    Blocked,
    /// An error occurred during the execution of the attack.
    Error,
}

/// A generic trait defining an adversarial attack vector.
#[async_trait]
pub trait NistAttackVector: Send + Sync {
    /// Executes the attack vector against the given cleanroom environment.
    async fn execute(
        &self,
        env: &crate::cleanroom::CleanroomEnvironment,
    ) -> Result<AttackResult, crate::error::CleanroomError>;
}

/// Orchestrates the execution of NIST adversarial attack vectors.
pub struct NistAdversarialEngine {
    vectors: Vec<Arc<dyn NistAttackVector>>,
}

impl NistAdversarialEngine {
    /// Creates a new, empty `NistAdversarialEngine`.
    pub fn new() -> Self {
        Self {
            vectors: Vec::new(),
        }
    }

    /// Registers a new attack vector with the engine.
    pub fn add_vector(&mut self, vector: Arc<dyn NistAttackVector>) {
        self.vectors.push(vector);
    }

    /// Executes all registered attack vectors.
    pub async fn execute_all(
        &self,
        env: &crate::cleanroom::CleanroomEnvironment,
    ) -> Result<Vec<AttackResult>, crate::error::CleanroomError> {
        let mut results = Vec::new();
        for vector in &self.vectors {
            let result = vector.execute(env).await?;
            results.push(result);
        }
        Ok(results)
    }
}

impl Default for NistAdversarialEngine {
    fn default() -> Self {
        Self::new()
    }
}
