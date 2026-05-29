use std::path::Path;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use clnrm_core::pqc::lattice::{PrivateKey, PublicKey, generate_keypair};

use crate::construct::{Graph, ProjectionEngine};
use crate::registry::{OntologyPack, RegistryService};
use crate::receipt::{
    Receipt, ReceiptEmissionEngine, SessionAttribution, ActorAttribution,
    TransportAttribution, ReceiptPayload
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Authority {
    Write,
    SensingLsp,
    SensingMcp,
    A2aObserver,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PartyPacket {
    pub party_id: String,
    pub payload: String,
    pub public_key: Vec<String>,
    pub signature: Vec<String>,
}

impl PartyPacket {
    pub fn verify(&self) -> bool {
        let mut pub_key = Vec::new();
        for pk_hex in &self.public_key {
            let Ok(bytes) = hex::decode(pk_hex) else { return false; };
            let Ok(arr) = bytes.try_into() else { return false; };
            pub_key.push(arr);
        }
        let mut sig = Vec::new();
        for sig_hex in &self.signature {
            let Ok(bytes) = hex::decode(sig_hex) else { return false; };
            let Ok(arr) = bytes.try_into() else { return false; };
            sig.push(arr);
        }
        crate::registry::wots::verify(&pub_key, &sig, self.payload.as_bytes())
    }
}

pub struct Actuator {
    pub private_key: PrivateKey,
    pub signing_seed: [u8; 32],
}

impl Actuator {
    pub fn new(private_key: PrivateKey, signing_seed: [u8; 32]) -> Self {
        Self {
            private_key,
            signing_seed,
        }
    }

    /// Primary sync engine method. Verifies inputs, ensures sensing routes cannot write,
    /// and materializes admitted artifacts to sync_dir.
    pub fn sync(
        &self,
        sync_dir: &Path,
        pack: &OntologyPack,
        input: &Graph,
        packets: &[PartyPacket],
        authority: Authority,
    ) -> Result<Receipt> {
        let start_time = chrono::Utc::now();

        // 1. Ensure sensing routes cannot write state
        if matches!(authority, Authority::SensingLsp | Authority::SensingMcp | Authority::A2aObserver) {
            return Err(anyhow!(
                "Sensing routes (LSP/MCP) and observers (A2A) are strictly non-mutating and cannot write state. Authority: {:?}",
                authority
            ));
        }

        // 2. Verify Ontology Pack Signature
        pack.verify_signature()
            .map_err(|e| anyhow!("Ontology pack signature verification failed: {}", e))?;

        // 3. Verify Party Packets
        for packet in packets {
            if !packet.verify() {
                return Err(anyhow!("PartyPacket signature verification failed for party: {}", packet.party_id));
            }
        }

        // 4. Validate Input Graph against Ontology Pack constraints
        let mut registry = RegistryService::new();
        let pack_json = serde_json::to_string(pack)?;
        registry.ingest(&pack_json)
            .map_err(|e| anyhow!("Failed to ingest ontology pack into temporary validator registry: {}", e))?;

        let records = ProjectionEngine::extract_records(input);
        for record in &records {
            // Strip angle brackets from term IRI to match the shapes defined in ontology
            let clean_type = record.record_type.trim_start_matches('<').trim_end_matches('>');
            
            // Build a JSON representation of record properties to validate against constraints
            let mut record_map = serde_json::Map::new();
            for (k, v) in &record.properties {
                // If it is numeric or boolean, parse it correctly for registry validator, otherwise keep as string
                let json_val = if let Ok(n) = v.parse::<i64>() {
                    serde_json::Value::Number(n.into())
                } else if let Ok(b) = v.parse::<bool>() {
                    serde_json::Value::Bool(b)
                } else {
                    serde_json::Value::String(v.clone())
                };
                record_map.insert(k.clone(), json_val);
            }
            let record_json = serde_json::Value::Object(record_map);

            registry.validate_instance(&pack.metadata.name, clean_type, &record_json)
                .map_err(|e| anyhow!("Input graph term validation failed for target_term '{}': {}", clean_type, e))?;
        }

        // 5. Run CONSTRUCT projection to materialize outputs
        let profile = build_profile_from_pack(pack);
        let projected_graph = ProjectionEngine::project(input, &profile, false)?;

        // 6. Materialize admitted artifacts to sync_dir
        std::fs::create_dir_all(sync_dir)?;
        
        let graph_path = sync_dir.join("projected_graph.json");
        std::fs::write(&graph_path, serde_json::to_string_pretty(&projected_graph)?)?;

        let projected_records = ProjectionEngine::extract_records(&projected_graph);
        let records_path = sync_dir.join("business_records.json");
        std::fs::write(&records_path, serde_json::to_string_pretty(&projected_records)?)?;

        let artifacts_dir = sync_dir.join("artifacts");
        std::fs::create_dir_all(&artifacts_dir)?;
        for record in &projected_records {
            let clean_id = record.id.trim_start_matches('<').trim_end_matches('>').replace('/', "_").replace(':', "_");
            let artifact_path = artifacts_dir.join(format!("{}.json", clean_id));
            std::fs::write(&artifact_path, serde_json::to_string_pretty(record)?)?;
        }

        // 7. Emit PQC-sealed Receipt
        let input_bytes = serde_json::to_vec(input)?;
        let input_hash = hex::encode(clnrm_core::pqc::hash::custom_hash(&input_bytes));

        let output_bytes = serde_json::to_vec(&projected_graph)?;
        let output_hash = hex::encode(clnrm_core::pqc::hash::custom_hash(&output_bytes));

        let closure_hash = pack.metadata.hash.clone();

        let end_time = chrono::Utc::now();
        let duration_ms = end_time.signed_duration_since(start_time).num_milliseconds().max(0) as u64;

        let session = SessionAttribution {
            session_id: format!("session_{}", uuid::Uuid::new_v4()),
            timestamp: start_time.to_rfc3339(),
            duration_ms,
        };

        let actor = ActorAttribution {
            actor_id: "authority_actuator".to_string(),
            role: "Actuator".to_string(),
            public_key: Some(hex::encode(custom_hash_key(&self.private_key.pub_key.a.coeffs))),
        };

        let transport = TransportAttribution {
            protocol: "ggen-sync-v1".to_string(),
            endpoint: "localhost".to_string(),
            client_version: "26.5.28".to_string(),
            metadata: std::collections::BTreeMap::new(),
        };

        let payload = ReceiptPayload {
            input_hash,
            output_hash,
            closure_hash,
        };

        let receipt = ReceiptEmissionEngine::emit(
            session,
            actor,
            transport,
            payload,
            &self.private_key,
            self.signing_seed,
        )?;

        Ok(receipt)
    }

    /// Replay verification logic
    pub fn replay(
        receipt: &Receipt,
        _registry: &RegistryService,
    ) -> Result<bool> {
        // 1. Verify receipt seal
        if !receipt.verify_seal()? {
            return Ok(false);
        }
        Ok(true)
    }
}

/// Helper to hash public key coefficients for display
fn custom_hash_key(coeffs: &[i64; 64]) -> [u8; 32] {
    let mut bytes = Vec::new();
    for &c in coeffs {
        bytes.extend_from_slice(&c.to_le_bytes());
    }
    clnrm_core::pqc::hash::custom_hash(&bytes)
}

fn build_profile_from_pack(pack: &OntologyPack) -> crate::construct::ConstructProfile {
    crate::construct::ConstructProfile {
        name: format!("{}_profile", pack.metadata.name),
        construct_clause: vec![
            crate::construct::TriplePattern {
                subject: crate::construct::PatternTerm::Variable("s".to_string()),
                predicate: crate::construct::PatternTerm::Variable("p".to_string()),
                object: crate::construct::PatternTerm::Variable("o".to_string()),
            }
        ],
        where_clause: vec![
            crate::construct::TriplePattern {
                subject: crate::construct::PatternTerm::Variable("s".to_string()),
                predicate: crate::construct::PatternTerm::Variable("p".to_string()),
                object: crate::construct::PatternTerm::Variable("o".to_string()),
            }
        ],
        filters: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::construct::{Term, Triple};
    use crate::registry::{
        PackMetadata, Vocabulary, VocabularyTerm, ShapeConstraints, ShapeConstraint,
        PropertyConstraint, DataType, wots, PqcSignature
    };
    use serde_json::json;

    fn sign_pack_helper(entropy: &[u8; 32], pack: &mut OntologyPack) {
        pack.metadata.hash = String::new();
        pack.signature = PqcSignature {
            public_key: vec![],
            signature_blocks: vec![],
        };

        let content_hash = pack.compute_content_hash().unwrap();
        pack.metadata.hash = content_hash.clone();

        let hash_bytes = hex::decode(&content_hash).unwrap();
        let (priv_key, pub_key) = wots::generate_keypair(entropy);
        let signature_blocks = wots::sign(&priv_key, &hash_bytes);

        pack.signature = PqcSignature {
            public_key: pub_key.iter().map(hex::encode).collect(),
            signature_blocks: signature_blocks.iter().map(hex::encode).collect(),
        };
    }

    fn create_test_ontology_pack() -> OntologyPack {
        let mut pack = OntologyPack {
            metadata: PackMetadata {
                name: "truex-test-ontology".to_string(),
                version: "1.0.0".to_string(),
                description: "Test pack for Actuator sync validation".to_string(),
                hash: String::new(),
            },
            vocabulary: Vocabulary {
                namespace: "truex.test".to_string(),
                terms: vec![
                    VocabularyTerm {
                        name: "UserRecord".to_string(),
                        term_type: "concept".to_string(),
                        data_type: None,
                        description: "User concept".to_string(),
                    },
                    VocabularyTerm {
                        name: "username".to_string(),
                        term_type: "property".to_string(),
                        data_type: Some(DataType::String),
                        description: "Standard name".to_string(),
                    },
                ],
            },
            shape_constraints: ShapeConstraints {
                shapes: vec![ShapeConstraint {
                    target_term: "UserRecord".to_string(),
                    property_constraints: vec![
                        PropertyConstraint {
                            property_name: "username".to_string(),
                            required: true,
                            expected_type: DataType::String,
                            min_value: None,
                            max_value: None,
                            pattern: Some("^[a-zA-Z0-9_]{3,15}$".to_string()),
                        },
                    ],
                }],
            },
            signature: PqcSignature {
                public_key: vec![],
                signature_blocks: vec![],
            },
        };
        sign_pack_helper(&[9u8; 32], &mut pack);
        pack
    }

    fn make_test_packets() -> Vec<PartyPacket> {
        let entropy = [11u8; 32];
        let payload = "agreement_payload";
        let (priv_key, pub_key) = wots::generate_keypair(&entropy);
        let signature = wots::sign(&priv_key, payload.as_bytes());

        vec![PartyPacket {
            party_id: "counterparty_1".to_string(),
            payload: payload.to_string(),
            public_key: pub_key.iter().map(hex::encode).collect(),
            signature: signature.iter().map(hex::encode).collect(),
        }]
    }

    fn make_valid_graph() -> Graph {
        let mut graph = Graph::new();
        graph.add_triple(Triple {
            subject: Term::IRI("user_bob".to_string()),
            predicate: Term::IRI("a".to_string()),
            object: Term::IRI("UserRecord".to_string()),
        });
        graph.add_triple(Triple {
            subject: Term::IRI("user_bob".to_string()),
            predicate: Term::IRI("username".to_string()),
            object: Term::Literal("bob_99".to_string()),
        });
        graph
    }

    #[test]
    fn test_sensing_routes_cannot_write() {
        let pack = create_test_ontology_pack();
        let graph = make_valid_graph();
        let packets = make_test_packets();

        let actuator_kp = generate_keypair([1u8; 32]);
        let actuator = Actuator::new(actuator_kp.secret, [2u8; 32]);

        let sync_dir = tempfile::tempdir().unwrap();

        // SensingLsp should fail
        let res = actuator.sync(sync_dir.path(), &pack, &graph, &packets, Authority::SensingLsp);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("strictly non-mutating"));

        // SensingMcp should fail
        let res = actuator.sync(sync_dir.path(), &pack, &graph, &packets, Authority::SensingMcp);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("strictly non-mutating"));

        // A2aObserver should fail
        let res = actuator.sync(sync_dir.path(), &pack, &graph, &packets, Authority::A2aObserver);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("strictly non-mutating"));
    }

    #[test]
    fn test_write_authority_success_and_replay() {
        let pack = create_test_ontology_pack();
        let graph = make_valid_graph();
        let packets = make_test_packets();

        let actuator_kp = generate_keypair([1u8; 32]);
        let actuator = Actuator::new(actuator_kp.secret, [2u8; 32]);

        let sync_dir = tempfile::tempdir().unwrap();

        // Write authority should succeed
        let res = actuator.sync(sync_dir.path(), &pack, &graph, &packets, Authority::Write);
        assert!(res.is_ok(), "Sync failed: {:?}", res);
        let receipt = res.unwrap();

        // Check materialized files
        assert!(sync_dir.path().join("projected_graph.json").exists());
        assert!(sync_dir.path().join("business_records.json").exists());
        assert!(sync_dir.path().join("artifacts/user_bob.json").exists());

        // Replay validation
        let registry = RegistryService::new();
        let replay_res = Actuator::replay(&receipt, &registry);
        assert!(replay_res.is_ok());
        assert!(replay_res.unwrap());
    }

    #[test]
    fn test_invalid_input_graph_violates_shapes_fails() {
        let pack = create_test_ontology_pack();
        let packets = make_test_packets();

        // Invalid graph: violates regex ^[a-zA-Z0-9_]{3,15}$ (too short)
        let mut graph = Graph::new();
        graph.add_triple(Triple {
            subject: Term::IRI("user_bob".to_string()),
            predicate: Term::IRI("a".to_string()),
            object: Term::IRI("UserRecord".to_string()),
        });
        graph.add_triple(Triple {
            subject: Term::IRI("user_bob".to_string()),
            predicate: Term::IRI("username".to_string()),
            object: Term::Literal("b".to_string()),
        });

        let actuator_kp = generate_keypair([1u8; 32]);
        let actuator = Actuator::new(actuator_kp.secret, [2u8; 32]);

        let sync_dir = tempfile::tempdir().unwrap();
        let res = actuator.sync(sync_dir.path(), &pack, &graph, &packets, Authority::Write);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("validation failed"));
    }
}
