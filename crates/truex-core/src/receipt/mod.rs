//! Receipt Emission Engine
//!
//! Compiles and emits structurally complete, PQC-sealed receipts for the
//! Truex Execution Trust Marketplace.
//!
//! Structurally complete receipts include:
//! - Input and Output hashes (attesting to execution states)
//! - Closure hash (attesting to ontology/consequence closure)
//! - Session attribution (session ID, execution time, duration)
//! - Actor attribution (actor ID, role, optional key/identity)
//! - Transport attribution (protocol, endpoint, client metadata)
//! - Cryptographic PQC Seal (Lattice-based Fiat-Shamir with Aborts signature)

use std::collections::BTreeMap;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use anyhow::{anyhow, Result};

// Reference cleanroom-core PQC primitives
use clnrm_core::pqc::hash::custom_hash;
use clnrm_core::pqc::lattice::{
    self, Poly, PublicKey as LPublicKey, PrivateKey as LPrivateKey, Signature as LSignature, N, Q, GAMMA, TAU
};

/// Post-Quantum Cryptographic (PQC) Seal wrapping the Lattice signature and public key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PqcSeal {
    /// Coefficients of the public key component 'a' (size N)
    pub pub_key_a: Vec<i64>,
    /// Coefficients of the public key component 't' (size N)
    pub pub_key_t: Vec<i64>,
    /// Coefficients of the signature component 'z' (size N)
    pub sig_z: Vec<i64>,
    /// Coefficients of the signature component 'c' (size N)
    pub sig_c: Vec<i64>,
    /// ISO 8601 UTC timestamp of when the seal was applied
    pub sealed_at: String,
}

/// Metadata and timing for the execution session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionAttribution {
    /// Unique identifier for the execution session
    pub session_id: String,
    /// ISO 8601 UTC timestamp when the session began
    pub timestamp: String,
    /// Total execution duration in milliseconds
    pub duration_ms: u64,
}

/// Actor/Counterparty details.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActorAttribution {
    /// Unique identifier for the initiating actor/counterparty
    pub actor_id: String,
    /// Role of the actor in the marketplace transaction
    pub role: String,
    /// Optional public key identifier or public identity string of the actor
    pub public_key: Option<String>,
}

/// Network and transport telemetry attribution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransportAttribution {
    /// Transport protocol (e.g., "gRPC", "QUIC", "A2A")
    pub protocol: String,
    /// Network endpoint address (e.g., "10.0.0.5:50051")
    pub endpoint: String,
    /// Version of the marketplace client or proxy
    pub client_version: String,
    /// Additional transport metadata (e.g., bytes sent/received, TLS ciphers)
    pub metadata: BTreeMap<String, String>,
}

/// Execution payloads and deterministic closure hashes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReceiptPayload {
    /// Hex-encoded representation of the input state graph hash
    pub input_hash: String,
    /// Hex-encoded representation of the output state graph hash
    pub output_hash: String,
    /// Hex-encoded representation of the deterministic consequence closure hash
    pub closure_hash: String,
}

/// The final, structurally complete execution receipt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Receipt {
    /// Session attribution details
    pub session: SessionAttribution,
    /// Actor attribution details
    pub actor: ActorAttribution,
    /// Transport/networking attribution details
    pub transport: TransportAttribution,
    /// Input/Output and closure payload hashes
    pub payload: ReceiptPayload,
    /// Cryptographic PQC seal attesting to all fields above (None if unsealed)
    pub seal: Option<PqcSeal>,
}

impl Receipt {
    /// Helper to validate that a string is a canonical, lowercase, 32-byte hex hash.
    fn validate_hex_hash(hash: &str) -> Result<()> {
        if hash.len() != 64 {
            return Err(anyhow!("Hash must be exactly 64 hex characters, got length {}", hash.len()));
        }
        for c in hash.chars() {
            if !c.is_ascii_digit() && !('a'..='f').contains(&c) {
                return Err(anyhow!("Hash must be lowercase hexadecimal characters only"));
            }
        }
        Ok(())
    }

    /// Validates all receipt headers and attribution fields to prevent malleability.
    pub fn validate_headers(&self) -> Result<()> {
        // Validate session attribution
        if self.session.session_id.trim().is_empty() {
            return Err(anyhow!("session_id cannot be empty or only whitespace"));
        }
        chrono::DateTime::parse_from_rfc3339(&self.session.timestamp)
            .map_err(|e| anyhow!("session timestamp must be a valid RFC 3339 string: {}", e))?;

        // Validate actor attribution
        if self.actor.actor_id.trim().is_empty() {
            return Err(anyhow!("actor_id cannot be empty or only whitespace"));
        }
        if self.actor.role.trim().is_empty() {
            return Err(anyhow!("actor role cannot be empty or only whitespace"));
        }
        if let Some(ref pk) = self.actor.public_key {
            if pk.trim().is_empty() {
                return Err(anyhow!("actor public key cannot be empty string if present"));
            }
        }

        // Validate transport attribution
        if self.transport.protocol.trim().is_empty() {
            return Err(anyhow!("transport protocol cannot be empty or only whitespace"));
        }
        if self.transport.endpoint.trim().is_empty() {
            return Err(anyhow!("transport endpoint cannot be empty or only whitespace"));
        }
        if self.transport.client_version.trim().is_empty() {
            return Err(anyhow!("transport client_version cannot be empty or only whitespace"));
        }

        // Validate payload hashes
        Self::validate_hex_hash(&self.payload.input_hash)?;
        Self::validate_hex_hash(&self.payload.output_hash)?;
        Self::validate_hex_hash(&self.payload.closure_hash)?;

        Ok(())
    }

    /// Computes the cryptographic hash of the receipt fields (excluding the seal itself).
    /// Uses the custom sponge-construction hash from `clnrm-core`.
    pub fn compute_payload_hash(&self) -> Result<[u8; 32]> {
        // Define a representation containing only the fields that are signed
        #[derive(Serialize)]
        struct HashableReceipt<'a> {
            session: &'a SessionAttribution,
            actor: &'a ActorAttribution,
            transport: &'a TransportAttribution,
            payload: &'a ReceiptPayload,
        }

        let hashable = HashableReceipt {
            session: &self.session,
            actor: &self.actor,
            transport: &self.transport,
            payload: &self.payload,
        };

        let serialized = serde_json::to_vec(&hashable)
            .map_err(|e| anyhow!("Failed to serialize receipt for hashing: {}", e))?;

        Ok(custom_hash(&serialized))
    }

    /// Verifies the PQC signature on the receipt using the embedded public key in the seal.
    ///
    /// Returns `Ok(true)` if the signature is valid, `Ok(false)` if it is invalid,
    /// or an `Err` if the seal is missing or malformed.
    pub fn verify_seal(&self) -> Result<bool> {
        // 1. Strict validation of receipt headers/fields
        self.validate_headers()?;

        let seal = self.seal.as_ref()
            .ok_or_else(|| anyhow!("Verification failed: Receipt is not sealed"))?;

        // 2. Validate seal timestamp and chronological ordering
        let parsed_session_time = chrono::DateTime::parse_from_rfc3339(&self.session.timestamp)
            .map_err(|e| anyhow!("Invalid session timestamp format: {}", e))?;
        let parsed_sealed_time = chrono::DateTime::parse_from_rfc3339(&seal.sealed_at)
            .map_err(|e| anyhow!("seal timestamp must be a valid RFC 3339 string: {}", e))?;
        if parsed_sealed_time < parsed_session_time {
            return Err(anyhow!("seal timestamp cannot be before session timestamp"));
        }

        // 3. Reconstruct and validate lattice public key coefficients
        let min_coeff = -Q / 2; // -4194304
        let max_coeff = Q / 2;  // 4194304

        let pub_key_a: [i64; N] = seal.pub_key_a.clone().try_into()
            .map_err(|_| anyhow!("Invalid pub_key_a length"))?;
        for (i, &val) in pub_key_a.iter().enumerate() {
            if val < min_coeff || val > max_coeff {
                return Err(anyhow!("pub_key_a coefficient at index {} is out of canonical range [{}, {}]", i, min_coeff, max_coeff));
            }
        }

        let pub_key_t: [i64; N] = seal.pub_key_t.clone().try_into()
            .map_err(|_| anyhow!("Invalid pub_key_t length"))?;
        for (i, &val) in pub_key_t.iter().enumerate() {
            if val < min_coeff || val > max_coeff {
                return Err(anyhow!("pub_key_t coefficient at index {} is out of canonical range [{}, {}]", i, min_coeff, max_coeff));
            }
        }

        let pk = LPublicKey {
            a: Poly { coeffs: pub_key_a },
            t: Poly { coeffs: pub_key_t },
        };

        // 4. Reconstruct and validate lattice signature coefficients
        let sig_z: [i64; N] = seal.sig_z.clone().try_into()
            .map_err(|_| anyhow!("Invalid sig_z length"))?;
        for (i, &val) in sig_z.iter().enumerate() {
            if val < min_coeff || val > max_coeff {
                return Err(anyhow!("sig_z coefficient at index {} is out of canonical range [{}, {}]", i, min_coeff, max_coeff));
            }
        }

        let sig_c: [i64; N] = seal.sig_c.clone().try_into()
            .map_err(|_| anyhow!("Invalid sig_c length"))?;
        for (i, &val) in sig_c.iter().enumerate() {
            if val < min_coeff || val > max_coeff {
                return Err(anyhow!("sig_c coefficient at index {} is out of canonical range [{}, {}]", i, min_coeff, max_coeff));
            }
        }

        let sig = LSignature {
            z: Poly { coeffs: sig_z },
            c: Poly { coeffs: sig_c },
        };

        // Strict verification of signature bounds
        if sig.z.norm_infty() > GAMMA - TAU as i64 {
            return Err(anyhow!("Signature component z violates norm bounds"));
        }
        if !sig.c.is_valid_challenge() {
            return Err(anyhow!("Signature component c is not a valid ternary challenge"));
        }

        // 5. Recompute the payload hash
        let msg_hash = self.compute_payload_hash()?;

        // Perform verification
        let is_valid = lattice::verify(&pk, &msg_hash, &sig);
        Ok(is_valid)
    }
}

/// Engine responsible for compiling, hashing, and sealing receipts.
pub struct ReceiptEmissionEngine;

impl ReceiptEmissionEngine {
    /// Compiles attributes and emits a PQC-sealed receipt.
    ///
    /// # Arguments
    ///
    /// * `session` - Session attribution details
    /// * `actor` - Actor attribution details
    /// * `transport` - Transport/network attribution details
    /// * `payload` - Hashes of input/output states and consequence closure
    /// * `sk` - Private key for signing the receipt
    /// * `signing_seed` - Deterministic signing seed for PQC Fiat-Shamir with Aborts
    pub fn emit(
        session: SessionAttribution,
        actor: ActorAttribution,
        transport: TransportAttribution,
        payload: ReceiptPayload,
        sk: &LPrivateKey,
        signing_seed: [u8; 32],
    ) -> Result<Receipt> {
        let mut receipt = Receipt {
            session,
            actor,
            transport,
            payload,
            seal: None,
        };

        // Hash the receipt content
        let hash = receipt.compute_payload_hash()?;

        // Sign the hash using lattice signature scheme
        let sig = lattice::sign(sk, &hash, signing_seed);

        // Capture current time for the seal metadata
        let sealed_at = Utc::now().to_rfc3339();

        // Construct the PQC Seal
        let seal = PqcSeal {
            pub_key_a: sk.pub_key.a.coeffs.to_vec(),
            pub_key_t: sk.pub_key.t.coeffs.to_vec(),
            sig_z: sig.z.coeffs.to_vec(),
            sig_c: sig.c.coeffs.to_vec(),
            sealed_at,
        };

        receipt.seal = Some(seal);
        Ok(receipt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clnrm_core::pqc::lattice::generate_keypair;

    fn make_test_attributions() -> (SessionAttribution, ActorAttribution, TransportAttribution, ReceiptPayload) {
        let session = SessionAttribution {
            session_id: "sess_8f3c832b-9fca-4d33-bb93-cb79d8c0b5f1".to_string(),
            timestamp: "2026-05-29T02:00:00Z".to_string(),
            duration_ms: 42,
        };

        let actor = ActorAttribution {
            actor_id: "actor_counterparty_alpha".to_string(),
            role: "Proposer".to_string(),
            public_key: Some("0x1234abcd".to_string()),
        };

        let mut transport_metadata = BTreeMap::new();
        transport_metadata.insert("tls_cipher".to_string(), "TLS_AES_256_GCM_SHA384".to_string());
        transport_metadata.insert("rtt_us".to_string(), "150".to_string());

        let transport = TransportAttribution {
            protocol: "HTTP/3".to_string(),
            endpoint: "192.168.1.100:443".to_string(),
            client_version: "truex-client-v1.0.0".to_string(),
            metadata: transport_metadata,
        };

        let payload = ReceiptPayload {
            input_hash: "a4f2c9e782bc4e7d95315ff186b89c62582910fae134267891823abf1064506c".to_string(),
            output_hash: "28e9cfb1837a4c90abefd019ab7612f0ea7c718a209e761cf9a74ba10abefc91".to_string(),
            closure_hash: "b5c1ab2f3479ad01ffeb9c882a17cf12a0e98c76abf10a82efcd09641abefd10".to_string(),
        };

        (session, actor, transport, payload)
    }

    #[test]
    fn test_receipt_emission_and_verification() -> Result<()> {
        let (session, actor, transport, payload) = make_test_attributions();

        // Generate PQC keys
        let keypair_seed = [42u8; 32];
        let kp = generate_keypair(keypair_seed);

        // Emit receipt
        let signing_seed = [99u8; 32];
        let receipt = ReceiptEmissionEngine::emit(
            session,
            actor,
            transport,
            payload,
            &kp.secret,
            signing_seed,
        )?;

        // Verify the seal
        let is_valid = receipt.verify_seal()?;
        assert!(is_valid, "Valid PQC seal failed verification");

        Ok(())
    }

    #[test]
    fn test_unsealed_receipt_verification_fails() {
        let (session, actor, transport, payload) = make_test_attributions();

        let unsealed_receipt = Receipt {
            session,
            actor,
            transport,
            payload,
            seal: None,
        };

        assert!(unsealed_receipt.verify_seal().is_err());
    }

    #[test]
    fn test_tempered_receipt_verification_fails() -> Result<()> {
        let (session, actor, transport, payload) = make_test_attributions();

        let keypair_seed = [42u8; 32];
        let kp = generate_keypair(keypair_seed);
        let signing_seed = [99u8; 32];

        let mut receipt = ReceiptEmissionEngine::emit(
            session,
            actor,
            transport,
            payload,
            &kp.secret,
            signing_seed,
        )?;

        // Tamper with output hash
        receipt.payload.output_hash = "0000000000000000000000000000000000000000000000000000000000000000".to_string();

        // Verification should fail (return false)
        let is_valid = receipt.verify_seal()?;
        assert!(!is_valid, "Tampered receipt should not pass verification");

        Ok(())
    }

    #[test]
    fn test_tempered_seal_verification_fails() -> Result<()> {
        let (session, actor, transport, payload) = make_test_attributions();

        let keypair_seed = [42u8; 32];
        let kp = generate_keypair(keypair_seed);
        let signing_seed = [99u8; 32];

        let mut receipt = ReceiptEmissionEngine::emit(
            session,
            actor,
            transport,
            payload,
            &kp.secret,
            signing_seed,
        )?;

        // Tamper with signature coefficients in seal
        if let Some(ref mut seal) = receipt.seal {
            seal.sig_z[0] = seal.sig_z[0].wrapping_add(1);
        }

        // Verification should fail (return false)
        let is_valid = receipt.verify_seal()?;
        assert!(!is_valid, "Tampered signature should not pass verification");

        Ok(())
    }

    #[test]
    fn test_invalid_header_fields_fail() -> Result<()> {
        let (session, actor, transport, payload) = make_test_attributions();
        let kp = generate_keypair([42u8; 32]);
        let receipt = ReceiptEmissionEngine::emit(session, actor, transport, payload, &kp.secret, [99u8; 32])?;

        // Empty session_id
        let mut r = receipt.clone();
        r.session.session_id = "   ".to_string();
        assert!(r.verify_seal().is_err());

        // Malformed session timestamp
        let mut r = receipt.clone();
        r.session.timestamp = "not-a-timestamp".to_string();
        assert!(r.verify_seal().is_err());

        // Empty actor_id
        let mut r = receipt.clone();
        r.actor.actor_id = "".to_string();
        assert!(r.verify_seal().is_err());

        // Empty actor role
        let mut r = receipt.clone();
        r.actor.role = "  ".to_string();
        assert!(r.verify_seal().is_err());

        // Empty transport protocol
        let mut r = receipt.clone();
        r.transport.protocol = "".to_string();
        assert!(r.verify_seal().is_err());

        // Malformed payload input hash
        let mut r = receipt.clone();
        r.payload.input_hash = "NOTHEX".to_string();
        assert!(r.verify_seal().is_err());

        // Capital hex payload input hash
        let mut r = receipt.clone();
        r.payload.input_hash = r.payload.input_hash.to_uppercase();
        assert!(r.verify_seal().is_err());

        Ok(())
    }

    #[test]
    fn test_out_of_bounds_seal_coefficients_fail() -> Result<()> {
        let (session, actor, transport, payload) = make_test_attributions();
        let kp = generate_keypair([42u8; 32]);
        let receipt = ReceiptEmissionEngine::emit(session, actor, transport, payload, &kp.secret, [99u8; 32])?;

        // Out of bounds coefficient in pub_key_a
        let mut r = receipt.clone();
        if let Some(ref mut seal) = r.seal {
            seal.pub_key_a[0] = 99999999;
        }
        assert!(r.verify_seal().is_err());

        // Out of bounds coefficient in sig_z
        let mut r = receipt.clone();
        if let Some(ref mut seal) = r.seal {
            seal.sig_z[5] = -88888888;
        }
        assert!(r.verify_seal().is_err());

        // Invalid challenge structure in sig_c (weight is not TAU)
        let mut r = receipt.clone();
        if let Some(ref mut seal) = r.seal {
            seal.sig_c.fill(0);
        }
        assert!(r.verify_seal().is_err());

        // Seal timestamp before session timestamp
        let mut r = receipt.clone();
        if let Some(ref mut seal) = r.seal {
            seal.sealed_at = "2026-05-29T01:59:59Z".to_string();
        }
        assert!(r.verify_seal().is_err());

        Ok(())
    }
}
