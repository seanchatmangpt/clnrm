//! Plugin Discovery System
//!
//! Handles plugin discovery from multiple sources including local filesystem,
//! remote registries, and community repositories.

use crate::error::{CleanroomError, Result};
use crate::marketplace::metadata::*;
use crate::marketplace::MarketplaceConfig;
use reqwest::Client;
use serde_json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;

/// Plugin discovery provider trait
pub trait PluginDiscoveryProvider: Send + Sync {
    /// Get provider name
    fn name(&self) -> &str;

    /// Check if provider is available
    fn is_available(&self) -> bool;

    /// Discover plugins from this provider
    async fn discover_plugins(&self) -> Result<Vec<PluginMetadata>>;

    /// Search for plugins
    async fn search_plugins(&self, query: &str) -> Result<Vec<PluginMetadata>>;

    /// Get specific plugin metadata
    async fn get_plugin_metadata(&self, plugin_name: &str) -> Result<PluginMetadata>;

    /// Download plugin binary
    async fn download_plugin(&self, metadata: &PluginMetadata) -> Result<PluginBinary>;
}

/// Main plugin discovery system
pub struct PluginDiscovery {
    config: MarketplaceConfig,
    providers: Vec<Box<dyn PluginDiscoveryProvider>>,
    http_client: Client,
}

impl PluginDiscovery {
    /// Create new discovery system
    pub fn new(config: &MarketplaceConfig) -> Result<Self> {
        let mut providers: Vec<Box<dyn PluginDiscoveryProvider>> = Vec::new();

        // Add local filesystem provider
        providers.push(Box::new(LocalPluginProvider::new(&config.install_dir)?));

        // Add remote registry providers
        for registry_url in &config.registry_urls {
            providers.push(Box::new(RemoteRegistryProvider::new(registry_url.clone())?));
        }

        Ok(Self {
            config: config.clone(),
            providers,
            http_client: Client::new(),
        })
    }

    /// Discover plugins from all providers
    pub async fn discover_all_plugins(&self) -> Result<Vec<PluginMetadata>> {
        let mut all_plugins = Vec::new();

        for provider in &self.providers {
            if provider.is_available() {
                match provider.discover_plugins().await {
                    Ok(plugins) => all_plugins.extend(plugins),
                    Err(e) => {
                        eprintln!("Failed to discover plugins from {}: {}", provider.name(), e);
                    }
                }
            }
        }

        // Remove duplicates based on name and version
        let mut unique_plugins = HashMap::new();
        for plugin in all_plugins {
            let key = (plugin.name.clone(), plugin.version.clone());
            unique_plugins.insert(key, plugin);
        }

        Ok(unique_plugins.into_values().collect())
    }

    /// Search for plugins across all providers
    pub async fn search_plugins(&self, query: &str) -> Result<Vec<PluginMetadata>> {
        let mut all_results = Vec::new();

        for provider in &self.providers {
            if provider.is_available() {
                match provider.search_plugins(query).await {
                    Ok(results) => all_results.extend(results),
                    Err(e) => {
                        eprintln!("Search failed for {}: {}", provider.name(), e);
                    }
                }
            }
        }

        // Remove duplicates
        let mut unique_results = HashMap::new();
        for plugin in all_results {
            let key = (plugin.name.clone(), plugin.version.clone());
            unique_results.insert(key, plugin);
        }

        Ok(unique_results.into_values().collect())
    }

    /// Get plugin metadata from any provider
    pub async fn get_plugin_metadata(&self, plugin_name: &str) -> Result<PluginMetadata> {
        for provider in &self.providers {
            if provider.is_available() {
                match provider.get_plugin_metadata(plugin_name).await {
                    Ok(metadata) => return Ok(metadata),
                    Err(_) => continue, // Try next provider
                }
            }
        }

        Err(CleanroomError::not_found(format!("Plugin '{}' not found", plugin_name)))
    }

    /// Download plugin from best available provider
    pub async fn download_plugin(&self, plugin_name: &str) -> Result<PluginBinary> {
        let metadata = self.get_plugin_metadata(plugin_name).await?;

        for provider in &self.providers {
            if provider.is_available() {
                match provider.download_plugin(&metadata).await {
                    Ok(binary) => return Ok(binary),
                    Err(_) => continue, // Try next provider
                }
            }
        }

        Err(CleanroomError::internal_error(format!("Failed to download plugin '{}' from any provider", plugin_name)))
    }

    /// Add a custom discovery provider
    pub fn add_provider(&mut self, provider: Box<dyn PluginDiscoveryProvider>) {
        self.providers.push(provider);
    }
}

/// Local filesystem plugin provider
pub struct LocalPluginProvider {
    plugin_dir: PathBuf,
}

impl LocalPluginProvider {
    pub fn new(plugin_dir: &Path) -> Result<Self> {
        Ok(Self {
            plugin_dir: plugin_dir.to_path_buf(),
        })
    }

    /// Scan local plugin directory for plugin metadata files
    async fn scan_local_plugins(&self) -> Result<Vec<PluginMetadata>> {
        let mut plugins = Vec::new();

        if !self.plugin_dir.exists() {
            return Ok(plugins);
        }

        let mut entries = fs::read_dir(&self.plugin_dir).await
            .map_err(|e| CleanroomError::internal_error("Failed to read plugin directory")
                .with_source(e.to_string()))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| CleanroomError::internal_error("Failed to read directory entry")
                .with_source(e.to_string()))? {

            let path = entry.path();

            // Look for plugin metadata files
            if path.is_dir() {
                let metadata_file = path.join("plugin.json");
                if metadata_file.exists() {
                    match self.load_plugin_metadata(&metadata_file).await {
                        Ok(metadata) => plugins.push(metadata),
                        Err(e) => {
                            eprintln!("Failed to load plugin metadata from {:?}: {}", metadata_file, e);
                        }
                    }
                }
            }
        }

        Ok(plugins)
    }

    /// Load plugin metadata from file
    async fn load_plugin_metadata(&self, metadata_file: &Path) -> Result<PluginMetadata> {
        let content = fs::read_to_string(metadata_file).await
            .map_err(|e| CleanroomError::internal_error("Failed to read plugin metadata file")
                .with_source(e.to_string()))?;

        let metadata: PluginMetadata = serde_json::from_str(&content)
            .map_err(|e| CleanroomError::internal_error("Failed to parse plugin metadata")
                .with_source(e.to_string()))?;

        Ok(metadata)
    }
}

impl PluginDiscoveryProvider for LocalPluginProvider {
    fn name(&self) -> &str {
        "local"
    }

    fn is_available(&self) -> bool {
        self.plugin_dir.exists()
    }

    async fn discover_plugins(&self) -> Result<Vec<PluginMetadata>> {
        self.scan_local_plugins().await
    }

    async fn search_plugins(&self, query: &str) -> Result<Vec<PluginMetadata>> {
        let all_plugins = self.discover_plugins().await?;
        let query_lower = query.to_lowercase();

        let matching_plugins: Vec<PluginMetadata> = all_plugins
            .into_iter()
            .filter(|plugin| {
                plugin.name.to_lowercase().contains(&query_lower) ||
                plugin.description.to_lowercase().contains(&query_lower) ||
                plugin.keywords.iter().any(|keyword| keyword.to_lowercase().contains(&query_lower)) ||
                plugin.author.to_lowercase().contains(&query_lower)
            })
            .collect();

        Ok(matching_plugins)
    }

    async fn get_plugin_metadata(&self, plugin_name: &str) -> Result<PluginMetadata> {
        let all_plugins = self.discover_plugins().await?;
        all_plugins
            .into_iter()
            .find(|plugin| plugin.name == plugin_name)
            .ok_or_else(|| CleanroomError::not_found(format!("Plugin '{}' not found locally", plugin_name)))
    }

    async fn download_plugin(&self, metadata: &PluginMetadata) -> Result<PluginBinary> {
        // For local plugins, just return the metadata as the "binary"
        // In a real implementation, this would copy the plugin files
        Ok(PluginBinary {
            metadata: metadata.clone(),
            binary_data: Vec::new(), // Local plugins don't need binary data
            checksum: "local".to_string(),
        })
    }
}

/// Remote registry plugin provider
pub struct RemoteRegistryProvider {
    registry_url: String,
    http_client: Client,
}

impl RemoteRegistryProvider {
    pub fn new(registry_url: String) -> Result<Self> {
        Ok(Self {
            registry_url,
            http_client: Client::new(),
        })
    }

    /// Fetch plugins from remote registry
    async fn fetch_remote_plugins(&self) -> Result<Vec<PluginMetadata>> {
        let url = format!("{}/api/plugins", self.registry_url);

        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| CleanroomError::network_error("Failed to connect to registry")
                .with_source(e.to_string()))?;

        if !response.status().is_success() {
            return Err(CleanroomError::network_error(format!(
                "Registry returned error: {}",
                response.status()
            )));
        }

        let plugins: Vec<PluginMetadata> = response.json().await
            .map_err(|e| CleanroomError::internal_error("Failed to parse registry response")
                .with_source(e.to_string()))?;

        Ok(plugins)
    }

    /// Search remote registry
    async fn search_remote_plugins(&self, query: &str) -> Result<Vec<PluginMetadata>> {
        let url = format!("{}/api/plugins/search?q={}", self.registry_url, query);

        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| CleanroomError::network_error("Failed to search registry")
                .with_source(e.to_string()))?;

        if !response.status().is_success() {
            return Err(CleanroomError::network_error(format!(
                "Registry search returned error: {}",
                response.status()
            )));
        }

        let plugins: Vec<PluginMetadata> = response.json().await
            .map_err(|e| CleanroomError::internal_error("Failed to parse search response")
                .with_source(e.to_string()))?;

        Ok(plugins)
    }

    /// Get specific plugin metadata from remote registry
    async fn get_remote_plugin_metadata(&self, plugin_name: &str) -> Result<PluginMetadata> {
        let url = format!("{}/api/plugins/{}", self.registry_url, plugin_name);

        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| CleanroomError::network_error("Failed to get plugin metadata")
                .with_source(e.to_string()))?;

        if !response.status().is_success() {
            return Err(CleanroomError::not_found(format!("Plugin '{}' not found in registry", plugin_name)));
        }

        let metadata: PluginMetadata = response.json().await
            .map_err(|e| CleanroomError::internal_error("Failed to parse plugin metadata")
                .with_source(e.to_string()))?;

        Ok(metadata)
    }
}

impl PluginDiscoveryProvider for RemoteRegistryProvider {
    fn name(&self) -> &str {
        "remote"
    }

    fn is_available(&self) -> bool {
        // For now, assume remote registries are always available
        // In production, this would check connectivity
        true
    }

    async fn discover_plugins(&self) -> Result<Vec<PluginMetadata>> {
        self.fetch_remote_plugins().await
    }

    async fn search_plugins(&self, query: &str) -> Result<Vec<PluginMetadata>> {
        self.search_remote_plugins(query).await
    }

    async fn get_plugin_metadata(&self, plugin_name: &str) -> Result<PluginMetadata> {
        self.get_remote_plugin_metadata(plugin_name).await
    }

    async fn download_plugin(&self, metadata: &PluginMetadata) -> Result<PluginBinary> {
        // Download plugin binary from registry
        let download_url = format!("{}/api/plugins/{}/download", self.registry_url, metadata.name);

        let response = self.http_client
            .get(&download_url)
            .send()
            .await
            .map_err(|e| CleanroomError::network_error("Failed to download plugin")
                .with_source(e.to_string()))?;

        if !response.status().is_success() {
            return Err(CleanroomError::network_error(format!(
                "Download failed with status: {}",
                response.status()
            )));
        }

        let binary_data = response.bytes().await
            .map_err(|e| CleanroomError::internal_error("Failed to read plugin binary")
                .with_source(e.to_string()))?
            .to_vec();

        Ok(PluginBinary {
            metadata: metadata.clone(),
            binary_data,
            checksum: "placeholder".to_string(), // TODO: Implement proper checksums
        })
    }
}

/// Plugin binary representation
#[derive(Debug, Clone)]
pub struct PluginBinary {
    pub metadata: PluginMetadata,
    pub binary_data: Vec<u8>,
    pub checksum: String,
}

impl PluginBinary {
    /// Verify binary integrity
    pub fn verify_integrity(&self) -> bool {
        // TODO: Implement proper checksum verification
        !self.binary_data.is_empty()
    }

    /// Get binary size
    pub fn size(&self) -> usize {
        self.binary_data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_local_plugin_discovery() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let plugin_dir = temp_dir.path().join("plugins");

        // Create plugin directory
        fs::create_dir_all(&plugin_dir).await?;

        // Create test plugin metadata
        let metadata = PluginMetadata::new("test-plugin", "1.0.0", "Test plugin", "Test Author")?;
        let metadata_json = serde_json::to_string_pretty(&metadata)?;

        // Write metadata file
        let plugin_path = plugin_dir.join("test-plugin");
        fs::create_dir_all(&plugin_path).await?;
        fs::write(plugin_path.join("plugin.json"), metadata_json).await?;

        // Test local provider
        let config = MarketplaceConfig {
            install_dir: plugin_dir,
            ..Default::default()
        };

        let provider = LocalPluginProvider::new(&config.install_dir)?;
        assert!(provider.is_available());

        let plugins = provider.discover_plugins().await?;
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name, "test-plugin");

        Ok(())
    }

    #[test]
    fn test_plugin_binary() {
        let metadata = PluginMetadata::new("test", "1.0.0", "Test", "Author").unwrap();
        let binary = PluginBinary {
            metadata,
            binary_data: vec![1, 2, 3, 4],
            checksum: "test".to_string(),
        };

        assert!(binary.verify_integrity());
        assert_eq!(binary.size(), 4);
    }
}
