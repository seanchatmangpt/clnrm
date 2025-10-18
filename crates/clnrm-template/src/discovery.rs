//! Template discovery and auto-loading system
//!
//! Provides convenient APIs for discovering and loading templates from:
//! - Directories (recursive)
//! - Glob patterns
//! - File paths
//! - Template collections/namespaces

use crate::error::{TemplateError, Result};
use crate::renderer::TemplateRenderer;
use crate::context::TemplateContext;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

/// Template discovery and loading system
///
/// Provides fluent API for discovering templates from various sources:
/// - Auto-discovery in directories
/// - Glob pattern matching
/// - Template namespaces
/// - Hot-reload support
#[derive(Debug)]
pub struct TemplateDiscovery {
    /// Base search paths for template discovery
    search_paths: Vec<PathBuf>,
    /// Template namespace mappings (namespace -> template content)
    namespaces: HashMap<String, String>,
    /// Glob patterns for template inclusion
    glob_patterns: Vec<String>,
    /// Enable recursive directory scanning
    recursive: bool,
    /// File extensions to include (default: .toml, .tera, .tpl)
    extensions: Vec<String>,
    /// Enable hot-reload (watch for file changes)
    hot_reload: bool,
    /// Template organization strategy
    organization: TemplateOrganization,
}

/// Template organization strategies
#[derive(Debug, Clone)]
pub enum TemplateOrganization {
    /// Flat organization (all templates in single namespace)
    Flat,
    /// Hierarchical organization (templates organized by directory structure)
    Hierarchical,
    /// Custom organization with prefixes
    Custom { prefix: String },
}

impl Default for TemplateDiscovery {
    fn default() -> Self {
        Self {
            search_paths: Vec::new(),
            namespaces: HashMap::new(),
            glob_patterns: Vec::new(),
            recursive: true,
            extensions: vec![
                "toml".to_string(),
                "tera".to_string(),
                "tpl".to_string(),
                "template".to_string(),
            ],
            hot_reload: false,
            organization: TemplateOrganization::Hierarchical,
        }
    }
}

impl TemplateDiscovery {
    /// Create new template discovery instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Add search path for template discovery
    ///
    /// # Arguments
    /// * `path` - Directory path to search for templates
    pub fn with_search_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.search_paths.push(path.as_ref().to_path_buf());
        self
    }

    /// Add multiple search paths
    pub fn with_search_paths<I, P>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        for path in paths {
            self.search_paths.push(path.as_ref().to_path_buf());
        }
        self
    }

    /// Add glob pattern for template inclusion
    ///
    /// # Arguments
    /// * `pattern` - Glob pattern (e.g., "**/*.toml", "tests/**/*.tera")
    pub fn with_glob_pattern(mut self, pattern: &str) -> Self {
        self.glob_patterns.push(pattern.to_string());
        self
    }

    /// Enable/disable recursive directory scanning
    pub fn recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    /// Set file extensions to include in discovery
    pub fn with_extensions<I, S>(mut self, extensions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.extensions = extensions.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Enable hot-reload for template files
    pub fn hot_reload(mut self, enabled: bool) -> Self {
        self.hot_reload = enabled;
        self
    }

    /// Set template organization strategy
    pub fn with_organization(mut self, organization: TemplateOrganization) -> Self {
        self.organization = organization;
        self
    }

    /// Add template to namespace
    ///
    /// # Arguments
    /// * `namespace` - Namespace name (e.g., "macros", "partials")
    /// * `content` - Template content
    pub fn with_namespace<S: Into<String>>(mut self, namespace: S, content: S) -> Self {
        self.namespaces.insert(namespace.into(), content.into());
        self
    }

    /// Discover and load all templates
    ///
    /// Returns a TemplateLoader with all discovered templates ready for rendering
    pub fn load(self) -> Result<TemplateLoader> {
        let mut templates = HashMap::new();

        // Load namespace templates first (highest priority)
        for (namespace, content) in &self.namespaces {
            templates.insert(namespace.to_string(), content.to_string());
        }

        // Discover templates from search paths
        for search_path in &self.search_paths {
            self.discover_from_path(search_path, &mut templates)?;
        }

        // Discover templates from glob patterns
        for pattern in &self.glob_patterns {
            self.discover_from_glob(pattern, &mut templates)?;
        }

        Ok(TemplateLoader {
            templates,
            hot_reload: self.hot_reload,
            organization: self.organization,
        })
    }

    /// Discover templates from a directory path
    fn discover_from_path(&self, path: &Path, templates: &mut HashMap<String, String>) -> Result<()> {
        if !path.exists() {
            return Ok(()); // Skip non-existent paths
        }

        if path.is_file() {
            // Single file
            if self.should_include_file(path) {
                let name = self.template_name_from_path(path);
                let content = std::fs::read_to_string(path)
                    .map_err(|e| TemplateError::IoError(format!("Failed to read template file {:?}: {}", path, e)))?;
                templates.insert(name, content);
            }
            return Ok(());
        }

        // Directory scanning
        self.scan_directory(path, templates)
    }

    /// Discover templates from glob pattern
    fn discover_from_glob(&self, pattern: &str, templates: &mut HashMap<String, String>) -> Result<()> {
        use globset::{Glob, GlobSetBuilder};

        let glob = Glob::new(pattern)
            .map_err(|e| TemplateError::ConfigError(format!("Invalid glob pattern '{}': {}", pattern, e)))?;

        let glob_set = GlobSetBuilder::new()
            .add(glob)
            .build()
            .map_err(|e| TemplateError::ConfigError(format!("Failed to build glob set for '{}': {}", pattern, e)))?;

        for search_path in &self.search_paths {
            self.scan_path_with_glob(search_path, &glob_set, templates)?;
        }

        Ok(())
    }

    /// Scan directory for template files
    fn scan_directory(&self, dir: &Path, templates: &mut HashMap<String, String>) -> Result<()> {
        use walkdir::WalkDir;

        let walker = if self.recursive {
            WalkDir::new(dir)
        } else {
            WalkDir::new(dir).max_depth(1)
        };

        for entry in walker {
            let entry = entry
                .map_err(|e| TemplateError::IoError(format!("Failed to read directory entry: {}", e)))?;

            if entry.file_type().is_file() && self.should_include_file(&entry.path()) {
                let name = self.template_name_from_path(&entry.path());
                let content = std::fs::read_to_string(entry.path())
                    .map_err(|e| TemplateError::IoError(format!("Failed to read template file {:?}: {}", entry.path(), e)))?;

                templates.insert(name, content);
            }
        }

        Ok(())
    }

    /// Scan path with glob pattern
    fn scan_path_with_glob(&self, path: &Path, glob_set: &globset::GlobSet, templates: &mut HashMap<String, String>) -> Result<()> {
        use walkdir::WalkDir;

        let walker = if self.recursive {
            WalkDir::new(path)
        } else {
            WalkDir::new(path).max_depth(1)
        };

        for entry in walker {
            let entry = entry
                .map_err(|e| TemplateError::IoError(format!("Failed to read directory entry: {}", e)))?;

            if entry.file_type().is_file() {
                let path_str = entry.path().to_string_lossy();
                if glob_set.is_match(&*path_str) && self.should_include_file(&entry.path()) {
                    let name = self.template_name_from_path(&entry.path());
                    let content = std::fs::read_to_string(entry.path())
                        .map_err(|e| TemplateError::IoError(format!("Failed to read template file {:?}: {}", entry.path(), e)))?;

                    templates.insert(name, content);
                }
            }
        }

        Ok(())
    }

    /// Check if file should be included based on extension
    fn should_include_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
            self.extensions.contains(&extension.to_string())
        } else {
            false
        }
    }

    /// Generate template name from file path
    fn template_name_from_path(&self, path: &Path) -> String {
        // Remove extension and convert path separators to dots
        let stem = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        // For relative paths within search paths, use relative structure
        for search_path in &self.search_paths {
            if let Ok(relative_path) = path.strip_prefix(search_path) {
                let relative_str = relative_path.to_string_lossy().replace(['/', '\\'], ".");
                let name_without_ext = Path::new(&relative_str).file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(stem);

                return match &self.organization {
                    TemplateOrganization::Flat => name_without_ext.to_string(),
                    TemplateOrganization::Hierarchical => {
                        // Use full relative path as template name
                        let parent = relative_path.parent()
                            .and_then(|p| p.to_str())
                            .unwrap_or("");
                        if parent.is_empty() {
                            name_without_ext.to_string()
                        } else {
                            format!("{}.{}", parent.replace(['/', '\\'], "."), name_without_ext)
                        }
                    }
                    TemplateOrganization::Custom { prefix } => {
                        format!("{}.{}", prefix, name_without_ext)
                    }
                };
            }
        }

        stem.to_string()
    }
}

/// Template loader with loaded templates ready for rendering
///
/// Provides template rendering with loaded template collection.
/// Supports hot-reload if enabled during discovery.
#[derive(Debug)]
pub struct TemplateLoader {
    /// Loaded templates (name -> content)
    templates: HashMap<String, String>,
    /// Hot-reload enabled
    hot_reload: bool,
    /// Template organization strategy
    organization: TemplateOrganization,
}

impl TemplateLoader {
    /// Create new template loader
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            hot_reload: false,
            organization: TemplateOrganization::Hierarchical,
        }
    }

    /// Get template content by name
    pub fn get_template(&self, name: &str) -> Option<&str> {
        self.templates.get(name).map(|s| s.as_str())
    }

    /// Check if template exists
    pub fn has_template(&self, name: &str) -> bool {
        self.templates.contains_key(name)
    }

    /// List all available template names
    pub fn template_names(&self) -> Vec<&str> {
        self.templates.keys().map(|s| s.as_str()).collect()
    }

    /// List templates by category (for hierarchical organization)
    pub fn templates_by_category(&self) -> HashMap<String, Vec<String>> {
        let mut categories = HashMap::new();

        for name in self.templates.keys() {
            let category = if let Some(dot_pos) = name.rfind('.') {
                name[..dot_pos].to_string()
            } else {
                "root".to_string()
            };

            categories.entry(category).or_insert_with(Vec::new).push(name.clone());
        }

        categories
    }

    /// Create template renderer with loaded templates
    ///
    /// # Arguments
    /// * `context` - Template context for rendering
    /// * `determinism` - Optional determinism configuration
    pub fn create_renderer(&self, context: crate::context::TemplateContext) -> Result<TemplateRenderer> {
        let mut renderer = TemplateRenderer::new()?;

        // Add all loaded templates
        for (name, content) in &self.templates {
            renderer.add_template(name, content)
                .map_err(|e| TemplateError::RenderError(format!("Failed to add template '{}': {}", name, e)))?;
        }

        Ok(renderer.with_context(context))
    }

    /// Render template by name
    ///
    /// # Arguments
    /// * `name` - Template name
    /// * `context` - Template context
    /// * `determinism` - Optional determinism configuration
    pub fn render(&self, name: &str, context: crate::context::TemplateContext) -> Result<String> {
        let mut renderer = self.create_renderer(context)?;
        renderer.render_str(&self.templates[name], name)
    }

    /// Render template with user variables
    ///
    /// Convenience method for simple rendering with user vars
    pub fn render_with_vars(&self, name: &str, user_vars: std::collections::HashMap<String, serde_json::Value>) -> Result<String> {
        let mut context = crate::context::TemplateContext::with_defaults();
        context.merge_user_vars(user_vars);
        self.render(name, context)
    }

    /// Save all templates to files (for template generation)
    ///
    /// # Arguments
    /// * `output_dir` - Directory to save templates to
    pub fn save_to_directory<P: AsRef<Path>>(&self, output_dir: P) -> Result<()> {
        let output_dir = output_dir.as_ref();

        // Create output directory if it doesn't exist
        std::fs::create_dir_all(output_dir)
            .map_err(|e| TemplateError::IoError(format!("Failed to create output directory: {}", e)))?;

        for (name, content) in &self.templates {
            let file_path = self.template_path_from_name(name, output_dir);
            std::fs::write(&file_path, content)
                .map_err(|e| TemplateError::IoError(format!("Failed to write template '{}': {}", name, e)))?;
        }

        Ok(())
    }

    /// Convert template name back to file path
    fn template_path_from_name(&self, name: &str, base_dir: &Path) -> PathBuf {
        match &self.organization {
            TemplateOrganization::Flat => {
                base_dir.join(format!("{}.toml", name))
            }
            TemplateOrganization::Hierarchical => {
                // Convert dots back to path separators
                let path_str = name.replace('.', "/");
                base_dir.join(format!("{}.toml", path_str))
            }
            TemplateOrganization::Custom { prefix } => {
                // Remove prefix and convert to path
                let path_part = if name.starts_with(&format!("{}.", prefix)) {
                    &name[prefix.len() + 1..]
                } else {
                    name
                };
                let path_str = path_part.replace('.', "/");
                base_dir.join(format!("{}.toml", path_str))
            }
        }
    }
}

/// Fluent API for template loading
pub struct TemplateLoaderBuilder {
    discovery: TemplateDiscovery,
}

impl TemplateLoaderBuilder {
    /// Start building template loader
    pub fn new() -> Self {
        Self {
            discovery: TemplateDiscovery::new(),
        }
    }

    /// Add search path
    pub fn search_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.discovery.search_paths.push(path.as_ref().to_path_buf());
        self
    }

    /// Add glob pattern
    pub fn glob_pattern(mut self, pattern: &str) -> Self {
        self.discovery.glob_patterns.push(pattern.to_string());
        self
    }

    /// Add namespace template
    pub fn namespace<S: Into<String>>(mut self, name: S, content: S) -> Self {
        self.discovery.namespaces.insert(name.into(), content.into());
        self
    }

    /// Enable hot reload
    pub fn hot_reload(mut self) -> Self {
        self.discovery.hot_reload = true;
        self
    }

    /// Set organization strategy
    pub fn organization(mut self, organization: TemplateOrganization) -> Self {
        self.discovery.organization = organization;
        self
    }

    /// Build the template loader
    pub fn build(self) -> Result<TemplateLoader> {
        self.discovery.load()
    }
}

impl Default for TemplateLoaderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_template_discovery_basic() -> Result<()> {
        let temp_dir = tempdir()?;
        let template_file = temp_dir.path().join("test.toml");
        std::fs::write(&template_file, "name = \"{{ test_var }}\"")?;

        let discovery = TemplateDiscovery::new()
            .with_search_path(&temp_dir)
            .recursive(false);

        let loader = discovery.load()?;

        assert!(loader.has_template("test"));
        assert_eq!(loader.get_template("test"), Some("name = \"{{ test_var }}\""));

        Ok(())
    }

    #[test]
    fn test_template_discovery_with_namespace() -> Result<()> {
        let discovery = TemplateDiscovery::new()
            .with_namespace("macros", "{% macro test() %}Hello{% endmacro %}");

        let loader = discovery.load()?;

        assert!(loader.has_template("macros"));
        assert_eq!(loader.get_template("macros"), Some("{% macro test() %}Hello{% endmacro %}"));

        Ok(())
    }

    #[test]
    fn test_template_loader_rendering() -> Result<()> {
        let temp_dir = tempdir()?;
        let template_file = temp_dir.path().join("config.toml");
        std::fs::write(&template_file, "service = \"{{ svc }}\"")?;

        let discovery = TemplateDiscovery::new()
            .with_search_path(&temp_dir);

        let loader = discovery.load()?;

        let mut vars = std::collections::HashMap::new();
        vars.insert("svc".to_string(), serde_json::Value::String("test-service".to_string()));

        let result = loader.render_with_vars("config", vars)?;
        assert_eq!(result.trim(), "service = \"test-service\"");

        Ok(())
    }

    #[test]
    fn test_hierarchical_organization() -> Result<()> {
        let temp_dir = tempdir()?;
        let subdir = temp_dir.path().join("services");
        std::fs::create_dir_all(&subdir)?;

        let template_file = subdir.join("api.toml");
        std::fs::write(&template_file, "service = \"api\"")?;

        let discovery = TemplateDiscovery::new()
            .with_search_path(&temp_dir)
            .with_organization(TemplateOrganization::Hierarchical);

        let loader = discovery.load()?;

        assert!(loader.has_template("services.api"));

        Ok(())
    }
}