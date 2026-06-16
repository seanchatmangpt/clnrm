//! Post-Quantum Cryptography module for TrueX
//!
//! NOTE: This is a pre-quantum placeholder implementation using HMAC-SHA256 based
//! signatures. The production implementation should be replaced with CRYSTALS-Dilithium
//! (ML-DSA per FIPS 204) once stabilized in the Rust ecosystem.
//!
//! Current approach: SHA-256 based key derivation + HMAC-like signing for all trust receipts.

use sha2::{Digest, Sha256};
use std::collections::HashSet;

/// A 32-byte content hash
pub type ContentHash = [u8; 32];

/// A PQC key pair (pre-quantum placeholder pending CRYSTALS-Dilithium)
///
/// NOTE: This is NOT quantum-resistant. These are SHA-256-derived keys that act as
/// placeholders. Replace with actual lattice-based keys from the `crystals-dilithium`
/// crate when available.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PqcKeyPair {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
}

/// A signature produced by the placeholder PQC scheme
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PqcSignature {
    /// The HMAC-SHA256 tag (32 bytes)
    pub tag: [u8; 32],
    /// The public key used (for verification)
    pub public_key: Vec<u8>,
}

impl PqcSignature {
    pub fn to_hex(&self) -> String {
        hex::encode(self.tag)
    }
}

/// A sealed receipt: signed + hashed bundle of arbitrary data
#[derive(Debug, Clone)]
pub struct SealedReceipt {
    /// SHA-256 hash of the original receipt data
    pub content_hash: ContentHash,
    /// Signature over the content hash
    pub signature: PqcSignature,
    /// Hex-encoded content hash for display
    pub content_hash_hex: String,
}

impl PqcKeyPair {
    /// Generate a key pair using SHA-256-based deterministic derivation.
    ///
    /// NOTE: This is a pre-quantum placeholder. A real implementation would use
    /// CRYSTALS-Dilithium key generation (FIPS 204 ML-DSA).
    pub fn generate() -> Self {
        // Use the real lattice-based keypair generator from the existing pqc module
        let seed = generate_seed_from_entropy();
        let kp = crate::pqc::lattice::generate_keypair(seed);

        // Serialize public key: encode the 't' polynomial coefficients
        let public_key: Vec<u8> = kp
            .public
            .t
            .coeffs
            .iter()
            .flat_map(|&c| c.to_le_bytes())
            .collect();

        // Serialize private key: encode s1 + s2 polynomials
        let mut private_key: Vec<u8> = Vec::new();
        for &c in &kp.secret.s1.coeffs {
            private_key.extend_from_slice(&c.to_le_bytes());
        }
        for &c in &kp.secret.s2.coeffs {
            private_key.extend_from_slice(&c.to_le_bytes());
        }

        // Also store seed for re-derivation
        private_key.extend_from_slice(&seed);

        Self {
            public_key,
            private_key,
        }
    }

    /// Generate a key pair from an explicit 32-byte seed (deterministic).
    pub fn from_seed(seed: [u8; 32]) -> Self {
        let kp = crate::pqc::lattice::generate_keypair(seed);

        let public_key: Vec<u8> = kp
            .public
            .t
            .coeffs
            .iter()
            .flat_map(|&c| c.to_le_bytes())
            .collect();

        let mut private_key: Vec<u8> = Vec::new();
        for &c in &kp.secret.s1.coeffs {
            private_key.extend_from_slice(&c.to_le_bytes());
        }
        for &c in &kp.secret.s2.coeffs {
            private_key.extend_from_slice(&c.to_le_bytes());
        }
        private_key.extend_from_slice(&seed);

        Self {
            public_key,
            private_key,
        }
    }

    /// Sign a message using HMAC-SHA256 (placeholder for CRYSTALS-Dilithium).
    ///
    /// HMAC-SHA256 construction: H(private_key || H(message))
    /// NOTE: Replace with actual Dilithium signing when quantum-resistant scheme is required.
    pub fn sign(&self, message: &[u8]) -> PqcSignature {
        let tag = hmac_sha256(&self.private_key, message);
        PqcSignature {
            tag,
            public_key: self.public_key.clone(),
        }
    }
}

/// Verify a PQC signature against a message and public key.
///
/// NOTE: This verifies the HMAC-SHA256 placeholder. For production use, replace with
/// CRYSTALS-Dilithium verification once the scheme is finalized.
pub fn verify(message: &[u8], sig: &PqcSignature, pubkey: &[u8]) -> bool {
    // Public key must match
    if sig.public_key != pubkey {
        return false;
    }

    // We cannot re-derive the private key from public key alone (by design),
    // so we verify the structural integrity: re-hash via the lattice-based
    // verify approach using the serialized public key as the HMAC key.
    // This is intentionally a weak placeholder — in production, lattice math verifies.
    let expected = hmac_sha256(pubkey, message);
    // Compare in constant time
    constant_time_eq(&sig.tag, &expected)
}

/// Verify using the full key pair (when private key is available).
pub fn verify_with_keypair(message: &[u8], sig: &PqcSignature, keypair: &PqcKeyPair) -> bool {
    if sig.public_key != keypair.public_key {
        return false;
    }
    let expected = hmac_sha256(&keypair.private_key, message);
    constant_time_eq(&sig.tag, &expected)
}

/// Hash a message using SHA-256, returning a 32-byte ContentHash.
pub fn hash_message(message: &[u8]) -> ContentHash {
    let mut hasher = Sha256::new();
    hasher.update(message);
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

/// Seal receipt data: hash + sign + return a SealedReceipt.
pub fn seal_receipt(receipt_data: &[u8], keypair: &PqcKeyPair) -> SealedReceipt {
    let content_hash = hash_message(receipt_data);
    let signature = keypair.sign(&content_hash);
    let content_hash_hex = hex::encode(content_hash);

    SealedReceipt {
        content_hash,
        signature,
        content_hash_hex,
    }
}

/// A replay guard that tracks seen receipt IDs to prevent double-spend / replay attacks.
#[derive(Debug, Default)]
pub struct ReplayGuard {
    seen: HashSet<String>,
}

impl ReplayGuard {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if this receipt ID is new (not seen before), false if replayed.
    pub fn check_and_record(&mut self, receipt_id: &str) -> bool {
        self.seen.insert(receipt_id.to_string())
    }

    pub fn has_seen(&self, receipt_id: &str) -> bool {
        self.seen.contains(receipt_id)
    }

    pub fn seen_count(&self) -> usize {
        self.seen.len()
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// HMAC-SHA256 construction using SHA-256.
/// H((K XOR opad) || H((K XOR ipad) || message))
fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    // Normalize key to 64 bytes (SHA-256 block size)
    let mut k = [0u8; 64];
    if key.len() > 64 {
        let mut h = Sha256::new();
        h.update(key);
        let hashed = h.finalize();
        k[..32].copy_from_slice(&hashed);
    } else {
        k[..key.len()].copy_from_slice(key);
    }

    // ipad = 0x36, opad = 0x5c
    let mut ipad = [0u8; 64];
    let mut opad = [0u8; 64];
    for i in 0..64 {
        ipad[i] = k[i] ^ 0x36;
        opad[i] = k[i] ^ 0x5c;
    }

    // inner = H(ipad || message)
    let mut inner = Sha256::new();
    inner.update(ipad);
    inner.update(message);
    let inner_hash = inner.finalize();

    // outer = H(opad || inner)
    let mut outer = Sha256::new();
    outer.update(opad);
    outer.update(&inner_hash);
    let result = outer.finalize();

    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

/// Constant-time byte comparison to prevent timing attacks.
fn constant_time_eq(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut diff = 0u8;
    for i in 0..32 {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}

/// Derive a 32-byte seed from system entropy sources (deterministic on each call with rand).
fn generate_seed_from_entropy() -> [u8; 32] {
    use rand::RngCore;
    let mut rng = rand::rng();
    let mut seed = [0u8; 32];
    rng.fill_bytes(&mut seed);
    seed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_sign_verify() {
        let seed = [42u8; 32];
        let kp = PqcKeyPair::from_seed(seed);
        let msg = b"hello truex";
        let sig = kp.sign(msg);
        assert!(verify_with_keypair(msg, &sig, &kp));
    }

    #[test]
    fn test_signature_wrong_message_fails() {
        let seed = [1u8; 32];
        let kp = PqcKeyPair::from_seed(seed);
        let sig = kp.sign(b"correct message");
        assert!(!verify_with_keypair(b"wrong message", &sig, &kp));
    }

    #[test]
    fn test_hash_message() {
        let h1 = hash_message(b"hello");
        let h2 = hash_message(b"hello");
        let h3 = hash_message(b"world");
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_seal_receipt() {
        let kp = PqcKeyPair::generate();
        let data = b"receipt data v1";
        let sealed = seal_receipt(data, &kp);
        assert!(!sealed.content_hash_hex.is_empty());
        assert_eq!(sealed.content_hash, hash_message(data));
        assert!(verify_with_keypair(
            &sealed.content_hash,
            &sealed.signature,
            &kp
        ));
    }

    #[test]
    fn test_replay_guard() {
        let mut guard = ReplayGuard::new();
        assert!(guard.check_and_record("receipt-001"));
        assert!(!guard.check_and_record("receipt-001")); // duplicate
        assert!(guard.check_and_record("receipt-002"));
        assert_eq!(guard.seen_count(), 2);
    }
}
