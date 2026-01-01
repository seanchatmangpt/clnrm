use std::collections::HashMap;
use std::fmt;

pub type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Failed,
}

#[derive(Debug, Clone)]
pub struct ServiceHandle {
    pub id: String,
    pub service_name: String,
    pub metadata: HashMap<String, String>,
    pub state: ServiceState,
    pub started_at: u64,
}

#[derive(Debug, Clone)]
pub struct TestStep {
    pub name: String,
    pub command: Vec<String>,
    pub expected_output: Option<String>,
    pub timeout_ms: u64,
    pub retries: u32,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub duration_ms: u64,
    pub output: String,
    pub error: Option<String>,
}

pub trait ServicePlugin: Send + Sync + fmt::Debug {
    fn name(&self) -> &str;
    fn start(&self) -> Result<ServiceHandle>;
    fn stop(&self, handle: ServiceHandle) -> Result<()>;
    fn health_check(&self, handle: &ServiceHandle) -> HealthStatus;
    fn get_port(&self) -> u16;
    fn get_image(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct SurrealDbService;

impl SurrealDbService {
    pub fn new() -> Self {
        Self
    }
}

impl ServicePlugin for SurrealDbService {
    fn name(&self) -> &str {
        "surrealdb"
    }

    fn start(&self) -> Result<ServiceHandle> {
        Ok(ServiceHandle {
            id: uuid::Uuid::new_v4().to_string(),
            service_name: self.name().to_string(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("image".to_string(), "surrealdb:latest".to_string());
                m.insert("port".to_string(), "8000".to_string());
                m
            },
            state: ServiceState::Running,
            started_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    fn stop(&self, _: ServiceHandle) -> Result<()> {
        Ok(())
    }

    fn health_check(&self, _: &ServiceHandle) -> HealthStatus {
        HealthStatus::Healthy
    }

    fn get_port(&self) -> u16 {
        8000
    }

    fn get_image(&self) -> &str {
        "surrealdb:latest"
    }
}

#[derive(Debug, Clone)]
pub struct PostgresService;

impl PostgresService {
    pub fn new() -> Self {
        Self
    }
}

impl ServicePlugin for PostgresService {
    fn name(&self) -> &str {
        "postgres"
    }

    fn start(&self) -> Result<ServiceHandle> {
        Ok(ServiceHandle {
            id: uuid::Uuid::new_v4().to_string(),
            service_name: self.name().to_string(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("image".to_string(), "postgres:15-alpine".to_string());
                m.insert("port".to_string(), "5432".to_string());
                m
            },
            state: ServiceState::Running,
            started_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    fn stop(&self, _: ServiceHandle) -> Result<()> {
        Ok(())
    }

    fn health_check(&self, _: &ServiceHandle) -> HealthStatus {
        HealthStatus::Healthy
    }

    fn get_port(&self) -> u16 {
        5432
    }

    fn get_image(&self) -> &str {
        "postgres:15-alpine"
    }
}

#[derive(Debug, Clone)]
pub struct OllamaService;

impl OllamaService {
    pub fn new() -> Self {
        Self
    }
}

impl ServicePlugin for OllamaService {
    fn name(&self) -> &str {
        "ollama"
    }

    fn start(&self) -> Result<ServiceHandle> {
        Ok(ServiceHandle {
            id: uuid::Uuid::new_v4().to_string(),
            service_name: self.name().to_string(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("image".to_string(), "ollama/ollama:latest".to_string());
                m.insert("port".to_string(), "11434".to_string());
                m
            },
            state: ServiceState::Running,
            started_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    fn stop(&self, _: ServiceHandle) -> Result<()> {
        Ok(())
    }

    fn health_check(&self, _: &ServiceHandle) -> HealthStatus {
        HealthStatus::Healthy
    }

    fn get_port(&self) -> u16 {
        11434
    }

    fn get_image(&self) -> &str {
        "ollama/ollama:latest"
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

    pub fn stop_all(&mut self) -> Result<()> {
        let ids: Vec<_> = self.active_services.keys().cloned().collect();
        for id in ids {
            self.stop_service(&id)?;
        }
        Ok(())
    }

    pub fn get_service_status(&self, id: &str) -> Option<&ServiceHandle> {
        self.active_services.get(id)
    }

    pub fn list_running_services(&self) -> Vec<(&str, &ServiceHandle)> {
        self.active_services
            .iter()
            .map(|(_, h)| (h.service_name.as_str(), h))
            .collect()
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TestExecutor {
    registry: ServiceRegistry,
    results: Vec<TestResult>,
}

impl TestExecutor {
    pub fn new(registry: ServiceRegistry) -> Self {
        Self {
            registry,
            results: Vec::new(),
        }
    }

    pub fn execute_test(&mut self, step: TestStep, service_handle: Option<&ServiceHandle>) -> TestResult {
        let start = std::time::Instant::now();

        let output = if let Some(service) = service_handle {
            format!(
                "Service: {} ({}:{})",
                service.service_name,
                service.metadata.get("image").unwrap_or(&"unknown".to_string()),
                self.registry
                    .plugins
                    .get(&service.service_name)
                    .map(|p| p.get_port().to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            )
        } else {
            String::from("No service")
        };

        let passed = match &step.expected_output {
            Some(expected) => output.contains(expected),
            None => true,
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        TestResult {
            name: step.name,
            passed,
            duration_ms,
            output,
            error: if passed { None } else { Some("Output mismatch".to_string()) },
        }
    }

    pub fn execute_steps(&mut self, steps: Vec<TestStep>) -> Vec<TestResult> {
        let mut results = Vec::new();
        let handles: Vec<ServiceHandle> = self.registry.list_running_services()
            .into_iter()
            .map(|(_, h)| h.clone())
            .collect();
        let first_handle = handles.first();

        for step in steps {
            let result = self.execute_test(step, first_handle);
            results.push(result);
        }

        self.results.extend(results.clone());
        results
    }

    pub fn get_results(&self) -> &[TestResult] {
        &self.results
    }

    pub fn summary(&self) -> (usize, usize, u64) {
        let total = self.results.len();
        let passed = self.results.iter().filter(|r| r.passed).count();
        let total_duration: u64 = self.results.iter().map(|r| r.duration_ms).sum();
        (passed, total, total_duration)
    }

    pub fn list_running_services(&self) -> Vec<(&str, &ServiceHandle)> {
        self.registry.list_running_services()
    }

    pub fn stop_all(&mut self) -> Result<()> {
        self.registry.stop_all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_registry_creation() {
        let mut registry = ServiceRegistry::new();
        registry.load_ggen_services().unwrap();
        assert_eq!(registry.plugins.len(), 3);
    }

    #[test]
    fn test_service_startup_and_health() {
        let mut registry = ServiceRegistry::new();
        registry.load_ggen_services().unwrap();

        let h = registry.start_service("surrealdb").unwrap();
        assert_eq!(h.service_name, "surrealdb");
        assert_eq!(h.state, ServiceState::Running);

        let plugin = registry.plugins.get("surrealdb").unwrap();
        assert_eq!(plugin.health_check(&h), HealthStatus::Healthy);

        registry.stop_service(&h.id).unwrap();
    }

    #[test]
    fn test_test_executor() {
        let mut registry = ServiceRegistry::new();
        registry.load_ggen_services().unwrap();

        let _h = registry.start_service("postgres").unwrap();

        let mut executor = TestExecutor::new(registry);

        let steps = vec![
            TestStep {
                name: "test-postgres".to_string(),
                command: vec!["psql".to_string()],
                expected_output: Some("postgres".to_string()),
                timeout_ms: 5000,
                retries: 0,
            },
        ];

        let results = executor.execute_steps(steps);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_multi_service_execution() {
        let mut registry = ServiceRegistry::new();
        registry.load_ggen_services().unwrap();

        let _h1 = registry.start_service("surrealdb").unwrap();
        let _h2 = registry.start_service("postgres").unwrap();

        assert_eq!(registry.list_running_services().len(), 2);

        registry.stop_all().unwrap();
        assert_eq!(registry.active_services.len(), 0);
    }

    #[test]
    fn test_service_port_and_image() {
        let sdb = SurrealDbService::new();
        let pg = PostgresService::new();
        let ollama = OllamaService::new();

        assert_eq!(sdb.get_port(), 8000);
        assert_eq!(pg.get_port(), 5432);
        assert_eq!(ollama.get_port(), 11434);

        assert_eq!(sdb.get_image(), "surrealdb:latest");
        assert_eq!(pg.get_image(), "postgres:15-alpine");
        assert_eq!(ollama.get_image(), "ollama/ollama:latest");
    }
}
