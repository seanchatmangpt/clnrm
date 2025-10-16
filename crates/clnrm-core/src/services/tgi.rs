//! Text Generation Inference (TGI) Service Plugin
//!
//! Provides integration with Hugging Face's Text Generation Inference server.
//! TGI is optimized for high-performance text generation with LLMs.

use crate::cleanroom::{HealthStatus, ServiceHandle, ServicePlugin};
use crate::error::{CleanroomError, Result};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// TGI service configuration
#[derive(Debug, Clone)]
pub struct TgiConfig {
    /// Service endpoint URL
    pub endpoint: String,
    /// Model to serve (Hugging Face model ID)
    pub model_id: String,
    /// Maximum total tokens per request
    pub max_total_tokens: Option<u32>,
    /// Maximum input length
    pub max_input_length: Option<u32>,
    /// Maximum batch size for prefill
    pub max_batch_prefill_tokens: Option<u32>,
    /// Maximum number of concurrent requests
    pub max_concurrent_requests: Option<u32>,
    /// Maximum batch size for total tokens
    pub max_batch_total_tokens: Option<u32>,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
}

/// TGI service plugin
pub struct TgiPlugin {
    name: String,
    config: TgiConfig,
    client: Arc<RwLock<Option<reqwest::Client>>>,
}

impl TgiPlugin {
    /// Create a new TGI plugin instance
    pub fn new(name: &str, config: TgiConfig) -> Self {
        Self {
            name: name.to_string(),
            config,
            client: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize the HTTP client for TGI API calls
    async fn init_client(&self) -> Result<reqwest::Client> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(self.config.timeout_seconds))
            .build()
            .map_err(|e| {
                CleanroomError::internal_error(format!("Failed to create HTTP client: {}", e))
            })?;

        Ok(client)
    }

    /// Test connection to TGI service
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
            CleanroomError::service_error(format!("Failed to connect to TGI: {}", e))
        })?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(CleanroomError::service_error("TGI service not responding"))
        }
    }

    /// Generate text using TGI API
    pub async fn generate_text(
        &self,
        inputs: &str,
        parameters: Option<TgiParameters>,
    ) -> Result<TgiResponse> {
        let mut client_guard = self.client.write().await;
        if client_guard.is_none() {
            *client_guard = Some(self.init_client().await?);
        }
        let client = client_guard
            .as_ref()
            .ok_or_else(|| CleanroomError::internal_error("HTTP client not initialized"))?;

        let url = format!("{}/generate", self.config.endpoint);

        let mut payload = json!({
            "inputs": inputs,
            "parameters": parameters.unwrap_or_default()
        });

        // Set default parameters if not provided
        let params = payload["parameters"].as_object_mut().ok_or_else(|| {
            CleanroomError::internal_error(
                "Invalid JSON structure: parameters field missing or not an object",
            )
        })?;
        params.entry("max_new_tokens").or_insert(json!(100));
        params.entry("temperature").or_insert(json!(0.7));
        params.entry("do_sample").or_insert(json!(true));

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
            let tgi_response: TgiResponse = response.json().await.map_err(|e| {
                CleanroomError::service_error(format!("Failed to parse response: {}", e))
            })?;

            Ok(tgi_response)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            Err(CleanroomError::service_error(format!(
                "TGI API error: {}",
                error_text
            )))
        }
    }

    /// Get model information from TGI
    pub async fn get_info(&self) -> Result<TgiInfo> {
        let mut client_guard = self.client.write().await;
        if client_guard.is_none() {
            *client_guard = Some(self.init_client().await?);
        }
        let client = client_guard
            .as_ref()
            .ok_or_else(|| CleanroomError::internal_error("HTTP client not initialized"))?;

        let url = format!("{}/info", self.config.endpoint);

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| CleanroomError::service_error(format!("Failed to get info: {}", e)))?;

        if response.status().is_success() {
            let info: TgiInfo = response.json().await.map_err(|e| {
                CleanroomError::service_error(format!("Failed to parse info: {}", e))
            })?;

            Ok(info)
        } else {
            Err(CleanroomError::service_error(
                "Failed to retrieve service info",
            ))
        }
    }
}

/// TGI generation parameters
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TgiParameters {
    /// Maximum number of new tokens to generate
    #[serde(rename = "max_new_tokens")]
    pub max_new_tokens: Option<u32>,
    /// Sampling temperature (0.0 to 2.0)
    pub temperature: Option<f32>,
    /// Top-p sampling parameter
    #[serde(rename = "top_p")]
    pub top_p: Option<f32>,
    /// Top-k sampling parameter
    #[serde(rename = "top_k")]
    pub top_k: Option<u32>,
    /// Whether to use sampling (vs greedy decoding)
    #[serde(rename = "do_sample")]
    pub do_sample: Option<bool>,
    /// Number of beams for beam search
    #[serde(rename = "num_beams")]
    pub num_beams: Option<u32>,
    /// Repetition penalty
    #[serde(rename = "repetition_penalty")]
    pub repetition_penalty: Option<f32>,
}

impl Default for TgiParameters {
    fn default() -> Self {
        Self {
            max_new_tokens: Some(100),
            temperature: Some(0.7),
            top_p: None,
            top_k: None,
            do_sample: Some(true),
            num_beams: None,
            repetition_penalty: None,
        }
    }
}

/// Response from TGI text generation
#[derive(Debug, serde::Deserialize)]
pub struct TgiResponse {
    /// Generated text
    pub generated_text: String,
    /// Input prompt that was used
    pub prompt: Option<String>,
    /// Generation details
    pub details: Option<TgiDetails>,
    /// Generation warnings
    pub warnings: Option<Vec<String>>,
}

/// Generation details
#[derive(Debug, serde::Deserialize)]
pub struct TgiDetails {
    /// Whether generation finished
    pub finish_reason: String,
    /// Number of generated tokens
    pub generated_tokens: u32,
    /// Seed used for generation
    pub seed: Option<u64>,
    /// Generation parameters used
    pub parameters: Option<TgiParameters>,
}

/// TGI service information
#[derive(Debug, serde::Deserialize)]
pub struct TgiInfo {
    /// Model ID being served
    pub model_id: String,
    /// Model SHA hash
    pub model_sha: Option<String>,
    /// Maximum total tokens supported
    pub max_total_tokens: u32,
    /// Maximum input length supported
    pub max_input_length: u32,
    /// Maximum batch size for prefill
    pub max_batch_prefill_tokens: u32,
    /// Maximum number of concurrent requests
    pub max_concurrent_requests: u32,
    /// Maximum batch size for total tokens
    pub max_batch_total_tokens: u32,
    /// Tokenization details
    pub tokenization: Option<TgiTokenization>,
    /// Model dtype
    pub model_dtype: Option<String>,
}

/// Tokenization information
#[derive(Debug, serde::Deserialize)]
pub struct TgiTokenization {
    /// Tokenizer class
    pub tokenizer_class: Option<String>,
    /// Whether tokenizer is slow
    pub tokenizer_slow: Option<bool>,
}

impl ServicePlugin for TgiPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        // Use tokio::task::block_in_place for async operations
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Test connection to TGI service
                let health_check = async {
                    match self.test_connection().await {
                        Ok(_) => HealthStatus::Healthy,
                        Err(_) => HealthStatus::Unhealthy,
                    }
                };

                let health = health_check.await;

                let mut metadata = HashMap::new();
                metadata.insert("endpoint".to_string(), self.config.endpoint.clone());
                metadata.insert("model_id".to_string(), self.config.model_id.clone());
                metadata.insert(
                    "timeout_seconds".to_string(),
                    self.config.timeout_seconds.to_string(),
                );
                metadata.insert("health_status".to_string(), format!("{:?}", health));

                if let Some(max_total_tokens) = self.config.max_total_tokens {
                    metadata.insert("max_total_tokens".to_string(), max_total_tokens.to_string());
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
    fn test_tgi_plugin_creation() {
        let config = TgiConfig {
            endpoint: "http://localhost:8080".to_string(),
            model_id: "microsoft/DialoGPT-medium".to_string(),
            max_total_tokens: Some(2048),
            max_input_length: Some(1024),
            max_batch_prefill_tokens: Some(4096),
            max_concurrent_requests: Some(32),
            max_batch_total_tokens: Some(8192),
            timeout_seconds: 60,
        };

        let plugin = TgiPlugin::new("test_tgi", config);
        assert_eq!(plugin.name(), "test_tgi");
    }

    #[test]
    fn test_tgi_config() {
        let config = TgiConfig {
            endpoint: "http://localhost:8080".to_string(),
            model_id: "microsoft/DialoGPT-medium".to_string(),
            max_total_tokens: Some(2048),
            max_input_length: Some(1024),
            max_batch_prefill_tokens: Some(4096),
            max_concurrent_requests: Some(32),
            max_batch_total_tokens: Some(8192),
            timeout_seconds: 60,
        };

        assert_eq!(config.endpoint, "http://localhost:8080");
        assert_eq!(config.model_id, "microsoft/DialoGPT-medium");
        assert_eq!(config.timeout_seconds, 60);
    }
}
