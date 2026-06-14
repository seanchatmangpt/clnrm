use super::reputation::{PeerId, ReputationEngine};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// New types
// ---------------------------------------------------------------------------

pub struct SlashingEvent {
    pub agent_id: String,
    pub penalty_points: f64,
    pub reason: String,
    pub timestamp_ms: u64,
}

pub struct SlashingRecord {
    pub agent_id: String,
    pub total_slashed: f64,
    pub events: Vec<SlashingEvent>,
    pub is_banned: bool,
}

pub struct ReputationSlasher {
    pub records: HashMap<String, SlashingRecord>,
    pub ban_threshold: f64,
}

impl ReputationSlasher {
    pub fn new(ban_threshold: f64) -> Self {
        Self {
            records: HashMap::new(),
            ban_threshold,
        }
    }

    /// Records a slashing event for agent_id and bans the agent if cumulative
    /// penalty >= ban_threshold. Returns the SlashingEvent that was recorded.
    pub fn slash(&mut self, agent_id: &str, penalty: f64, reason: &str) -> SlashingEvent {
        // Use a fixed timestamp of 0 when no wall-clock is available.
        let timestamp_ms: u64 = 0;

        let event = SlashingEvent {
            agent_id: agent_id.to_string(),
            penalty_points: penalty,
            reason: reason.to_string(),
            timestamp_ms,
        };

        let record = self
            .records
            .entry(agent_id.to_string())
            .or_insert_with(|| SlashingRecord {
                agent_id: agent_id.to_string(),
                total_slashed: 0.0,
                events: Vec::new(),
                is_banned: false,
            });

        record.total_slashed += penalty;
        record.events.push(SlashingEvent {
            agent_id: event.agent_id.clone(),
            penalty_points: event.penalty_points,
            reason: event.reason.clone(),
            timestamp_ms: event.timestamp_ms,
        });

        if record.total_slashed >= self.ban_threshold {
            record.is_banned = true;
        }

        event
    }

    pub fn is_banned(&self, agent_id: &str) -> bool {
        self.records
            .get(agent_id)
            .map(|r| r.is_banned)
            .unwrap_or(false)
    }

    pub fn total_slashed(&self, agent_id: &str) -> f64 {
        self.records
            .get(agent_id)
            .map(|r| r.total_slashed)
            .unwrap_or(0.0)
    }

    pub fn get_record(&self, agent_id: &str) -> Option<&SlashingRecord> {
        self.records.get(agent_id)
    }

    // ---------------------------------------------------------------------------
    // Backward-compatible function (kept from original implementation)
    // ---------------------------------------------------------------------------

    /// Applies an immediate penalty to the agent's local trust score in the
    /// provided ReputationEngine. Kept for backward compatibility.
    pub fn slash_agent(engine: &mut ReputationEngine, agent_id: &str, _penalty_points: f64) {
        // Apply immediate penalty to agent's local trust score.
        // In a real system, this intercepts DurableRefusals.
        engine.record_interaction(
            PeerId(agent_id.to_string()),
            PeerId("SYSTEM".to_string()),
            false,
        );
    }
}
