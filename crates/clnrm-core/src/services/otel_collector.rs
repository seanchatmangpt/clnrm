use crate::cleanroom::{HealthStatus, ServiceHandle, ServicePlugin};
use crate::error::Result;

#[derive(Debug)]
pub struct OtelCollectorPlugin {
    pub name: String,
}

impl OtelCollectorPlugin {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl ServicePlugin for OtelCollectorPlugin {
    fn name(&self) -> &str {
        &self.name
    }
    fn start(&self) -> Result<crate::cleanroom::ServiceHandle> {
        Ok(ServiceHandle::new(&self.name))
    }
    fn stop(&self, _handle: ServiceHandle) -> Result<()> {
        Ok(())
    }
    fn health_check(&self, _handle: &ServiceHandle) -> HealthStatus {
        HealthStatus::Healthy
    }
}
