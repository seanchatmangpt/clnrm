//! Ollama AI Service Plugin
//!
//! Provides integration with Ollama AI services for testing AI functionality.
//! Supports model loading, text generation, and health monitoring.

use crate::cleanroom::{HealthStatus, ServiceHandle, ServicePlugin};
use crate::error::{CleanroomError, Result};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Ollama AI service plugin configuration
#[derive(Debug, Clone)]
pub struct OllamaConfig {
    /// Service endpoint URL
    pub endpoint: String,
    /// Default model to use
    pub default_model: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
}

/// Ollama AI service plugin
#[derive(Debug)]
pub struct OllamaPlugin {
    name: String,
    config: OllamaConfig,
    client: Arc<RwLock<Option<reqwest::Client>>>,
}

impl OllamaPlugin {
    /// Create a new Ollama plugin instance
    pub fn new(name: &str, config: OllamaConfig) -> Self {
        Self {
            name: name.to_string(),
            config,
            client: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize the HTTP client for Ollama API calls
    async fn init_client(&self) -> Result<reqwest::Client> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(self.config.timeout_seconds))
            .build()
            .map_err(|e| {
                CleanroomError::internal_error(format!("Failed to create HTTP client: {}", e))
            })?;

        Ok(client)
    }

    /// Test connection to Ollama service
    async fn test_connection(&self) -> Result<()> {
        let client = self.init_client().await?;
        let url = format!("{}/api/version", self.config.endpoint);

        let response = client.get(&url).send().await.map_err(|e| {
            CleanroomError::service_error(format!("Failed to connect to Ollama: {}", e))
        })?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(CleanroomError::service_error(
                "Ollama service not responding",
            ))
        }
    }

    /// Generate text using Ollama AI model
    pub async fn generate_text(&self, model: &str, prompt: &str) -> Result<OllamaResponse> {
        let mut client_guard = self.client.write().await;
        if client_guard.is_none() {
            *client_guard = Some(self.init_client().await?);
        }
        let client = client_guard
            .as_ref()
            .ok_or_else(|| CleanroomError::internal_error("HTTP client not initialized"))?;

        let url = format!("{}/api/generate", self.config.endpoint);
        let payload = json!({
            "model": model,
            "prompt": prompt,
            "stream": false
        });

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                CleanroomError::service_error(format!("Failed to generate text: {}", e))
            })?;

        if response.status().is_success() {
            let ollama_response: OllamaResponse = response.json().await.map_err(|e| {
                CleanroomError::service_error(format!("Failed to parse response: {}", e))
            })?;

            Ok(ollama_response)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            Err(CleanroomError::service_error(format!(
                "Ollama API error: {}",
                error_text
            )))
        }
    }

    /// Get available models from Ollama
    pub async fn list_models(&self) -> Result<Vec<OllamaModel>> {
        let mut client_guard = self.client.write().await;
        if client_guard.is_none() {
            *client_guard = Some(self.init_client().await?);
        }
        let client = client_guard
            .as_ref()
            .ok_or_else(|| CleanroomError::internal_error("HTTP client not initialized"))?;

        let url = format!("{}/api/tags", self.config.endpoint);

        let response =
            client.get(&url).send().await.map_err(|e| {
                CleanroomError::service_error(format!("Failed to list models: {}", e))
            })?;

        if response.status().is_success() {
            let model_list: OllamaModelList = response.json().await.map_err(|e| {
                CleanroomError::service_error(format!("Failed to parse model list: {}", e))
            })?;

            Ok(model_list.models)
        } else {
            Err(CleanroomError::service_error(
                "Failed to retrieve model list",
            ))
        }
    }
}

/// Response from Ollama text generation
#[derive(Debug, serde::Deserialize)]
pub struct OllamaResponse {
    /// Generated text response
    pub response: String,
    /// Model used for generation
    pub model: String,
    /// Creation timestamp
    pub created_at: String,
    /// Whether generation is complete
    pub done: bool,
    /// Reason for completion
    pub done_reason: String,
    /// Total duration in nanoseconds
    pub total_duration: u64,
    /// Load duration in nanoseconds
    pub load_duration: u64,
    /// Number of tokens in prompt
    pub prompt_eval_count: u32,
    /// Duration for prompt evaluation
    pub prompt_eval_duration: u64,
    /// Number of tokens generated
    pub eval_count: u32,
    /// Duration for token generation
    pub eval_duration: u64,
}

/// Ollama model information
#[derive(Debug, serde::Deserialize)]
pub struct OllamaModel {
    /// Model name
    pub name: String,
    /// Model identifier
    pub model: String,
    /// Last modified timestamp
    pub modified_at: String,
    /// Model size in bytes
    pub size: u64,
    /// Model digest
    pub digest: String,
    /// Model details
    pub details: OllamaModelDetails,
}

/// Ollama model details
#[derive(Debug, serde::Deserialize)]
pub struct OllamaModelDetails {
    /// Parent model
    pub parent_model: String,
    /// Model format
    pub format: String,
    /// Model family
    pub family: String,
    /// Model families
    pub families: Vec<String>,
    /// Parameter size
    pub parameter_size: String,
    /// Quantization level
    pub quantization_level: String,
}

/// List of available models
#[derive(Debug, serde::Deserialize)]
pub struct OllamaModelList {
    /// List of available models
    pub models: Vec<OllamaModel>,
}

impl ServicePlugin for OllamaPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        // Use tokio::task::block_in_place for async operations
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Test connection to Ollama service
                let health_check = async {
                    // Simple health check - try to connect
                    match self.test_connection().await {
                        Ok(_) => HealthStatus::Healthy,
                        Err(_) => HealthStatus::Unhealthy,
                    }
                };

                let health = health_check.await;

                let mut metadata = HashMap::new();
                metadata.insert("endpoint".to_string(), self.config.endpoint.clone());
                metadata.insert(
                    "default_model".to_string(),
                    self.config.default_model.clone(),
                );
                metadata.insert(
                    "timeout_seconds".to_string(),
                    self.config.timeout_seconds.to_string(),
                );
                metadata.insert("health_status".to_string(), format!("{:?}", health));

                Ok(ServiceHandle {
                    id: Uuid::new_v4().to_string(),
                    service_name: self.name.clone(),
                    metadata,
                })
            })
        })
    }

    fn stop(&self, _handle: ServiceHandle) -> Result<()> {
        // HTTP-based service, no cleanup needed beyond dropping the client
        Ok(())
    }

    fn health_check(&self, handle: &ServiceHandle) -> HealthStatus {
        if let Some(health_status) = handle.metadata.get("health_status") {
            match health_status.as_str() {
                "Healthy" => HealthStatus::Healthy,
                "Unhealthy" => HealthStatus::Unhealthy,
                _ => HealthStatus::Unknown,
            }
        } else {
            HealthStatus::Unknown
        }
    }
}
