//! SHA-256 digest for reproducibility
//!
//! Generates cryptographic hashes of span data to ensure reproducible test results.

use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

// ---------------------------------------------------------------------------
// DigestReport – structured report with SHA-256 trace digest
// ---------------------------------------------------------------------------

/// A structured report capturing a SHA-256 digest of a set of span IDs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DigestReport {
    /// Name of the test this report belongs to
    pub test_name: String,
    /// Hex-encoded SHA-256 of sorted span_ids joined by newline
    pub trace_digest: String,
    /// Number of spans included in the digest
    pub span_count: usize,
    /// Number of events (currently mirrors span_count; callers may override)
    pub event_count: usize,
    /// ISO-8601 creation timestamp
    pub created_at: String,
    /// Schema version
    pub version: String,
}

impl DigestReport {
    /// Create a new `DigestReport`.
    ///
    /// The `trace_digest` is the hex-encoded SHA-256 of all `span_ids` sorted
    /// lexicographically and joined by `\n`.
    pub fn new(test_name: &str, span_ids: &[&str]) -> Self {
        let mut sorted: Vec<&str> = span_ids.to_vec();
        sorted.sort_unstable();
        let joined = sorted.join("\n");

        let mut hasher = Sha256::new();
        hasher.update(joined.as_bytes());
        let trace_digest = format!("{:x}", hasher.finalize());

        Self {
            test_name: test_name.to_string(),
            trace_digest,
            span_count: span_ids.len(),
            event_count: span_ids.len(),
            created_at: chrono::Utc::now().to_rfc3339(),
            version: "1".to_string(),
        }
    }

    /// Return `true` if both reports have the same `trace_digest`
    pub fn matches(&self, other: &DigestReport) -> bool {
        self.trace_digest == other.trace_digest
    }

    /// Serialize this report to a JSON string (panics on failure, use from_json to round-trip)
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("DigestReport is always serializable")
    }

    /// Deserialize a `DigestReport` from a JSON string
    pub fn from_json(s: &str) -> std::result::Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    /// Return a list of human-readable differences between `self` and `other`
    pub fn diff(&self, other: &DigestReport) -> Vec<String> {
        let mut diffs = Vec::new();

        if self.test_name != other.test_name {
            diffs.push(format!(
                "test_name: {:?} vs {:?}",
                self.test_name, other.test_name
            ));
        }
        if self.trace_digest != other.trace_digest {
            diffs.push(format!(
                "trace_digest: {} vs {}",
                self.trace_digest, other.trace_digest
            ));
        }
        if self.span_count != other.span_count {
            diffs.push(format!(
                "span_count: {} vs {}",
                self.span_count, other.span_count
            ));
        }
        if self.event_count != other.event_count {
            diffs.push(format!(
                "event_count: {} vs {}",
                self.event_count, other.event_count
            ));
        }
        if self.version != other.version {
            diffs.push(format!(
                "version: {:?} vs {:?}",
                self.version, other.version
            ));
        }

        diffs
    }
}

// ---------------------------------------------------------------------------
// DigestReporter – existing file-based SHA-256 helper (kept intact)
// ---------------------------------------------------------------------------

/// SHA-256 digest generator for reproducibility
pub struct DigestReporter;

impl DigestReporter {
    /// Write SHA-256 digest to file
    ///
    /// # Arguments
    /// * `path` - File path for digest output
    /// * `spans_json` - JSON string of spans to hash
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Errors
    /// Returns error if file write fails
    pub fn write(path: &Path, spans_json: &str) -> Result<()> {
        let digest = Self::compute_digest(spans_json);
        Self::write_file(path, &digest)
    }

    /// Compute SHA-256 digest of input string
    ///
    /// # Arguments
    /// * `spans_json` - JSON string to hash
    ///
    /// # Returns
    /// * Hexadecimal string representation of SHA-256 hash
    pub fn compute_digest(spans_json: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(spans_json.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Write digest to file with newline
    fn write_file(path: &Path, digest: &str) -> Result<()> {
        std::fs::write(path, format!("{}\n", digest))
            .map_err(|e| CleanroomError::report_error(format!("Failed to write digest: {}", e)))
    }
}
