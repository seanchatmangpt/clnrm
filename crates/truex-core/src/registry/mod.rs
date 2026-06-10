use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use sha2::{Sha256, Digest};
use std::convert::TryInto;
use semver::Version;

/// PQC (Post-Quantum Cryptography) Signature verification and signing using Winternitz OTS.
pub mod wots {
    use super::*;

    pub const W_VAL: usize = 16;
    pub const L1: usize = 64; // For 256-bit message digest, 64 nibbles
    pub const L2: usize = 3;  // Checksum fits in 3 nibbles (max sum is 64 * 15 = 960)
    pub const L: usize = L1 + L2; // 67 blocks

    fn hash_block(block: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(block);
        hasher.finalize().into()
    }

    fn hash_chain(block: &[u8; 32], steps: usize) -> [u8; 32] {
        let mut current = *block;
        for _ in 0..steps {
            current = hash_block(&current);
        }
        current
    }

    /// Convert a 32-byte message digest into L (67) nibble-based lengths (0-15) plus checksum.
    pub fn message_to_lengths(message_digest: &[u8; 32]) -> Vec<usize> {
        let mut lengths = Vec::with_capacity(L);
        // Extract 64 nibbles
        for &byte in message_digest {
            lengths.push((byte >> 4) as usize);
            lengths.push((byte & 0x0F) as usize);
        }

        // Compute checksum
        let mut checksum = 0;
        for &len in &lengths {
            checksum += (W_VAL - 1) - len;
        }

        // Append checksum as 3 nibbles (base 16)
        lengths.push((checksum >> 8) & 0x0F);
        lengths.push((checksum >> 4) & 0x0F);
        lengths.push(checksum & 0x0F);

        lengths
    }

    /// Generate public and private keys from a 32-byte seed/entropy source.
    pub fn generate_keypair(entropy: &[u8; 32]) -> (Vec<[u8; 32]>, Vec<[u8; 32]>) {
        let mut private_key = Vec::with_capacity(L);
        for i in 0..L {
            let mut hasher = Sha256::new();
            hasher.update(entropy);
            hasher.update(&(i as u32).to_be_bytes());
            private_key.push(hasher.finalize().into());
        }

        let mut public_key = Vec::with_capacity(L);
        for block in &private_key {
            public_key.push(hash_chain(block, W_VAL - 1));
        }

        (private_key, public_key)
    }

    /// Sign a message using WOTS private key blocks.
    pub fn sign(private_key: &[[u8; 32]], message: &[u8]) -> Vec<[u8; 32]> {
        let mut hasher = Sha256::new();
        hasher.update(message);
        let digest: [u8; 32] = hasher.finalize().into();

        let lengths = message_to_lengths(&digest);
        let mut signature = Vec::with_capacity(L);
        for i in 0..L {
            signature.push(hash_chain(&private_key[i], lengths[i]));
        }
        signature
    }

    /// Verify a WOTS signature using WOTS public key blocks.
    pub fn verify(public_key: &[[u8; 32]], signature: &[[u8; 32]], message: &[u8]) -> bool {
        if public_key.len() != L || signature.len() != L {
            return false;
        }

        let mut hasher = Sha256::new();
        hasher.update(message);
        let digest: [u8; 32] = hasher.finalize().into();

        let lengths = message_to_lengths(&digest);
        for i in 0..L {
            let steps = (W_VAL - 1) - lengths[i];
            let block = hash_chain(&signature[i], steps);
            if block != public_key[i] {
                return false;
            }
        }
        true
    }
}

/// Errors returned by the Registry Service.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RegistryError {
    #[error("Failed to parse JSON representation: {0}")]
    JsonError(String),

    #[error("Invalid version format '{0}': {1}")]
    InvalidVersion(String, String),

    #[error("Version '{new_version}' is not newer than existing version '{existing_version}' for pack '{pack_name}'")]
    VersionDowngrade {
        pack_name: String,
        new_version: String,
        existing_version: String,
    },

    #[error("Content hash mismatch: computed '{computed}', but metadata specifies '{specified}'")]
    HashMismatch {
        computed: String,
        specified: String,
    },

    #[error("PQC signature verification failed")]
    SignatureVerificationFailed,

    #[error("Invalid shape constraint for target term '{0}': {1}")]
    InvalidShapeConstraint(String, String),

    #[error("Ontology pack '{0}' not found")]
    PackNotFound(String),

    #[error("Shape constraint for term '{0}' not found in pack '{1}'")]
    ShapeNotFound(String, String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Cryptographic format error: {0}")]
    CryptoFormatError(String),

    #[error("Winternitz OTS key reuse detected on upgrade for pack '{0}'")]
    KeyReuseDetected(String),

    #[error("Public key pinning validation failed for pack '{0}': transition signature missing or invalid")]
    KeyPinningViolation(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DataType {
    String,
    Integer,
    Boolean,
    Float,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VocabularyTerm {
    pub name: String,
    pub term_type: String, // e.g. "concept", "property", "relation"
    pub data_type: Option<DataType>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Vocabulary {
    pub namespace: String,
    pub terms: Vec<VocabularyTerm>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PropertyConstraint {
    pub property_name: String,
    pub required: bool,
    pub expected_type: DataType,
    pub min_value: Option<i64>,
    pub max_value: Option<i64>,
    pub pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShapeConstraint {
    pub target_term: String,
    pub property_constraints: Vec<PropertyConstraint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShapeConstraints {
    pub shapes: Vec<ShapeConstraint>,
}

fn decode_blocks(blocks: &[String], name: &str) -> Result<Vec<[u8; 32]>, RegistryError> {
    if blocks.len() != wots::L {
        return Err(RegistryError::CryptoFormatError(format!(
            "{} must have length {}",
            name,
            wots::L
        )));
    }
    let mut decoded = Vec::with_capacity(wots::L);
    for block_hex in blocks {
        let bytes = hex::decode(block_hex)
            .map_err(|e| RegistryError::CryptoFormatError(format!("Invalid {} block hex: {}", name, e)))?;
        let arr: [u8; 32] = bytes.try_into().map_err(|_| {
            RegistryError::CryptoFormatError(format!("{} block must be 32 bytes", name))
        })?;
        decoded.push(arr);
    }
    Ok(decoded)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PqcSignature {
    /// 67 public key blocks hex-encoded.
    pub public_key: Vec<String>,
    /// 67 signature blocks hex-encoded.
    pub signature_blocks: Vec<String>,
    /// Optional transition signature: the previous key signing the new public key.
    /// Required on upgrades to prevent takeover.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transition_signature: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackMetadata {
    pub name: String,
    pub version: String, // SemVer
    pub description: String,
    pub hash: String, // Content hash of vocabulary and shape constraints
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OntologyPack {
    pub metadata: PackMetadata,
    pub vocabulary: Vocabulary,
    pub shape_constraints: ShapeConstraints,
    pub signature: PqcSignature,
}

impl OntologyPack {
    /// Computes the SHA-256 content hash of the Ontology Pack excluding the hash and signature fields.
    pub fn compute_content_hash(&self) -> Result<String, serde_json::Error> {
        let mut temp_pack = self.clone();
        temp_pack.metadata.hash = String::new();
        temp_pack.signature = PqcSignature {
            public_key: vec![],
            signature_blocks: vec![],
            transition_signature: None,
        };
        let serialized = serde_json::to_vec(&temp_pack)?;
        let mut hasher = Sha256::new();
        hasher.update(&serialized);
        let result = hasher.finalize();
        Ok(hex::encode(result))
    }

    /// Verifies the WOTS signature against the pack's content hash.
    pub fn verify_signature(&self) -> Result<(), RegistryError> {
        let pub_key = decode_blocks(&self.signature.public_key, "Public key")?;
        let signature = decode_blocks(&self.signature.signature_blocks, "Signature")?;

        let hash_bytes = hex::decode(&self.metadata.hash)
            .map_err(|e| RegistryError::CryptoFormatError(format!("Invalid metadata hash hex: {}", e)))?;

        if !wots::verify(&pub_key, &signature, &hash_bytes) {
            return Err(RegistryError::SignatureVerificationFailed);
        }

        Ok(())
    }
}

/// In-memory Registry Service implementing ingestion, verification, and data validation.
#[derive(Debug, Default)]
pub struct RegistryService {
    packs: HashMap<String, OntologyPack>,
}

impl RegistryService {
    /// Creates a new empty RegistryService.
    pub fn new() -> Self {
        Self {
            packs: HashMap::new(),
        }
    }

    /// Ingests a raw public ontology pack (JSON format), validating version, hash, shape constraints, and PQC signatures.
    pub fn ingest(&mut self, pack_json: &str) -> Result<(), RegistryError> {
        let pack: OntologyPack = serde_json::from_str(pack_json)
            .map_err(|e| RegistryError::JsonError(e.to_string()))?;

        // 1. Validate Version Format
        let version = Version::parse(&pack.metadata.version)
            .map_err(|e| RegistryError::InvalidVersion(pack.metadata.version.clone(), e.to_string()))?;

        // 2. Validate version upgrade semantics
        if let Some(existing) = self.packs.get(&pack.metadata.name) {
            let existing_ver = Version::parse(&existing.metadata.version).unwrap();
            if version <= existing_ver {
                return Err(RegistryError::VersionDowngrade {
                    pack_name: pack.metadata.name.clone(),
                    new_version: pack.metadata.version.clone(),
                    existing_version: existing.metadata.version.clone(),
                });
            }

            // A. Prevent Winternitz OTS key reuse on successive upgrades
            if pack.signature.public_key == existing.signature.public_key {
                return Err(RegistryError::KeyReuseDetected(pack.metadata.name.clone()));
            }

            // B. Public key pinning via transition signature verification
            let transition_sig_blocks = pack.signature.transition_signature.as_ref()
                .ok_or_else(|| RegistryError::KeyPinningViolation(pack.metadata.name.clone()))?;

            let existing_pub_key = decode_blocks(&existing.signature.public_key, "Existing public key")?;
            let transition_sig = decode_blocks(transition_sig_blocks, "Transition signature")?;

            // The transition signature signs the hash of the new public key
            let mut hasher = Sha256::new();
            for block in &pack.signature.public_key {
                hasher.update(block.as_bytes());
            }
            let new_pub_key_hash: [u8; 32] = hasher.finalize().into();

            if !wots::verify(&existing_pub_key, &transition_sig, &new_pub_key_hash) {
                return Err(RegistryError::KeyPinningViolation(pack.metadata.name.clone()));
            }
        }

        // 3. Validate Shape Constraints Syntax
        for shape in &pack.shape_constraints.shapes {
            for constraint in &shape.property_constraints {
                if let Some(pattern) = &constraint.pattern {
                    if let Err(e) = regex::Regex::new(pattern) {
                        return Err(RegistryError::InvalidShapeConstraint(
                            shape.target_term.clone(),
                            format!("Invalid regex pattern '{}': {}", pattern, e),
                        ));
                    }
                }
            }
        }

        // 4. Validate Content Hash
        let computed = pack.compute_content_hash()
            .map_err(|e| RegistryError::JsonError(e.to_string()))?;
        if computed != pack.metadata.hash {
            return Err(RegistryError::HashMismatch {
                computed,
                specified: pack.metadata.hash.clone(),
            });
        }

        // 5. Validate PQC Signature
        pack.verify_signature()?;

        // 6. Store validated pack
        self.packs.insert(pack.metadata.name.clone(), pack);
        Ok(())
    }

    /// Retrieves a registered pack by name.
    pub fn get_pack(&self, name: &str) -> Option<&OntologyPack> {
        self.packs.get(name)
    }

    /// Validates a concrete JSON instance against the shape constraints of a registered ontology pack term.
    pub fn validate_instance(
        &self,
        pack_name: &str,
        target_term: &str,
        instance: &serde_json::Value,
    ) -> Result<(), RegistryError> {
        let pack = self.packs.get(pack_name)
            .ok_or_else(|| RegistryError::PackNotFound(pack_name.to_string()))?;

        let shape = pack.shape_constraints.shapes.iter()
            .find(|s| s.target_term == target_term)
            .ok_or_else(|| RegistryError::ShapeNotFound(target_term.to_string(), pack_name.to_string()))?;

        let obj = instance.as_object()
            .ok_or_else(|| RegistryError::ValidationFailed("Instance must be a JSON object".to_string()))?;

        for constraint in &shape.property_constraints {
            let val = obj.get(&constraint.property_name);
            match val {
                None => {
                    if constraint.required {
                        return Err(RegistryError::ValidationFailed(format!(
                            "Required property '{}' is missing",
                            constraint.property_name
                        )));
                    }
                }
                Some(value) => {
                    match constraint.expected_type {
                        DataType::String => {
                            let s = value.as_str().ok_or_else(|| {
                                RegistryError::ValidationFailed(format!(
                                    "Property '{}' must be a string",
                                    constraint.property_name
                                ))
                            })?;
                            if let Some(pattern) = &constraint.pattern {
                                let re = regex::Regex::new(pattern).unwrap();
                                if !re.is_match(s) {
                                    return Err(RegistryError::ValidationFailed(format!(
                                        "Property '{}' does not match pattern '{}'",
                                        constraint.property_name, pattern
                                    )));
                                }
                            }
                        }
                        DataType::Integer => {
                            let n = value.as_i64().ok_or_else(|| {
                                RegistryError::ValidationFailed(format!(
                                    "Property '{}' must be an integer",
                                    constraint.property_name
                                ))
                            })?;
                            if let Some(min) = constraint.min_value {
                                if n < min {
                                    return Err(RegistryError::ValidationFailed(format!(
                                        "Property '{}' value {} is less than min {}",
                                        constraint.property_name, n, min
                                    )));
                                }
                            }
                            if let Some(max) = constraint.max_value {
                                if n > max {
                                    return Err(RegistryError::ValidationFailed(format!(
                                        "Property '{}' value {} is greater than max {}",
                                        constraint.property_name, n, max
                                    )));
                                }
                            }
                        }
                        DataType::Float => {
                            if !value.is_f64() && !value.is_i64() {
                                return Err(RegistryError::ValidationFailed(format!(
                                    "Property '{}' must be a float",
                                    constraint.property_name
                                )));
                            }
                        }
                        DataType::Boolean => {
                            if !value.is_boolean() {
                                return Err(RegistryError::ValidationFailed(format!(
                                    "Property '{}' must be a boolean",
                                    constraint.property_name
                                )));
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Helper to sign an OntologyPack content hash and populate metadata/signature fields.
    fn sign_pack_helper(entropy: &[u8; 32], pack: &mut OntologyPack) {
        pack.metadata.hash = String::new();
        pack.signature = PqcSignature {
            public_key: vec![],
            signature_blocks: vec![],
        };

        // Compute actual content hash
        let content_hash = pack.compute_content_hash().unwrap();
        pack.metadata.hash = content_hash.clone();

        // Sign the hash using WOTS
        let hash_bytes = hex::decode(&content_hash).unwrap();
        let (priv_key, pub_key) = wots::generate_keypair(entropy);
        let signature_blocks = wots::sign(&priv_key, &hash_bytes);

        pack.signature = PqcSignature {
            public_key: pub_key.iter().map(hex::encode).collect(),
            signature_blocks: signature_blocks.iter().map(hex::encode).collect(),
        };
    }

    fn create_test_pack() -> OntologyPack {
        OntologyPack {
            metadata: PackMetadata {
                name: "test-ontology".to_string(),
                version: "1.0.0".to_string(),
                description: "A test ontology pack for Truex Registry".to_string(),
                hash: String::new(),
            },
            vocabulary: Vocabulary {
                namespace: "truex.test".to_string(),
                terms: vec![
                    VocabularyTerm {
                        name: "UserRecord".to_string(),
                        term_type: "concept".to_string(),
                        data_type: None,
                        description: "Standard User Record".to_string(),
                    },
                    VocabularyTerm {
                        name: "username".to_string(),
                        term_type: "property".to_string(),
                        data_type: Some(DataType::String),
                        description: "Name of the user".to_string(),
                    },
                    VocabularyTerm {
                        name: "age".to_string(),
                        term_type: "property".to_string(),
                        data_type: Some(DataType::Integer),
                        description: "Age in years".to_string(),
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
                        PropertyConstraint {
                            property_name: "age".to_string(),
                            required: false,
                            expected_type: DataType::Integer,
                            min_value: Some(0),
                            max_value: Some(150),
                            pattern: None,
                        },
                    ],
                }],
            },
            signature: PqcSignature {
                public_key: vec![],
                signature_blocks: vec![],
            },
        }
    }

    #[test]
    fn test_wots_correctness() {
        let entropy = [42u8; 32];
        let message = b"Hello, Truex execution trust!";
        let (priv_key, pub_key) = wots::generate_keypair(&entropy);

        let signature = wots::sign(&priv_key, message);
        assert!(wots::verify(&pub_key, &signature, message));

        // Tamper test
        let mut tampered_signature = signature.clone();
        tampered_signature[0][0] ^= 1;
        assert!(!wots::verify(&pub_key, &tampered_signature, message));

        // Different message test
        assert!(!wots::verify(&pub_key, &signature, b"Different message"));
    }

    #[test]
    fn test_pack_ingestion_success() {
        let entropy = [7u8; 32];
        let mut pack = create_test_pack();
        sign_pack_helper(&entropy, &mut pack);

        let pack_json = serde_json::to_string(&pack).unwrap();
        let mut service = RegistryService::new();
        let res = service.ingest(&pack_json);
        assert!(res.is_ok(), "Ingestion failed: {:?}", res);

        let retrieved = service.get_pack("test-ontology").unwrap();
        assert_eq!(retrieved.metadata.version, "1.0.0");
    }

    fn sign_pack_upgrade_helper(
        old_entropy: &[u8; 32],
        new_entropy: &[u8; 32],
        pack: &mut OntologyPack,
    ) {
        sign_pack_helper(new_entropy, pack);

        let (old_priv_key, _old_pub_key) = wots::generate_keypair(old_entropy);
        let mut hasher = Sha256::new();
        for block in &pack.signature.public_key {
            hasher.update(block.as_bytes());
        }
        let new_pub_key_hash: [u8; 32] = hasher.finalize().into();
        let transition_sig = wots::sign(&old_priv_key, &new_pub_key_hash);

        pack.signature.transition_signature = Some(
            transition_sig.iter().map(hex::encode).collect()
        );
    }

    #[test]
    fn test_pack_ingestion_invalid_version() {
        let entropy = [7u8; 32];
        let mut pack = create_test_pack();
        pack.metadata.version = "invalid-version-string".to_string();
        sign_pack_helper(&entropy, &mut pack);

        let pack_json = serde_json::to_string(&pack).unwrap();
        let mut service = RegistryService::new();
        let res = service.ingest(&pack_json);
        assert!(matches!(res, Err(RegistryError::InvalidVersion(..))));
    }

    #[test]
    fn test_pack_ingestion_version_downgrade() {
        let entropy1 = [7u8; 32];
        let entropy2 = [8u8; 32];
        let mut service = RegistryService::new();

        // Ingest version 1.0.0
        let mut pack1 = create_test_pack();
        pack1.metadata.version = "1.0.0".to_string();
        sign_pack_helper(&entropy1, &mut pack1);
        service.ingest(&serde_json::to_string(&pack1).unwrap()).unwrap();

        // Ingest version 1.0.0 again or lower
        let mut pack2 = create_test_pack();
        pack2.metadata.version = "0.9.9".to_string();
        sign_pack_helper(&entropy1, &mut pack2);
        let res = service.ingest(&serde_json::to_string(&pack2).unwrap());
        assert!(matches!(res, Err(RegistryError::VersionDowngrade { .. })));

        // Ingest version 1.0.1 (upgrade) -> should succeed with valid transition signature and key change
        let mut pack3 = create_test_pack();
        pack3.metadata.version = "1.0.1".to_string();
        sign_pack_upgrade_helper(&entropy1, &entropy2, &mut pack3);
        let res = service.ingest(&serde_json::to_string(&pack3).unwrap());
        assert!(res.is_ok(), "Upgrade ingestion failed: {:?}", res);
    }

    #[test]
    fn test_pack_upgrade_key_reuse_prevention() {
        let entropy = [7u8; 32];
        let mut service = RegistryService::new();

        // Ingest version 1.0.0
        let mut pack1 = create_test_pack();
        pack1.metadata.version = "1.0.0".to_string();
        sign_pack_helper(&entropy, &mut pack1);
        service.ingest(&serde_json::to_string(&pack1).unwrap()).unwrap();

        // Try to upgrade with the same key
        let mut pack2 = create_test_pack();
        pack2.metadata.version = "1.0.1".to_string();
        sign_pack_helper(&entropy, &mut pack2);
        let res = service.ingest(&serde_json::to_string(&pack2).unwrap());
        assert!(
            matches!(res, Err(RegistryError::KeyReuseDetected(..))),
            "Expected KeyReuseDetected, got: {:?}",
            res
        );
    }

    #[test]
    fn test_pack_upgrade_takeover_prevention() {
        let entropy1 = [7u8; 32];
        let entropy2 = [8u8; 32];
        let mut service = RegistryService::new();

        // Ingest version 1.0.0
        let mut pack1 = create_test_pack();
        pack1.metadata.version = "1.0.0".to_string();
        sign_pack_helper(&entropy1, &mut pack1);
        service.ingest(&serde_json::to_string(&pack1).unwrap()).unwrap();

        // Try to upgrade with a different key but NO transition signature (takeover attempt)
        let mut pack2 = create_test_pack();
        pack2.metadata.version = "1.0.1".to_string();
        sign_pack_helper(&entropy2, &mut pack2);
        let res = service.ingest(&serde_json::to_string(&pack2).unwrap());
        assert!(
            matches!(res, Err(RegistryError::KeyPinningViolation(..))),
            "Expected KeyPinningViolation, got: {:?}",
            res
        );

        // Try to upgrade with an invalid transition signature (tampered)
        let mut pack3 = create_test_pack();
        pack3.metadata.version = "1.0.1".to_string();
        sign_pack_upgrade_helper(&entropy1, &entropy2, &mut pack3);
        // Tamper the transition signature
        if let Some(ref mut sig) = pack3.signature.transition_signature {
            sig[0] = hex::encode([0u8; 32]);
        }
        let res = service.ingest(&serde_json::to_string(&pack3).unwrap());
        assert!(
            matches!(res, Err(RegistryError::KeyPinningViolation(..))),
            "Expected KeyPinningViolation (invalid signature), got: {:?}",
            res
        );
    }

    #[test]
    fn test_pack_ingestion_hash_mismatch() {
        let entropy = [7u8; 32];
        let mut pack = create_test_pack();
        sign_pack_helper(&entropy, &mut pack);

        // Tamper metadata description without re-signing or updating hash
        pack.metadata.description = "Tampered description".to_string();

        let pack_json = serde_json::to_string(&pack).unwrap();
        let mut service = RegistryService::new();
        let res = service.ingest(&pack_json);
        assert!(matches!(res, Err(RegistryError::HashMismatch { .. })));
    }

    #[test]
    fn test_pack_ingestion_signature_mismatch() {
        let entropy = [7u8; 32];
        let mut pack = create_test_pack();
        sign_pack_helper(&entropy, &mut pack);

        // Tamper signature block
        pack.signature.signature_blocks[0] = hex::encode([0u8; 32]);

        let pack_json = serde_json::to_string(&pack).unwrap();
        let mut service = RegistryService::new();
        let res = service.ingest(&pack_json);
        assert!(matches!(res, Err(RegistryError::SignatureVerificationFailed)));
    }

    #[test]
    fn test_shape_constraint_validation() {
        let entropy = [7u8; 32];
        let mut pack = create_test_pack();
        sign_pack_helper(&entropy, &mut pack);

        let mut service = RegistryService::new();
        service.ingest(&serde_json::to_string(&pack).unwrap()).unwrap();

        // Valid UserRecord
        let valid_instance = json!({
            "username": "alice_123",
            "age": 28
        });
        assert!(service.validate_instance("test-ontology", "UserRecord", &valid_instance).is_ok());

        // Missing required field
        let invalid_missing = json!({
            "age": 28
        });
        let res = service.validate_instance("test-ontology", "UserRecord", &invalid_missing);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("Required property 'username' is missing"));

        // Invalid regex pattern
        let invalid_regex = json!({
            "username": "a", // Too short
            "age": 28
        });
        let res = service.validate_instance("test-ontology", "UserRecord", &invalid_regex);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("does not match pattern"));

        // Age out of bounds
        let invalid_age = json!({
            "username": "alice_123",
            "age": 160
        });
        let res = service.validate_instance("test-ontology", "UserRecord", &invalid_age);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("is greater than max 150"));
    }

    #[test]
    fn test_registry_stress_benchmark() {
        use std::time::Instant;

        println!("\n=== Starting TruexRegistry Stress-Test & Benchmark ===");

        let num_packs = 100;
        let mut service = RegistryService::new();
        
        let mut keygen_durations = Vec::new();
        let mut signing_durations = Vec::new();
        let mut ingestion_durations = Vec::new();
        let mut validation_durations = Vec::new();

        let mut packs = Vec::with_capacity(num_packs);

        // Pre-generate packs & measure keygen / signing latency
        for i in 0..num_packs {
            let start_keygen = Instant::now();
            let mut seed = [0u8; 32];
            // Unique seed for each pack
            seed[0..8].copy_from_slice(&(i as u64).to_be_bytes());
            let (priv_key, pub_key) = wots::generate_keypair(&seed);
            keygen_durations.push(start_keygen.elapsed());

            let mut pack = create_test_pack();
            pack.metadata.name = format!("stress-ontology-{}", i);
            pack.vocabulary.namespace = format!("truex.stress.{}", i);

            // Compute hash & sign
            let start_signing = Instant::now();
            pack.metadata.hash = String::new();
            pack.signature = PqcSignature {
                public_key: pub_key.iter().map(hex::encode).collect(),
                signature_blocks: vec![],
                transition_signature: None,
            };

            let content_hash = pack.compute_content_hash().unwrap();
            pack.metadata.hash = content_hash.clone();
            let hash_bytes = hex::decode(&content_hash).unwrap();
            let signature_blocks = wots::sign(&priv_key, &hash_bytes);
            pack.signature.signature_blocks = signature_blocks.iter().map(hex::encode).collect();
            signing_durations.push(start_signing.elapsed());

            packs.push(pack);
        }

        // Measure Ingestion Throughput and Latency
        let start_all_ingest = Instant::now();
        for pack in &packs {
            let pack_json = serde_json::to_string(pack).unwrap();
            let start_ingest = Instant::now();
            service.ingest(&pack_json).unwrap();
            ingestion_durations.push(start_ingest.elapsed());
        }
        let total_ingest_duration = start_all_ingest.elapsed();

        // Measure Verification & Shape Constraint Validation Latency
        let valid_instance = json!({
            "username": "bob_99",
            "age": 42
        });

        for i in 0..num_packs {
            let pack_name = format!("stress-ontology-{}", i);
            let start_val = Instant::now();
            service.validate_instance(&pack_name, "UserRecord", &valid_instance).unwrap();
            validation_durations.push(start_val.elapsed());
        }

        // Print results to stdout
        let sum_keygen: f64 = keygen_durations.iter().map(|d| d.as_secs_f64()).sum();
        let avg_keygen = sum_keygen / (num_packs as f64) * 1000.0; // ms

        let sum_signing: f64 = signing_durations.iter().map(|d| d.as_secs_f64()).sum();
        let avg_signing = sum_signing / (num_packs as f64) * 1000.0; // ms

        let sum_ingestion: f64 = ingestion_durations.iter().map(|d| d.as_secs_f64()).sum();
        let avg_ingestion = sum_ingestion / (num_packs as f64) * 1000.0; // ms

        let sum_validation: f64 = validation_durations.iter().map(|d| d.as_secs_f64()).sum();
        let avg_validation = sum_validation / (num_packs as f64) * 1000.0; // ms

        let throughput = (num_packs as f64) / total_ingest_duration.as_secs_f64();

        println!("--------------------------------------------------");
        println!("Stress Test Configuration:");
        println!("  Number of Ontology Packs: {}", num_packs);
        println!("  PQC algorithm: Winternitz OTS (w=16, L=67)");
        println!("Performance Metrics:");
        println!("  Avg WOTS Key Pair Gen Latency:   {:.3} ms", avg_keygen);
        println!("  Avg WOTS Sign + Hash Latency:    {:.3} ms", avg_signing);
        println!("  Avg Ingestion + Verify Latency:  {:.3} ms", avg_ingestion);
        println!("  Avg Shape Validate Latency:      {:.3} ms", avg_validation);
        println!("  Total Registration Duration:     {:.3} ms", total_ingest_duration.as_secs_f64() * 1000.0);
        println!("  Ingestion Throughput:            {:.2} packs/sec", throughput);
        println!("--------------------------------------------------");

        assert_eq!(service.packs.len(), num_packs);
    }
}
