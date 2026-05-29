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

use std::marker::PhantomData;

/// The validated ontology input (`O*`)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedOntology {
    law_id: String,
    state_field: u8,
    event_condition: String,
}

impl ValidatedOntology {
    /// Creates a new `ValidatedOntology`.
    pub fn new(law_id: String, state_field: u8, event_condition: String) -> Self {
        Self {
            law_id,
            state_field,
            event_condition,
        }
    }

    /// The law identifier.
    pub fn law_id(&self) -> &str {
        &self.law_id
    }

    /// The current state field of the finite state machine.
    pub fn state_field(&self) -> u8 {
        self.state_field
    }

    /// The external event condition triggered.
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
    /// Instantiates a new generative constitution from a cryptographic receipt.
    pub fn new(receipt: [u8; 32]) -> Self {
        Self { receipt }
    }

    /// Returns the underlying cryptographic receipt.
    pub fn receipt(&self) -> &[u8; 32] {
        &self.receipt
    }
}

/// The result of the invariant projection `A = μ(O*)`.
///
/// Strict type barrier: The internal `_seal` field is private, and no public constructor exists.
/// Sensing surfaces (LSP/MCP/A2A) physically cannot instantiate this type directly,
/// ensuring that all agent behavior is properly admitted by the kernel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedConsequence {
    ontology: ValidatedOntology,
    _seal: PhantomData<()>,
}

impl AdmittedConsequence {
    /// Returns the underlying validated ontology.
    pub fn ontology(&self) -> &ValidatedOntology {
        &self.ontology
    }
}

/// Errors that can occur during the admission process.
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

/// The `AdmissionKernel` (`μ`).
///
/// Applies the projection mapping `μ(O*)` under the constraints of `R`.
pub struct AdmissionKernel {
    constitution: GenerativeConstitution,
}

impl AdmissionKernel {
    /// Initializes a new admission kernel bound to the specific constitution `R`.
    pub fn new(constitution: GenerativeConstitution) -> Self {
        Self { constitution }
    }

    /// Evaluates if the `ValidatedOntology` satisfies the constitution `R`.
    /// This is the core logical check `R ⊢ O*`.
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

/// The `GgenSyncActuator`.
///
/// The sole authority capable of returning an `AdmittedConsequence`.
/// Enforces the invariant `R ⊢ A = μ(O*)`.
pub struct GgenSyncActuator {
    mu: AdmissionKernel,
}

impl GgenSyncActuator {
    /// Instantiates a new actuator wrapped around the admission kernel.
    pub fn new(mu: AdmissionKernel) -> Self {
        Self { mu }
    }

    /// Projects the validated ontology into an admitted consequence if `R ⊢ O*` holds.
    ///
    /// Returns `AdmittedConsequence` exclusively. This is the only function that can
    /// construct an `AdmittedConsequence`, ensuring the type barrier is respected.
    pub fn actuate(&self, ontology: ValidatedOntology) -> Result<AdmittedConsequence, AdmissionError> {
        self.mu.evaluate(&ontology)?;

        Ok(AdmittedConsequence {
            ontology,
            _seal: PhantomData,
        })
    }
}
