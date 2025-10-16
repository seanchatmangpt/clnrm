//! Plugin Package Management
//!
//! Handles plugin packaging, installation, and dependency management.

use crate::error::{CleanroomError, Result};
use crate::marketplace::metadata::*;
use crate::marketplace::MarketplaceConfig;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;

/// Plugin installer for managing plugin lifecycle
pub struct PluginInstaller {
    config: MarketplaceConfig,
}

impl PluginInstaller {
    /// Create new plugin installer
    pub fn new(config: &MarketplaceConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }

    /// Install a plugin
    pub async fn install_plugin(&self, metadata: &PluginMetadata) -> Result<PluginMetadata> {
        // Ensure installation directory exists
        fs::create_dir_all(&self.config.install_dir).await
            .map_err(|e| CleanroomError::internal_error("Failed to create install directory")
                .with_source(e.to_string()))?;

        // Create plugin directory
        let plugin_dir = self.config.install_dir.join(&metadata.name);
        fs::create_dir_all(&plugin_dir).await
            .map_err(|e| CleanroomError::internal_error("Failed to create plugin directory")
                .with_source(e.to_string()))?;

        // Save plugin metadata
        let metadata_file = plugin_dir.join("plugin.json");
        let metadata_json = serde_json::to_string_pretty(metadata)
            .map_err(|e| CleanroomError::internal_error("Failed to serialize plugin metadata")
                .with_source(e.to_string()))?;

        fs::write(&metadata_file, metadata_json).await
            .map_err(|e| CleanroomError::internal_error("Failed to write plugin metadata")
                .with_source(e.to_string()))?;

        // Run post-installation steps if provided
        if !metadata.installation.post_install.is_empty() {
            for step in &metadata.installation.post_install {
                println!("Running post-install step: {}", step);
                // TODO: Execute post-install commands
            }
        }

        println!("✅ Plugin '{}' installed successfully", metadata.name);
        Ok(metadata.clone())
    }

    /// Update an existing plugin
    pub async fn update_plugin(
        &self,
        current: &PluginMetadata,
        latest: &PluginMetadata,
    ) -> Result<()> {
        if current.version >= latest.version {
            return Ok(()); // Already up to date
        }

        // Backup current plugin
        let backup_dir = self.config.cache_dir.join("backups").join(&format!("{}-{}", current.name, current.version));
        let current_dir = self.config.install_dir.join(&current.name);

        if current_dir.exists() {
            // TODO: Implement backup functionality
            println!("Backing up current plugin version...");
        }

        // Install new version
        self.install_plugin(latest).await?;

        // Remove old version
        if current_dir.exists() {
            fs::remove_dir_all(&current_dir).await
                .map_err(|e| CleanroomError::internal_error("Failed to remove old plugin version")
                    .with_source(e.to_string()))?;
        }

        println!("✅ Plugin '{}' updated from {} to {}", current.name, current.version, latest.version);
        Ok(())
    }

    /// Uninstall a plugin
    pub async fn uninstall_plugin(&self, plugin_name: &str) -> Result<()> {
        let plugin_dir = self.config.install_dir.join(plugin_name);

        if !plugin_dir.exists() {
            return Err(CleanroomError::not_found(format!("Plugin '{}' not installed", plugin_name)));
        }

        // Check for dependencies before uninstalling
        if let Ok(dependencies) = self.check_dependencies(plugin_name).await {
            if !dependencies.is_empty() {
                return Err(CleanroomError::validation_error(format!(
                    "Cannot uninstall plugin '{}' - it is required by: {}",
                    plugin_name,
                    dependencies.join(", ")
                )));
            }
        }

        fs::remove_dir_all(&plugin_dir).await
            .map_err(|e| CleanroomError::internal_error("Failed to uninstall plugin")
                .with_source(e.to_string()))?;

        println!("✅ Plugin '{}' uninstalled successfully", plugin_name);
        Ok(())
    }

    /// Check plugin dependencies
    async fn check_dependencies(&self, plugin_name: &str) -> Result<Vec<String>> {
        let mut dependent_plugins = Vec::new();

        // Scan all installed plugins for dependencies
        let mut entries = fs::read_dir(&self.config.install_dir).await
            .map_err(|e| CleanroomError::internal_error("Failed to read install directory")
                .with_source(e.to_string()))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| CleanroomError::internal_error("Failed to read directory entry")
                .with_source(e.to_string()))? {

            let path = entry.path();
            if path.is_dir() && path != self.config.install_dir.join(plugin_name) {
                if let Ok(metadata) = self.load_plugin_metadata(&path).await {
                    for dep in &metadata.dependencies {
                        if dep.name == plugin_name {
                            dependent_plugins.push(metadata.name);
                        }
                    }
                }
            }
        }

        Ok(dependent_plugins)
    }

    /// Load plugin metadata from installed plugin
    async fn load_plugin_metadata(&self, plugin_dir: &Path) -> Result<PluginMetadata> {
        let metadata_file = plugin_dir.join("plugin.json");
        let content = fs::read_to_string(&metadata_file).await
            .map_err(|e| CleanroomError::internal_error("Failed to read plugin metadata")
                .with_source(e.to_string()))?;

        let metadata: PluginMetadata = serde_json::from_str(&content)
            .map_err(|e| CleanroomError::internal_error("Failed to parse plugin metadata")
                .with_source(e.to_string()))?;

        Ok(metadata)
    }

    /// Verify plugin integrity
    pub async fn verify_plugin(&self, plugin_name: &str) -> Result<bool> {
        let plugin_dir = self.config.install_dir.join(plugin_name);

        if !plugin_dir.exists() {
            return Ok(false);
        }

        // Check if metadata file exists and is valid
        let metadata_file = plugin_dir.join("plugin.json");
        if !metadata_file.exists() {
            return Ok(false);
        }

        // Try to parse metadata
        match self.load_plugin_metadata(&plugin_dir).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// List all installed plugins
    pub async fn list_installed_plugins(&self) -> Result<Vec<PluginMetadata>> {
        let mut plugins = Vec::new();

        let mut entries = fs::read_dir(&self.config.install_dir).await
            .map_err(|e| CleanroomError::internal_error("Failed to read install directory")
                .with_source(e.to_string()))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| CleanroomError::internal_error("Failed to read directory entry")
                .with_source(e.to_string()))? {

            let path = entry.path();
            if path.is_dir() {
                if let Ok(metadata) = self.load_plugin_metadata(&path).await {
                    plugins.push(metadata);
                }
            }
        }

        Ok(plugins)
    }

    /// Get plugin installation path
    pub fn get_plugin_path(&self, plugin_name: &str) -> PathBuf {
        self.config.install_dir.join(plugin_name)
    }

    /// Check if plugin is installed
    pub async fn is_installed(&self, plugin_name: &str) -> bool {
        self.get_plugin_path(plugin_name).exists()
    }
}

/// Plugin packaging utilities
pub struct PluginPackager;

impl PluginPackager {
    /// Create a plugin package from metadata and binary
    pub fn create_package(metadata: PluginMetadata, binary_data: Vec<u8>) -> Result<PluginPackage> {
        let package = PluginPackage {
            metadata,
            binary: PluginBinary {
                metadata: metadata.clone(),
                binary_data,
                checksum: "placeholder".to_string(), // TODO: Calculate actual checksum
            },
            signatures: Vec::new(), // TODO: Implement signing
        };

        Ok(package)
    }

    /// Validate plugin package integrity
    pub fn validate_package(package: &PluginPackage) -> Result<()> {
        // Verify metadata
        package.metadata.validate()?;

        // Verify binary integrity
        if !package.binary.verify_integrity() {
            return Err(CleanroomError::validation_error("Plugin binary integrity check failed"));
        }

        // TODO: Verify signatures

        Ok(())
    }
}

/// Plugin package for distribution
#[derive(Debug, Clone)]
pub struct PluginPackage {
    pub metadata: PluginMetadata,
    pub binary: PluginBinary,
    pub signatures: Vec<PluginSignature>,
}

/// Plugin signature for security
#[derive(Debug, Clone)]
pub struct PluginSignature {
    pub algorithm: String,
    pub signature: String,
    pub public_key: String,
}

/// Plugin dependency resolver
pub struct DependencyResolver;

impl DependencyResolver {
    /// Resolve plugin dependencies
    pub async fn resolve_dependencies(
        &self,
        plugin_name: &str,
        registry: &crate::marketplace::registry::PluginRegistry,
    ) -> Result<Vec<PluginMetadata>> {
        let mut resolved = Vec::new();
        let mut to_resolve = vec![plugin_name.to_string()];
        let mut resolved_names = std::collections::HashSet::new();

        while let Some(current_name) = to_resolve.pop() {
            if resolved_names.contains(&current_name) {
                continue; // Already resolved
            }

            let metadata = registry.get_plugin(&current_name)?;
            resolved.push(metadata.clone());

            // Add dependencies to resolution queue
            for dep in &metadata.dependencies {
                if !resolved_names.contains(&dep.name) {
                    to_resolve.push(dep.name.clone());
                }
            }

            resolved_names.insert(current_name);
        }

        Ok(resolved)
    }

    /// Check for dependency conflicts
    pub fn check_conflicts(&self, plugins: &[PluginMetadata]) -> Result<Vec<DependencyConflict>> {
        let mut conflicts = Vec::new();
        let mut dependency_versions: std::collections::HashMap<String, Vec<Version>> = std::collections::HashMap::new();

        // Collect all dependency versions
        for plugin in plugins {
            for dep in &plugin.dependencies {
                dependency_versions.entry(dep.name.clone())
                    .or_insert_with(Vec::new)
                    .push(plugin.version.clone());
            }
        }

        // Check for version conflicts
        for (dep_name, versions) in dependency_versions {
            if versions.len() > 1 {
                conflicts.push(DependencyConflict {
                    dependency_name: dep_name,
                    conflicting_versions: versions,
                });
            }
        }

        Ok(conflicts)
    }
}

/// Dependency conflict information
#[derive(Debug, Clone)]
pub struct DependencyConflict {
    pub dependency_name: String,
    pub conflicting_versions: Vec<Version>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_plugin_installation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = MarketplaceConfig {
            install_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let installer = PluginInstaller::new(&config)?;
        let metadata = PluginMetadata::new("test-plugin", "1.0.0", "Test plugin", "Test Author")?;

        // Install plugin
        let installed = installer.install_plugin(&metadata).await?;
        assert_eq!(installed.name, "test-plugin");

        // Verify installation
        assert!(installer.is_installed("test-plugin").await);

        // List installed plugins
        let installed_list = installer.list_installed_plugins().await?;
        assert_eq!(installed_list.len(), 1);

        Ok(())
    }
}
