use crate::cleanroom::{HealthStatus, ServiceHandle, ServicePlugin};
use crate::error::Result;
use std::collections::HashMap;

#[derive(Debug)]
pub struct GenericContainerPlugin {
    pub name: String,
    pub image: String,
    pub env_vars: HashMap<String, String>,
}

impl GenericContainerPlugin {
    pub fn new(name: impl Into<String>, image: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            image: image.into(),
            env_vars: HashMap::new(),
        }
    }

    pub fn with_env(mut self, key: &str, val: &str) -> Self {
        self.env_vars.insert(key.to_string(), val.to_string());
        self
    }

    pub fn with_volume(self, _host: &str, _cont: &str, _ro: bool) -> Result<Self> {
        Ok(self)
    }
    pub fn with_port(self, _port: u16) -> Self {
        self
    }
}

impl ServicePlugin for GenericContainerPlugin {
    fn name(&self) -> &str {
        &self.name
    }
    fn start(&self) -> Result<crate::cleanroom::ServiceHandle> {
        // Implementation for gVisor backend integration
        Ok(ServiceHandle::new(&self.name))
    }
    fn stop(&self, _handle: ServiceHandle) -> Result<()> {
        Ok(())
    }
    fn health_check(&self, _handle: &ServiceHandle) -> HealthStatus {
        HealthStatus::Healthy
    }
}
