//! Plugin Registry Management
//!
//! Manages the local plugin registry including installation tracking,
//! version management, and plugin lifecycle operations.

use crate::error::{CleanroomError, Result};
use crate::marketplace::{metadata::*, MarketplaceConfig};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tokio::sync::RwLock;

/// Plugin registry for managing installed and available plugins
pub struct PluginRegistry {
    config: MarketplaceConfig,
    /// Local registry database
    registry_db: RwLock<RegistryDatabase>,
}

/// Internal registry database structure
#[derive(Debug, Clone, Default)]
struct RegistryDatabase {
    /// Installed plugins keyed by name
    installed: HashMap<String, PluginMetadata>,
    /// Available plugins from remote registries
    available: HashMap<String, PluginMetadata>,
    /// Installation records
    installations: HashMap<String, InstallationRecord>,
}

/// Installation record for tracking plugin lifecycle
#[derive(Debug, Clone)]
struct InstallationRecord {
    plugin_name: String,
    version: semver::Version,
    installed_at: chrono::DateTime<chrono::Utc>,
    install_path: PathBuf,
    active: bool,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new(config: &MarketplaceConfig) -> Result<Self> {
        // Ensure directories exist
        fs::create_dir_all(&config.cache_dir).map_err(|e| {
            CleanroomError::internal_error(format!("Failed to create cache directory: {}", e))
        })?;

        fs::create_dir_all(&config.install_dir).map_err(|e| {
            CleanroomError::internal_error(format!("Failed to create install directory: {}", e))
        })?;

        let registry = Self {
            config: config.clone(),
            registry_db: RwLock::new(RegistryDatabase::default()),
        };

        Ok(registry)
    }

    /// Load registry from disk
    pub async fn load(&self) -> Result<()> {
        let registry_path = self.config.cache_dir.join("registry.json");

        if registry_path.exists() {
            let content = fs::read_to_string(&registry_path).map_err(|e| {
                CleanroomError::internal_error(format!("Failed to read registry: {}", e))
            })?;

            let db: RegistryDatabase = serde_json::from_str(&content).map_err(|e| {
                CleanroomError::internal_error(format!("Failed to parse registry: {}", e))
            })?;

            let mut registry = self.registry_db.write().await;
            *registry = db;
        }

        Ok(())
    }

    /// Save registry to disk
    pub async fn save(&self) -> Result<()> {
        let registry_path = self.config.cache_dir.join("registry.json");

        let db = self.registry_db.read().await;
        let content = serde_json::to_string_pretty(&*db).map_err(|e| {
            CleanroomError::internal_error(format!("Failed to serialize registry: {}", e))
        })?;

        fs::write(&registry_path, content).map_err(|e| {
            CleanroomError::internal_error(format!("Failed to write registry: {}", e))
        })?;

        Ok(())
    }

    /// Register a plugin in the available plugins list
    pub async fn register_plugin(&self, metadata: PluginMetadata) -> Result<()> {
        metadata.validate()?;

        let mut db = self.registry_db.write().await;
        db.available.insert(metadata.name.clone(), metadata);

        drop(db);
        self.save().await?;

        Ok(())
    }

    /// Get plugin metadata by name
    pub fn get_plugin(&self, name: &str) -> Result<PluginMetadata> {
        let db = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.registry_db.read())
        });

        db.installed
            .get(name)
            .or_else(|| db.available.get(name))
            .cloned()
            .ok_or_else(|| CleanroomError::validation_error(format!("Plugin '{}' not found", name)))
    }

    /// List all installed plugins
    pub fn list_installed_plugins(&self) -> Result<Vec<PluginMetadata>> {
        let db = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.registry_db.read())
        });

        Ok(db.installed.values().cloned().collect())
    }

    /// List all available plugins
    pub async fn list_available_plugins(&self) -> Result<Vec<PluginMetadata>> {
        let db = self.registry_db.read().await;
        Ok(db.available.values().cloned().collect())
    }

    /// Check if plugin is installed
    pub async fn is_installed(&self, name: &str) -> bool {
        let db = self.registry_db.read().await;
        db.installed.contains_key(name)
    }

    /// Record plugin installation
    pub async fn record_installation(&self, name: &str) -> Result<()> {
        let mut db = self.registry_db.write().await;

        let metadata = db
            .available
            .get(name)
            .ok_or_else(|| {
                CleanroomError::validation_error(format!(
                    "Plugin '{}' not found in available plugins",
                    name
                ))
            })?
            .clone();

        let install_path = self.config.install_dir.join(&metadata.name);

        let record = InstallationRecord {
            plugin_name: metadata.name.clone(),
            version: metadata.version.clone(),
            installed_at: chrono::Utc::now(),
            install_path,
            active: true,
        };

        db.installations.insert(name.to_string(), record);
        db.installed.insert(name.to_string(), metadata);

        drop(db);
        self.save().await?;

        Ok(())
    }

    /// Remove plugin from registry
    pub async fn remove_plugin(&self, name: &str) -> Result<()> {
        let mut db = self.registry_db.write().await;

        if let Some(record) = db.installations.remove(name) {
            // Remove installation directory
            if record.install_path.exists() {
                fs::remove_dir_all(&record.install_path).map_err(|e| {
                    CleanroomError::internal_error(format!(
                        "Failed to remove plugin directory: {}",
                        e
                    ))
                })?;
            }
        }

        db.installed.remove(name);

        drop(db);
        self.save().await?;

        Ok(())
    }

    /// Update plugin metadata
    pub async fn update_plugin_metadata(&self, metadata: PluginMetadata) -> Result<()> {
        metadata.validate()?;

        let mut db = self.registry_db.write().await;

        if db.installed.contains_key(&metadata.name) {
            db.installed.insert(metadata.name.clone(), metadata.clone());
        }

        db.available.insert(metadata.name.clone(), metadata);

        drop(db);
        self.save().await?;

        Ok(())
    }

    /// Get plugin installation path
    pub async fn get_install_path(&self, name: &str) -> Result<PathBuf> {
        let db = self.registry_db.read().await;

        db.installations
            .get(name)
            .map(|record| record.install_path.clone())
            .ok_or_else(|| {
                CleanroomError::validation_error(format!("Plugin '{}' is not installed", name))
            })
    }

    /// Rate a plugin
    pub async fn rate_plugin(&self, name: &str, rating: u8) -> Result<()> {
        if rating > 5 {
            return Err(CleanroomError::validation_error(
                "Rating must be between 1 and 5",
            ));
        }

        let mut db = self.registry_db.write().await;

        if let Some(metadata) = db.available.get_mut(name) {
            let total =
                metadata.community.average_rating * (metadata.community.rating_count as f64);
            metadata.community.rating_count += 1;
            metadata.community.average_rating =
                (total + rating as f64) / (metadata.community.rating_count as f64);
            metadata.community.updated_at = chrono::Utc::now();
        } else {
            return Err(CleanroomError::validation_error(format!(
                "Plugin '{}' not found",
                name
            )));
        }

        drop(db);
        self.save().await?;

        Ok(())
    }

    /// Add a review for a plugin
    pub async fn add_review(&self, name: &str, review: String) -> Result<()> {
        let mut db = self.registry_db.write().await;

        if let Some(metadata) = db.available.get_mut(name) {
            metadata.community.reviews.push(review);
            metadata.community.updated_at = chrono::Utc::now();
        } else {
            return Err(CleanroomError::validation_error(format!(
                "Plugin '{}' not found",
                name
            )));
        }

        drop(db);
        self.save().await?;

        Ok(())
    }

    /// Get plugin statistics
    pub fn get_plugin_stats(&self, name: &str) -> Result<PluginStatistics> {
        let db = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.registry_db.read())
        });

        let metadata = db
            .available
            .get(name)
            .or_else(|| db.installed.get(name))
            .ok_or_else(|| {
                CleanroomError::validation_error(format!("Plugin '{}' not found", name))
            })?
            .clone();

        Ok(PluginStatistics {
            metadata: metadata.clone(),
            community: metadata.community.clone(),
            usage_stats: PluginUsageStats::default(),
            performance_metrics: PluginPerformanceMetrics::default(),
        })
    }

    /// Sync with remote registries
    pub async fn sync_remote(&self) -> Result<Vec<PluginMetadata>> {
        let mut synced_plugins = Vec::new();

        for registry_url in &self.config.registry_urls {
            match self.fetch_registry_catalog(registry_url).await {
                Ok(plugins) => {
                    for plugin in plugins {
                        self.register_plugin(plugin.clone()).await?;
                        synced_plugins.push(plugin);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to sync with registry {}: {}", registry_url, e);
                }
            }
        }

        Ok(synced_plugins)
    }

    /// Fetch plugin catalog from remote registry
    async fn fetch_registry_catalog(&self, _registry_url: &str) -> Result<Vec<PluginMetadata>> {
        // TODO: Implement actual HTTP fetch from remote registry
        // For now, return empty list
        Ok(Vec::new())
    }
}

impl serde::Serialize for RegistryDatabase {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("RegistryDatabase", 3)?;
        state.serialize_field("installed", &self.installed)?;
        state.serialize_field("available", &self.available)?;
        state.serialize_field("installations", &self.installations)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for RegistryDatabase {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct DatabaseHelper {
            installed: HashMap<String, PluginMetadata>,
            available: HashMap<String, PluginMetadata>,
            installations: HashMap<String, InstallationRecord>,
        }

        let helper = DatabaseHelper::deserialize(deserializer)?;
        Ok(RegistryDatabase {
            installed: helper.installed,
            available: helper.available,
            installations: helper.installations,
        })
    }
}

impl serde::Serialize for InstallationRecord {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("InstallationRecord", 5)?;
        state.serialize_field("plugin_name", &self.plugin_name)?;
        state.serialize_field("version", &self.version.to_string())?;
        state.serialize_field("installed_at", &self.installed_at)?;
        state.serialize_field("install_path", &self.install_path.to_string_lossy())?;
        state.serialize_field("active", &self.active)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for InstallationRecord {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct RecordHelper {
            plugin_name: String,
            version: String,
            installed_at: chrono::DateTime<chrono::Utc>,
            install_path: String,
            active: bool,
        }

        let helper = RecordHelper::deserialize(deserializer)?;
        Ok(InstallationRecord {
            plugin_name: helper.plugin_name,
            version: semver::Version::parse(&helper.version).map_err(serde::de::Error::custom)?,
            installed_at: helper.installed_at,
            install_path: PathBuf::from(helper.install_path),
            active: helper.active,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_registry_creation() -> Result<()> {
        let temp_dir = std::env::temp_dir().join(format!("clnrm-test-{}", uuid::Uuid::new_v4()));
        let config = MarketplaceConfig {
            registry_urls: vec![],
            cache_dir: temp_dir.join("cache"),
            install_dir: temp_dir.join("plugins"),
            community_enabled: false,
            auto_update: false,
        };
        let registry = PluginRegistry::new(&config)?;
        assert!(registry.list_installed_plugins()?.is_empty());
        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_plugin_registration() -> Result<()> {
        let temp_dir = std::env::temp_dir().join(format!("clnrm-test-{}", uuid::Uuid::new_v4()));
        let config = MarketplaceConfig {
            registry_urls: vec![],
            cache_dir: temp_dir.join("cache"),
            install_dir: temp_dir.join("plugins"),
            community_enabled: false,
            auto_update: false,
        };
        let registry = PluginRegistry::new(&config)?;

        let mut metadata =
            PluginMetadata::new("test-plugin", "1.0.0", "Test plugin", "Test Author")?;
        metadata
            .capabilities
            .push(standard_capabilities::database_capability());

        registry.register_plugin(metadata.clone()).await?;

        let available = registry.list_available_plugins().await?;
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].name, "test-plugin");

        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_plugin_installation_tracking() -> Result<()> {
        let temp_dir = std::env::temp_dir().join(format!("clnrm-test-{}", uuid::Uuid::new_v4()));
        let config = MarketplaceConfig {
            registry_urls: vec![],
            cache_dir: temp_dir.join("cache"),
            install_dir: temp_dir.join("plugins"),
            community_enabled: false,
            auto_update: false,
        };
        let registry = PluginRegistry::new(&config)?;

        let mut metadata =
            PluginMetadata::new("test-plugin", "1.0.0", "Test plugin", "Test Author")?;
        metadata
            .capabilities
            .push(standard_capabilities::database_capability());

        registry.register_plugin(metadata).await?;
        registry.record_installation("test-plugin").await?;

        assert!(registry.is_installed("test-plugin").await);

        let installed = registry.list_installed_plugins()?;
        assert_eq!(installed.len(), 1);

        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }
}
