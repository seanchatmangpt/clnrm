//! vLLM Service Plugin
//!
//! Provides integration with vLLM (Very Large Language Model) inference server.
//! vLLM is a high-throughput, memory-efficient inference engine for LLMs.

use crate::cleanroom::{HealthStatus, ServiceHandle, ServicePlugin};
use crate::error::{CleanroomError, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// vLLM service configuration
#[derive(Debug, Clone)]
pub struct VllmConfig {
    /// Service endpoint URL
    pub endpoint: String,
    /// Model to serve
    pub model: String,
    /// Maximum number of sequences per iteration
    pub max_num_seqs: Option<u32>,
    /// Maximum model length
    pub max_model_len: Option<u32>,
    /// Tensor parallelism degree
    pub tensor_parallel_size: Option<u32>,
    /// GPU memory utilization
    pub gpu_memory_utilization: Option<f32>,
    /// Enable automatic prefix caching
    pub enable_prefix_caching: Option<bool>,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
}

/// vLLM service plugin
#[derive(Debug)]
pub struct VllmPlugin {
    name: String,
    config: VllmConfig,
    client: Arc<RwLock<Option<reqwest::Client>>>,
}

impl VllmPlugin {
    /// Create a new vLLM plugin instance
    pub fn new(name: &str, config: VllmConfig) -> Self {
        Self {
            name: name.to_string(),
            config,
            client: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize the HTTP client for vLLM API calls
    async fn init_client(&self) -> Result<reqwest::Client> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(self.config.timeout_seconds))
            .build()
            .map_err(|e| {
                CleanroomError::internal_error(format!("Failed to create HTTP client: {}", e))
            })?;

        Ok(client)
    }

    /// Test connection to vLLM service
    async fn test_connection(&self) -> Result<()> {
        let mut client_guard = self.client.write().await;
        if client_guard.is_none() {
            *client_guard = Some(self.init_client().await?);
        }
        let client = client_guard
            .as_ref()
            .ok_or_else(|| CleanroomError::internal_error("HTTP client not initialized"))?;

        let url = format!("{}/health", self.config.endpoint);

        let response = client.get(&url).send().await.map_err(|e| {
            CleanroomError::service_error(format!("Failed to connect to vLLM: {}", e))
        })?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(CleanroomError::service_error("vLLM service not responding"))
        }
    }

    /// Generate text using vLLM API (OpenAI-compatible)
    pub async fn generate_text(
        &self,
        prompt: &str,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    ) -> Result<VllmResponse> {
        let mut client_guard = self.client.write().await;
        if client_guard.is_none() {
            *client_guard = Some(self.init_client().await?);
        }
        let client = client_guard
            .as_ref()
            .ok_or_else(|| CleanroomError::internal_error("HTTP client not initialized"))?;

        let url = format!("{}/v1/completions", self.config.endpoint);

        let mut payload = json!({
            "model": self.config.model,
            "prompt": prompt,
            "stream": false
        });

        if let Some(max_tokens) = max_tokens {
            payload["max_tokens"] = json!(max_tokens);
        }

        if let Some(temperature) = temperature {
            payload["temperature"] = json!(temperature);
        }

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
            let vllm_response: VllmResponse = response.json().await.map_err(|e| {
                CleanroomError::service_error(format!("Failed to parse response: {}", e))
            })?;

            Ok(vllm_response)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            Err(CleanroomError::service_error(format!(
                "vLLM API error: {}",
                error_text
            )))
        }
    }

    /// Get model information from vLLM
    pub async fn get_model_info(&self) -> Result<VllmModelInfo> {
        let mut client_guard = self.client.write().await;
        if client_guard.is_none() {
            *client_guard = Some(self.init_client().await?);
        }
        let client = client_guard
            .as_ref()
            .ok_or_else(|| CleanroomError::internal_error("HTTP client not initialized"))?;

        let url = format!("{}/v1/models", self.config.endpoint);

        let response = client.get(&url).send().await.map_err(|e| {
            CleanroomError::service_error(format!("Failed to get model info: {}", e))
        })?;

        if response.status().is_success() {
            let model_info: VllmModelInfo = response.json().await.map_err(|e| {
                CleanroomError::service_error(format!("Failed to parse model info: {}", e))
            })?;

            Ok(model_info)
        } else {
            Err(CleanroomError::service_error(
                "Failed to retrieve model info",
            ))
        }
    }
}

/// Response from vLLM text generation (OpenAI-compatible)
#[derive(Debug, serde::Deserialize)]
pub struct VllmResponse {
    /// Unique ID for the request
    pub id: String,
    /// Object type (always "text_completion")
    pub object: String,
    /// Creation timestamp
    pub created: u64,
    /// Model used for generation
    pub model: String,
    /// Generated choices
    pub choices: Vec<VllmChoice>,
    /// Usage statistics
    pub usage: VllmUsage,
}

/// Individual choice in vLLM response
#[derive(Debug, serde::Deserialize)]
pub struct VllmChoice {
    /// Generated text
    pub text: String,
    /// Index of this choice
    pub index: u32,
    /// Log probabilities (if requested)
    pub logprobs: Option<Value>,
    /// Reason for finishing
    pub finish_reason: String,
}

/// Token usage information
#[derive(Debug, serde::Deserialize)]
pub struct VllmUsage {
    /// Number of prompt tokens
    pub prompt_tokens: u32,
    /// Number of completion tokens
    pub completion_tokens: u32,
    /// Total tokens used
    pub total_tokens: u32,
}

/// Model information response
#[derive(Debug, serde::Deserialize)]
pub struct VllmModelInfo {
    /// Object type (always "list")
    pub object: String,
    /// List of available models
    pub data: Vec<VllmModelData>,
}

/// Individual model data
#[derive(Debug, serde::Deserialize)]
pub struct VllmModelData {
    /// Model identifier
    pub id: String,
    /// Object type (always "model")
    pub object: String,
    /// Creation timestamp
    pub created: u64,
    /// Model owner
    pub owned_by: String,
}

impl ServicePlugin for VllmPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        // Use tokio::task::block_in_place for async operations
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Test connection to vLLM service
                let health_check = async {
                    match self.test_connection().await {
                        Ok(_) => HealthStatus::Healthy,
                        Err(_) => HealthStatus::Unhealthy,
                    }
                };

                let health = health_check.await;

                let mut metadata = HashMap::new();
                metadata.insert("endpoint".to_string(), self.config.endpoint.clone());
                metadata.insert("model".to_string(), self.config.model.clone());
                metadata.insert(
                    "timeout_seconds".to_string(),
                    self.config.timeout_seconds.to_string(),
                );
                metadata.insert("health_status".to_string(), format!("{:?}", health));

                if let Some(max_num_seqs) = self.config.max_num_seqs {
                    metadata.insert("max_num_seqs".to_string(), max_num_seqs.to_string());
                }

                if let Some(max_model_len) = self.config.max_model_len {
                    metadata.insert("max_model_len".to_string(), max_model_len.to_string());
                }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vllm_plugin_creation() {
        let config = VllmConfig {
            endpoint: "http://localhost:8000".to_string(),
            model: "microsoft/DialoGPT-medium".to_string(),
            max_num_seqs: Some(100),
            max_model_len: Some(2048),
            tensor_parallel_size: Some(1),
            gpu_memory_utilization: Some(0.9),
            enable_prefix_caching: Some(true),
            timeout_seconds: 60,
        };

        let plugin = VllmPlugin::new("test_vllm", config);
        assert_eq!(plugin.name(), "test_vllm");
    }

    #[test]
    fn test_vllm_config() {
        let config = VllmConfig {
            endpoint: "http://localhost:8000".to_string(),
            model: "microsoft/DialoGPT-medium".to_string(),
            max_num_seqs: Some(100),
            max_model_len: Some(2048),
            tensor_parallel_size: Some(1),
            gpu_memory_utilization: Some(0.9),
            enable_prefix_caching: Some(true),
            timeout_seconds: 60,
        };

        assert_eq!(config.endpoint, "http://localhost:8000");
        assert_eq!(config.model, "microsoft/DialoGPT-medium");
        assert_eq!(config.timeout_seconds, 60);
    }
}
