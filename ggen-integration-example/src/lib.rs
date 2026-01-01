use std::collections::HashMap;
use std::fmt;

pub type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ServiceHandle {
    pub id: String,
    pub service_name: String,
    pub metadata: HashMap<String, String>,
}

pub trait ServicePlugin: Send + Sync + fmt::Debug {
    fn name(&self) -> &str;
    fn start(&self) -> Result<ServiceHandle>;
    fn stop(&self, handle: ServiceHandle) -> Result<()>;
    fn health_check(&self, handle: &ServiceHandle) -> HealthStatus;
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
        Ok(ServiceHandle {
            id: uuid::Uuid::new_v4().to_string(),
            service_name: self.name.clone(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("image".to_string(), self.image.clone());
                m.insert("port".to_string(), self.port.to_string());
                m.insert("type".to_string(), "SurrealDbPlugin".to_string());
                m
            },
        })
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
        Ok(ServiceHandle {
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
        })
    }

    fn stop(&self, _handle: ServiceHandle) -> Result<()> {
        Ok(())
    }

    fn health_check(&self, _handle: &ServiceHandle) -> HealthStatus {
        HealthStatus::Healthy
    }
}

#[derive(Debug, Clone)]
pub struct OllamaService {
    name: String,
    image: String,
    port: u16,
}

impl OllamaService {
    pub fn new() -> Self {
        Self {
            name: "ollama".to_string(),
            image: "ollama/ollama:latest".to_string(),
            port: 11434,
        }
    }
}

impl ServicePlugin for OllamaService {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        Ok(ServiceHandle {
            id: uuid::Uuid::new_v4().to_string(),
            service_name: self.name.clone(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("image".to_string(), self.image.clone());
                m.insert("port".to_string(), self.port.to_string());
                m.insert("type".to_string(), "OllamaPlugin".to_string());
                m
            },
        })
    }

    fn stop(&self, _handle: ServiceHandle) -> Result<()> {
        Ok(())
    }

    fn health_check(&self, _handle: &ServiceHandle) -> HealthStatus {
        HealthStatus::Healthy
    }
}

pub struct ServiceRegistry {
    pub plugins: HashMap<String, Box<dyn ServicePlugin>>,
    pub active_services: HashMap<String, ServiceHandle>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            active_services: HashMap::new(),
        }
    }

    pub fn register_plugin(&mut self, plugin: Box<dyn ServicePlugin>) {
        self.plugins.insert(plugin.name().to_string(), plugin);
    }

    pub fn load_ggen_services(&mut self) -> Result<()> {
        self.register_plugin(Box::new(SurrealDbService::new()));
        self.register_plugin(Box::new(PostgresService::new()));
        self.register_plugin(Box::new(OllamaService::new()));
        Ok(())
    }

    pub fn list_services(&self) -> Vec<&str> {
        self.plugins.keys().map(|k| k.as_str()).collect()
    }

    pub fn start_service(&mut self, name: &str) -> Result<ServiceHandle> {
        let plugin = self.plugins.get(name)
            .ok_or(format!("Service not found: {}", name))?;

        let handle = plugin.start()?;
        self.active_services.insert(handle.id.clone(), handle.clone());
        Ok(handle)
    }

    pub fn stop_service(&mut self, id: &str) -> Result<()> {
        if let Some(handle) = self.active_services.remove(id) {
            let plugin = self.plugins.get(&handle.service_name)
                .ok_or(format!("Service plugin not found"))?;
            plugin.stop(handle)?;
        }
        Ok(())
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_services() {
        let mut registry = ServiceRegistry::new();
        assert!(registry.load_ggen_services().is_ok());
        assert_eq!(registry.list_services().len(), 3);
    }

    #[test]
    fn test_service_names() {
        let mut registry = ServiceRegistry::new();
        registry.load_ggen_services().unwrap();
        let services = registry.list_services();
        assert!(services.contains(&"surrealdb"));
        assert!(services.contains(&"postgres"));
        assert!(services.contains(&"ollama"));
    }

    #[test]
    fn test_start_stop_service() {
        let mut registry = ServiceRegistry::new();
        registry.load_ggen_services().unwrap();

        let handle = registry.start_service("surrealdb").unwrap();
        assert_eq!(handle.service_name, "surrealdb");
        assert!(!handle.id.is_empty());
        assert!(handle.metadata.contains_key("image"));

        registry.stop_service(&handle.id).unwrap();
    }

    #[test]
    fn test_health_check() {
        let service = SurrealDbService::new();
        let handle = service.start().unwrap();
        let health = service.health_check(&handle);
        assert_eq!(health, HealthStatus::Healthy);
        service.stop(handle).unwrap();
    }

    #[test]
    fn test_multiple_services() {
        let mut registry = ServiceRegistry::new();
        registry.load_ggen_services().unwrap();

        let h1 = registry.start_service("surrealdb").unwrap();
        let h2 = registry.start_service("postgres").unwrap();
        let h3 = registry.start_service("ollama").unwrap();

        assert_eq!(registry.active_services.len(), 3);

        registry.stop_service(&h1.id).unwrap();
        registry.stop_service(&h2.id).unwrap();
        registry.stop_service(&h3.id).unwrap();

        assert_eq!(registry.active_services.len(), 0);
    }

    #[test]
    fn test_surrealdb_metadata() {
        let service = SurrealDbService::new();
        let handle = service.start().unwrap();

        assert_eq!(handle.metadata.get("image").unwrap(), "surrealdb:latest");
        assert_eq!(handle.metadata.get("port").unwrap(), "8000");
        assert_eq!(handle.metadata.get("type").unwrap(), "SurrealDbPlugin");
    }

    #[test]
    fn test_postgres_metadata() {
        let service = PostgresService::new();
        let handle = service.start().unwrap();

        assert_eq!(handle.metadata.get("image").unwrap(), "postgres:15-alpine");
        assert_eq!(handle.metadata.get("port").unwrap(), "5432");
        assert_eq!(handle.metadata.get("env.POSTGRES_PASSWORD").unwrap(), "testpassword");
    }

    #[test]
    fn test_service_health_status() {
        let sdb = SurrealDbService::new();
        let pg = PostgresService::new();
        let ollama = OllamaService::new();

        let h1 = sdb.start().unwrap();
        let h2 = pg.start().unwrap();
        let h3 = ollama.start().unwrap();

        assert_eq!(sdb.health_check(&h1), HealthStatus::Healthy);
        assert_eq!(pg.health_check(&h2), HealthStatus::Healthy);
        assert_eq!(ollama.health_check(&h3), HealthStatus::Healthy);
    }

    #[test]
    fn test_service_names_unique() {
        let s1 = SurrealDbService::new();
        let s2 = PostgresService::new();
        let s3 = OllamaService::new();

        let names = vec![s1.name(), s2.name(), s3.name()];
        assert_eq!(names.len(), 3);
        assert_eq!(names.iter().collect::<std::collections::HashSet<_>>().len(), 3);
    }
}
