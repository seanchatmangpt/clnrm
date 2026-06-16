//! Service templates for common databases and services
//!
//! Provides pre-configured templates that can be extended and customized.

use crate::error::Result;
use crate::service::definition::{ImageRef, ResourceSpec, ServiceDefinition};
use crate::service::health::{HealthCheck, HttpScheme, ReadinessProbe};
use crate::service::network::{PortMapping, Protocol};
use std::collections::HashMap;

/// Service template collection
pub struct ServiceTemplates {
    templates: HashMap<String, ServiceDefinition>,
}

impl ServiceTemplates {
    /// Create new template collection with built-in templates
    pub fn new() -> Self {
        let mut templates = HashMap::new();

        // Add built-in templates
        templates.insert("surrealdb".to_string(), Self::surrealdb_template());
        templates.insert("postgresql".to_string(), Self::postgresql_template());
        templates.insert("mysql".to_string(), Self::mysql_template());
        templates.insert("redis".to_string(), Self::redis_template());
        templates.insert("mongodb".to_string(), Self::mongodb_template());

        Self { templates }
    }

    /// Get template by name
    pub fn get(&self, name: &str) -> Option<&ServiceDefinition> {
        self.templates.get(name)
    }

    /// Register custom template
    pub fn register(&mut self, name: String, template: ServiceDefinition) {
        self.templates.insert(name, template);
    }

    /// SurrealDB template
    fn surrealdb_template() -> ServiceDefinition {
        let mut env = HashMap::new();
        env.insert("SURREAL_USER".to_string(), "root".to_string());
        env.insert("SURREAL_PASS".to_string(), "root".to_string());
        env.insert("SURREAL_PATH".to_string(), "memory".to_string());

        ServiceDefinition {
            name: "surrealdb".to_string(),
            image: ImageRef {
                registry: Some("docker.io".to_string()),
                repository: "surrealdb/surrealdb".to_string(),
                tag: "v1.0.0".to_string(),
                digest: None,
            },
            command: Some(vec![
                "surreal".to_string(),
                "start".to_string(),
                "--bind".to_string(),
                "0.0.0.0:8000".to_string(),
            ]),
            args: None,
            env,
            ports: vec![PortMapping {
                container: 8000,
                host: None,
                protocol: Protocol::Tcp,
            }],
            volumes: vec![],
            health_check: Some(HealthCheck::Http {
                path: "/health".to_string(),
                port: 8000,
                scheme: HttpScheme::Http,
                interval: "5s".to_string(),
                timeout: "3s".to_string(),
                retries: 3,
            }),
            resources: ResourceSpec {
                memory_limit: Some("512M".to_string()),
                memory_swap: None,
                cpu_limit: Some(1.0),
                cpu_shares: None,
                pids_limit: Some(100),
            },
            depends_on: vec![],
            readiness: Some(ReadinessProbe::Tcp {
                port: 8000,
                initial_delay: "2s".to_string(),
                timeout: "30s".to_string(),
            }),
            extends: None,
        }
    }

    /// PostgreSQL template
    fn postgresql_template() -> ServiceDefinition {
        let mut env = HashMap::new();
        env.insert("POSTGRES_USER".to_string(), "postgres".to_string());
        env.insert("POSTGRES_PASSWORD".to_string(), "postgres".to_string());
        env.insert("POSTGRES_DB".to_string(), "testdb".to_string());

        ServiceDefinition {
            name: "postgresql".to_string(),
            image: ImageRef {
                registry: Some("docker.io".to_string()),
                repository: "library/postgres".to_string(),
                tag: "14".to_string(),
                digest: None,
            },
            command: None,
            args: None,
            env,
            ports: vec![PortMapping {
                container: 5432,
                host: None,
                protocol: Protocol::Tcp,
            }],
            volumes: vec![],
            health_check: Some(HealthCheck::Exec {
                command: vec![
                    "pg_isready".to_string(),
                    "-U".to_string(),
                    "postgres".to_string(),
                ],
                interval: "5s".to_string(),
                timeout: "3s".to_string(),
                retries: 3,
            }),
            resources: ResourceSpec {
                memory_limit: Some("512M".to_string()),
                memory_swap: None,
                cpu_limit: Some(1.0),
                cpu_shares: None,
                pids_limit: Some(100),
            },
            depends_on: vec![],
            readiness: Some(ReadinessProbe::Tcp {
                port: 5432,
                initial_delay: "3s".to_string(),
                timeout: "30s".to_string(),
            }),
            extends: None,
        }
    }

    /// MySQL template
    fn mysql_template() -> ServiceDefinition {
        let mut env = HashMap::new();
        env.insert("MYSQL_ROOT_PASSWORD".to_string(), "root".to_string());
        env.insert("MYSQL_DATABASE".to_string(), "testdb".to_string());
        env.insert("MYSQL_USER".to_string(), "user".to_string());
        env.insert("MYSQL_PASSWORD".to_string(), "password".to_string());

        ServiceDefinition {
            name: "mysql".to_string(),
            image: ImageRef {
                registry: Some("docker.io".to_string()),
                repository: "library/mysql".to_string(),
                tag: "8.0".to_string(),
                digest: None,
            },
            command: None,
            args: None,
            env,
            ports: vec![PortMapping {
                container: 3306,
                host: None,
                protocol: Protocol::Tcp,
            }],
            volumes: vec![],
            health_check: Some(HealthCheck::Exec {
                command: vec![
                    "mysqladmin".to_string(),
                    "ping".to_string(),
                    "-h".to_string(),
                    "localhost".to_string(),
                ],
                interval: "5s".to_string(),
                timeout: "3s".to_string(),
                retries: 3,
            }),
            resources: ResourceSpec {
                memory_limit: Some("512M".to_string()),
                memory_swap: None,
                cpu_limit: Some(1.0),
                cpu_shares: None,
                pids_limit: Some(100),
            },
            depends_on: vec![],
            readiness: Some(ReadinessProbe::Tcp {
                port: 3306,
                initial_delay: "5s".to_string(),
                timeout: "30s".to_string(),
            }),
            extends: None,
        }
    }

    /// Redis template
    fn redis_template() -> ServiceDefinition {
        ServiceDefinition {
            name: "redis".to_string(),
            image: ImageRef {
                registry: Some("docker.io".to_string()),
                repository: "library/redis".to_string(),
                tag: "7".to_string(),
                digest: None,
            },
            command: None,
            args: None,
            env: HashMap::new(),
            ports: vec![PortMapping {
                container: 6379,
                host: None,
                protocol: Protocol::Tcp,
            }],
            volumes: vec![],
            health_check: Some(HealthCheck::Exec {
                command: vec!["redis-cli".to_string(), "ping".to_string()],
                interval: "5s".to_string(),
                timeout: "3s".to_string(),
                retries: 3,
            }),
            resources: ResourceSpec {
                memory_limit: Some("256M".to_string()),
                memory_swap: None,
                cpu_limit: Some(0.5),
                cpu_shares: None,
                pids_limit: Some(50),
            },
            depends_on: vec![],
            readiness: Some(ReadinessProbe::Tcp {
                port: 6379,
                initial_delay: "1s".to_string(),
                timeout: "10s".to_string(),
            }),
            extends: None,
        }
    }

    /// MongoDB template
    fn mongodb_template() -> ServiceDefinition {
        let mut env = HashMap::new();
        env.insert("MONGO_INITDB_ROOT_USERNAME".to_string(), "root".to_string());
        env.insert("MONGO_INITDB_ROOT_PASSWORD".to_string(), "root".to_string());

        ServiceDefinition {
            name: "mongodb".to_string(),
            image: ImageRef {
                registry: Some("docker.io".to_string()),
                repository: "library/mongo".to_string(),
                tag: "6.0".to_string(),
                digest: None,
            },
            command: None,
            args: None,
            env,
            ports: vec![PortMapping {
                container: 27017,
                host: None,
                protocol: Protocol::Tcp,
            }],
            volumes: vec![],
            health_check: Some(HealthCheck::Exec {
                command: vec![
                    "mongosh".to_string(),
                    "--eval".to_string(),
                    "db.adminCommand('ping')".to_string(),
                ],
                interval: "5s".to_string(),
                timeout: "3s".to_string(),
                retries: 3,
            }),
            resources: ResourceSpec {
                memory_limit: Some("512M".to_string()),
                memory_swap: None,
                cpu_limit: Some(1.0),
                cpu_shares: None,
                pids_limit: Some(100),
            },
            depends_on: vec![],
            readiness: Some(ReadinessProbe::Tcp {
                port: 27017,
                initial_delay: "3s".to_string(),
                timeout: "30s".to_string(),
            }),
            extends: None,
        }
    }
}

impl Default for ServiceTemplates {
    fn default() -> Self {
        Self::new()
    }
}

/// Save templates to TOML file for reference
pub fn save_templates_to_toml() -> Result<String> {
    let templates = ServiceTemplates::new();

    let mut toml_content = String::new();
    toml_content.push_str("# Service Templates for gVisor Backend\n\n");

    for (name, template) in &templates.templates {
        toml_content.push_str(&format!("# {} Template\n", name));
        toml_content.push_str(&format!("[template.{}]\n", name));
        toml_content.push_str("plugin = \"gvisor_container\"\n");
        toml_content.push_str(&format!("image = \"{}\"\n", template.image));

        if let Some(ref command) = template.command {
            toml_content.push_str(&format!("command = {:?}\n", command));
        }

        if !template.env.is_empty() {
            toml_content.push_str(&format!("\n[template.{}.env]\n", name));
            for (k, v) in &template.env {
                toml_content.push_str(&format!("{} = \"{}\"\n", k, v));
            }
        }

        toml_content.push('\n');
    }

    Ok(toml_content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surrealdb_template() {
        let templates = ServiceTemplates::new();
        let template = templates.get("surrealdb").unwrap();

        assert_eq!(template.name, "surrealdb");
        assert_eq!(template.ports.len(), 1);
        assert_eq!(template.ports[0].container, 8000);
        assert!(template.health_check.is_some());
        assert!(template.readiness.is_some());
    }

    #[test]
    fn test_all_templates_valid() {
        let templates = ServiceTemplates::new();

        for template in templates.templates.values() {
            assert!(template.validate().is_ok());
        }
    }
}
