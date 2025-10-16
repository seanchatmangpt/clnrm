//! Plugin Registry System
//!
//! Manages plugin metadata, installation tracking, and community features.
//! Provides the central registry for plugin discovery and management.

use crate::error::{CleanroomError, Result};
use crate::marketplace::metadata::*;
use crate::marketplace::MarketplaceConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio::fs;

/// In-memory plugin registry with persistence
#[derive(Debug)]
pub struct PluginRegistry {
    /// Plugin metadata storage
    plugins: Arc<RwLock<HashMap<String, PluginMetadata>>>,
    /// Plugin ratings and reviews
    ratings: Arc<RwLock<HashMap<String, Vec<PluginRating>>>>,
    /// Plugin reviews
    reviews: Arc<RwLock<HashMap<String, Vec<PluginReview>>>>,
    /// Plugin installation tracking
    installations: Arc<RwLock<HashMap<String, InstallationRecord>>>,
    /// Registry configuration
    config: MarketplaceConfig,
    /// Registry file path for persistence
    registry_file: PathBuf,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new(config: &MarketplaceConfig) -> Result<Self> {
        let registry_file = config.cache_dir.join("registry.json");

        Ok(Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            ratings: Arc::new(RwLock::new(HashMap::new())),
            reviews: Arc::new(RwLock::new(HashMap::new())),
            installations: Arc::new(RwLock::new(HashMap::new())),
            config: config.clone(),
            registry_file,
        })
    }

    /// Initialize registry by loading from disk
    pub async fn initialize(&self) -> Result<()> {
        if self.registry_file.exists() {
            let content = fs::read_to_string(&self.registry_file).await
                .map_err(|e| CleanroomError::internal_error("Failed to read registry file")
                    .with_source(e.to_string()))?;

            let registry_data: RegistryData = serde_json::from_str(&content)
                .map_err(|e| CleanroomError::internal_error("Failed to parse registry data")
                    .with_source(e.to_string()))?;

            // Load plugins
            let mut plugins = self.plugins.write().unwrap();
            for plugin in registry_data.plugins {
                plugins.insert(plugin.name.clone(), plugin);
            }

            // Load ratings and reviews if community features are enabled
            if self.config.community_enabled {
                let mut ratings = self.ratings.write().unwrap();
                for (name, plugin_ratings) in registry_data.ratings {
                    ratings.insert(name, plugin_ratings);
                }

                let mut reviews = self.reviews.write().unwrap();
                for (name, plugin_reviews) in registry_data.reviews {
                    reviews.insert(name, plugin_reviews);
                }
            }
        }

        Ok(())
    }

    /// Save registry to disk
    pub async fn save(&self) -> Result<()> {
        let registry_data = self.export_registry_data();

        let json_data = serde_json::to_string_pretty(&registry_data)
            .map_err(|e| CleanroomError::internal_error("Failed to serialize registry")
                .with_source(e.to_string()))?;

        // Ensure directory exists
        if let Some(parent) = self.registry_file.parent() {
            fs::create_dir_all(parent).await
                .map_err(|e| CleanroomError::internal_error("Failed to create registry directory")
                    .with_source(e.to_string()))?;
        }

        fs::write(&self.registry_file, json_data).await
            .map_err(|e| CleanroomError::internal_error("Failed to write registry file")
                .with_source(e.to_string()))?;

        Ok(())
    }

    /// Register a plugin in the registry
    pub async fn register_plugin(&self, metadata: PluginMetadata) -> Result<()> {
        metadata.validate()?;

        let mut plugins = self.plugins.write().unwrap();
        plugins.insert(metadata.name.clone(), metadata);

        // Save registry after registration
        self.save().await?;

        Ok(())
    }

    /// Get plugin metadata by name
    pub fn get_plugin(&self, name: &str) -> Result<PluginMetadata> {
        let plugins = self.plugins.read().unwrap();
        plugins.get(name)
            .cloned()
            .ok_or_else(|| CleanroomError::not_found(format!("Plugin '{}' not found", name)))
    }

    /// List all registered plugins
    pub fn list_plugins(&self) -> Result<Vec<PluginMetadata>> {
        let plugins = self.plugins.read().unwrap();
        Ok(plugins.values().cloned().collect())
    }

    /// List installed plugins
    pub fn list_installed_plugins(&self) -> Result<Vec<PluginMetadata>> {
        let installations = self.installations.read().unwrap();
        let mut installed_plugins = Vec::new();

        for (plugin_name, _) in installations.iter() {
            if let Ok(metadata) = self.get_plugin(plugin_name) {
                installed_plugins.push(metadata);
            }
        }

        Ok(installed_plugins)
    }

    /// Search plugins by query
    pub fn search_plugins(&self, query: &str) -> Result<Vec<PluginMetadata>> {
        let plugins = self.plugins.read().unwrap();
        let query_lower = query.to_lowercase();

        let matching_plugins: Vec<PluginMetadata> = plugins
            .values()
            .filter(|plugin| {
                plugin.name.to_lowercase().contains(&query_lower) ||
                plugin.description.to_lowercase().contains(&query_lower) ||
                plugin.keywords.iter().any(|keyword| keyword.to_lowercase().contains(&query_lower)) ||
                plugin.author.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect();

        Ok(matching_plugins)
    }

    /// Rate a plugin
    pub async fn rate_plugin(&self, plugin_name: &str, rating: u8) -> Result<()> {
        if !self.config.community_enabled {
            return Err(CleanroomError::validation_error("Community features are disabled"));
        }

        if rating > 5 {
            return Err(CleanroomError::validation_error("Rating must be between 1 and 5"));
        }

        let rating_record = PluginRating {
            user_id: "anonymous".to_string(), // TODO: Implement user authentication
            rating,
            timestamp: chrono::Utc::now(),
        };

        let mut ratings = self.ratings.write().unwrap();
        ratings.entry(plugin_name.to_string()).or_insert_with(Vec::new).push(rating_record);

        // Update plugin's community info
        if let Ok(mut plugin) = self.get_plugin(plugin_name) {
            plugin.community.add_rating(rating);
            let mut plugins = self.plugins.write().unwrap();
            plugins.insert(plugin_name.to_string(), plugin);
        }

        self.save().await?;
        Ok(())
    }

    /// Add a review for a plugin
    pub async fn add_review(&self, plugin_name: &str, review: String) -> Result<()> {
        if !self.config.community_enabled {
            return Err(CleanroomError::validation_error("Community features are disabled"));
        }

        let review_record = PluginReview {
            user_id: "anonymous".to_string(), // TODO: Implement user authentication
            review,
            timestamp: chrono::Utc::now(),
        };

        let mut reviews = self.reviews.write().unwrap();
        reviews.entry(plugin_name.to_string()).or_insert_with(Vec::new).push(review_record);

        self.save().await?;
        Ok(())
    }

    /// Record plugin installation
    pub async fn record_installation(&self, plugin_name: &str) -> Result<()> {
        let record = InstallationRecord {
            plugin_name: plugin_name.to_string(),
            installed_at: chrono::Utc::now(),
            version: self.get_plugin(plugin_name)?.version,
        };

        let mut installations = self.installations.write().unwrap();
        installations.insert(plugin_name.to_string(), record);

        // Increment download count
        if let Ok(mut plugin) = self.get_plugin(plugin_name) {
            plugin.community.increment_downloads();
            let mut plugins = self.plugins.write().unwrap();
            plugins.insert(plugin_name.to_string(), plugin);
        }

        self.save().await?;
        Ok(())
    }

    /// Get plugin statistics
    pub fn get_plugin_stats(&self, plugin_name: &str) -> Result<PluginStatistics> {
        let metadata = self.get_plugin(plugin_name)?;

        let ratings = self.ratings.read().unwrap();
        let reviews = self.reviews.read().unwrap();

        let community = CommunityInfo {
            average_rating: ratings.get(plugin_name)
                .map(|r| {
                    let sum: u8 = r.iter().map(|rating| rating.rating).sum();
                    sum as f32 / r.len() as f32
                })
                .unwrap_or(0.0),
            rating_count: ratings.get(plugin_name).map(|r| r.len() as u32).unwrap_or(0),
            download_count: metadata.community.download_count,
            created_at: metadata.community.created_at,
            updated_at: metadata.community.updated_at,
            github_stars: metadata.community.github_stars,
            open_issues: metadata.community.open_issues,
            last_commit: metadata.community.last_commit,
        };

        let installations = self.installations.read().unwrap();
        let usage_stats = if let Some(_) = installations.get(plugin_name) {
            UsageStatistics {
                installations: 1,
                active_installations: 1,
                daily_usage: 0.0,
                peak_usage: 1,
                avg_session_duration: 0.0,
                error_rate: 0.0,
            }
        } else {
            UsageStatistics::default()
        };

        Ok(PluginStatistics {
            metadata,
            community,
            usage_stats,
            performance_metrics: PerformanceMetrics::default(),
        })
    }

    /// Export registry data for serialization
    fn export_registry_data(&self) -> RegistryData {
        let plugins = self.plugins.read().unwrap();
        let ratings = self.ratings.read().unwrap();
        let reviews = self.reviews.read().unwrap();

        RegistryData {
            plugins: plugins.values().cloned().collect(),
            ratings: ratings.clone(),
            reviews: reviews.clone(),
        }
    }

    /// Import registry data from external source
    pub async fn import_plugins(&self, plugins: Vec<PluginMetadata>) -> Result<()> {
        for plugin in plugins {
            self.register_plugin(plugin).await?;
        }
        Ok(())
    }

    /// Remove a plugin from the registry
    pub async fn remove_plugin(&self, plugin_name: &str) -> Result<()> {
        let mut plugins = self.plugins.write().unwrap();
        if plugins.remove(plugin_name).is_none() {
            return Err(CleanroomError::not_found(format!("Plugin '{}' not found", plugin_name)));
        }

        // Remove associated data
        let mut ratings = self.ratings.write().unwrap();
        ratings.remove(plugin_name);

        let mut reviews = self.reviews.write().unwrap();
        reviews.remove(plugin_name);

        let mut installations = self.installations.write().unwrap();
        installations.remove(plugin_name);

        self.save().await?;
        Ok(())
    }
}

/// Registry data for serialization
#[derive(Debug, Serialize, Deserialize)]
struct RegistryData {
    plugins: Vec<PluginMetadata>,
    ratings: HashMap<String, Vec<PluginRating>>,
    reviews: HashMap<String, Vec<PluginReview>>,
}

/// Plugin rating record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRating {
    pub user_id: String,
    pub rating: u8,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Plugin review record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginReview {
    pub user_id: String,
    pub review: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Installation record for tracking plugin installations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationRecord {
    pub plugin_name: String,
    pub installed_at: chrono::DateTime<chrono::Utc>,
    pub version: Version,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_registry_operations() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = MarketplaceConfig {
            cache_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let registry = PluginRegistry::new(&config)?;
        registry.initialize().await?;

        // Create test plugin metadata
        let mut metadata = PluginMetadata::new("test-plugin", "1.0.0", "Test plugin", "Test Author")?;
        metadata.capabilities.push(PluginCapability::new(
            "test",
            PluginCategory::Testing,
            "Test capability",
        ));

        // Register plugin
        registry.register_plugin(metadata.clone()).await?;

        // Verify plugin is registered
        let retrieved = registry.get_plugin("test-plugin")?;
        assert_eq!(retrieved.name, "test-plugin");
        assert_eq!(retrieved.version.to_string(), "1.0.0");

        // List plugins
        let plugins = registry.list_plugins()?;
        assert_eq!(plugins.len(), 1);

        // Search plugins
        let search_results = registry.search_plugins("test")?;
        assert_eq!(search_results.len(), 1);

        Ok(())
    }
}
