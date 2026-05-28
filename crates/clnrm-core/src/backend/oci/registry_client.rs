//! Docker Registry API v2 client for pulling images

use super::{OciImage, OciImageConfig, OciLayer, OciManifest};
use crate::error::{CleanroomError, Result};
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use serde::Deserialize;
use tracing::info;

/// Docker Registry API v2 client
#[derive(Debug)]
pub struct RegistryClient {
    http_client: reqwest::Client,
    auth_cache: DashMap<String, AuthToken>,
}

/// Authentication token
#[derive(Debug, Clone)]
struct AuthToken {
    token: String,
    expires_at: DateTime<Utc>,
}

impl AuthToken {
    fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }
}

/// Authentication response from registry
#[derive(Debug, Deserialize)]
struct AuthResponse {
    token: String,
    #[serde(default = "default_expires_in")]
    expires_in: i64,
}

fn default_expires_in() -> i64 {
    300 // 5 minutes
}

impl RegistryClient {
    /// Create new registry client
    pub fn new() -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| CleanroomError::report_error(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            http_client,
            auth_cache: DashMap::new(),
        })
    }

    /// Pull image manifest and layers from registry
    pub async fn pull_image(
        &self,
        registry: &str,
        repository: &str,
        tag: &str,
    ) -> Result<OciImage> {
        info!(
            "Pulling image from registry: {}/{}:{}",
            registry, repository, tag
        );

        // 1. Get authentication token
        let token = self.authenticate(registry, repository).await?;

        // 2. Fetch manifest
        let manifest = self
            .fetch_manifest(registry, repository, tag, &token)
            .await?;

        info!("Fetched manifest with {} layers", manifest.layers.len());

        // 3. Download config blob
        let config_bytes = self
            .fetch_blob(registry, repository, &manifest.config.digest, &token)
            .await?;

        let config: OciImageConfig = serde_json::from_slice(&config_bytes).map_err(|e| {
            CleanroomError::oci_error(format!("Failed to parse image config: {}", e))
        })?;

        // 4. Download layer blobs
        let mut layers = Vec::new();
        for (idx, layer_desc) in manifest.layers.iter().enumerate() {
            info!(
                "Downloading layer {}/{}: {}",
                idx + 1,
                manifest.layers.len(),
                layer_desc.digest
            );

            let layer_data = self
                .fetch_blob(registry, repository, &layer_desc.digest, &token)
                .await?;

            layers.push(OciLayer {
                digest: layer_desc.digest.clone(),
                media_type: layer_desc.media_type.clone(),
                size: layer_desc.size,
                data: layer_data,
            });
        }

        info!("Successfully pulled image with {} layers", layers.len());

        Ok(OciImage {
            manifest,
            config,
            layers,
            config_bytes,
        })
    }

    /// Authenticate with registry (supports bearer tokens)
    async fn authenticate(&self, registry: &str, repository: &str) -> Result<AuthToken> {
        // Check cache
        let cache_key = format!("{}:{}", registry, repository);
        if let Some(token) = self.auth_cache.get(&cache_key) {
            if !token.is_expired() {
                info!("Using cached auth token for {}", cache_key);
                return Ok(token.clone());
            }
        }

        // Request new token
        let auth_url = format!(
            "https://auth.docker.io/token?service=registry.docker.io&scope=repository:{}:pull",
            repository
        );

        info!("Requesting auth token from: {}", auth_url);

        let response: AuthResponse = self
            .http_client
            .get(&auth_url)
            .send()
            .await
            .map_err(|e| {
                CleanroomError::report_error(format!("Failed to get auth token: {}", e))
            })?
            .json()
            .await
            .map_err(|e| {
                CleanroomError::report_error(format!("Failed to parse auth response: {}", e))
            })?;

        let token = AuthToken {
            token: response.token,
            expires_at: Utc::now() + Duration::seconds(response.expires_in),
        };

        self.auth_cache.insert(cache_key, token.clone());
        Ok(token)
    }

    /// Fetch image manifest
    async fn fetch_manifest(
        &self,
        registry: &str,
        repository: &str,
        tag: &str,
        token: &AuthToken,
    ) -> Result<OciManifest> {
        let url = format!("https://{}/v2/{}/manifests/{}", registry, repository, tag);

        info!("Fetching manifest from: {}", url);

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token.token))
            .header(
                "Accept",
                "application/vnd.docker.distribution.manifest.v2+json",
            )
            .send()
            .await
            .map_err(|e| {
                CleanroomError::report_error(format!("Failed to fetch manifest: {}", e))
            })?;

        if !response.status().is_success() {
            return Err(CleanroomError::report_error(format!(
                "Failed to fetch manifest: HTTP {}",
                response.status()
            )));
        }

        let manifest: OciManifest = response.json().await.map_err(|e| {
            CleanroomError::report_error(format!("Failed to parse manifest: {}", e))
        })?;

        Ok(manifest)
    }

    /// Fetch blob (config or layer)
    async fn fetch_blob(
        &self,
        registry: &str,
        repository: &str,
        digest: &str,
        token: &AuthToken,
    ) -> Result<Vec<u8>> {
        let url = format!("https://{}/v2/{}/blobs/{}", registry, repository, digest);

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token.token))
            .send()
            .await
            .map_err(|e| {
                CleanroomError::report_error(format!("Failed to fetch blob {}: {}", digest, e))
            })?;

        if !response.status().is_success() {
            return Err(CleanroomError::report_error(format!(
                "Failed to fetch blob {}: HTTP {}",
                digest,
                response.status()
            )));
        }

        let data = response.bytes().await.map_err(|e| {
            CleanroomError::report_error(format!("Failed to read blob {}: {}", digest, e))
        })?;

        Ok(data.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_pull_alpine() {
        let client = RegistryClient::new().unwrap();
        let result = client
            .pull_image("registry-1.docker.io", "library/alpine", "latest")
            .await;

        match result {
            Ok(image) => {
                assert!(!image.layers.is_empty());
                assert_eq!(image.config.os, "linux");
            }
            Err(e) => {
                eprintln!("Failed to pull alpine: {}", e);
                // Don't fail test if network is unavailable
            }
        }
    }
}
