use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum StatusError {
    #[error("Component unhealthy: {0}")]
    ComponentUnhealthy(String),
    #[error("Status mismatch: {0}")]
    StatusMismatch(String),
    #[error("Integration broken: {0}")]
    IntegrationBroken(String),
}

pub type Result<T> = std::result::Result<T, StatusError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntegrationStatus {
    Connected,
    PartiallyConnected,
    Disconnected,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStatus {
    pub component_id: String,
    pub component_name: String,
    pub health: ComponentHealth,
    pub ready: bool,
    pub version: String,
    pub uptime_ms: u64,
    pub error_count: u64,
    pub last_update: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCoherence {
    pub total_assertions: usize,
    pub passed_assertions: usize,
    pub failed_assertions: Vec<String>,
    pub coherent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub snapshot_id: String,
    pub timestamp: u64,
    pub components: HashMap<String, ComponentStatus>,
    pub integration_status: HashMap<String, IntegrationStatus>,
    pub coherence: SystemCoherence,
    pub alerts: Vec<SystemAlert>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemAlert {
    pub alert_id: String,
    pub severity: AlertSeverity,
    pub component: String,
    pub message: String,
    pub timestamp: u64,
    pub acknowledged: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

pub struct SystemStatus {
    current_status: Option<SystemSnapshot>,
    status_history: Vec<SystemSnapshot>,
    integration_graph: HashMap<String, Vec<String>>,
}

impl SystemStatus {
    pub fn new() -> Self {
        Self {
            current_status: None,
            status_history: Vec::new(),
            integration_graph: HashMap::new(),
        }
    }

    pub fn register_integration(&mut self, source: String, targets: Vec<String>) -> Result<()> {
        self.integration_graph.insert(source, targets);
        Ok(())
    }

    pub fn collect_status(&mut self, components: HashMap<String, ComponentStatus>) -> Result<SystemSnapshot> {
        let snapshot_id = Uuid::new_v4().to_string();
        let timestamp = now_millis();

        let mut integration_status = HashMap::new();
        for (source, targets) in &self.integration_graph {
            let source_ok = components.get(source).map(|c| c.health == ComponentHealth::Healthy).unwrap_or(false);

            let mut all_targets_ok = true;
            for target in targets {
                let target_ok = components.get(target).map(|c| c.ready).unwrap_or(false);
                if !target_ok {
                    all_targets_ok = false;
                }
            }

            let status = match (source_ok, all_targets_ok) {
                (true, true) => IntegrationStatus::Connected,
                (true, false) => IntegrationStatus::PartiallyConnected,
                (false, _) => IntegrationStatus::Disconnected,
            };

            integration_status.insert(format!("{}", source), status);
        }

        let mut coherence = SystemCoherence {
            total_assertions: 0,
            passed_assertions: 0,
            failed_assertions: Vec::new(),
            coherent: true,
        };

        // Check component health invariants
        for component in components.values() {
            coherence.total_assertions += 1;
            if component.health == ComponentHealth::Healthy && component.ready {
                coherence.passed_assertions += 1;
            } else {
                coherence.coherent = false;
                coherence.failed_assertions.push(
                    format!("{}: health={:?}, ready={}", component.component_name, component.health, component.ready)
                );
            }
        }

        // Check integration invariants
        for (source, targets) in &self.integration_graph {
            for target in targets {
                coherence.total_assertions += 1;
                if let (Some(src), Some(tgt)) = (components.get(source), components.get(target)) {
                    if src.health == ComponentHealth::Healthy && tgt.ready {
                        coherence.passed_assertions += 1;
                    } else {
                        coherence.coherent = false;
                        coherence.failed_assertions.push(
                            format!("Integration {} -> {} broken", source, target)
                        );
                    }
                } else {
                    coherence.coherent = false;
                    coherence.failed_assertions.push(
                        format!("Integration {} -> {} missing components", source, target)
                    );
                }
            }
        }

        let mut alerts = Vec::new();

        // Generate alerts for unhealthy components
        for component in components.values() {
            if component.health == ComponentHealth::Unhealthy {
                alerts.push(SystemAlert {
                    alert_id: Uuid::new_v4().to_string(),
                    severity: AlertSeverity::Critical,
                    component: component.component_name.clone(),
                    message: format!("{} is unhealthy", component.component_name),
                    timestamp,
                    acknowledged: false,
                });
            } else if component.health == ComponentHealth::Degraded {
                alerts.push(SystemAlert {
                    alert_id: Uuid::new_v4().to_string(),
                    severity: AlertSeverity::Warning,
                    component: component.component_name.clone(),
                    message: format!("{} is degraded", component.component_name),
                    timestamp,
                    acknowledged: false,
                });
            }
        }

        // Generate alerts for broken integrations
        for (source, status) in &integration_status {
            if *status != IntegrationStatus::Connected {
                alerts.push(SystemAlert {
                    alert_id: Uuid::new_v4().to_string(),
                    severity: AlertSeverity::Critical,
                    component: source.clone(),
                    message: format!("Integration status: {:?}", status),
                    timestamp,
                    acknowledged: false,
                });
            }
        }

        let snapshot = SystemSnapshot {
            snapshot_id,
            timestamp,
            components,
            integration_status,
            coherence,
            alerts,
        };

        self.current_status = Some(snapshot.clone());
        self.status_history.push(snapshot.clone());

        Ok(snapshot)
    }

    pub fn get_current_status(&self) -> Option<&SystemSnapshot> {
        self.current_status.as_ref()
    }

    pub fn get_status_history(&self) -> &[SystemSnapshot] {
        &self.status_history
    }

    pub fn verify_coherence(&self) -> Result<()> {
        if let Some(status) = &self.current_status {
            if !status.coherence.coherent {
                return Err(StatusError::IntegrationBroken(
                    format!("{} failed assertions", status.coherence.failed_assertions.len())
                ));
            }
            Ok(())
        } else {
            Err(StatusError::StatusMismatch("No status collected".to_string()))
        }
    }

    pub fn get_unhealthy_components(&self) -> Vec<&ComponentStatus> {
        if let Some(status) = &self.current_status {
            status.components.values()
                .filter(|c| c.health != ComponentHealth::Healthy || !c.ready)
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_broken_integrations(&self) -> Vec<(String, IntegrationStatus)> {
        if let Some(status) = &self.current_status {
            status.integration_status.iter()
                .filter(|(_, s)| **s != IntegrationStatus::Connected)
                .map(|(k, v)| (k.clone(), *v))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn acknowledge_alert(&mut self, alert_id: &str) -> Result<()> {
        if let Some(status) = &mut self.current_status {
            if let Some(alert) = status.alerts.iter_mut().find(|a| a.alert_id == alert_id) {
                alert.acknowledged = true;
                return Ok(());
            }
        }
        Err(StatusError::StatusMismatch("Alert not found".to_string()))
    }
}

impl Default for SystemStatus {
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

    fn create_component(id: &str, name: &str, healthy: bool) -> ComponentStatus {
        ComponentStatus {
            component_id: id.to_string(),
            component_name: name.to_string(),
            health: if healthy { ComponentHealth::Healthy } else { ComponentHealth::Unhealthy },
            ready: healthy,
            version: "1.0.0".to_string(),
            uptime_ms: 3600000,
            error_count: 0,
            last_update: now_millis(),
        }
    }

    #[test]
    fn test_system_creation() {
        let system = SystemStatus::new();
        assert!(system.get_current_status().is_none());
    }

    #[test]
    fn test_component_collection() {
        let mut system = SystemStatus::new();
        let mut components = HashMap::new();

        components.insert("comp-1".to_string(), create_component("comp-1", "Component 1", true));

        let status = system.collect_status(components).unwrap();
        assert_eq!(status.components.len(), 1);
    }

    #[test]
    fn test_coherence_check() {
        let mut system = SystemStatus::new();
        let mut components = HashMap::new();

        components.insert("comp-1".to_string(), create_component("comp-1", "Component 1", true));
        system.collect_status(components).unwrap();

        assert!(system.verify_coherence().is_ok());
    }

    #[test]
    fn test_integration_tracking() {
        let mut system = SystemStatus::new();
        system.register_integration(
            "orchestrator".to_string(),
            vec!["service-mesh".to_string(), "config".to_string()],
        ).unwrap();

        let mut components = HashMap::new();
        components.insert("orchestrator".to_string(), create_component("orch", "Orchestrator", true));
        components.insert("service-mesh".to_string(), create_component("mesh", "Service Mesh", true));
        components.insert("config".to_string(), create_component("config", "Config", true));

        system.collect_status(components).unwrap();
        assert!(system.verify_coherence().is_ok());
    }

    #[test]
    fn test_unhealthy_detection() {
        let mut system = SystemStatus::new();
        let mut components = HashMap::new();

        components.insert("comp-1".to_string(), create_component("comp-1", "Component 1", false));
        system.collect_status(components).unwrap();

        let unhealthy = system.get_unhealthy_components();
        assert!(!unhealthy.is_empty());
    }

    #[test]
    fn test_alert_generation() {
        let mut system = SystemStatus::new();
        let mut components = HashMap::new();

        components.insert("comp-1".to_string(), create_component("comp-1", "Component 1", false));
        let status = system.collect_status(components).unwrap();

        assert!(!status.alerts.is_empty());
    }
}
