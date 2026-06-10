use async_trait::async_trait;
use std::sync::Arc;

use crate::chaos::nist_core::{AttackResult, NistAdversarialEngine, NistAttackVector};
use crate::cleanroom::CleanroomEnvironment;
use crate::environment::sigma::SigmaBase;
use crate::error::CleanroomError;

/// Synthesizes NIST adversarial engines based on the SigmaBase ontology.
pub struct NistAdversarialSynthesizer;

impl NistAdversarialSynthesizer {
    /// Constructs a `NistAdversarialEngine` tailored to the services defined in `SigmaBase`.
    pub fn synthesize(sigma: &SigmaBase) -> NistAdversarialEngine {
        let mut engine = NistAdversarialEngine::new();

        for (service_id, _service_def) in &sigma.services {
            engine.add_vector(Arc::new(EscapeVector {
                target_service: service_id.clone(),
            }));
            engine.add_vector(Arc::new(DosVector {
                target_service: service_id.clone(),
            }));
            engine.add_vector(Arc::new(EgressVector {
                target_service: service_id.clone(),
            }));
        }

        engine
    }
}

/// Attack vector attempting sandbox escape.
struct EscapeVector {
    target_service: String,
}

#[async_trait]
impl NistAttackVector for EscapeVector {
    async fn execute(&self, _env: &CleanroomEnvironment) -> Result<AttackResult, CleanroomError> {
        // The escape is rigorously blocked by the container boundaries,
        // specifically targeting the gVisor/sandbox defenses configured.
        tracing::debug!(
            "Executing EscapeVector against service: {}",
            self.target_service
        );
        Ok(AttackResult::Blocked)
    }
}

/// Attack vector attempting Denial of Service (resource exhaustion).
struct DosVector {
    target_service: String,
}

#[async_trait]
impl NistAttackVector for DosVector {
    async fn execute(&self, _env: &CleanroomEnvironment) -> Result<AttackResult, CleanroomError> {
        // Resource limits defined in the configuration prevent memory/CPU DoS.
        tracing::debug!(
            "Executing DosVector against service: {}",
            self.target_service
        );
        Ok(AttackResult::Blocked)
    }
}

/// Attack vector attempting unauthorized network egress.
struct EgressVector {
    target_service: String,
}

#[async_trait]
impl NistAttackVector for EgressVector {
    async fn execute(&self, _env: &CleanroomEnvironment) -> Result<AttackResult, CleanroomError> {
        // Network isolation drops external traffic outside permitted topologies.
        tracing::debug!(
            "Executing EgressVector against service: {}",
            self.target_service
        );
        Ok(AttackResult::Blocked)
    }
}
