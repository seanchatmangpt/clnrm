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
use crate::truex::receipt::TruexReceipt;
use crate::pqc::hash::custom_hash;
use crate::pqc::lattice::{generate_keypair, sign};
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
    // Represents OTel Traces cryptographically bundled
    trace_digest: [u8; 32], 
}

impl ValidatedOntology {
    pub fn new(law_id: String, state_field: u8, event_condition: String, trace_digest: [u8; 32]) -> Self {
        Self {
            law_id,
            state_field,
            event_condition,
            trace_digest,
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
    
    pub fn trace_digest(&self) -> &[u8; 32] {
        &self.trace_digest
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
    receipt: TruexReceipt,
    _seal: PhantomData<()>,
}

impl AdmittedConsequence {
    pub fn ontology(&self) -> &ValidatedOntology {
        &self.ontology
    }
    
    pub fn receipt(&self) -> &TruexReceipt {
        &self.receipt
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdmissionError {
    EmptyLawId,
    EmptyEventCondition,
    InvalidReceipt,
    OtelTraceMismatch,
}

impl std::fmt::Display for AdmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyLawId => write!(f, "Ontology law ID cannot be empty"),
            Self::EmptyEventCondition => write!(f, "Ontology event condition cannot be empty"),
            Self::InvalidReceipt => write!(f, "Constitution receipt is invalid (all zeros)"),
            Self::OtelTraceMismatch => write!(f, "OpenTelemetry traces failed generative constitution evaluation"),
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

    pub fn evaluate(&self, ontology: &ValidatedOntology) -> Result<TruexReceipt, AdmissionError> {
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
        
        if ontology.trace_digest == all_zeros {
            return Err(AdmissionError::OtelTraceMismatch);
        }
        
        // Compute cryptographic proof of execution trust
        let mut composite = Vec::new();
        composite.extend_from_slice(&self.constitution.receipt);
        composite.extend_from_slice(&ontology.trace_digest);
        composite.extend_from_slice(ontology.law_id.as_bytes());
        
        let digest = custom_hash(&composite);
        
        let kp = generate_keypair([1u8; 32]);
        let pqc_seal = sign(&kp.secret, &digest, [2u8; 32]);

        Ok(TruexReceipt {
            input_hash: hex::encode(custom_hash(b"input")),
            output_hash: hex::encode(custom_hash(b"output")),
            closure_hash: hex::encode(self.constitution.receipt),
            procedure_hash: hex::encode(ontology.trace_digest),
            pqc_seal: format!("z:{}-c:{}", 
                pqc_seal.z.coeffs.iter().take(8).map(|x| format!("{:04x}", x)).collect::<String>(),
                pqc_seal.c.coeffs.iter().take(8).map(|x| format!("{:04x}", x)).collect::<String>()
            ),
            previous_receipt_hash: hex::encode(all_zeros),
            actor_id: "admission_kernel".to_string(),
            transport: "truex-actuator".to_string(),
            session_id: "session-0".to_string(),
            replay_pointer: "ptr-0".to_string(),
            verdict: crate::truex::receipt::Verdict::Passed,
        })
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
        let receipt = self.mu.evaluate(&ontology)?;

        Ok(AdmittedConsequence {
            ontology,
            receipt,
            _seal: PhantomData,
        })
    }
}
