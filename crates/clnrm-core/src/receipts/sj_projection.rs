//! Semantic Jira projection: `TestReceipt` → `sj:Receipt`.
//!
//! Deterministic mapping per the spec at
//! `docs/sjira/v26.10.8/RECEIPT-PROJECTION.md`. Standing is computed, not
//! declared: ALIVE iff `validate()` passes, the hermeticity witness is fully
//! clean, and the chain link resolves (genesis `None` counts as resolved);
//! otherwise BLOCKED. Nothing rounds up: any failed input yields BLOCKED.
//!
//! TTL emission is deterministic: individuals are sorted by identity id, and
//! every emitted literal is escaped.

use crate::receipts::receipt::TestReceipt;
use serde::Serialize;

/// Projected `sj:Receipt` individual.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SjReceipt {
    /// `sj:identity` — content address, subject, base ontology hash.
    pub identity: SjIdentity,
    /// `sj:standing` — computed, not declared.
    pub standing: SjStanding,
    /// `sj:replay` — replay commands and durable storage location.
    pub replay: SjReplay,
    /// `sj:receipt` — chain hash: previous_receipt id when linked, own id at genesis.
    pub receipt: String,
}

/// `sj:identity` — identity block of a projected receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SjIdentity {
    /// Content address (`TestReceipt::id`).
    pub id: String,
    /// What was tested (`scenario_id`).
    pub subject: String,
    /// Environment ontology hash the run was manufactured against (`sigma_hash`).
    pub base_sha: String,
}

/// Standing vocabulary for projected receipts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SjStanding {
    /// Fully verified: validate() + clean witness + resolved chain.
    ALIVE,
    /// Any verification failure — validate error, dirty witness, or unresolved chain link.
    BLOCKED,
}

impl SjStanding {
    /// Turtle individual for the standing.
    pub fn as_str(&self) -> &'static str {
        match self {
            SjStanding::ALIVE => "sj:ALIVE",
            SjStanding::BLOCKED => "sj:BLOCKED",
        }
    }
}

/// `sj:replay` — replay path for a projected receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SjReplay {
    /// Commands that re-verify the receipt.
    pub commands: Vec<String>,
    /// Durable (out-of-subject) storage location for this receipt.
    pub durable_location: String,
}

/// Project a `TestReceipt` to its `sj:Receipt` form.
///
/// Deterministic: same receipt in, same `SjReceipt` out. The replay command
/// set is fixed (`weaver proof verify`); the durable location is the
/// content-addressed store key.
pub fn to_sj_receipt(r: &TestReceipt) -> SjReceipt {
    let standing = compute_standing(r);
    SjReceipt {
        identity: SjIdentity {
            id: r.id.as_str().to_string(),
            subject: r.scenario_id.0.clone(),
            base_sha: r.sigma_hash.as_str().to_string(),
        },
        standing,
        replay: SjReplay {
            commands: vec!["weaver proof verify".to_string()],
            durable_location: format!("clnrm://receipts/{}", r.id.as_str()),
        },
        // Chain hash: the previous link when present, else the receipt's own id
        // (genesis anchors the chain with its own content address).
        receipt: match &r.previous_receipt {
            Some(prev) => prev.as_str().to_string(),
            None => r.id.as_str().to_string(),
        },
    }
}

/// Compute `sj:standing` for a receipt per the spec's standing law.
///
/// ALIVE iff all of:
/// 1. `validate()` passes (declared id == computed content hash, hermeticity
///    constraint consistent with witness),
/// 2. the hermeticity witness is fully clean (network/filesystem/process
///    isolation, deterministic, empty violation vectors),
/// 3. the chain link resolves: `previous_receipt` is `None` (genesis) — a
///    receipt viewed in isolation cannot resolve a `Some` link, so an
///    un-resolved link is BLOCKED (orphan), never rounded up.
fn compute_standing(r: &TestReceipt) -> SjStanding {
    // (1) content-hash validation
    if r.validate().is_err() {
        return SjStanding::BLOCKED;
    }

    // (2) fully clean hermeticity witness
    let w = &r.hermeticity_witness;
    let witness_clean = w.network_isolated
        && w.filesystem_isolated
        && w.process_isolated
        && w.deterministic
        && w.external_connections.is_empty()
        && w.non_hermetic_paths.is_empty()
        && w.determinism_violations.is_empty();
    if !witness_clean {
        return SjStanding::BLOCKED;
    }

    // (3) resolved chain: only genesis (None) resolves in isolation.
    if r.previous_receipt.is_some() {
        return SjStanding::BLOCKED;
    }

    // Weaver validation is part of verification evidence: a proof that
    // explicitly failed blocks standing.
    if let Some(proof) = &r.weaver_proof {
        if !proof.validation_passed {
            return SjStanding::BLOCKED;
        }
    }

    SjStanding::ALIVE
}

/// Escape a string for emission inside a Turtle quoted literal.
fn escape_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

/// Emit deterministic Turtle for a set of projected receipts.
///
/// Individuals are sorted by identity id; each receipt is emitted as an
/// `sj:Receipt` individual with identity, standing, replay, and chain
/// properties. Output is byte-identical for the same input set.
pub fn to_sj_ttl(receipts: &[SjReceipt]) -> String {
    let mut sorted: Vec<&SjReceipt> = receipts.iter().collect();
    sorted.sort_by(|a, b| a.identity.id.cmp(&b.identity.id));

    let mut out = String::new();
    out.push_str("@prefix sj: <http://sjira.example/sj#> .\n\n");
    for r in sorted {
        out.push_str(&format!(
            "sj:receipt-{} a sj:Receipt ;\n    sj:identity \"{}\" ;\n    sj:subject \"{}\" ;\n    sj:baseSha \"{}\" ;\n    sj:standing {} ;\n    sj:replayCommand \"{}\" ;\n    sj:durableLocation \"{}\" ;\n    sj:receipt \"{}\" .\n\n",
            escape_literal(&r.identity.id),
            escape_literal(&r.identity.id),
            escape_literal(&r.identity.subject),
            escape_literal(&r.identity.base_sha),
            r.standing.as_str(),
            r.replay
                .commands
                .iter()
                .map(|c| escape_literal(c))
                .collect::<Vec<_>>()
                .join("\", \""),
            escape_literal(&r.replay.durable_location),
            escape_literal(&r.receipt),
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capabilities::scenario::{CapabilityId, ScenarioId};
    use crate::environment::sigma::ContentHash;
    use crate::receipts::receipt::{OtelGraphProof, WeaverProof};
    use std::collections::HashMap;

    /// Build a valid genesis ALIVE receipt (content hash consistent).
    fn alive_receipt(scenario: &str) -> TestReceipt {
        let mut r = TestReceipt {
            scenario_id: ScenarioId(scenario.to_string()),
            ..TestReceipt::default()
        };
        r.id = r.compute_id();
        r
    }

    fn weaver_passed() -> Option<WeaverProof> {
        Some(WeaverProof {
            registry_path: "/tmp/registry".to_string(),
            schema_version: "1.0.0".to_string(),
            validation_passed: true,
            warnings: Vec::new(),
            otel_graph: OtelGraphProof {
                trace_ids: vec!["trace-1".to_string()],
                span_counts: HashMap::new(),
                metrics: Vec::new(),
                log_entries: 0,
            },
            validated_at: "2026-10-08T00:00:00Z".to_string(),
        })
    }

    #[test]
    fn test_genesis_receipt_is_alive() {
        let mut r = alive_receipt("scenario-a");
        r.weaver_proof = weaver_passed();
        r.id = r.compute_id(); // recompute after adding proof
        let sj = to_sj_receipt(&r);
        assert_eq!(sj.standing, SjStanding::ALIVE);
        assert_eq!(sj.identity.subject, "scenario-a");
        assert_eq!(sj.replay.commands, vec!["weaver proof verify"]);
        assert_eq!(sj.receipt, sj.identity.id);
    }

    #[test]
    fn test_id_mismatch_is_blocked() {
        let mut r = alive_receipt("scenario-b");
        r.scenario_id = ScenarioId("mutated".to_string()); // id no longer matches body
        let sj = to_sj_receipt(&r);
        assert_eq!(sj.standing, SjStanding::BLOCKED);
    }

    #[test]
    fn test_dirty_witness_is_blocked() {
        let mut r = alive_receipt("scenario-c");
        r.hermeticity_witness
            .external_connections
            .push("db:5432".to_string());
        r.id = r.compute_id();
        let sj = to_sj_receipt(&r);
        assert_eq!(sj.standing, SjStanding::BLOCKED);
    }

    #[test]
    fn test_unresolved_chain_link_is_blocked() {
        let mut r = alive_receipt("scenario-d");
        r.previous_receipt = Some(ContentHash::from_string("missing-parent"));
        r.id = r.compute_id();
        let sj = to_sj_receipt(&r);
        assert_eq!(sj.standing, SjStanding::BLOCKED);
        // chain hash still projected faithfully
        assert_eq!(sj.receipt, "missing-parent");
    }

    #[test]
    fn test_failed_weaver_proof_is_blocked() {
        let mut r = alive_receipt("scenario-e");
        let mut proof = weaver_passed().unwrap();
        proof.validation_passed = false;
        r.weaver_proof = Some(proof);
        r.id = r.compute_id();
        assert_eq!(to_sj_receipt(&r).standing, SjStanding::BLOCKED);
    }

    #[test]
    fn test_ttl_is_deterministic_and_sorted() {
        let ra = to_sj_receipt(&alive_receipt("alpha"));
        let rb = to_sj_receipt(&alive_receipt("beta"));
        let forward = to_sj_ttl(&[ra.clone(), rb.clone()]);
        let reverse = to_sj_ttl(&[rb, ra]);
        assert_eq!(forward, reverse, "TTL must be order-independent");
        assert!(forward.contains("a sj:Receipt"));
        assert!(forward.contains("sj:standing sj:ALIVE"));
    }

    #[test]
    fn test_ttl_exact_match() {
        let r = to_sj_receipt(&alive_receipt("exact"));
        let expected = format!(
            "@prefix sj: <http://sjira.example/sj#> .\n\nsj:receipt-{} a sj:Receipt ;\n    sj:identity \"{}\" ;\n    sj:subject \"exact\" ;\n    sj:baseSha \"{}\" ;\n    sj:standing sj:ALIVE ;\n    sj:replayCommand \"weaver proof verify\" ;\n    sj:durableLocation \"clnrm://receipts/{}\" ;\n    sj:receipt \"{}\" .\n\n",
            r.identity.id, r.identity.id, r.identity.base_sha, r.identity.id, r.identity.id
        );
        assert_eq!(to_sj_ttl(&[r]), expected);
    }

    #[test]
    fn test_ttl_escapes_quotes_and_newlines() {
        let mut r = alive_receipt("has \"quote\" and\nnewline");
        r.id = r.compute_id();
        let sj = to_sj_receipt(&r);
        let ttl = to_sj_ttl(&[sj]);
        assert!(!ttl.contains("\"has \""));
        assert!(ttl.contains("\\\"quote\\\""));
        assert!(ttl.contains("\\n"));
    }

    #[test]
    fn test_capabilities_projected_into_identity_subject_scope() {
        let mut r = alive_receipt("scenario-f");
        r.capabilities = vec![CapabilityId("cap.postgres".to_string())];
        r.id = r.compute_id();
        let sj = to_sj_receipt(&r);
        assert_eq!(sj.standing, SjStanding::ALIVE);
        assert_eq!(sj.identity.subject, "scenario-f");
    }
}
