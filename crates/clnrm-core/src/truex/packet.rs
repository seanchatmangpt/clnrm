use crate::pqc::hash::{custom_hash, Digest};
use crate::pqc::lattice::{sign, verify, PrivateKey, PublicKey, Signature};

/// A cryptographically sealed consequence packet.
/// Uses Post-Quantum Cryptography for both hashing and signing.
#[derive(Clone, Debug)]
pub struct ConsequencePacket {
    /// The hash of the sealed object event, produced by `custom_hash`.
    pub object_event_hash: Digest,
    /// The lattice-based Post-Quantum signature.
    pub pqc_signature: Signature,
    /// The actual payload/event data.
    pub payload: Vec<u8>,
}

impl ConsequencePacket {
    /// Creates a new `ConsequencePacket`, cryptographically sealing the payload.
    /// This prevents depreciation or tampering.
    pub fn new(payload: &[u8], sk: &PrivateKey, seed: [u8; 32]) -> Self {
        // Seal the payload using our custom post-quantum hash
        let object_event_hash = custom_hash(payload);

        // Sign the hash (or the payload, but signing the hash is standard and fits)
        // using our custom post-quantum lattice signature scheme
        let pqc_signature = sign(sk, &object_event_hash, seed);

        Self {
            object_event_hash,
            pqc_signature,
            payload: payload.to_vec(),
        }
    }

    /// Verifies the cryptographic seal of the packet to ensure it has not been
    /// tampered with and is authentic.
    pub fn verify(&self, pk: &PublicKey) -> bool {
        // First, verify the payload matches the expected hash
        let computed_hash = custom_hash(&self.payload);

        // Secure comparison (though standard `!=` is constant-time enough for slices in some contexts,
        // we'll just use simple equality for the array)
        if computed_hash != self.object_event_hash {
            return false;
        }

        // Verify the lattice-based signature
        verify(pk, &self.object_event_hash, &self.pqc_signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pqc::lattice::generate_keypair;

    #[test]
    fn test_consequence_packet_seal_and_verify() {
        let payload = b"Test consequence event payload";
        let keypair_seed = [42u8; 32];
        let sig_seed = [99u8; 32];

        // Generate PQC keypair
        let kp = generate_keypair(keypair_seed);

        // Create and seal the packet
        let packet = ConsequencePacket::new(payload, &kp.secret, sig_seed);

        // Verify the packet is cryptographically sealed and authentic
        assert!(packet.verify(&kp.public), "Packet verification failed");

        // Verify tampering is detected
        let mut tampered_packet = packet.clone();
        tampered_packet.payload[0] ^= 1; // Alter payload
        assert!(
            !tampered_packet.verify(&kp.public),
            "Tampered payload should fail verification"
        );

        let mut tampered_hash_packet = packet.clone();
        tampered_hash_packet.object_event_hash[0] ^= 1; // Alter hash
        assert!(
            !tampered_hash_packet.verify(&kp.public),
            "Tampered hash should fail verification"
        );
    }
}
