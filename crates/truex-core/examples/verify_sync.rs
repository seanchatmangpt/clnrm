use std::fs;
use std::path::Path;
use std::process::Command;
use anyhow::{anyhow, Result};
use truex_core::construct::{Graph, Term, Triple};
use truex_core::registry::{
    OntologyPack, PackMetadata, Vocabulary, VocabularyTerm, ShapeConstraints, ShapeConstraint,
    PropertyConstraint, DataType, wots, PqcSignature, RegistryService
};
use truex_core::actuator::{Actuator, PartyPacket};
use truex_core::receipt::Receipt;

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

fn main() -> Result<()> {
    println!("🚀 Starting dynamic CLI registration & sync verification example...");

    // Create temporary directory for fixtures and sync output
    let temp_dir = tempfile::tempdir()?;
    let pack_path = temp_dir.path().join("pack.json");
    let graph_path = temp_dir.path().join("graph.json");
    let packets_path = temp_dir.path().join("packets.json");
    let sync_dir = temp_dir.path().join("sync_out");

    // Write input files
    let pack = create_test_ontology_pack();
    fs::write(&pack_path, serde_json::to_string_pretty(&pack)?)?;

    let graph = make_valid_graph();
    fs::write(&graph_path, serde_json::to_string_pretty(&graph)?)?;

    let packets = make_test_packets();
    fs::write(&packets_path, serde_json::to_string_pretty(&packets)?)?;

    // Find clnrm binary. Let's execute it via cargo run to ensure it is up to date.
    println!("📦 Executing 'cargo run --bin clnrm -- ggen sync'...");
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "clnrm",
            "--",
            "ggen",
            "sync",
            "--sync_dir",
            sync_dir.to_str().unwrap(),
            "--pack_path",
            pack_path.to_str().unwrap(),
            "--input_path",
            graph_path.to_str().unwrap(),
            "--packets_path",
            packets_path.to_str().unwrap(),
        ])
        .output()?;

    if !output.status.success() {
        println!("❌ ggen sync command failed!");
        println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
        return Err(anyhow!("ggen sync execution returned non-zero exit code"));
    }

    println!("✅ ggen sync command exited successfully.");
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    println!("Command output:\n{}", stdout_str);

    // Verify output files exist
    let receipt_path = sync_dir.join("receipt.json");
    let proj_graph_path = sync_dir.join("projected_graph.json");
    let biz_records_path = sync_dir.join("business_records.json");

    assert!(receipt_path.exists(), "receipt.json was not materialized!");
    assert!(proj_graph_path.exists(), "projected_graph.json was not materialized!");
    assert!(biz_records_path.exists(), "business_records.json was not materialized!");
    println!("✅ All output artifacts successfully materialized.");

    // Parse and verify receipt
    let receipt_content = fs::read_to_string(&receipt_path)?;
    let receipt: Receipt = serde_json::from_str(&receipt_content)?;

    println!("🔒 Verifying materialized receipt PQC signature...");
    let verified = receipt.verify_seal()?;
    if verified {
        println!("✅ Receipt signature verified successfully!");
    } else {
        return Err(anyhow!("Receipt seal verification failed!"));
    }

    println!("🔁 Replaying receipt against registry...");
    let registry = RegistryService::new();
    let replayed = Actuator::replay(&receipt, &registry)?;
    if replayed {
        println!("✅ Receipt replay check succeeded!");
    } else {
        return Err(anyhow!("Receipt replay check failed!"));
    }

    println!("🎉 All CLI verification checks PASSED successfully!");
    Ok(())
}
