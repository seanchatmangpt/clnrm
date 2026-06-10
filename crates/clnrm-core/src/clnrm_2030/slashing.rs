use super::reputation::{PeerId, ReputationEngine};

pub struct ReputationSlasher;

impl ReputationSlasher {
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
