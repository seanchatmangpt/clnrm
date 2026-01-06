use crate::cleanroom::{ServiceHandle, ServicePlugin, HealthStatus};
use crate::error::{Result, CleanroomError};
use std::collections::HashMap;
use std::fmt;

pub struct GenGenServiceLoader;

impl GenGenServiceLoader {
    pub fn load_services() -> Result<Vec<Box<dyn ServicePlugin>>> {
        Ok(vec![
            Box::new(SurrealDbService::new()),
            Box::new(OllamaGenGenService::new()),
            Box::new(VllmGenGenService::new()),
            Box::new(TgiGenGenService::new()),
            Box::new(PostgresService::new()),
            Box::new(OtelCollectorService::new()),
        ])
    }
}

#[derive(Debug, Clone)]
pub struct SurrealDbService {
    name: String,
    image: String,
    port: u16,
}

impl SurrealDbService {
    pub fn new() -> Self {
        Self {
            name: "surrealdb".to_string(),
            image: "surrealdb:latest".to_string(),
            port: 8000,
        }
    }
}

impl ServicePlugin for SurrealDbService {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        let handle = ServiceHandle {
            id: uuid::Uuid::new_v4().to_string(),
            service_name: self.name.clone(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("image".to_string(), self.image.clone());
                m.insert("port".to_string(), self.port.to_string());
                m.insert("type".to_string(), "SurrealDbPlugin".to_string());
                m
            },
        };
        Ok(handle)
    }

    fn stop(&self, _handle: ServiceHandle) -> Result<()> {
        Ok(())
    }

    fn health_check(&self, _handle: &ServiceHandle) -> HealthStatus {
        HealthStatus::Healthy
    }
}

#[derive(Debug, Clone)]
pub struct OllamaGenGenService {
    name: String,
    image: String,
    port: u16,
}

impl OllamaGenGenService {
    pub fn new() -> Self {
        Self {
            name: "ollama".to_string(),
            image: "ollama/ollama:latest".to_string(),
            port: 11434,
        }
    }
}

impl ServicePlugin for OllamaGenGenService {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        let handle = ServiceHandle {
            id: uuid::Uuid::new_v4().to_string(),
            service_name: self.name.clone(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("image".to_string(), self.image.clone());
                m.insert("port".to_string(), self.port.to_string());
                m.insert("type".to_string(), "OllamaPlugin".to_string());
                m
            },
        };
        Ok(handle)
    }

    fn stop(&self, _handle: ServiceHandle) -> Result<()> {
        Ok(())
    }

    fn health_check(&self, _handle: &ServiceHandle) -> HealthStatus {
        HealthStatus::Healthy
    }
}

#[derive(Debug, Clone)]
pub struct VllmGenGenService {
    name: String,
    image: String,
    port: u16,
}

impl VllmGenGenService {
    pub fn new() -> Self {
        Self {
            name: "vllm".to_string(),
            image: "vllm/vllm:latest".to_string(),
            port: 8000,
        }
    }
}

impl ServicePlugin for VllmGenGenService {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        let handle = ServiceHandle {
            id: uuid::Uuid::new_v4().to_string(),
            service_name: self.name.clone(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("image".to_string(), self.image.clone());
                m.insert("port".to_string(), self.port.to_string());
                m.insert("type".to_string(), "VllmPlugin".to_string());
                m
            },
        };
        Ok(handle)
    }

    fn stop(&self, _handle: ServiceHandle) -> Result<()> {
        Ok(())
    }

    fn health_check(&self, _handle: &ServiceHandle) -> HealthStatus {
        HealthStatus::Healthy
    }
}

#[derive(Debug, Clone)]
pub struct TgiGenGenService {
    name: String,
    image: String,
    port: u16,
}

impl TgiGenGenService {
    pub fn new() -> Self {
        Self {
            name: "tgi".to_string(),
            image: "ghcr.io/huggingface/text-generation-inference:latest".to_string(),
            port: 8080,
        }
    }
}

impl ServicePlugin for TgiGenGenService {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        let handle = ServiceHandle {
            id: uuid::Uuid::new_v4().to_string(),
            service_name: self.name.clone(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("image".to_string(), self.image.clone());
                m.insert("port".to_string(), self.port.to_string());
                m.insert("type".to_string(), "TgiPlugin".to_string());
                m
            },
        };
        Ok(handle)
    }

    fn stop(&self, _handle: ServiceHandle) -> Result<()> {
        Ok(())
    }

    fn health_check(&self, _handle: &ServiceHandle) -> HealthStatus {
        HealthStatus::Healthy
    }
}

#[derive(Debug, Clone)]
pub struct PostgresService {
    name: String,
    image: String,
    port: u16,
}

impl PostgresService {
    pub fn new() -> Self {
        Self {
            name: "postgres".to_string(),
            image: "postgres:15-alpine".to_string(),
            port: 5432,
        }
    }
}

impl ServicePlugin for PostgresService {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        let handle = ServiceHandle {
            id: uuid::Uuid::new_v4().to_string(),
            service_name: self.name.clone(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("image".to_string(), self.image.clone());
                m.insert("port".to_string(), self.port.to_string());
                m.insert("type".to_string(), "PostgresPlugin".to_string());
                m.insert("env.POSTGRES_PASSWORD".to_string(), "testpassword".to_string());
                m
            },
        };
        Ok(handle)
    }

    fn stop(&self, _handle: ServiceHandle) -> Result<()> {
        Ok(())
    }

    fn health_check(&self, _handle: &ServiceHandle) -> HealthStatus {
        HealthStatus::Healthy
    }
}

#[derive(Debug, Clone)]
pub struct OtelCollectorService {
    name: String,
    image: String,
    port: u16,
}

impl OtelCollectorService {
    pub fn new() -> Self {
        Self {
            name: "otel-collector".to_string(),
            image: "otel/opentelemetry-collector:latest".to_string(),
            port: 4317,
        }
    }
}

impl ServicePlugin for OtelCollectorService {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        let handle = ServiceHandle {
            id: uuid::Uuid::new_v4().to_string(),
            service_name: self.name.clone(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("image".to_string(), self.image.clone());
                m.insert("port".to_string(), self.port.to_string());
                m.insert("type".to_string(), "OtelCollectorPlugin".to_string());
                m
            },
        };
        Ok(handle)
    }

    fn stop(&self, _handle: ServiceHandle) -> Result<()> {
        Ok(())
    }

    fn health_check(&self, _handle: &ServiceHandle) -> HealthStatus {
        HealthStatus::Healthy
    }
}

impl Default for SurrealDbService {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for OllamaGenGenService {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for VllmGenGenService {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TgiGenGenService {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PostgresService {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for OtelCollectorService {
    fn default() -> Self {
        Self::new()
    }
}
