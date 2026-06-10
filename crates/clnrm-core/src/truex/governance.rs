use crate::error::{CleanroomError, Result}; // Assuming this is the correct error module
use crate::truex::admission_types::{Graph, PartyPacket};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use tracing::{info, warn};

/// Manifest for ontology-based consequences, cryptographically linked to a Consequence Grammar.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OntologyPack {
    pub pack_id: String,
    pub grammar: Graph,
    pub signature: String,
    pub public_key: String,
}

/// The authoritative registry for all admitted laws.
pub struct RegistryService {
    admitted_laws: RwLock<HashMap<String, OntologyPack>>,
}

impl RegistryService {
    pub fn new() -> Self {
        Self {
            admitted_laws: RwLock::new(HashMap::new()),
        }
    }

    /// Admit a new ontology pack into the registry, performing strict PQC signature validation.
    pub fn admit(&self, pack: OntologyPack) -> Result<()> {
        let packet = PartyPacket {
            sender: pack.pack_id.clone(),
            payload: serde_json::to_string(&pack.grammar)
                .map_err(|e| CleanroomError::serialization_error(e.to_string()))?,
            nonce: 0,
            signature_hex: Some(pack.signature.clone()),
            public_key_hex: Some(pack.public_key.clone()),
        };

        if !packet
            .verify_signature()
            .map_err(|e| CleanroomError::validation_error(e))?
        {
            return Err(CleanroomError::validation_error(
                "Signature validation failed",
            ));
        }

        let pack_id = pack.pack_id.clone();
        let mut laws = self
            .admitted_laws
            .write()
            .map_err(|_| CleanroomError::internal_error("Lock poisoned"))?;
        laws.insert(pack_id.clone(), pack);

        info!(pack_id = %pack_id, "Ontology pack admitted to registry.");
        Ok(())
    }

    pub fn is_admitted(&self, pack_id: &str) -> bool {
        self.admitted_laws
            .read()
            .map(|laws| laws.contains_key(pack_id))
            .unwrap_or(false)
    }

    /// Validates a consequence against the grammar of an admitted pack.
    pub fn validate_consequence(&self, pack_id: &str, graph: &Graph) -> bool {
        let laws = self.admitted_laws.read().expect("Lock poisoned");
        if let Some(pack) = laws.get(pack_id) {
            // Functional grammar validation: ensure all records in the consequence graph
            // are compliant with the admitted ontology grammar.
            graph
                .records
                .iter()
                .all(|r| pack.grammar.records.contains(r))
        } else {
            warn!(pack_id = %pack_id, "Ontology pack not found in registry.");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::truex::admission_types::Record;

    #[test]
    fn test_registry_service() {
        let registry = RegistryService::new();
        let pack = OntologyPack {
            pack_id: "test-pack".to_string(),
            grammar: Graph {
                records: vec![Record {
                    entity: "A".into(),
                    attribute: "B".into(),
                    value: "C".into(),
                }],
            },
            signature: "".to_string(),
            public_key: "".to_string(),
        };
        // Signature missing/empty, should fail
        assert!(registry.admit(pack).is_err());
    }
}
