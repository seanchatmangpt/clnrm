use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum ReconciliationError {
    #[error("State conflict: {0}")]
    StateConflict(String),
    #[error("Drift detected: {0}")]
    DriftDetected(String),
    #[error("Reconciliation failed: {0}")]
    ReconciliationFailed(String),
    #[error("Invalid state: {0}")]
    InvalidState(String),
}

pub type Result<T> = std::result::Result<T, ReconciliationError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DesiredState {
    Running,
    Stopped,
    Updating,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActualState {
    Running,
    Stopped,
    Unknown,
    Degraded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriftType {
    None,
    VersionMismatch,
    ReplicaMismatch,
    HealthMismatch,
    ConfigurationMismatch,
    StateConflict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceState {
    pub service_id: String,
    pub service_name: String,
    pub desired_state: DesiredState,
    pub actual_state: ActualState,
    pub desired_version: String,
    pub actual_version: String,
    pub desired_replicas: u32,
    pub actual_replicas: u32,
    pub healthy_replicas: u32,
    pub drift_type: DriftType,
    pub last_observed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationAction {
    pub id: String,
    pub service_id: String,
    pub action_type: ActionType,
    pub reason: String,
    pub timestamp: u64,
    pub completed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    Start,
    Stop,
    Restart,
    ScaleUp,
    ScaleDown,
    Upgrade,
    HealthRemediation,
    ConfigApply,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationCycle {
    pub cycle_id: String,
    pub timestamp: u64,
    pub services_observed: usize,
    pub services_with_drift: usize,
    pub actions_generated: usize,
    pub conflicts_detected: Vec<String>,
}

pub struct ReconciliationEngine {
    desired_states: HashMap<String, ServiceState>,
    actual_states: HashMap<String, ServiceState>,
    pending_actions: Vec<ReconciliationAction>,
    completed_actions: Vec<ReconciliationAction>,
    cycle_history: Vec<ReconciliationCycle>,
    conflict_log: Vec<(u64, String)>,
}

impl ReconciliationEngine {
    pub fn new() -> Self {
        Self {
            desired_states: HashMap::new(),
            actual_states: HashMap::new(),
            pending_actions: Vec::new(),
            completed_actions: Vec::new(),
            cycle_history: Vec::new(),
            conflict_log: Vec::new(),
        }
    }

    pub fn set_desired_state(&mut self, service_id: String, state: ServiceState) -> Result<()> {
        if service_id.is_empty() {
            return Err(ReconciliationError::InvalidState("Service ID cannot be empty".to_string()));
        }

        self.desired_states.insert(service_id, state);
        Ok(())
    }

    pub fn observe_actual_state(&mut self, service_id: String, state: ServiceState) -> Result<()> {
        if service_id.is_empty() {
            return Err(ReconciliationError::InvalidState("Service ID cannot be empty".to_string()));
        }

        self.actual_states.insert(service_id, state);
        Ok(())
    }

    pub fn detect_drift(&self, service_id: &str) -> Result<DriftType> {
        let desired = self
            .desired_states
            .get(service_id)
            .ok_or(ReconciliationError::InvalidState(
                format!("No desired state for {}", service_id),
            ))?;

        let actual = self
            .actual_states
            .get(service_id)
            .ok_or(ReconciliationError::DriftDetected(
                format!("No actual state observed for {}", service_id),
            ))?;

        if desired.desired_version != actual.actual_version {
            return Ok(DriftType::VersionMismatch);
        }

        if desired.desired_replicas != actual.actual_replicas {
            return Ok(DriftType::ReplicaMismatch);
        }

        if desired.desired_state != DesiredState::Stopped && actual.actual_state == ActualState::Stopped {
            return Ok(DriftType::StateConflict);
        }

        if actual.healthy_replicas < (actual.actual_replicas / 2) {
            return Ok(DriftType::HealthMismatch);
        }

        Ok(DriftType::None)
    }

    pub fn reconcile(&mut self) -> Result<ReconciliationCycle> {
        let cycle_id = Uuid::new_v4().to_string();
        let timestamp = now_millis();
        let mut services_with_drift = 0;
        let mut actions_generated = 0;
        let mut conflicts = Vec::new();

        for service_id in self.desired_states.keys() {
            match self.detect_drift(service_id) {
                Ok(DriftType::None) => {}
                Ok(drift_type) => {
                    services_with_drift += 1;

                    let action = match drift_type {
                        DriftType::VersionMismatch => self.generate_upgrade_action(service_id, drift_type)?,
                        DriftType::ReplicaMismatch => self.generate_scaling_action(service_id, drift_type)?,
                        DriftType::StateConflict => {
                            conflicts.push(format!("State conflict on {}", service_id));
                            self.generate_conflict_resolution(service_id, drift_type)?
                        }
                        DriftType::HealthMismatch => self.generate_remediation_action(service_id, drift_type)?,
                        _ => self.generate_config_action(service_id, drift_type)?,
                    };

                    self.pending_actions.push(action);
                    actions_generated += 1;
                }
                Err(e) => {
                    conflicts.push(e.to_string());
                }
            }
        }

        for conflict in &conflicts {
            self.conflict_log.push((timestamp, conflict.clone()));
        }

        let cycle = ReconciliationCycle {
            cycle_id,
            timestamp,
            services_observed: self.desired_states.len(),
            services_with_drift,
            actions_generated,
            conflicts_detected: conflicts,
        };

        self.cycle_history.push(cycle.clone());
        Ok(cycle)
    }

    fn generate_upgrade_action(&self, service_id: &str, drift: DriftType) -> Result<ReconciliationAction> {
        Ok(ReconciliationAction {
            id: Uuid::new_v4().to_string(),
            service_id: service_id.to_string(),
            action_type: ActionType::Upgrade,
            reason: format!("Version drift detected: {:?}", drift),
            timestamp: now_millis(),
            completed: false,
        })
    }

    fn generate_scaling_action(&self, service_id: &str, _drift: DriftType) -> Result<ReconciliationAction> {
        let desired = self.desired_states.get(service_id).unwrap();
        let actual = self.actual_states.get(service_id).unwrap();

        let action_type = if desired.desired_replicas > actual.actual_replicas {
            ActionType::ScaleUp
        } else {
            ActionType::ScaleDown
        };

        Ok(ReconciliationAction {
            id: Uuid::new_v4().to_string(),
            service_id: service_id.to_string(),
            action_type,
            reason: format!("Replica mismatch: desired={}, actual={}", desired.desired_replicas, actual.actual_replicas),
            timestamp: now_millis(),
            completed: false,
        })
    }

    fn generate_remediation_action(&self, service_id: &str, drift: DriftType) -> Result<ReconciliationAction> {
        Ok(ReconciliationAction {
            id: Uuid::new_v4().to_string(),
            service_id: service_id.to_string(),
            action_type: ActionType::HealthRemediation,
            reason: format!("Health degradation: {:?}", drift),
            timestamp: now_millis(),
            completed: false,
        })
    }

    fn generate_conflict_resolution(&self, service_id: &str, drift: DriftType) -> Result<ReconciliationAction> {
        Ok(ReconciliationAction {
            id: Uuid::new_v4().to_string(),
            service_id: service_id.to_string(),
            action_type: ActionType::Restart,
            reason: format!("State conflict resolution: {:?}", drift),
            timestamp: now_millis(),
            completed: false,
        })
    }

    fn generate_config_action(&self, service_id: &str, drift: DriftType) -> Result<ReconciliationAction> {
        Ok(ReconciliationAction {
            id: Uuid::new_v4().to_string(),
            service_id: service_id.to_string(),
            action_type: ActionType::ConfigApply,
            reason: format!("Configuration mismatch: {:?}", drift),
            timestamp: now_millis(),
            completed: false,
        })
    }

    pub fn execute_action(&mut self, action_id: &str) -> Result<()> {
        let action = self
            .pending_actions
            .iter_mut()
            .find(|a| a.id == action_id)
            .ok_or(ReconciliationError::ReconciliationFailed(
                format!("Action not found: {}", action_id),
            ))?;

        action.completed = true;

        if let Some(pos) = self.pending_actions.iter().position(|a| a.id == action_id) {
            let completed = self.pending_actions.remove(pos);
            self.completed_actions.push(completed);
        }

        Ok(())
    }

    pub fn get_pending_actions(&self) -> &[ReconciliationAction] {
        &self.pending_actions
    }

    pub fn get_completed_actions(&self) -> &[ReconciliationAction] {
        &self.completed_actions
    }

    pub fn get_drift_summary(&self) -> Result<DriftSummary> {
        let mut summary = DriftSummary::default();

        for service_id in self.desired_states.keys() {
            match self.detect_drift(service_id) {
                Ok(drift_type) => {
                    summary.total_services += 1;
                    if drift_type != DriftType::None {
                        summary.services_with_drift += 1;
                        match drift_type {
                            DriftType::VersionMismatch => summary.version_mismatches += 1,
                            DriftType::ReplicaMismatch => summary.replica_mismatches += 1,
                            DriftType::HealthMismatch => summary.health_mismatches += 1,
                            DriftType::StateConflict => summary.state_conflicts += 1,
                            DriftType::ConfigurationMismatch => summary.config_mismatches += 1,
                            DriftType::None => {}
                        }
                    }
                }
                Err(_) => {
                    summary.observation_errors += 1;
                }
            }
        }

        Ok(summary)
    }

    pub fn get_conflict_log(&self) -> &[(u64, String)] {
        &self.conflict_log
    }

    pub fn get_cycle_history(&self) -> &[ReconciliationCycle] {
        &self.cycle_history
    }
}

impl Default for ReconciliationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DriftSummary {
    pub total_services: usize,
    pub services_with_drift: usize,
    pub version_mismatches: usize,
    pub replica_mismatches: usize,
    pub health_mismatches: usize,
    pub state_conflicts: usize,
    pub config_mismatches: usize,
    pub observation_errors: usize,
}

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_service(id: &str, name: &str) -> ServiceState {
        ServiceState {
            service_id: id.to_string(),
            service_name: name.to_string(),
            desired_state: DesiredState::Running,
            actual_state: ActualState::Running,
            desired_version: "1.0.0".to_string(),
            actual_version: "1.0.0".to_string(),
            desired_replicas: 3,
            actual_replicas: 3,
            healthy_replicas: 3,
            drift_type: DriftType::None,
            last_observed: now_millis(),
        }
    }

    #[test]
    fn test_engine_creation() {
        let engine = ReconciliationEngine::new();
        assert!(engine.desired_states.is_empty());
    }

    #[test]
    fn test_set_desired_state() {
        let mut engine = ReconciliationEngine::new();
        let service = create_service("svc-1", "Service 1");

        assert!(engine.set_desired_state("svc-1".to_string(), service).is_ok());
        assert_eq!(engine.desired_states.len(), 1);
    }

    #[test]
    fn test_no_drift_detection() {
        let mut engine = ReconciliationEngine::new();
        let service = create_service("svc-1", "Service 1");

        engine.set_desired_state("svc-1".to_string(), service.clone()).unwrap();
        engine.observe_actual_state("svc-1".to_string(), service).unwrap();

        let drift = engine.detect_drift("svc-1").unwrap();
        assert_eq!(drift, DriftType::None);
    }

    #[test]
    fn test_version_drift_detection() {
        let mut engine = ReconciliationEngine::new();
        let mut desired = create_service("svc-1", "Service 1");
        let mut actual = create_service("svc-1", "Service 1");

        desired.desired_version = "2.0.0".to_string();
        actual.actual_version = "1.0.0".to_string();

        engine.set_desired_state("svc-1".to_string(), desired).unwrap();
        engine.observe_actual_state("svc-1".to_string(), actual).unwrap();

        let drift = engine.detect_drift("svc-1").unwrap();
        assert_eq!(drift, DriftType::VersionMismatch);
    }

    #[test]
    fn test_replica_drift_detection() {
        let mut engine = ReconciliationEngine::new();
        let mut desired = create_service("svc-1", "Service 1");
        let mut actual = create_service("svc-1", "Service 1");

        desired.desired_replicas = 5;
        actual.actual_replicas = 3;

        engine.set_desired_state("svc-1".to_string(), desired).unwrap();
        engine.observe_actual_state("svc-1".to_string(), actual).unwrap();

        let drift = engine.detect_drift("svc-1").unwrap();
        assert_eq!(drift, DriftType::ReplicaMismatch);
    }

    #[test]
    fn test_reconciliation_cycle() {
        let mut engine = ReconciliationEngine::new();
        let mut desired = create_service("svc-1", "Service 1");
        let mut actual = create_service("svc-1", "Service 1");

        desired.desired_version = "2.0.0".to_string();
        actual.actual_version = "1.0.0".to_string();

        engine.set_desired_state("svc-1".to_string(), desired).unwrap();
        engine.observe_actual_state("svc-1".to_string(), actual).unwrap();

        let cycle = engine.reconcile().unwrap();
        assert_eq!(cycle.services_observed, 1);
        assert_eq!(cycle.services_with_drift, 1);
        assert_eq!(cycle.actions_generated, 1);
    }

    #[test]
    fn test_action_execution() {
        let mut engine = ReconciliationEngine::new();
        let mut desired = create_service("svc-1", "Service 1");
        let mut actual = create_service("svc-1", "Service 1");

        desired.desired_version = "2.0.0".to_string();
        actual.actual_version = "1.0.0".to_string();

        engine.set_desired_state("svc-1".to_string(), desired).unwrap();
        engine.observe_actual_state("svc-1".to_string(), actual).unwrap();

        let _cycle = engine.reconcile().unwrap();
        assert!(!engine.pending_actions.is_empty());

        let action_id = engine.pending_actions[0].id.clone();
        assert!(engine.execute_action(&action_id).is_ok());
        assert!(engine.pending_actions.is_empty());
        assert_eq!(engine.completed_actions.len(), 1);
    }

    #[test]
    fn test_drift_summary() {
        let mut engine = ReconciliationEngine::new();
        let mut desired = create_service("svc-1", "Service 1");
        let mut actual = create_service("svc-1", "Service 1");

        desired.desired_version = "2.0.0".to_string();
        actual.actual_version = "1.0.0".to_string();

        engine.set_desired_state("svc-1".to_string(), desired).unwrap();
        engine.observe_actual_state("svc-1".to_string(), actual).unwrap();

        let summary = engine.get_drift_summary().unwrap();
        assert_eq!(summary.total_services, 1);
        assert_eq!(summary.services_with_drift, 1);
        assert_eq!(summary.version_mismatches, 1);
    }
}
