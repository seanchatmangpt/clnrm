//! Cleanroom Marketplace - Plugin Ecosystem Management
//!
//! Provides a comprehensive plugin marketplace with discovery, installation,
//! management, and community features for the Cleanroom testing framework.

pub mod commands;
pub mod community;
pub mod discovery;
pub mod metadata;
pub mod package;
pub mod registry;
pub mod security;

pub use community::*;
pub use discovery::*;
pub use metadata::*;
pub use package::*;
pub use registry::*;
pub use security::*;

// Re-export command types
pub use commands::{
    execute_marketplace_command, initialize_sample_marketplace, MarketplaceCommands,
    MarketplaceSubcommands, UpdateResult,
};

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
    registry: registry::PluginRegistry,
    discovery: discovery::PluginDiscovery,
    installer: package::PluginInstaller,
}

impl Marketplace {
    /// Create a new marketplace instance
    pub fn new(config: MarketplaceConfig) -> Result<Self> {
        let registry = registry::PluginRegistry::new(&config)?;
        let discovery = discovery::PluginDiscovery::new(&config)?;
        let installer = package::PluginInstaller::new(&config)?;

        Ok(Self {
            config,
            registry,
            discovery,
            installer,
        })
    }

    /// Initialize marketplace with default configuration
    pub async fn default() -> Result<Self> {
        let marketplace = Self::new(MarketplaceConfig::default())?;

        // Initialize sample plugins for demonstration
        let mut sample_plugins = Vec::new();

        // PostgreSQL plugin
        let mut postgres_plugin = crate::marketplace::metadata::PluginMetadata::new(
            "postgres-plugin",
            "1.2.3",
            "Production-ready PostgreSQL testing plugin with advanced features",
            "Cleanroom Core Team",
        )?;
        postgres_plugin.keywords = vec![
            "database".to_string(),
            "postgresql".to_string(),
            "sql".to_string(),
            "testing".to_string(),
        ];
        postgres_plugin
            .capabilities
            .push(crate::marketplace::metadata::standard_capabilities::database_capability());
        postgres_plugin.community.average_rating = 4.8;
        postgres_plugin.community.rating_count = 127;
        postgres_plugin.community.download_count = 2341;
        sample_plugins.push(postgres_plugin);

        // Redis plugin
        let mut redis_plugin = crate::marketplace::metadata::PluginMetadata::new(
            "redis-plugin",
            "2.0.1",
            "High-performance Redis testing with cache and session management",
            "Community Maintainers",
        )?;
        redis_plugin.keywords = vec![
            "cache".to_string(),
            "redis".to_string(),
            "session".to_string(),
            "performance".to_string(),
        ];
        redis_plugin
            .capabilities
            .push(crate::marketplace::metadata::PluginCapability::new(
                "cache",
                crate::marketplace::metadata::PluginCategory::Storage,
                "Provides Redis cache testing capabilities",
            ));
        redis_plugin.community.average_rating = 4.6;
        redis_plugin.community.rating_count = 89;
        redis_plugin.community.download_count = 1542;
        sample_plugins.push(redis_plugin);

        // Kafka plugin
        let mut kafka_plugin = crate::marketplace::metadata::PluginMetadata::new(
            "kafka-plugin",
            "1.5.0",
            "Apache Kafka streaming and message queue testing plugin",
            "Enterprise Solutions Inc",
        )?;
        kafka_plugin.keywords = vec![
            "kafka".to_string(),
            "streaming".to_string(),
            "messaging".to_string(),
            "event".to_string(),
        ];
        kafka_plugin
            .capabilities
            .push(crate::marketplace::metadata::PluginCapability::new(
                "messaging",
                crate::marketplace::metadata::PluginCategory::MessageQueue,
                "Provides Kafka streaming and message queue testing",
            ));
        kafka_plugin.community.average_rating = 4.4;
        kafka_plugin.community.rating_count = 203;
        kafka_plugin.community.download_count = 892;
        sample_plugins.push(kafka_plugin);

        // AI testing plugin
        let mut ai_plugin = crate::marketplace::metadata::PluginMetadata::new(
            "ai-testing-plugin",
            "0.8.2",
            "Comprehensive AI/ML model testing and validation framework",
            "AI Testing Collective",
        )?;
        ai_plugin.keywords = vec![
            "ai".to_string(),
            "machine-learning".to_string(),
            "testing".to_string(),
            "validation".to_string(),
        ];
        ai_plugin
            .capabilities
            .push(crate::marketplace::metadata::standard_capabilities::ai_ml_capability());
        ai_plugin.community.average_rating = 4.9;
        ai_plugin.community.rating_count = 67;
        ai_plugin.community.download_count = 342;
        sample_plugins.push(ai_plugin);

        // MongoDB plugin
        let mut mongodb_plugin = crate::marketplace::metadata::PluginMetadata::new(
            "mongodb-plugin",
            "1.1.0",
            "MongoDB NoSQL database testing with document validation",
            "Database Community",
        )?;
        mongodb_plugin.keywords = vec![
            "mongodb".to_string(),
            "nosql".to_string(),
            "document".to_string(),
            "database".to_string(),
        ];
        mongodb_plugin
            .capabilities
            .push(crate::marketplace::metadata::PluginCapability::new(
                "database",
                crate::marketplace::metadata::PluginCategory::Database,
                "Provides MongoDB document database testing",
            ));
        mongodb_plugin.community.average_rating = 4.3;
        mongodb_plugin.community.rating_count = 156;
        mongodb_plugin.community.download_count = 678;
        sample_plugins.push(mongodb_plugin);

        // Register all sample plugins
        for plugin in sample_plugins {
            let _ = marketplace.registry.register_plugin(plugin).await;
        }

        Ok(marketplace)
    }

    /// Search for plugins
    pub async fn search(&self, query: &str) -> Result<Vec<metadata::PluginMetadata>> {
        self.discovery.search_plugins(query).await
    }

    /// Install a plugin
    pub async fn install(&self, plugin_name: &str) -> Result<metadata::PluginMetadata> {
        let metadata = self.registry.get_plugin(plugin_name)?;
        self.installer.install_plugin(&metadata).await
    }

    /// List installed plugins
    pub fn list_installed(&self) -> Result<Vec<metadata::PluginMetadata>> {
        self.registry.list_installed_plugins()
    }

    /// Get plugin information
    pub fn get_plugin_info(&self, plugin_name: &str) -> Result<metadata::PluginMetadata> {
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
            Ok(UpdateResult::Updated(
                plugin_name.to_string(),
                latest.version,
            ))
        } else {
            Ok(UpdateResult::NoUpdate(plugin_name.to_string()))
        }
    }

    /// Rate a plugin
    pub async fn rate_plugin(&self, plugin_name: &str, rating: u8) -> Result<()> {
        if rating > 5 {
            return Err(CleanroomError::validation_error(
                "Rating must be between 1 and 5",
            ));
        }

        self.registry.rate_plugin(plugin_name, rating).await
    }

    /// Add a review for a plugin
    pub async fn review_plugin(&self, plugin_name: &str, review: &str) -> Result<()> {
        self.registry
            .add_review(plugin_name, review.to_string())
            .await
    }

    /// Get plugin statistics
    pub fn get_plugin_stats(&self, plugin_name: &str) -> Result<metadata::PluginStatistics> {
        self.registry.get_plugin_stats(plugin_name)
    }
}
