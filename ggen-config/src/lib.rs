use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Invalid environment: {0}")]
    InvalidEnvironment(String),
}

pub type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &str {
        match self {
            Environment::Development => "development",
            Environment::Staging => "staging",
            Environment::Production => "production",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(Environment::Development),
            "staging" | "stage" => Ok(Environment::Staging),
            "production" | "prod" => Ok(Environment::Production),
            _ => Err(ConfigError::InvalidEnvironment(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub image: String,
    pub port: u16,
    pub version: String,
    pub dependencies: Vec<String>,
    pub environment: HashMap<String, String>,
    pub resources: ResourceConfig,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub cpu_cores: f32,
    pub memory_mb: u32,
    pub disk_gb: u32,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            cpu_cores: 0.5,
            memory_mb: 256,
            disk_gb: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceGroupConfig {
    pub name: String,
    pub description: String,
    pub environment: String,
    pub services: HashMap<String, ServiceConfig>,
    pub constraints: GroupConstraints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupConstraints {
    pub max_parallel_starts: usize,
    pub startup_timeout_ms: u64,
    pub health_check_interval_ms: u64,
    pub auto_restart: bool,
}

impl Default for GroupConstraints {
    fn default() -> Self {
        Self {
            max_parallel_starts: 3,
            startup_timeout_ms: 30000,
            health_check_interval_ms: 5000,
            auto_restart: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub version: String,
    pub environments: HashMap<String, EnvironmentConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    pub name: String,
    pub registry: String,
    pub namespace: String,
    pub groups: Vec<String>,
}

pub struct ConfigManager {
    base_path: PathBuf,
}

impl ConfigManager {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Result<Self> {
        let path = base_path.as_ref().to_path_buf();
        if !path.exists() {
            fs::create_dir_all(&path).map_err(|e| ConfigError::IoError(e.to_string()))?;
        }

        Ok(Self { base_path: path })
    }

    pub fn load_group_config(&self, name: &str, env: Environment) -> Result<ServiceGroupConfig> {
        let filename = format!("{}-{}.json", name, env.as_str());
        let path = self.base_path.join(&filename);

        if !path.exists() {
            return Err(ConfigError::NotFound(filename));
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;

        let config: ServiceGroupConfig = serde_json::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        self.validate_group(&config)?;
        Ok(config)
    }

    pub fn save_group_config(&self, config: &ServiceGroupConfig) -> Result<()> {
        self.validate_group(config)?;

        let filename = format!("{}-{}.json", config.name, config.environment);
        let path = self.base_path.join(&filename);

        let content = serde_json::to_string_pretty(config)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        fs::write(&path, content)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;

        Ok(())
    }

    pub fn load_project_config(&self) -> Result<ProjectConfig> {
        let path = self.base_path.join("project.json");

        if !path.exists() {
            return Err(ConfigError::NotFound("project.json".to_string()));
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;

        let config: ProjectConfig = serde_json::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        Ok(config)
    }

    pub fn save_project_config(&self, config: &ProjectConfig) -> Result<()> {
        let path = self.base_path.join("project.json");

        let content = serde_json::to_string_pretty(config)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        fs::write(&path, content)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;

        Ok(())
    }

    pub fn list_configs(&self) -> Result<Vec<String>> {
        let entries = fs::read_dir(&self.base_path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;

        let mut configs = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| ConfigError::IoError(e.to_string()))?;
            let filename = entry.file_name();
            let name = filename.to_string_lossy();

            if name.ends_with(".json") && name != "project.json" {
                configs.push(name.to_string());
            }
        }

        Ok(configs)
    }

    pub fn validate_group(&self, config: &ServiceGroupConfig) -> Result<()> {
        if config.name.is_empty() {
            return Err(ConfigError::ValidationError("Group name cannot be empty".to_string()));
        }

        if config.services.is_empty() {
            return Err(ConfigError::ValidationError(
                format!("Group {} has no services", config.name),
            ));
        }

        for (service_id, service) in &config.services {
            if service.name.is_empty() {
                return Err(ConfigError::ValidationError(
                    format!("Service {} has empty name", service_id),
                ));
            }

            if service.image.is_empty() {
                return Err(ConfigError::ValidationError(
                    format!("Service {} has empty image", service_id),
                ));
            }

            if service.port == 0 {
                return Err(ConfigError::ValidationError(
                    format!("Service {} has invalid port", service_id),
                ));
            }

            for dep in &service.dependencies {
                if !config.services.contains_key(dep) {
                    return Err(ConfigError::ValidationError(
                        format!("Service {} depends on missing service {}", service_id, dep),
                    ));
                }
            }
        }

        Ok(())
    }

    pub fn merge_config(&self, base: &ServiceGroupConfig, override_: &ServiceGroupConfig) -> Result<ServiceGroupConfig> {
        let mut merged = base.clone();

        for (service_id, override_service) in &override_.services {
            if let Some(base_service) = merged.services.get_mut(service_id) {
                merge_service_config(base_service, override_service);
            } else {
                merged.services.insert(service_id.clone(), override_service.clone());
            }
        }

        self.validate_group(&merged)?;
        Ok(merged)
    }
}

fn merge_service_config(base: &mut ServiceConfig, override_: &ServiceConfig) {
    if !override_.image.is_empty() {
        base.image = override_.image.clone();
    }
    if override_.port != 0 {
        base.port = override_.port;
    }
    if !override_.version.is_empty() {
        base.version = override_.version.clone();
    }

    for (key, value) in &override_.environment {
        base.environment.insert(key.clone(), value.clone());
    }

    for (key, value) in &override_.metadata {
        base.metadata.insert(key.clone(), value.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_service() -> ServiceConfig {
        ServiceConfig {
            name: "test-service".to_string(),
            image: "test:latest".to_string(),
            port: 8080,
            version: "1.0.0".to_string(),
            dependencies: vec![],
            environment: HashMap::new(),
            resources: ResourceConfig::default(),
            metadata: HashMap::new(),
        }
    }

    fn create_test_group() -> ServiceGroupConfig {
        let mut services = HashMap::new();
        services.insert("test".to_string(), create_test_service());

        ServiceGroupConfig {
            name: "test-group".to_string(),
            description: "Test group".to_string(),
            environment: "development".to_string(),
            services,
            constraints: GroupConstraints::default(),
        }
    }

    #[test]
    fn test_environment_parsing() {
        assert_eq!(Environment::from_str("dev").unwrap(), Environment::Development);
        assert_eq!(Environment::from_str("staging").unwrap(), Environment::Staging);
        assert_eq!(Environment::from_str("prod").unwrap(), Environment::Production);
    }

    #[test]
    fn test_config_manager_creation() {
        let temp = TempDir::new().unwrap();
        let manager = ConfigManager::new(temp.path()).unwrap();
        assert!(manager.list_configs().unwrap().is_empty());
    }

    #[test]
    fn test_save_and_load_group() {
        let temp = TempDir::new().unwrap();
        let manager = ConfigManager::new(temp.path()).unwrap();

        let config = create_test_group();
        assert!(manager.save_group_config(&config).is_ok());

        let loaded = manager
            .load_group_config("test-group", Environment::Development)
            .unwrap();
        assert_eq!(loaded.name, config.name);
    }

    #[test]
    fn test_config_validation() {
        let temp = TempDir::new().unwrap();
        let manager = ConfigManager::new(temp.path()).unwrap();

        let mut invalid = create_test_group();
        invalid.services.clear();

        assert!(manager.validate_group(&invalid).is_err());
    }

    #[test]
    fn test_list_configs() {
        let temp = TempDir::new().unwrap();
        let manager = ConfigManager::new(temp.path()).unwrap();

        let config = create_test_group();
        manager.save_group_config(&config).unwrap();

        let configs = manager.list_configs().unwrap();
        assert!(configs.len() > 0);
    }

    #[test]
    fn test_merge_configurations() {
        let temp = TempDir::new().unwrap();
        let manager = ConfigManager::new(temp.path()).unwrap();

        let mut base = create_test_group();
        let mut override_ = create_test_group();

        if let Some(service) = base.services.get_mut("test") {
            service.version = "1.0.0".to_string();
        }

        if let Some(service) = override_.services.get_mut("test") {
            service.version = "2.0.0".to_string();
            service.port = 9090;
        }

        let merged = manager.merge_config(&base, &override_).unwrap();
        if let Some(service) = merged.services.get("test") {
            assert_eq!(service.version, "2.0.0");
            assert_eq!(service.port, 9090);
        }
    }
}
