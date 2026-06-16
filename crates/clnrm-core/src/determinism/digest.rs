//! Deterministic digest generation for trace verification
//!
//! Provides a builder-style SHA-256 hasher and convenience helpers for
//! generating and verifying content digests in reproducible ways.

use sha2::{Digest, Sha256};

/// Generate SHA-256 digest from raw byte data (legacy helper retained for compatibility).
///
/// # Returns
/// * Lowercase hex-encoded SHA-256 digest string
pub fn generate_digest(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Verify that `data` matches an expected lowercase-hex SHA-256 digest.
///
/// # Returns
/// * `true` when the actual digest equals `expected_digest`
pub fn verify_digest(data: &[u8], expected_digest: &str) -> bool {
    generate_digest(data) == expected_digest
}

// ── Builder ───────────────────────────────────────────────────────────────────

/// Incremental SHA-256 digest builder.
///
/// Feed heterogeneous data into the hasher through a fluent API, then
/// call [`DigestBuilder::finalize`] or [`DigestBuilder::finalize_hex`] to
/// obtain the result.
///
/// # Example
/// ```no_run
/// use clnrm_core::determinism::digest::DigestBuilder;
///
/// let hex = DigestBuilder::new()
///     .update_str("hello")
///     .update_u64(42)
///     .finalize_hex();
/// assert_eq!(hex.len(), 64);
/// ```
pub struct DigestBuilder {
    hasher: Sha256,
}

impl DigestBuilder {
    /// Create a new, empty `DigestBuilder`.
    pub fn new() -> Self {
        Self {
            hasher: Sha256::new(),
        }
    }

    /// Feed a UTF-8 string into the hasher.
    pub fn update_str(&mut self, s: &str) -> &mut Self {
        self.hasher.update(s.as_bytes());
        self
    }

    /// Feed a `u64` value into the hasher (little-endian byte representation).
    pub fn update_u64(&mut self, v: u64) -> &mut Self {
        self.hasher.update(v.to_le_bytes());
        self
    }

    /// Feed raw bytes into the hasher.
    pub fn update_bytes(&mut self, b: &[u8]) -> &mut Self {
        self.hasher.update(b);
        self
    }

    /// Consume the builder and return the raw 32-byte digest.
    pub fn finalize(self) -> [u8; 32] {
        self.hasher.finalize().into()
    }

    /// Consume the builder and return a lowercase hex-encoded digest string.
    pub fn finalize_hex(self) -> String {
        format!("{:x}", self.hasher.finalize())
    }
}

impl Default for DigestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ── Convenience helpers ───────────────────────────────────────────────────────

/// Compute a deterministic hex digest over a sorted list of span IDs.
///
/// Spans are sorted before hashing so that the result is independent of
/// the order in which spans were observed.
///
/// # Example
/// ```no_run
/// use clnrm_core::determinism::digest::compute_trace_digest;
///
/// let digest = compute_trace_digest(&["span-b", "span-a"]);
/// assert_eq!(digest.len(), 64);
/// ```
pub fn compute_trace_digest(span_ids: &[impl AsRef<str>]) -> String {
    let mut sorted: Vec<&str> = span_ids.iter().map(|s| s.as_ref()).collect();
    sorted.sort_unstable();

    let mut builder = DigestBuilder::new();
    for id in sorted {
        builder.update_str(id);
    }
    builder.finalize_hex()
}
