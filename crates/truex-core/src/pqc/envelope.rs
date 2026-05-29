//! Post-Quantum Cryptographic (PQC) Envelopes and Sealed Packets.
//! Provides wrapping, cryptographic signing, and verification mechanisms 
//! for Counterparty Packets and Execution Receipts.

use serde::{Serialize, de::DeserializeOwned, Deserialize};
use anyhow::{anyhow, Result};

use super::hash::{custom_hash, Digest};
use super::lattice::{self, PrivateKey, PublicKey, Signature};

/// A counterparty transaction packet sent to the marketplace.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PartyPacket {
    pub sender: String,
    pub receiver: String,
    pub action: String,
    pub payload: Vec<u8>,
    pub nonce: u64,
    pub timestamp: u64,
}

/// A replayable execution receipt issued by the marketplace actuator.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Receipt {
    pub verdict: String, // e.g., "Admit" or "Refuse"
    pub failing_rules: Vec<String>,
    pub transport_attribution: String,
    pub actor_attribution: String,
    pub session_attribution: String,
    pub timestamp: u64,
    pub input_hash: Digest,
    pub output_hash: Digest,
}

/// A cryptographically secured PQC envelope wrapping a payload.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SealedEnvelope<T> {
    pub payload: T,
    pub signature: Signature,
    pub signer_key: PublicKey,
    pub payload_hash: Digest,
}

impl<T> SealedEnvelope<T>
where
    T: Serialize + DeserializeOwned,
{
    /// Seals a payload into a PQC envelope by generating a custom signature over its hash.
    pub fn seal(payload: T, sk: &PrivateKey, seed: [u8; 32]) -> Result<Self> {
        let serialized = serde_json::to_vec(&payload)
            .map_err(|e| anyhow!("Failed to serialize envelope payload: {}", e))?;
        
        let payload_hash = custom_hash(&serialized);
        
        let signature = lattice::sign(sk, &payload_hash, seed);
        
        Ok(SealedEnvelope {
            payload,
            signature,
            signer_key: sk.pub_key.clone(),
            payload_hash,
        })
    }

    /// Verifies the envelope's cryptographic signature and payload hash integrity.
    pub fn verify(&self) -> Result<bool> {
        let serialized = serde_json::to_vec(&self.payload)
            .map_err(|e| anyhow!("Failed to serialize envelope payload for verification: {}", e))?;
        
        let computed_hash = custom_hash(&serialized);
        if computed_hash != self.payload_hash {
            return Ok(false);
        }
        
        let ok = lattice::verify(&self.signer_key, &self.payload_hash, &self.signature);
        Ok(ok)
    }
}

/// Alias for a cryptographically sealed party packet.
pub type SealedPartyPacket = SealedEnvelope<PartyPacket>;

/// Alias for a cryptographically sealed receipt.
pub type SealedReceipt = SealedEnvelope<Receipt>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pqc::lattice::generate_keypair;

    #[test]
    fn test_seal_and_verify_packet() {
        let seed = [3u8; 32];
        let kp = generate_keypair(seed);
        
        let packet = PartyPacket {
            sender: "Alice".to_string(),
            receiver: "Bob".to_string(),
            action: "transfer".to_string(),
            payload: b"amount=100".to_vec(),
            nonce: 42,
            timestamp: 1716952800,
        };
        
        let sig_seed = [4u8; 32];
        let sealed = SealedPartyPacket::seal(packet.clone(), &kp.secret, sig_seed)
            .expect("Failed to seal packet");
        
        // Verify valid envelope
        let is_valid = sealed.verify().expect("Failed to verify envelope");
        assert!(is_valid, "Valid envelope signature verification failed");
        
        // Verify payload is identical
        assert_eq!(sealed.payload, packet);
        
        // Tamper payload data and verify failure
        let mut tampered = sealed.clone();
        tampered.payload.sender = "Eve".to_string();
        let is_valid_tampered = tampered.verify().expect("Failed to verify tampered envelope");
        assert!(!is_valid_tampered, "Tampered envelope should not verify successfully");
    }

    #[test]
    fn test_seal_and_verify_receipt() {
        let seed = [5u8; 32];
        let kp = generate_keypair(seed);
        
        let receipt = Receipt {
            verdict: "Admit".to_string(),
            failing_rules: vec![],
            transport_attribution: "grpc-v1".to_string(),
            actor_attribution: "operator-0".to_string(),
            session_attribution: "session-abc".to_string(),
            timestamp: 1716952850,
            input_hash: [1u8; 32],
            output_hash: [2u8; 32],
        };
        
        let sig_seed = [6u8; 32];
        let sealed = SealedReceipt::seal(receipt.clone(), &kp.secret, sig_seed)
            .expect("Failed to seal receipt");
        
        let is_valid = sealed.verify().expect("Failed to verify receipt");
        assert!(is_valid, "Valid receipt signature verification failed");
        
        // Test JSON serialization of the sealed envelope itself
        let json = serde_json::to_string(&sealed).expect("Failed to serialize envelope to JSON");
        let deserialized: SealedReceipt = serde_json::from_str(&json)
            .expect("Failed to deserialize envelope from JSON");
        
        assert_eq!(deserialized, sealed);
        assert!(deserialized.verify().unwrap(), "Deserialized envelope should verify successfully");
    }
}
