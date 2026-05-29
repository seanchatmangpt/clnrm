//! Replay Surface
//!
//! Provides a verification module to replay issued receipts against
//! the same state/inputs to deterministically reproduce identical outcomes or refuse.

use crate::construct::{Graph as ConstructGraph, ConstructProfile, ProjectionEngine, Term};
use crate::admission::{Graph as AdmissionGraph, PartyPacket, EscrowPolicy, AdmissionKernel, AdmissionOutcome, Record};
use crate::receipt::Receipt;
use crate::registry::RegistryService;
use serde::{Serialize, Deserialize};
use std::path::Path;
use anyhow::{anyhow, Result};

/// A complete bundle needed to replay an execution of the Truex Execution Trust Marketplace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayFixture {
    /// The original issued receipt (PQC-sealed)
    pub receipt: Receipt,
    /// The input graph state before CONSTRUCT projection
    pub input_graph: ConstructGraph,
    /// The CONSTRUCT profile used to project the input graph
    pub construct_profile: ConstructProfile,
    /// The list of party packets submitted
    pub party_packets: Vec<PartyPacket>,
    /// The escrow policy that was evaluated
    pub escrow_policy: EscrowPolicy,
    /// The expected verdict
    pub expected_outcome: ReplayOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReplayOutcome {
    Admit {
        consensus_hash: String,
    },
    Refuse {
        reasons: Vec<String>,
    },
}

pub struct ReplayService;

impl ReplayService {
    /// Loads a ReplayFixture from a JSON string.
    pub fn load_fixture_from_json(json_str: &str) -> Result<ReplayFixture> {
        serde_json::from_str(json_str).map_err(|e| anyhow!("Failed to deserialize ReplayFixture: {}", e))
    }

    /// Loads a ReplayFixture from a JSON file.
    pub fn load_fixture_from_file(path: &Path) -> Result<ReplayFixture> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow!("Failed to read ReplayFixture file: {}", e))?;
        Self::load_fixture_from_json(&content)
    }

    /// Executes the transactional replay for the given fixture.
    /// Returns Ok(()) if the verdict is identical and all verifications pass,
    /// or Err detailing why the replay failed or was refused.
    pub fn replay_transaction(fixture: &ReplayFixture, _registry: &RegistryService) -> Result<()> {
        // 1. Verify the receipt seal/signature
        let is_valid_seal = fixture.receipt.verify_seal()
            .map_err(|e| anyhow!("Receipt seal verification failed: {}", e))?;
        if !is_valid_seal {
            return Err(anyhow!("Receipt signature is invalid"));
        }

        // 2. Validate input graph matches receipt input hash
        let input_bytes = serde_json::to_vec(&fixture.input_graph)
            .map_err(|e| anyhow!("Failed to serialize input graph: {}", e))?;
        let computed_input_hash = hex::encode(clnrm_core::pqc::hash::custom_hash(&input_bytes));
        if computed_input_hash != fixture.receipt.payload.input_hash {
            return Err(anyhow!(
                "Replay refused: Input graph hash mismatch. Expected {}, computed {}",
                fixture.receipt.payload.input_hash,
                computed_input_hash
            ));
        }

        // 3. Re-run CONSTRUCT projection
        let projected_graph = ProjectionEngine::project_local(&fixture.input_graph, &fixture.construct_profile)
            .map_err(|e| anyhow!("Projection failed during replay: {}", e))?;

        if let ReplayOutcome::Admit { .. } = &fixture.expected_outcome {
            // Verify projected graph matches output hash in receipt
            let projected_bytes = serde_json::to_vec(&projected_graph)
                .map_err(|e| anyhow!("Failed to serialize projected graph: {}", e))?;
            let computed_output_hash = hex::encode(clnrm_core::pqc::hash::custom_hash(&projected_bytes));

            if computed_output_hash != fixture.receipt.payload.output_hash {
                return Err(anyhow!(
                    "Replay refused: Projected output graph hash mismatch. Expected {}, computed {}",
                    fixture.receipt.payload.output_hash,
                    computed_output_hash
                ));
            }
        }

        // 4. Map projected construct::Graph to admission::Graph
        let mut admission_records = Vec::new();
        for triple in &projected_graph.triples {
            let val_str = match &triple.object {
                Term::Literal(val) => val.clone(),
                other => other.to_string(),
            };
            admission_records.push(Record {
                entity: triple.subject.to_string(),
                attribute: triple.predicate.to_string(),
                value: val_str,
            });
        }
        let admission_graph = AdmissionGraph { records: admission_records };

        // 5. Parse evaluation timestamp
        let datetime = chrono::DateTime::parse_from_rfc3339(&fixture.receipt.session.timestamp)
            .or_else(|_| chrono::DateTime::parse_from_str(&fixture.receipt.session.timestamp, "%+"))
            .map_err(|e| anyhow!("Invalid timestamp in receipt: {}", e))?;
        let evaluation_timestamp = datetime.timestamp();

        // 6. Run Admission Kernel
        let outcome = AdmissionKernel::evaluate(
            &admission_graph,
            &fixture.party_packets,
            &fixture.escrow_policy,
            evaluation_timestamp,
            &[],
        );

        // 7. Verify verdict identical
        match (&fixture.expected_outcome, outcome) {
            (ReplayOutcome::Admit { consensus_hash: expected_hash }, AdmissionOutcome::Admit(receipt)) => {
                if receipt.consensus_hash != *expected_hash {
                    return Err(anyhow!(
                        "Replay refused: Consensus hash mismatch. Expected {}, computed {}",
                        expected_hash,
                        receipt.consensus_hash
                    ));
                }
                Ok(())
            }
            (ReplayOutcome::Refuse { reasons: expected_reasons }, AdmissionOutcome::Refuse(refusal)) => {
                // Check if all expected reasons are present in the refusal reasons
                for req in expected_reasons {
                    if !refusal.reasons.iter().any(|r| r.contains(req)) {
                        return Err(anyhow!(
                            "Replay refused: Refusal reasons do not match. Expected reason '{}', got {:?}",
                            req,
                            refusal.reasons
                        ));
                    }
                }
                Ok(())
            }
            (ReplayOutcome::Admit { .. }, AdmissionOutcome::Refuse(refusal)) => {
                Err(anyhow!(
                    "Replay refused: Expected Admit, but outcome was Refuse. Reasons: {:?}",
                    refusal.reasons
                ))
            }
            (ReplayOutcome::Refuse { .. }, AdmissionOutcome::Admit(_)) => {
                Err(anyhow!("Replay refused: Expected Refuse, but outcome was Admit"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::construct::{Term, Triple, TriplePattern, PatternTerm};
    use crate::admission::{ConstraintOperator, RecordConstraint};
    use crate::receipt::{SessionAttribution, ActorAttribution, TransportAttribution, ReceiptPayload, ReceiptEmissionEngine};
    use clnrm_core::pqc::lattice::generate_keypair;
    use std::collections::BTreeMap;

    fn build_test_fixture(admit: bool) -> (ReplayFixture, RegistryService) {
        let registry = RegistryService::new();

        let mut input_graph = ConstructGraph::new();
        input_graph.add_triple(Triple {
            subject: Term::IRI("user1".to_string()),
            predicate: Term::IRI("a".to_string()),
            object: Term::IRI("User".to_string()),
        });
        input_graph.add_triple(Triple {
            subject: Term::IRI("user1".to_string()),
            predicate: Term::IRI("status".to_string()),
            object: Term::Literal("active".to_string()),
        });
        input_graph.canonicalize();

        let construct_profile = ConstructProfile {
            name: "test_projection".to_string(),
            construct_clause: vec![
                TriplePattern {
                    subject: PatternTerm::Variable("user".to_string()),
                    predicate: PatternTerm::Constant(Term::IRI("a".to_string())),
                    object: PatternTerm::Constant(Term::IRI("ProjectedUser".to_string())),
                },
                TriplePattern {
                    subject: PatternTerm::Variable("user".to_string()),
                    predicate: PatternTerm::Constant(Term::IRI("status".to_string())),
                    object: PatternTerm::Variable("status".to_string()),
                },
            ],
            where_clause: vec![
                TriplePattern {
                    subject: PatternTerm::Variable("user".to_string()),
                    predicate: PatternTerm::Constant(Term::IRI("a".to_string())),
                    object: PatternTerm::Constant(Term::IRI("User".to_string())),
                },
                TriplePattern {
                    subject: PatternTerm::Variable("user".to_string()),
                    predicate: PatternTerm::Constant(Term::IRI("status".to_string())),
                    object: PatternTerm::Variable("status".to_string()),
                },
            ],
            filters: vec![],
        };

        // Generate PQC keys for sealing the receipt
        let kp = generate_keypair([42u8; 32]);
        let signing_seed = [99u8; 32];

        let input_bytes = serde_json::to_vec(&input_graph).unwrap();
        let input_hash = hex::encode(clnrm_core::pqc::hash::custom_hash(&input_bytes));

        let projected_graph = ProjectionEngine::project_local(&input_graph, &construct_profile).unwrap();
        let output_bytes = serde_json::to_vec(&projected_graph).unwrap();
        let output_hash = hex::encode(clnrm_core::pqc::hash::custom_hash(&output_bytes));

        let session = SessionAttribution {
            session_id: "sess_123".to_string(),
            timestamp: "2026-05-29T02:00:00Z".to_string(),
            duration_ms: 5,
        };

        let actor = ActorAttribution {
            actor_id: "act_1".to_string(),
            role: "Actuator".to_string(),
            public_key: None,
        };

        let transport = TransportAttribution {
            protocol: "HTTP".to_string(),
            endpoint: "127.0.0.1".to_string(),
            client_version: "1.0".to_string(),
            metadata: BTreeMap::new(),
        };

        let payload = ReceiptPayload {
            input_hash,
            output_hash,
            closure_hash: "dummy_closure_hash".to_string(),
        };

        let receipt = ReceiptEmissionEngine::emit(
            session,
            actor,
            transport,
            payload,
            &kp.secret,
            signing_seed,
        ).unwrap();

        let party_packets = vec![
            PartyPacket {
                sender: "Alice".to_string(),
                payload: "Agree".to_string(),
                signature_hex: None,
                public_key_hex: None,
            }
        ];

        let escrow_policy = EscrowPolicy {
            policy_id: "TestPolicy".to_string(),
            required_senders: vec!["Alice".to_string()],
            verify_signatures: false,
            record_constraints: vec![
                RecordConstraint {
                    entity: "<user1>".to_string(),
                    attribute: "<status>".to_string(),
                    operator: ConstraintOperator::Equals,
                    expected_value: if admit { "active".to_string() } else { "inactive".to_string() },
                }
            ],
            time_lock: None,
        };

        // Convert the projected graph to expected records to compute consensus hash
        let mut admission_records = Vec::new();
        for triple in &projected_graph.triples {
            let val_str = match &triple.object {
                Term::Literal(val) => val.clone(),
                other => other.to_string(),
            };
            admission_records.push(Record {
                entity: triple.subject.to_string(),
                attribute: triple.predicate.to_string(),
                value: val_str,
            });
        }
        let admission_graph = AdmissionGraph { records: admission_records };
        let outcome = AdmissionKernel::evaluate(&admission_graph, &party_packets, &escrow_policy, 1779988800);

        let expected_outcome = match outcome {
            AdmissionOutcome::Admit(rec) => ReplayOutcome::Admit {
                consensus_hash: rec.consensus_hash,
            },
            AdmissionOutcome::Refuse(refusal) => ReplayOutcome::Refuse {
                reasons: refusal.reasons,
            },
        };

        (
            ReplayFixture {
                receipt,
                input_graph,
                construct_profile,
                party_packets,
                escrow_policy,
                expected_outcome,
            },
            registry,
        )
    }

    #[test]
    fn test_replay_admit_success() {
        let (fixture, registry) = build_test_fixture(true);
        let res = ReplayService::replay_transaction(&fixture, &registry);
        assert!(res.is_ok(), "Replay failed: {:?}", res);
    }

    #[test]
    fn test_replay_refuse_success() {
        let (fixture, registry) = build_test_fixture(false);
        let res = ReplayService::replay_transaction(&fixture, &registry);
        assert!(res.is_ok(), "Replay failed: {:?}", res);
    }

    #[test]
    fn test_replay_fails_on_tempered_input() {
        let (mut fixture, registry) = build_test_fixture(true);
        // Temper with input graph
        fixture.input_graph.add_triple(Triple {
            subject: Term::IRI("tampered_user".to_string()),
            predicate: Term::IRI("status".to_string()),
            object: Term::Literal("active".to_string()),
        });

        let res = ReplayService::replay_transaction(&fixture, &registry);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("Input graph hash mismatch"));
    }

    #[test]
    fn test_replay_refuse_fails_on_tempered_input() {
        let (mut fixture, registry) = build_test_fixture(false);
        // Temper with input graph
        fixture.input_graph.add_triple(Triple {
            subject: Term::IRI("tampered_user".to_string()),
            predicate: Term::IRI("status".to_string()),
            object: Term::Literal("active".to_string()),
        });

        let res = ReplayService::replay_transaction(&fixture, &registry);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("Input graph hash mismatch"));
    }

    #[test]
    fn test_replay_refuse_fails_on_tempered_signature() {
        let (mut fixture, registry) = build_test_fixture(false);
        // Tamper with signature coefficients in seal
        if let Some(ref mut seal) = fixture.receipt.seal {
            seal.sig_z[0] = seal.sig_z[0].wrapping_add(1);
        }

        let res = ReplayService::replay_transaction(&fixture, &registry);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("Receipt signature is invalid"));
    }
}
