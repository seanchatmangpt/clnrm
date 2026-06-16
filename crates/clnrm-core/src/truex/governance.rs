use crate::error::{CleanroomError, Result};
use crate::truex::admission_types::{Graph, PartyPacket};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, RwLock};
use tracing::{info, warn};
use uuid::Uuid;

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
        let laws = self.admitted_laws.read().expect("Lock poisoned"); // OK: RwLock poisoning not expected
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

// ── Governance voting registry ─────────────────────────────────────────────────

/// Opaque node identifier (proposer/voter).
pub type NodeId = String;

/// Opaque proposal identifier.
pub type ProposalId = String;

/// A governance proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    /// Short human-readable title.
    pub title: String,
    /// Detailed description of what this proposal changes.
    pub description: String,
    /// Minimum fraction of eligible voters required for a valid vote (0.0 – 1.0).
    /// E.g. 0.5 means at least 50 % of eligible voters must cast a vote.
    pub quorum_threshold: f64,
    /// Minimum fraction of YES votes (out of cast votes) to pass (0.0 – 1.0).
    /// E.g. 0.51 means simple majority.
    pub pass_threshold: f64,
    /// When the voting window closes.
    pub voting_deadline: DateTime<Utc>,
}

/// A single vote.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Vote {
    Yes,
    No,
    Abstain,
}

/// Status of a proposal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProposalStatus {
    Open,
    Passed,
    Rejected,
    Executed,
    Vetoed,
}

/// Internal proposal record with votes collected.
#[allow(dead_code)]
struct ProposalRecord {
    id: ProposalId,
    proposer: NodeId,
    proposal: Proposal,
    status: ProposalStatus,
    votes: HashMap<NodeId, Vote>,
    created_at: DateTime<Utc>,
    executed_at: Option<DateTime<Utc>>,
}

/// Tally result after counting votes.
#[derive(Debug, Clone)]
pub struct TallyResult {
    pub proposal_id: ProposalId,
    pub yes_votes: usize,
    pub no_votes: usize,
    pub abstain_votes: usize,
    pub total_cast: usize,
    pub eligible_voters: usize,
    pub quorum_met: bool,
    pub passed: bool,
    pub status: ProposalStatus,
}

/// The TrueX governance registry.
///
/// Manages proposals, voting, tally, execution, and emergency pause.
pub struct GovernanceRegistry {
    proposals: Mutex<HashMap<ProposalId, ProposalRecord>>,
    /// Set of all node IDs eligible to vote.
    eligible_voters: RwLock<HashSet<NodeId>>,
    /// When true, all governance actions are halted (emergency pause).
    paused: std::sync::atomic::AtomicBool,
}

impl GovernanceRegistry {
    /// Create a new registry with a set of eligible voters.
    pub fn new(eligible_voters: impl IntoIterator<Item = NodeId>) -> Self {
        Self {
            proposals: Mutex::new(HashMap::new()),
            eligible_voters: RwLock::new(eligible_voters.into_iter().collect()),
            paused: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Register a new eligible voter.
    pub fn add_voter(&self, voter: NodeId) -> Result<()> {
        self.assert_not_paused()?;
        self.eligible_voters
            .write()
            .map_err(|_| CleanroomError::internal_error("Lock poisoned"))?
            .insert(voter);
        Ok(())
    }

    /// Submit a new proposal. Returns the assigned `ProposalId`.
    pub fn propose(&self, proposer: NodeId, proposal: Proposal) -> Result<ProposalId> {
        self.assert_not_paused()?;
        if proposer.is_empty() {
            return Err(CleanroomError::validation_error(
                "Proposer ID cannot be empty",
            ));
        }
        if proposal.title.is_empty() {
            return Err(CleanroomError::validation_error(
                "Proposal title cannot be empty",
            ));
        }
        if proposal.quorum_threshold < 0.0 || proposal.quorum_threshold > 1.0 {
            return Err(CleanroomError::validation_error(
                "Quorum threshold must be between 0.0 and 1.0",
            ));
        }
        if proposal.pass_threshold < 0.0 || proposal.pass_threshold > 1.0 {
            return Err(CleanroomError::validation_error(
                "Pass threshold must be between 0.0 and 1.0",
            ));
        }

        let id = Uuid::new_v4().to_string();
        let record = ProposalRecord {
            id: id.clone(),
            proposer: proposer.clone(),
            proposal,
            status: ProposalStatus::Open,
            votes: HashMap::new(),
            created_at: Utc::now(),
            executed_at: None,
        };

        self.proposals
            .lock()
            .map_err(|_| CleanroomError::internal_error("Lock poisoned"))?
            .insert(id.clone(), record);

        info!(proposal_id = %id, proposer = %proposer, "New governance proposal submitted.");
        Ok(id)
    }

    /// Cast a vote on a proposal.
    ///
    /// Returns an error if:
    /// - The registry is paused.
    /// - The proposal does not exist.
    /// - The voter is not eligible.
    /// - The proposal is no longer open.
    /// - The voter has already voted (double-vote prevention).
    pub fn vote(&self, voter: NodeId, proposal_id: ProposalId, vote: Vote) -> Result<()> {
        self.assert_not_paused()?;

        let eligible = self
            .eligible_voters
            .read()
            .map_err(|_| CleanroomError::internal_error("Lock poisoned"))?
            .contains(&voter);

        if !eligible {
            return Err(CleanroomError::validation_error(format!(
                "Voter '{}' is not eligible",
                voter
            )));
        }

        let mut proposals = self
            .proposals
            .lock()
            .map_err(|_| CleanroomError::internal_error("Lock poisoned"))?;

        let record = proposals.get_mut(&proposal_id).ok_or_else(|| {
            CleanroomError::validation_error(format!("Proposal not found: {}", proposal_id))
        })?;

        if record.status != ProposalStatus::Open {
            return Err(CleanroomError::validation_error(format!(
                "Proposal {} is not open for voting (status: {:?})",
                proposal_id, record.status
            )));
        }

        if Utc::now() > record.proposal.voting_deadline {
            return Err(CleanroomError::validation_error(format!(
                "Voting window for proposal {} has closed",
                proposal_id
            )));
        }

        if record.votes.contains_key(&voter) {
            return Err(CleanroomError::validation_error(format!(
                "Voter '{}' has already voted on proposal {}",
                voter, proposal_id
            )));
        }

        record.votes.insert(voter.clone(), vote.clone());
        info!(voter = %voter, proposal_id = %proposal_id, ?vote, "Vote recorded.");
        Ok(())
    }

    /// Tally the votes for a proposal.
    ///
    /// Does not modify proposal status — call `execute` to finalize.
    pub fn tally(&self, proposal_id: &str) -> Result<TallyResult> {
        let proposals = self
            .proposals
            .lock()
            .map_err(|_| CleanroomError::internal_error("Lock poisoned"))?;

        let record = proposals.get(proposal_id).ok_or_else(|| {
            CleanroomError::validation_error(format!("Proposal not found: {}", proposal_id))
        })?;

        let eligible_voters = self
            .eligible_voters
            .read()
            .map_err(|_| CleanroomError::internal_error("Lock poisoned"))?
            .len();

        let yes = record.votes.values().filter(|v| **v == Vote::Yes).count();
        let no = record.votes.values().filter(|v| **v == Vote::No).count();
        let abstain = record
            .votes
            .values()
            .filter(|v| **v == Vote::Abstain)
            .count();
        let total_cast = yes + no + abstain;

        let quorum_met = if eligible_voters == 0 {
            false
        } else {
            (total_cast as f64 / eligible_voters as f64) >= record.proposal.quorum_threshold
        };

        let pass_fraction = if total_cast == 0 {
            0.0
        } else {
            yes as f64 / total_cast as f64
        };

        let passed = quorum_met && pass_fraction >= record.proposal.pass_threshold;

        Ok(TallyResult {
            proposal_id: proposal_id.to_string(),
            yes_votes: yes,
            no_votes: no,
            abstain_votes: abstain,
            total_cast,
            eligible_voters,
            quorum_met,
            passed,
            status: record.status.clone(),
        })
    }

    /// Execute a proposal that has passed quorum and threshold.
    ///
    /// Finalizes the proposal status to `Executed` (if passed) or `Rejected` (if not).
    /// Returns an error if the proposal cannot be executed (paused, not open, etc.).
    pub fn execute(&self, proposal_id: &str) -> Result<()> {
        self.assert_not_paused()?;

        let tally = self.tally(proposal_id)?;

        let mut proposals = self
            .proposals
            .lock()
            .map_err(|_| CleanroomError::internal_error("Lock poisoned"))?;

        let record = proposals.get_mut(proposal_id).ok_or_else(|| {
            CleanroomError::validation_error(format!("Proposal not found: {}", proposal_id))
        })?;

        if record.status != ProposalStatus::Open {
            return Err(CleanroomError::validation_error(format!(
                "Proposal {} is not open (status: {:?})",
                proposal_id, record.status
            )));
        }

        if tally.passed {
            record.status = ProposalStatus::Executed;
            record.executed_at = Some(Utc::now());
            info!(proposal_id = %proposal_id, "Governance proposal executed.");
        } else {
            record.status = ProposalStatus::Rejected;
            info!(proposal_id = %proposal_id, "Governance proposal rejected (quorum={}, passed={}).",
                tally.quorum_met, tally.passed);
        }

        Ok(())
    }

    /// Emergency pause: halts all governance actions immediately.
    ///
    /// Once paused, proposals cannot be submitted, voted on, or executed.
    /// Unpause by calling `resume`.
    pub fn emergency_pause(&self) {
        self.paused.store(true, std::sync::atomic::Ordering::SeqCst);
        warn!("Governance registry EMERGENCY PAUSED.");
    }

    /// Resume from emergency pause.
    pub fn resume(&self) {
        self.paused
            .store(false, std::sync::atomic::Ordering::SeqCst);
        info!("Governance registry resumed.");
    }

    /// Returns true if the registry is paused.
    pub fn is_paused(&self) -> bool {
        self.paused.load(std::sync::atomic::Ordering::SeqCst)
    }

    fn assert_not_paused(&self) -> Result<()> {
        if self.is_paused() {
            Err(CleanroomError::policy_violation(
                "Governance registry is emergency-paused; no actions allowed",
            ))
        } else {
            Ok(())
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

    fn make_proposal(deadline_secs: i64) -> Proposal {
        Proposal {
            title: "Test Proposal".to_string(),
            description: "A test governance proposal".to_string(),
            quorum_threshold: 0.5,
            pass_threshold: 0.5,
            voting_deadline: DateTime::from_timestamp(
                chrono::Utc::now().timestamp() + deadline_secs,
                0,
            )
            .unwrap(),
        }
    }

    #[test]
    fn test_propose_and_vote_and_execute() {
        let voters = vec!["alice".to_string(), "bob".to_string(), "carol".to_string()];
        let registry = GovernanceRegistry::new(voters);

        let pid = registry
            .propose("alice".to_string(), make_proposal(3600))
            .unwrap();

        registry
            .vote("alice".to_string(), pid.clone(), Vote::Yes)
            .unwrap();
        registry
            .vote("bob".to_string(), pid.clone(), Vote::Yes)
            .unwrap();
        registry
            .vote("carol".to_string(), pid.clone(), Vote::No)
            .unwrap();

        let tally = registry.tally(&pid).unwrap();
        assert!(tally.quorum_met);
        assert!(tally.passed); // 2/3 yes > 0.5 threshold
        assert_eq!(tally.yes_votes, 2);
        assert_eq!(tally.no_votes, 1);

        registry.execute(&pid).unwrap();
        let tally2 = registry.tally(&pid).unwrap();
        assert_eq!(tally2.status, ProposalStatus::Executed);
    }

    #[test]
    fn test_double_vote_rejected() {
        let registry = GovernanceRegistry::new(vec!["alice".to_string()]);
        let pid = registry
            .propose("alice".to_string(), make_proposal(3600))
            .unwrap();
        registry
            .vote("alice".to_string(), pid.clone(), Vote::Yes)
            .unwrap();
        let result = registry.vote("alice".to_string(), pid.clone(), Vote::No);
        assert!(result.is_err());
    }

    #[test]
    fn test_emergency_pause() {
        let registry = GovernanceRegistry::new(vec!["alice".to_string()]);
        registry.emergency_pause();
        assert!(registry.is_paused());
        let result = registry.propose("alice".to_string(), make_proposal(3600));
        assert!(result.is_err()); // paused
        registry.resume();
        assert!(!registry.is_paused());
        let result = registry.propose("alice".to_string(), make_proposal(3600));
        assert!(result.is_ok()); // resumed
    }

    #[test]
    fn test_no_quorum_rejects() {
        let registry = GovernanceRegistry::new(vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ]);
        let pid = registry
            .propose("a".to_string(), make_proposal(3600))
            .unwrap();
        // Only 1/4 votes = 25% < 50% quorum
        registry
            .vote("a".to_string(), pid.clone(), Vote::Yes)
            .unwrap();

        let tally = registry.tally(&pid).unwrap();
        assert!(!tally.quorum_met);
        assert!(!tally.passed);

        registry.execute(&pid).unwrap();
        let tally2 = registry.tally(&pid).unwrap();
        assert_eq!(tally2.status, ProposalStatus::Rejected);
    }
}
