//! Cleanroom Marketplace - Plugin Ecosystem Management
//!
//! Provides a comprehensive plugin marketplace with discovery, installation,
//! management, and community features for the Cleanroom testing framework.

pub mod metadata;
pub mod registry;
pub mod discovery;
pub mod commands;
pub mod package;
pub mod security;
pub mod community;

pub use metadata::*;
pub use registry::*;
pub use discovery::*;
pub use commands::*;
pub use package::*;
pub use security::*;
pub use community::*;

use crate::error::{CleanroomError, Result};
use std::path::PathBuf;

/// Marketplace configuration
#[derive(Debug, Clone)]
pub struct MarketplaceConfig {
    /// Registry endpoints
    pub registry_urls: Vec<String>,
    /// Local plugin cache directory
    pub cache_dir: PathBuf,
    /// Plugin installation directory
    pub install_dir: PathBuf,
    /// Enable community features
    pub community_enabled: bool,
    /// Auto-update plugins
    pub auto_update: bool,
}

impl Default for MarketplaceConfig {
    fn default() -> Self {
        Self {
            registry_urls: vec![
                "https://registry.cleanroom.dev".to_string(),
                "https://plugins.cleanroom.dev".to_string(),
            ],
            cache_dir: std::env::temp_dir().join("cleanroom").join("marketplace"),
            install_dir: std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("plugins"),
            community_enabled: true,
            auto_update: false,
        }
    }
}

/// Main marketplace client
pub struct Marketplace {
    config: MarketplaceConfig,
    registry: PluginRegistry,
    discovery: PluginDiscovery,
    installer: PluginInstaller,
}

impl Marketplace {
    /// Create a new marketplace instance
    pub fn new(config: MarketplaceConfig) -> Result<Self> {
        let registry = PluginRegistry::new(&config)?;
        let discovery = PluginDiscovery::new(&config)?;
        let installer = PluginInstaller::new(&config)?;

        Ok(Self {
            config,
            registry,
            discovery,
            installer,
        })
    }

    /// Initialize marketplace with default configuration
    pub fn default() -> Result<Self> {
        Self::new(MarketplaceConfig::default())
    }

    /// Search for plugins
    pub async fn search(&self, query: &str) -> Result<Vec<PluginMetadata>> {
        self.discovery.search_plugins(query).await
    }

    /// Install a plugin
    pub async fn install(&self, plugin_name: &str) -> Result<PluginMetadata> {
        let metadata = self.registry.get_plugin(plugin_name)?;
        self.installer.install_plugin(&metadata).await
    }

    /// List installed plugins
    pub fn list_installed(&self) -> Result<Vec<PluginMetadata>> {
        self.registry.list_installed_plugins()
    }

    /// Get plugin information
    pub fn get_plugin_info(&self, plugin_name: &str) -> Result<PluginMetadata> {
        self.registry.get_plugin(plugin_name)
    }

    /// Update all installed plugins
    pub async fn update_all(&self) -> Result<Vec<UpdateResult>> {
        let installed = self.list_installed()?;
        let mut results = Vec::new();

        for plugin in installed {
            match self.update_plugin(&plugin.name).await {
                Ok(result) => results.push(result),
                Err(e) => results.push(UpdateResult::Failed(plugin.name, e.to_string())),
            }
        }

        Ok(results)
    }

    /// Update a specific plugin
    pub async fn update_plugin(&self, plugin_name: &str) -> Result<UpdateResult> {
        let current = self.registry.get_plugin(plugin_name)?;
        let latest = self.discovery.get_plugin_metadata(plugin_name).await?;

        if latest.version > current.version {
            self.installer.update_plugin(&current, &latest).await?;
            Ok(UpdateResult::Updated(plugin_name.to_string(), latest.version))
        } else {
            Ok(UpdateResult::NoUpdate(plugin_name.to_string()))
        }
    }

    /// Rate a plugin
    pub async fn rate_plugin(&self, plugin_name: &str, rating: u8) -> Result<()> {
        if rating > 5 {
            return Err(CleanroomError::validation_error("Rating must be between 1 and 5"));
        }

        self.registry.rate_plugin(plugin_name, rating).await
    }

    /// Add a review for a plugin
    pub async fn review_plugin(&self, plugin_name: &str, review: &str) -> Result<()> {
        self.registry.add_review(plugin_name, review.to_string()).await
    }

    /// Get plugin statistics
    pub fn get_plugin_stats(&self, plugin_name: &str) -> Result<PluginStatistics> {
        self.registry.get_plugin_stats(plugin_name)
    }
}

/// Update operation result
#[derive(Debug, Clone)]
pub enum UpdateResult {
    Updated(String, semver::Version),
    NoUpdate(String),
    Failed(String, String),
}

impl std::fmt::Display for UpdateResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateResult::Updated(name, version) => {
                write!(f, "✅ {} updated to {}", name, version)
            }
            UpdateResult::NoUpdate(name) => {
                write!(f, "✅ {} is already up to date", name)
            }
            UpdateResult::Failed(name, error) => {
                write!(f, "❌ {} failed to update: {}", name, error)
            }
        }
    }
}
