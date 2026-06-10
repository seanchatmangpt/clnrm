//! Admission Kernel & Actuator for TrueX
//!
//! Enforces the core mathematical invariant: `R ⊢ A = μ(O*)`
//! Where:
//! - R: Generative Constitution (Rules)
//! - A: Agent behavior (Consequence / AdmittedConsequence)
//! - μ: AdmissionKernel (mu)
//! - O*: Validated Ontology (Input)
//!
//! Provides strict type barriers to ensure only the actuator can instantiate
//! an `AdmittedConsequence`.

use crate::truex::ocel::OCELEvent;
use crate::truex::ontology::OntologyLaw;
use std::marker::PhantomData;

/// Verifies conformance against an ontology law.
pub fn verify_conformance(event: &OCELEvent, law: &OntologyLaw) -> bool {
    law.transitions
        .iter()
        .any(|t| t.select_condition == event.activity)
}

/// The validated ontology input (`O*`)
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ValidatedOntology {
    law_id: String,
    state_field: u8,
    event_condition: String,
}

impl ValidatedOntology {
    pub fn new(law_id: String, state_field: u8, event_condition: String) -> Self {
        Self {
            law_id,
            state_field,
            event_condition,
        }
    }

    pub fn law_id(&self) -> &str {
        &self.law_id
    }

    pub fn state_field(&self) -> u8 {
        self.state_field
    }

    pub fn event_condition(&self) -> &str {
        &self.event_condition
    }
}

/// A cryptographic token representing the generative constitution `R`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerativeConstitution {
    receipt: [u8; 32],
}

impl GenerativeConstitution {
    pub fn new(receipt: [u8; 32]) -> Self {
        Self { receipt }
    }

    pub fn receipt(&self) -> &[u8; 32] {
        &self.receipt
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedConsequence {
    ontology: ValidatedOntology,
    _seal: PhantomData<()>,
}

impl AdmittedConsequence {
    pub fn ontology(&self) -> &ValidatedOntology {
        &self.ontology
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdmissionError {
    EmptyLawId,
    EmptyEventCondition,
    InvalidReceipt,
}

impl std::fmt::Display for AdmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyLawId => write!(f, "Ontology law ID cannot be empty"),
            Self::EmptyEventCondition => write!(f, "Ontology event condition cannot be empty"),
            Self::InvalidReceipt => write!(f, "Constitution receipt is invalid (all zeros)"),
        }
    }
}

impl std::error::Error for AdmissionError {}

pub struct AdmissionKernel {
    constitution: GenerativeConstitution,
}

impl AdmissionKernel {
    pub fn new(constitution: GenerativeConstitution) -> Self {
        Self { constitution }
    }

    pub fn evaluate(&self, ontology: &ValidatedOntology) -> Result<(), AdmissionError> {
        if ontology.law_id.is_empty() {
            return Err(AdmissionError::EmptyLawId);
        }
        if ontology.event_condition.is_empty() {
            return Err(AdmissionError::EmptyEventCondition);
        }

        let all_zeros = [0u8; 32];
        if self.constitution.receipt == all_zeros {
            return Err(AdmissionError::InvalidReceipt);
        }

        Ok(())
    }
}

pub struct GgenSyncActuator {
    mu: AdmissionKernel,
}

impl GgenSyncActuator {
    pub fn new(mu: AdmissionKernel) -> Self {
        Self { mu }
    }

    pub fn actuate(
        &self,
        ontology: ValidatedOntology,
    ) -> Result<AdmittedConsequence, AdmissionError> {
        self.mu.evaluate(&ontology)?;

        Ok(AdmittedConsequence {
            ontology,
            _seal: PhantomData,
        })
    }
}
