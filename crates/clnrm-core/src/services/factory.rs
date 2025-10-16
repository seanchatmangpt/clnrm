//! Service factory for creating plugins from configuration
//!
//! Provides centralized plugin creation from TOML ServiceConfig,
//! handling type-specific configuration and validation.

use crate::cleanroom::ServicePlugin;
use crate::config::ServiceConfig;
use crate::error::{CleanroomError, Result};
use crate::services::{
    generic::GenericContainerPlugin,
    ollama::{OllamaConfig, OllamaPlugin},
    surrealdb::SurrealDbPlugin,
    tgi::{TgiConfig, TgiPlugin},
    vllm::{VllmConfig, VllmPlugin},
};

/// Service factory for creating plugins from configuration
pub struct ServiceFactory;

impl ServiceFactory {
    /// Create a service plugin from configuration
    ///
    /// # Arguments
    ///
    /// * `name` - Service name identifier
    /// * `config` - Service configuration from TOML
    ///
    /// # Returns
    ///
    /// A boxed `ServicePlugin` implementation matching the service type
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Service type is unknown or unsupported
    /// - Required configuration fields are missing
    /// - Configuration values are invalid
    ///
    /// # Example
    ///
    /// ```no_run
    /// use clnrm_core::services::factory::ServiceFactory;
    /// use clnrm_core::config::ServiceConfig;
    /// use std::collections::HashMap;
    ///
    /// let mut config = ServiceConfig {
    ///     r#type: "surrealdb".to_string(),
    ///     plugin: "surrealdb".to_string(),
    ///     image: Some("surrealdb/surrealdb:latest".to_string()),
    ///     env: None,
    ///     ports: None,
    ///     volumes: None,
    ///     health_check: None,
    /// };
    ///
    /// let plugin = ServiceFactory::create_plugin("my_db", &config)?;
    /// # Ok::<(), clnrm_core::error::CleanroomError>(())
    /// ```
    pub fn create_plugin(name: &str, config: &ServiceConfig) -> Result<Box<dyn ServicePlugin>> {
        // Validate configuration before processing
        config.validate()?;

        // Determine service type from plugin field (normalized to lowercase)
        let service_type = config.plugin.to_lowercase();

        match service_type.as_str() {
            "surrealdb" => Self::create_surrealdb_plugin(name, config),
            "generic_container" => Self::create_generic_plugin(name, config),
            "ollama" => Self::create_ollama_plugin(name, config),
            "tgi" => Self::create_tgi_plugin(name, config),
            "vllm" => Self::create_vllm_plugin(name, config),
            _ => Err(CleanroomError::configuration_error(format!(
                "Unknown service type: '{}'. Supported types: surrealdb, generic_container, ollama, tgi, vllm",
                config.plugin
            ))),
        }
    }

    /// Create a SurrealDB plugin from configuration
    fn create_surrealdb_plugin(
        _name: &str,
        config: &ServiceConfig,
    ) -> Result<Box<dyn ServicePlugin>> {
        // Extract credentials from environment variables or config
        let username = Self::get_env_or_config(config, "SURREALDB_USER", "username")
            .unwrap_or_else(|| "root".to_string());

        let password = Self::get_env_or_config(config, "SURREALDB_PASS", "password")
            .unwrap_or_else(|| "root".to_string());

        // Extract strict mode flag (default: false)
        let strict = Self::get_config_bool(config, "strict").unwrap_or(false);

        // Create plugin with credentials
        let plugin = SurrealDbPlugin::with_credentials(&username, &password).with_strict(strict);

        Ok(Box::new(plugin))
    }

    /// Create a generic container plugin from configuration
    fn create_generic_plugin(name: &str, config: &ServiceConfig) -> Result<Box<dyn ServicePlugin>> {
        // Image is required for generic containers
        let image = config.image.as_ref().ok_or_else(|| {
            CleanroomError::configuration_error(
                "Generic container requires 'image' field in configuration",
            )
        })?;

        // Create base plugin
        let mut plugin = GenericContainerPlugin::new(name, image);

        // Add environment variables if present
        if let Some(ref env_vars) = config.env {
            for (key, value) in env_vars.iter() {
                plugin = plugin.with_env(key, value);
            }
        }

        // Add port mappings if present
        if let Some(ref ports) = config.ports {
            for port in ports {
                plugin = plugin.with_port(*port);
            }
        }

        // Add volume mounts if present
        if let Some(ref volumes) = config.volumes {
            for volume in volumes {
                plugin = plugin
                    .with_volume(
                        &volume.host_path,
                        &volume.container_path,
                        volume.read_only.unwrap_or(false),
                    )
                    .map_err(|e| {
                        CleanroomError::configuration_error(format!(
                            "Invalid volume configuration: {}",
                            e
                        ))
                    })?;
            }
        }

        Ok(Box::new(plugin))
    }

    /// Create an Ollama plugin from configuration
    fn create_ollama_plugin(name: &str, config: &ServiceConfig) -> Result<Box<dyn ServicePlugin>> {
        // Extract endpoint (required)
        let endpoint = Self::get_config_string(config, "endpoint").ok_or_else(|| {
            CleanroomError::configuration_error(
                "Ollama service requires 'endpoint' in env configuration",
            )
        })?;

        // Extract default model (required)
        let default_model = Self::get_config_string(config, "default_model")
            .or_else(|| Self::get_config_string(config, "model"))
            .ok_or_else(|| {
                CleanroomError::configuration_error(
                    "Ollama service requires 'default_model' or 'model' in env configuration",
                )
            })?;

        // Extract timeout (optional, default: 60 seconds)
        let timeout_seconds = Self::get_config_string(config, "timeout_seconds")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(60);

        let ollama_config = OllamaConfig {
            endpoint,
            default_model,
            timeout_seconds,
        };

        let plugin = OllamaPlugin::new(name, ollama_config);
        Ok(Box::new(plugin))
    }

    /// Create a TGI (Text Generation Inference) plugin from configuration
    fn create_tgi_plugin(name: &str, config: &ServiceConfig) -> Result<Box<dyn ServicePlugin>> {
        // Extract endpoint (required)
        let endpoint = Self::get_config_string(config, "endpoint").ok_or_else(|| {
            CleanroomError::configuration_error(
                "TGI service requires 'endpoint' in env configuration",
            )
        })?;

        // Extract model_id (required)
        let model_id = Self::get_config_string(config, "model_id")
            .or_else(|| Self::get_config_string(config, "model"))
            .ok_or_else(|| {
                CleanroomError::configuration_error(
                    "TGI service requires 'model_id' or 'model' in env configuration",
                )
            })?;

        // Extract optional configuration
        let max_total_tokens =
            Self::get_config_string(config, "max_total_tokens").and_then(|s| s.parse::<u32>().ok());

        let max_input_length =
            Self::get_config_string(config, "max_input_length").and_then(|s| s.parse::<u32>().ok());

        let max_batch_prefill_tokens = Self::get_config_string(config, "max_batch_prefill_tokens")
            .and_then(|s| s.parse::<u32>().ok());

        let max_concurrent_requests = Self::get_config_string(config, "max_concurrent_requests")
            .and_then(|s| s.parse::<u32>().ok());

        let max_batch_total_tokens = Self::get_config_string(config, "max_batch_total_tokens")
            .and_then(|s| s.parse::<u32>().ok());

        let timeout_seconds = Self::get_config_string(config, "timeout_seconds")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(60);

        let tgi_config = TgiConfig {
            endpoint,
            model_id,
            max_total_tokens,
            max_input_length,
            max_batch_prefill_tokens,
            max_concurrent_requests,
            max_batch_total_tokens,
            timeout_seconds,
        };

        let plugin = TgiPlugin::new(name, tgi_config);
        Ok(Box::new(plugin))
    }

    /// Create a vLLM plugin from configuration
    fn create_vllm_plugin(name: &str, config: &ServiceConfig) -> Result<Box<dyn ServicePlugin>> {
        // Extract endpoint (required)
        let endpoint = Self::get_config_string(config, "endpoint").ok_or_else(|| {
            CleanroomError::configuration_error(
                "vLLM service requires 'endpoint' in env configuration",
            )
        })?;

        // Extract model (required)
        let model = Self::get_config_string(config, "model").ok_or_else(|| {
            CleanroomError::configuration_error(
                "vLLM service requires 'model' in env configuration",
            )
        })?;

        // Extract optional configuration
        let max_num_seqs =
            Self::get_config_string(config, "max_num_seqs").and_then(|s| s.parse::<u32>().ok());

        let max_model_len =
            Self::get_config_string(config, "max_model_len").and_then(|s| s.parse::<u32>().ok());

        let tensor_parallel_size = Self::get_config_string(config, "tensor_parallel_size")
            .and_then(|s| s.parse::<u32>().ok());

        let gpu_memory_utilization = Self::get_config_string(config, "gpu_memory_utilization")
            .and_then(|s| s.parse::<f32>().ok());

        let enable_prefix_caching = Self::get_config_bool(config, "enable_prefix_caching");

        let timeout_seconds = Self::get_config_string(config, "timeout_seconds")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(60);

        let vllm_config = VllmConfig {
            endpoint,
            model,
            max_num_seqs,
            max_model_len,
            tensor_parallel_size,
            gpu_memory_utilization,
            enable_prefix_caching,
            timeout_seconds,
        };

        let plugin = VllmPlugin::new(name, vllm_config);
        Ok(Box::new(plugin))
    }

    // Helper functions for extracting configuration values

    /// Get value from environment variable or config env map
    fn get_env_or_config(
        config: &ServiceConfig,
        env_var: &str,
        config_key: &str,
    ) -> Option<String> {
        // First try environment variable
        std::env::var(env_var)
            .ok()
            // Then try config env map
            .or_else(|| {
                config
                    .env
                    .as_ref()
                    .and_then(|env_map| env_map.get(config_key).cloned())
            })
    }

    /// Get string value from config env map
    fn get_config_string(config: &ServiceConfig, key: &str) -> Option<String> {
        config
            .env
            .as_ref()
            .and_then(|env_map| env_map.get(key).cloned())
    }

    /// Get boolean value from config env map
    fn get_config_bool(config: &ServiceConfig, key: &str) -> Option<bool> {
        Self::get_config_string(config, key).and_then(|s| s.parse::<bool>().ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_create_surrealdb_plugin() -> Result<()> {
        let config = ServiceConfig {
            r#type: "database".to_string(),
            plugin: "surrealdb".to_string(),
            image: Some("surrealdb/surrealdb:latest".to_string()),
            env: None,
            ports: None,
            volumes: None,
            health_check: None,
            username: None,
            password: None,
            strict: None,
        };

        let plugin = ServiceFactory::create_plugin("test_db", &config)?;
        assert_eq!(plugin.name(), "surrealdb");
        Ok(())
    }

    #[test]
    fn test_create_surrealdb_plugin_with_credentials() -> Result<()> {
        let mut env = HashMap::new();
        env.insert("username".to_string(), "admin".to_string());
        env.insert("password".to_string(), "secret".to_string());
        env.insert("strict".to_string(), "true".to_string());

        let config = ServiceConfig {
            r#type: "database".to_string(),
            plugin: "surrealdb".to_string(),
            image: Some("surrealdb/surrealdb:latest".to_string()),
            env: Some(env),
            ports: None,
            volumes: None,
            health_check: None,
            username: None,
            password: None,
            strict: None,
        };

        let plugin = ServiceFactory::create_plugin("test_db", &config)?;
        assert_eq!(plugin.name(), "surrealdb");
        Ok(())
    }

    #[test]
    fn test_create_generic_plugin() -> Result<()> {
        let config = ServiceConfig {
            r#type: "container".to_string(),
            plugin: "generic_container".to_string(),
            image: Some("alpine:latest".to_string()),
            env: None,
            ports: None,
            volumes: None,
            health_check: None,
            username: None,
            password: None,
            strict: None,
        };

        let plugin = ServiceFactory::create_plugin("test_container", &config)?;
        assert_eq!(plugin.name(), "test_container");
        Ok(())
    }

    #[test]
    fn test_create_generic_plugin_with_env_and_ports() -> Result<()> {
        let mut env = HashMap::new();
        env.insert("KEY1".to_string(), "value1".to_string());
        env.insert("KEY2".to_string(), "value2".to_string());

        let config = ServiceConfig {
            r#type: "container".to_string(),
            plugin: "generic_container".to_string(),
            image: Some("nginx:latest".to_string()),
            env: Some(env),
            ports: Some(vec![8080, 8443]),
            volumes: None,
            health_check: None,
            username: None,
            password: None,
            strict: None,
        };

        let plugin = ServiceFactory::create_plugin("nginx", &config)?;
        assert_eq!(plugin.name(), "nginx");
        Ok(())
    }

    #[test]
    fn test_create_ollama_plugin() -> Result<()> {
        let mut env = HashMap::new();
        env.insert("endpoint".to_string(), "http://localhost:11434".to_string());
        env.insert("default_model".to_string(), "llama2".to_string());
        env.insert("timeout_seconds".to_string(), "120".to_string());

        let config = ServiceConfig {
            r#type: "ollama".to_string(), // Changed from "ai_service"
            plugin: "ollama".to_string(),
            image: None,
            env: Some(env),
            ports: None,
            volumes: None,
            health_check: None,
            username: None,
            password: None,
            strict: None,
        };

        let plugin = ServiceFactory::create_plugin("ollama_service", &config)?;
        assert_eq!(plugin.name(), "ollama_service");
        Ok(())
    }

    #[test]
    fn test_create_tgi_plugin() -> Result<()> {
        let mut env = HashMap::new();
        env.insert("endpoint".to_string(), "http://localhost:8080".to_string());
        env.insert(
            "model_id".to_string(),
            "microsoft/DialoGPT-medium".to_string(),
        );
        env.insert("max_total_tokens".to_string(), "2048".to_string());

        let config = ServiceConfig {
            r#type: "network_service".to_string(), // Changed from "ai_service"
            plugin: "tgi".to_string(),
            image: None,
            env: Some(env),
            ports: None,
            volumes: None,
            health_check: None,
            username: None,
            password: None,
            strict: None,
        };

        let plugin = ServiceFactory::create_plugin("tgi_service", &config)?;
        assert_eq!(plugin.name(), "tgi_service");
        Ok(())
    }

    #[test]
    fn test_create_vllm_plugin() -> Result<()> {
        let mut env = HashMap::new();
        env.insert("endpoint".to_string(), "http://localhost:8000".to_string());
        env.insert("model".to_string(), "facebook/opt-125m".to_string());
        env.insert("max_num_seqs".to_string(), "100".to_string());

        let config = ServiceConfig {
            r#type: "network_service".to_string(), // Changed from "ai_service"
            plugin: "vllm".to_string(),
            image: None,
            env: Some(env),
            ports: None,
            volumes: None,
            health_check: None,
            username: None,
            password: None,
            strict: None,
        };

        let plugin = ServiceFactory::create_plugin("vllm_service", &config)?;
        assert_eq!(plugin.name(), "vllm_service");
        Ok(())
    }

    #[test]
    fn test_unknown_service_type_returns_error() {
        let config = ServiceConfig {
            r#type: "unknown".to_string(),
            plugin: "unknown_plugin".to_string(),
            image: Some("some:image".to_string()),
            env: None,
            ports: None,
            volumes: None,
            health_check: None,
            username: None,
            password: None,
            strict: None,
        };

        let result = ServiceFactory::create_plugin("test", &config);
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(e.message.contains("Unknown service type"));
        }
    }

    #[test]
    fn test_generic_container_without_image_returns_error() {
        let config = ServiceConfig {
            r#type: "container".to_string(),
            plugin: "generic_container".to_string(),
            image: None, // Missing required field
            env: None,
            ports: None,
            volumes: None,
            health_check: None,
            username: None,
            password: None,
            strict: None,
        };

        let result = ServiceFactory::create_plugin("test", &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_ollama_without_endpoint_returns_error() {
        let mut env = HashMap::new();
        env.insert("default_model".to_string(), "llama2".to_string());
        // Missing endpoint

        let config = ServiceConfig {
            r#type: "ollama".to_string(), // Changed from "ai_service"
            plugin: "ollama".to_string(),
            image: None,
            env: Some(env),
            ports: None,
            volumes: None,
            health_check: None,
            username: None,
            password: None,
            strict: None,
        };

        let result = ServiceFactory::create_plugin("test", &config);
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(e.message.contains("endpoint"));
        }
    }

    #[test]
    fn test_case_insensitive_plugin_type() -> Result<()> {
        let config = ServiceConfig {
            r#type: "database".to_string(),
            plugin: "SurrealDB".to_string(), // Mixed case
            image: Some("surrealdb/surrealdb:latest".to_string()),
            env: None,
            ports: None,
            volumes: None,
            health_check: None,
            username: None,
            password: None,
            strict: None,
        };

        let plugin = ServiceFactory::create_plugin("test_db", &config)?;
        assert_eq!(plugin.name(), "surrealdb");
        Ok(())
    }
}
