use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum ControlError {
    #[error("Policy violation: {0}")]
    PolicyViolation(String),
    #[error("Action conflict: {0}")]
    ActionConflict(String),
    #[error("Invalid policy: {0}")]
    InvalidPolicy(String),
    #[error("Control loop error: {0}")]
    ControlLoopError(String),
}

pub type Result<T> = std::result::Result<T, ControlError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyType {
    MaxConcurrentActions,
    RateLimiting,
    DependencyOrdering,
    ResourceConstraint,
    HighAvailability,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: String,
    pub policy_type: PolicyType,
    pub name: String,
    pub enabled: bool,
    pub parameters: HashMap<String, String>,
    pub violation_severity: ViolationSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolation {
    pub id: String,
    pub policy_id: String,
    pub severity: ViolationSeverity,
    pub message: String,
    pub timestamp: u64,
    pub resolved: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionState {
    Pending,
    Queued,
    Executing,
    Completed,
    Failed,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledAction {
    pub id: String,
    pub action_type: String,
    pub target_service: String,
    pub dependencies: Vec<String>,
    pub priority: u32,
    pub state: ActionState,
    pub reason: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlCycle {
    pub cycle_id: String,
    pub timestamp: u64,
    pub actions_scheduled: usize,
    pub actions_executed: usize,
    pub policy_violations: usize,
    pub control_decisions: Vec<String>,
}

pub struct ControlLoop {
    policies: HashMap<String, Policy>,
    action_queue: VecDeque<ScheduledAction>,
    completed_actions: Vec<ScheduledAction>,
    violations: Vec<PolicyViolation>,
    cycle_history: Vec<ControlCycle>,
}

impl ControlLoop {
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
            action_queue: VecDeque::new(),
            completed_actions: Vec::new(),
            violations: Vec::new(),
            cycle_history: Vec::new(),
        }
    }

    pub fn add_policy(&mut self, policy: Policy) -> Result<()> {
        if policy.name.is_empty() {
            return Err(ControlError::InvalidPolicy("Policy name cannot be empty".to_string()));
        }

        self.policies.insert(policy.id.clone(), policy);
        Ok(())
    }

    pub fn schedule_action(&mut self, action: ScheduledAction) -> Result<()> {
        self.action_queue.push_back(action);
        Ok(())
    }

    pub fn evaluate_policies(&mut self, action: &ScheduledAction) -> Result<Vec<PolicyViolation>> {
        let mut violations = Vec::new();

        for policy in self.policies.values() {
            if !policy.enabled {
                continue;
            }

            match policy.policy_type {
                PolicyType::MaxConcurrentActions => {
                    if let Some(max_str) = policy.parameters.get("max_concurrent") {
                        if let Ok(max) = max_str.parse::<usize>() {
                            let executing = self
                                .action_queue
                                .iter()
                                .filter(|a| a.state == ActionState::Executing)
                                .count();

                            if executing >= max {
                                violations.push(PolicyViolation {
                                    id: Uuid::new_v4().to_string(),
                                    policy_id: policy.id.clone(),
                                    severity: ViolationSeverity::Warning,
                                    message: format!(
                                        "Max concurrent actions ({}) exceeded",
                                        max
                                    ),
                                    timestamp: now_millis(),
                                    resolved: false,
                                });
                            }
                        }
                    }
                }
                PolicyType::DependencyOrdering => {
                    for dep in &action.dependencies {
                        let dep_completed = self.completed_actions.iter().any(|a| a.id == *dep);
                        let dep_executing = self
                            .action_queue
                            .iter()
                            .any(|a| a.id == *dep && a.state == ActionState::Executing);

                        if !dep_completed && !dep_executing {
                            violations.push(PolicyViolation {
                                id: Uuid::new_v4().to_string(),
                                policy_id: policy.id.clone(),
                                severity: ViolationSeverity::Critical,
                                message: format!("Dependency {} not satisfied", dep),
                                timestamp: now_millis(),
                                resolved: false,
                            });
                        }
                    }
                }
                PolicyType::HighAvailability => {
                    if let Some(min_running) = policy.parameters.get("min_replicas") {
                        if let Ok(min) = min_running.parse::<u32>() {
                            // In real implementation, check actual running replicas
                            // For now, this is a placeholder check
                            if action.action_type.contains("Stop") {
                                // Hypothetical: check if stopping would violate HA
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        for violation in &violations {
            self.violations.push(violation.clone());
        }

        Ok(violations)
    }

    pub fn execute_cycle(&mut self) -> Result<ControlCycle> {
        let cycle_id = Uuid::new_v4().to_string();
        let timestamp = now_millis();
        let mut actions_scheduled = 0;
        let mut actions_executed = 0;
        let mut control_decisions = Vec::new();
        let violations_before = self.violations.len();

        while !self.action_queue.is_empty() {
            let action = self.action_queue.front().unwrap().clone();
            let violations = self.evaluate_policies(&action)?;

            if violations.is_empty() {
                if let Some(mut scheduled) = self.action_queue.pop_front() {
                    scheduled.state = ActionState::Executing;
                    control_decisions.push(format!(
                        "Execute {} on {}",
                        scheduled.action_type, scheduled.target_service
                    ));

                    scheduled.state = ActionState::Completed;
                    self.completed_actions.push(scheduled.clone());
                    actions_executed += 1;
                }
            } else {
                control_decisions.push(format!(
                    "Blocked action {} due to {} policy violations",
                    action.id,
                    violations.len()
                ));

                if let Some(blocked) = self.action_queue.front_mut() {
                    blocked.state = ActionState::Blocked;
                }

                break;
            }
        }

        actions_scheduled = self.action_queue.len();
        let policy_violations = self.violations.len() - violations_before;

        let cycle = ControlCycle {
            cycle_id,
            timestamp,
            actions_scheduled,
            actions_executed,
            policy_violations,
            control_decisions,
        };

        self.cycle_history.push(cycle.clone());
        Ok(cycle)
    }

    pub fn get_action_queue(&self) -> &VecDeque<ScheduledAction> {
        &self.action_queue
    }

    pub fn get_violations(&self) -> &[PolicyViolation] {
        &self.violations
    }

    pub fn get_completed_actions(&self) -> &[ScheduledAction] {
        &self.completed_actions
    }

    pub fn get_cycle_history(&self) -> &[ControlCycle] {
        &self.cycle_history
    }

    pub fn get_policies(&self) -> Vec<&Policy> {
        self.policies.values().collect()
    }

    pub fn resolve_violation(&mut self, violation_id: &str) -> Result<()> {
        if let Some(violation) = self.violations.iter_mut().find(|v| v.id == violation_id) {
            violation.resolved = true;
            Ok(())
        } else {
            Err(ControlError::ControlLoopError(
                format!("Violation not found: {}", violation_id),
            ))
        }
    }
}

impl Default for ControlLoop {
    fn default() -> Self {
        Self::new()
    }
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

    fn create_test_policy(ptype: PolicyType) -> Policy {
        Policy {
            id: Uuid::new_v4().to_string(),
            policy_type: ptype,
            name: format!("{:?} Policy", ptype),
            enabled: true,
            parameters: HashMap::new(),
            violation_severity: ViolationSeverity::Warning,
        }
    }

    fn create_test_action(id: &str) -> ScheduledAction {
        ScheduledAction {
            id: id.to_string(),
            action_type: "Upgrade".to_string(),
            target_service: "api-service".to_string(),
            dependencies: vec![],
            priority: 1,
            state: ActionState::Pending,
            reason: "Version mismatch".to_string(),
            timestamp: now_millis(),
        }
    }

    #[test]
    fn test_loop_creation() {
        let loop_inst = ControlLoop::new();
        assert!(loop_inst.policies.is_empty());
    }

    #[test]
    fn test_add_policy() {
        let mut loop_inst = ControlLoop::new();
        let policy = create_test_policy(PolicyType::MaxConcurrentActions);

        assert!(loop_inst.add_policy(policy).is_ok());
        assert_eq!(loop_inst.get_policies().len(), 1);
    }

    #[test]
    fn test_schedule_action() {
        let mut loop_inst = ControlLoop::new();
        let action = create_test_action("action-1");

        assert!(loop_inst.schedule_action(action).is_ok());
        assert_eq!(loop_inst.get_action_queue().len(), 1);
    }

    #[test]
    fn test_execute_cycle_no_violations() {
        let mut loop_inst = ControlLoop::new();
        let action = create_test_action("action-1");

        loop_inst.schedule_action(action).unwrap();

        let cycle = loop_inst.execute_cycle().unwrap();
        assert_eq!(cycle.actions_executed, 1);
        assert_eq!(cycle.actions_scheduled, 0);
    }

    #[test]
    fn test_dependency_policy() {
        let mut loop_inst = ControlLoop::new();
        let mut policy = create_test_policy(PolicyType::DependencyOrdering);
        policy.violation_severity = ViolationSeverity::Critical;

        loop_inst.add_policy(policy).unwrap();

        let mut action = create_test_action("action-1");
        action.dependencies = vec!["prereq-1".to_string()];

        let violations = loop_inst.evaluate_policies(&action).unwrap();
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_violation_resolution() {
        let mut loop_inst = ControlLoop::new();
        let policy = create_test_policy(PolicyType::DependencyOrdering);

        loop_inst.add_policy(policy).unwrap();

        let mut action = create_test_action("action-1");
        action.dependencies = vec!["prereq-1".to_string()];

        let violations = loop_inst.evaluate_policies(&action).unwrap();
        assert!(!violations.is_empty());

        let violation_id = violations[0].id.clone();
        assert!(loop_inst.resolve_violation(&violation_id).is_ok());

        let all_violations = loop_inst.get_violations();
        assert!(all_violations[0].resolved);
    }

    #[test]
    fn test_cycle_history() {
        let mut loop_inst = ControlLoop::new();
        let action = create_test_action("action-1");

        loop_inst.schedule_action(action).unwrap();
        loop_inst.execute_cycle().unwrap();

        let history = loop_inst.get_cycle_history();
        assert_eq!(history.len(), 1);
    }
}
