//! Plugin Package Management
//!
//! Handles plugin installation, updates, dependency resolution,
//! and package lifecycle management.

use crate::error::{CleanroomError, Result};
use crate::marketplace::{MarketplaceConfig, metadata::*};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

/// Plugin installer and package manager
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
        // Validate plugin
        metadata.validate()?;

        // Check compatibility
        let current_version = semver::Version::new(0, 3, 2);
        if !metadata.is_compatible_with(&current_version) {
            return Err(CleanroomError::validation_error(format!(
                "Plugin requires cleanroom version >= {}, current version is {}",
                metadata.min_cleanroom_version, current_version
            )));
        }

        // Resolve dependencies
        let dep_order = self.resolve_dependencies(metadata).await?;

        // Install dependencies first
        for dep_name in dep_order {
            if dep_name == metadata.name {
                continue;
            }

            tracing::info!("Installing dependency: {}", dep_name);
            // TODO: Actually install dependency
        }

        // Create installation directory
        let install_path = self.config.install_dir.join(&metadata.name);
        fs::create_dir_all(&install_path)
            .map_err(|e| CleanroomError::internal_error(format!("Failed to create install directory: {}", e)))?;

        // Download and extract plugin package
        self.download_plugin(metadata, &install_path).await?;

        // Validate installation
        self.validate_installation(&install_path, metadata)?;

        tracing::info!("Plugin '{}' installed successfully at {:?}", metadata.name, install_path);

        Ok(metadata.clone())
    }

    /// Update a plugin to new version
    pub async fn update_plugin(
        &self,
        current: &PluginMetadata,
        new: &PluginMetadata,
    ) -> Result<PluginMetadata> {
        if new.version <= current.version {
            return Err(CleanroomError::validation_error(format!(
                "New version {} is not newer than current version {}",
                new.version, current.version
            )));
        }

        // Backup current installation
        let install_path = self.config.install_dir.join(&current.name);
        let backup_path = install_path.with_extension("backup");

        if install_path.exists() {
            fs::rename(&install_path, &backup_path)
                .map_err(|e| CleanroomError::internal_error(format!("Failed to backup current installation: {}", e)))?;
        }

        // Install new version
        match self.install_plugin(new).await {
            Ok(installed) => {
                // Remove backup on success
                if backup_path.exists() {
                    let _ = fs::remove_dir_all(&backup_path);
                }
                Ok(installed)
            }
            Err(e) => {
                // Restore backup on failure
                if backup_path.exists() {
                    let _ = fs::rename(&backup_path, &install_path);
                }
                Err(e)
            }
        }
    }

    /// Uninstall a plugin
    pub async fn uninstall_plugin(&self, metadata: &PluginMetadata) -> Result<()> {
        let install_path = self.config.install_dir.join(&metadata.name);

        if !install_path.exists() {
            return Err(CleanroomError::validation_error(format!(
                "Plugin '{}' is not installed",
                metadata.name
            )));
        }

        fs::remove_dir_all(&install_path)
            .map_err(|e| CleanroomError::internal_error(format!("Failed to remove plugin directory: {}", e)))?;

        tracing::info!("Plugin '{}' uninstalled successfully", metadata.name);

        Ok(())
    }

    /// Resolve plugin dependencies
    async fn resolve_dependencies(&self, metadata: &PluginMetadata) -> Result<Vec<String>> {
        let mut resolved = Vec::new();
        let mut visited = HashSet::new();

        Self::resolve_recursive_sync(metadata, &mut resolved, &mut visited)?;

        Ok(resolved)
    }

    /// Recursive dependency resolution (simplified version without async)
    fn resolve_recursive_sync(
        metadata: &PluginMetadata,
        resolved: &mut Vec<String>,
        visited: &mut HashSet<String>,
    ) -> Result<()> {
        if visited.contains(&metadata.name) {
            // Circular dependency detected
            if !resolved.contains(&metadata.name) {
                return Err(CleanroomError::validation_error(format!(
                    "Circular dependency detected: {}",
                    metadata.name
                )));
            }
            return Ok(());
        }

        visited.insert(metadata.name.clone());

        for dep in &metadata.dependencies {
            if dep.optional {
                continue;
            }

            // TODO: Fetch dependency metadata and resolve recursively
            // For now, just add to resolved list
            if !resolved.contains(&dep.name) {
                resolved.push(dep.name.clone());
            }
        }

        resolved.push(metadata.name.clone());

        Ok(())
    }

    /// Download plugin package
    async fn download_plugin(&self, _metadata: &PluginMetadata, _install_path: &PathBuf) -> Result<()> {
        // TODO: Implement actual download from registry
        // For now, create a placeholder file
        tracing::info!("Downloading plugin package (simulated)");
        Ok(())
    }

    /// Validate plugin installation
    fn validate_installation(&self, install_path: &PathBuf, _metadata: &PluginMetadata) -> Result<()> {
        if !install_path.exists() {
            return Err(CleanroomError::validation_error("Installation directory not found"));
        }

        // TODO: Add more validation checks
        // - Check for required files
        // - Validate plugin manifest
        // - Verify checksums

        Ok(())
    }

    /// Check for plugin updates
    pub async fn check_updates(&self, current: &PluginMetadata) -> Result<Option<PluginMetadata>> {
        // TODO: Check remote registry for newer versions
        // For now, return None (no updates)
        Ok(None)
    }

    /// Get plugin dependencies
    pub async fn get_dependencies(&self, metadata: &PluginMetadata) -> Result<Vec<PluginDependency>> {
        Ok(metadata.dependencies.clone())
    }

    /// Check if plugin can be safely removed
    pub async fn can_remove(&self, _plugin_name: &str, _installed_plugins: &[PluginMetadata]) -> Result<bool> {
        // TODO: Check if other plugins depend on this one
        Ok(true)
    }

    /// Verify plugin integrity
    pub async fn verify_integrity(&self, _metadata: &PluginMetadata) -> Result<bool> {
        // TODO: Verify checksums and signatures
        Ok(true)
    }
}

/// Dependency resolver for complex dependency graphs
pub struct DependencyResolver {
    /// Resolved dependency order
    resolution_order: Vec<String>,
    /// Dependency graph
    graph: HashMap<String, Vec<String>>,
}

impl DependencyResolver {
    pub fn new() -> Self {
        Self {
            resolution_order: Vec::new(),
            graph: HashMap::new(),
        }
    }

    /// Add a dependency relationship
    pub fn add_dependency(&mut self, plugin: String, depends_on: String) {
        self.graph.entry(plugin).or_insert_with(Vec::new).push(depends_on);
    }

    /// Resolve all dependencies
    pub fn resolve(&mut self) -> Result<Vec<String>> {
        let mut visited = HashSet::new();
        let mut temp_mark = HashSet::new();

        for plugin in self.graph.keys().cloned().collect::<Vec<_>>() {
            if !visited.contains(&plugin) {
                self.visit(&plugin, &mut visited, &mut temp_mark)?;
            }
        }

        Ok(self.resolution_order.clone())
    }

    fn visit(
        &mut self,
        plugin: &str,
        visited: &mut HashSet<String>,
        temp_mark: &mut HashSet<String>,
    ) -> Result<()> {
        if temp_mark.contains(plugin) {
            return Err(CleanroomError::validation_error(format!(
                "Circular dependency detected at: {}",
                plugin
            )));
        }

        if visited.contains(plugin) {
            return Ok(());
        }

        temp_mark.insert(plugin.to_string());

        // Clone deps to avoid borrow issues
        let deps = self.graph.get(plugin).cloned();
        if let Some(deps) = deps {
            for dep in &deps {
                self.visit(dep, visited, temp_mark)?;
            }
        }

        temp_mark.remove(plugin);
        visited.insert(plugin.to_string());
        self.resolution_order.push(plugin.to_string());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_installer_creation() -> Result<()> {
        let config = MarketplaceConfig::default();
        let installer = PluginInstaller::new(&config)?;
        assert!(installer.config.install_dir.to_string_lossy().contains("plugins"));
        Ok(())
    }

    #[tokio::test]
    async fn test_dependency_resolution() -> Result<()> {
        let config = MarketplaceConfig::default();
        let installer = PluginInstaller::new(&config)?;

        let mut metadata = PluginMetadata::new(
            "test-plugin",
            "1.0.0",
            "Test plugin",
            "Test"
        )?;
        metadata.capabilities.push(standard_capabilities::database_capability());

        let deps = installer.resolve_dependencies(&metadata).await?;
        assert!(deps.contains(&"test-plugin".to_string()));

        Ok(())
    }

    #[test]
    fn test_dependency_resolver() -> Result<()> {
        let mut resolver = DependencyResolver::new();

        resolver.add_dependency("plugin-a".to_string(), "plugin-b".to_string());
        resolver.add_dependency("plugin-b".to_string(), "plugin-c".to_string());

        let order = resolver.resolve()?;

        // plugin-c should come before plugin-b, which should come before plugin-a
        let pos_a = order.iter().position(|p| p == "plugin-a").unwrap();
        let pos_b = order.iter().position(|p| p == "plugin-b").unwrap();
        let pos_c = order.iter().position(|p| p == "plugin-c").unwrap();

        assert!(pos_c < pos_b);
        assert!(pos_b < pos_a);

        Ok(())
    }

    #[test]
    fn test_circular_dependency_detection() -> Result<()> {
        let mut resolver = DependencyResolver::new();

        resolver.add_dependency("plugin-a".to_string(), "plugin-b".to_string());
        resolver.add_dependency("plugin-b".to_string(), "plugin-a".to_string());

        let result = resolver.resolve();
        assert!(result.is_err());

        Ok(())
    }
}
