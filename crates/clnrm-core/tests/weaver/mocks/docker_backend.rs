//! DockerBackendMock - Mock Docker operations for testing
//!
//! This mock simulates Docker container operations without requiring actual Docker.
//! It tracks container lifecycle and validates contract compliance.

use std::collections::HashMap;

/// Container state from container_lifecycle.yaml schema
#[derive(Debug, Clone, PartialEq)]
pub enum ContainerState {
    Creating,
    Running,
    Stopped,
    Error,
    Destroyed,
}

/// Mock container state tracking
#[derive(Debug, Clone)]
pub struct ContainerMockState {
    pub id: String,
    pub image: String,
    pub state: ContainerState,
    pub created_at: String,
    pub started_at: Option<String>,
    pub destroyed_at: Option<String>,
    pub exit_code: Option<i32>,
}

/// Mock Docker backend
#[derive(Debug, Default)]
pub struct DockerBackendMock {
    pub containers: HashMap<String, ContainerMockState>,
    pub create_calls: Vec<String>,  // Image names
    pub destroy_calls: Vec<String>, // Container IDs
    pub expected_lifecycle: Vec<ContainerState>,
    pub next_container_id: usize,
}

impl DockerBackendMock {
    /// Create a new mock backend
    pub fn new() -> Self {
        Self::default()
    }

    /// Simulate creating a container
    pub fn create_container(&mut self, image: &str) -> Result<String, String> {
        self.create_calls.push(image.to_string());

        let container_id = format!("mock-container-{}", self.next_container_id);
        self.next_container_id += 1;

        let container = ContainerMockState {
            id: container_id.clone(),
            image: image.to_string(),
            state: ContainerState::Creating,
            created_at: chrono::Utc::now().to_rfc3339(),
            started_at: None,
            destroyed_at: None,
            exit_code: None,
        };

        self.containers.insert(container_id.clone(), container);
        Ok(container_id)
    }

    /// Simulate starting a container
    pub fn start_container(&mut self, container_id: &str) -> Result<(), String> {
        let container = self.containers.get_mut(container_id)
            .ok_or_else(|| format!("Container not found: {}", container_id))?;

        container.state = ContainerState::Running;
        container.started_at = Some(chrono::Utc::now().to_rfc3339());
        Ok(())
    }

    /// Simulate destroying a container
    pub fn destroy_container(&mut self, container_id: &str) -> Result<(), String> {
        self.destroy_calls.push(container_id.to_string());

        let container = self.containers.get_mut(container_id)
            .ok_or_else(|| format!("Container not found: {}", container_id))?;

        container.state = ContainerState::Destroyed;
        container.destroyed_at = Some(chrono::Utc::now().to_rfc3339());
        container.exit_code = Some(0);
        Ok(())
    }

    /// Check if container was destroyed (cleanup validation)
    pub fn verify_destroyed(&self, container_id: &str) -> bool {
        self.containers
            .get(container_id)
            .map(|c| c.destroyed_at.is_some())
            .unwrap_or(false)
    }

    /// Get containers that leaked (not destroyed)
    pub fn get_leaked_containers(&self) -> Vec<String> {
        self.containers
            .iter()
            .filter(|(_, c)| c.destroyed_at.is_none())
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Simulate container with exit code
    pub fn set_exit_code(&mut self, container_id: &str, exit_code: i32) -> Result<(), String> {
        let container = self.containers.get_mut(container_id)
            .ok_or_else(|| format!("Container not found: {}", container_id))?;

        container.exit_code = Some(exit_code);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_container_generates_id() {
        let mut mock = DockerBackendMock::new();
        let id = mock.create_container("alpine:latest").unwrap();

        assert!(!id.is_empty());
        assert_eq!(mock.create_calls.len(), 1);
        assert!(mock.containers.contains_key(&id));
    }

    #[test]
    fn test_start_container_updates_state() {
        let mut mock = DockerBackendMock::new();
        let id = mock.create_container("alpine:latest").unwrap();

        mock.start_container(&id).unwrap();

        let container = mock.containers.get(&id).unwrap();
        assert_eq!(container.state, ContainerState::Running);
        assert!(container.started_at.is_some());
    }

    #[test]
    fn test_destroy_container_marks_destroyed() {
        let mut mock = DockerBackendMock::new();
        let id = mock.create_container("alpine:latest").unwrap();

        mock.destroy_container(&id).unwrap();

        assert!(mock.verify_destroyed(&id));
        let container = mock.containers.get(&id).unwrap();
        assert_eq!(container.state, ContainerState::Destroyed);
        assert!(container.destroyed_at.is_some());
    }

    #[test]
    fn test_leaked_containers_detection() {
        let mut mock = DockerBackendMock::new();
        let id1 = mock.create_container("alpine:latest").unwrap();
        let id2 = mock.create_container("postgres:15").unwrap();

        // Destroy only id1
        mock.destroy_container(&id1).unwrap();

        let leaked = mock.get_leaked_containers();
        assert_eq!(leaked.len(), 1);
        assert!(leaked.contains(&id2));
    }
}
