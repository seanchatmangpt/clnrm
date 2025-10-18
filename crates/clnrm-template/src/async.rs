//! Async template rendering support
//!
//! Provides async versions of template rendering functions for use in async applications:
//! - Async template rendering
//! - Async file operations
//! - Async template discovery
//! - Async caching and hot-reload

use crate::error::{TemplateError, Result};
use crate::context::TemplateContext;
use crate::renderer::{TemplateRenderer, OutputFormat};
use std::collections::HashMap;
use std::path::Path;
use serde_json::Value;

/// Async template renderer for async applications
///
/// Provides async versions of all template rendering operations
pub struct AsyncTemplateRenderer {
    /// Base template renderer
    renderer: TemplateRenderer,
}

impl AsyncTemplateRenderer {
    /// Create new async template renderer
    pub async fn new() -> Result<Self> {
        let renderer = TemplateRenderer::new()?;
        Ok(Self { renderer })
    }

    /// Create renderer with default context
    pub async fn with_defaults() -> Result<Self> {
        let renderer = TemplateRenderer::with_defaults()?;
        Ok(Self { renderer })
    }

    /// Set template context
    pub fn with_context(mut self, context: TemplateContext) -> Self {
        self.renderer = self.renderer.with_context(context);
        self
    }

    /// Render template string asynchronously
    ///
    /// # Arguments
    /// * `template` - Template content
    /// * `name` - Template name for error reporting
    pub async fn render_str(&mut self, template: &str, name: &str) -> Result<String> {
        // Template rendering is CPU-bound, so we run it in a blocking task
        tokio::task::spawn_blocking(move || {
            self.renderer.render_str(template, name)
        })
        .await
        .map_err(|e| TemplateError::InternalError(format!("Async rendering failed: {}", e)))?
    }

    /// Render template to specific format
    ///
    /// # Arguments
    /// * `template` - Template content
    /// * `name` - Template name
    /// * `format` - Output format
    pub async fn render_to_format(&mut self, template: &str, name: &str, format: OutputFormat) -> Result<String> {
        let rendered = self.render_str(template, name).await?;

        match format {
            OutputFormat::Toml => Ok(rendered),
            OutputFormat::Json => crate::simple::convert_to_json(&rendered),
            OutputFormat::Yaml => crate::simple::convert_to_yaml(&rendered),
            OutputFormat::Plain => crate::simple::strip_template_syntax(&rendered),
        }
    }

    /// Render template file asynchronously
    ///
    /// # Arguments
    /// * `path` - Path to template file
    pub async fn render_file<P: AsRef<Path>>(&mut self, path: P) -> Result<String> {
        let path = path.as_ref().to_path_buf();
        tokio::task::spawn_blocking(move || {
            let mut renderer = TemplateRenderer::new()?;
            renderer.render_file(&path)
        })
        .await
        .map_err(|e| TemplateError::InternalError(format!("Async file rendering failed: {}", e)))?
    }

    /// Merge user variables into context
    pub fn merge_user_vars(&mut self, user_vars: HashMap<String, Value>) {
        self.renderer.merge_user_vars(user_vars);
    }

    /// Access the underlying renderer
    pub fn renderer(&self) -> &TemplateRenderer {
        &self.renderer
    }

    /// Access the underlying renderer mutably
    pub fn renderer_mut(&mut self) -> &mut TemplateRenderer {
        &mut self.renderer
    }
}

/// Async convenience functions for simple template rendering

/// Render template string asynchronously
///
/// # Arguments
/// * `template` - Template content
/// * `vars` - Variables as key-value pairs
pub async fn async_render(template: &str, vars: HashMap<&str, &str>) -> Result<String> {
    let mut json_vars = HashMap::new();
    for (key, value) in vars {
        json_vars.insert(key.to_string(), Value::String(value.to_string()));
    }

    let mut renderer = AsyncTemplateRenderer::new().await?;
    renderer.merge_user_vars(json_vars);
    renderer.render_str(template, "async_template").await
}

/// Render template file asynchronously
///
/// # Arguments
/// * `path` - Path to template file
/// * `vars` - Variables as key-value pairs
pub async fn async_render_file<P: AsRef<Path>>(path: P, vars: HashMap<&str, &str>) -> Result<String> {
    let mut json_vars = HashMap::new();
    for (key, value) in vars {
        json_vars.insert(key.to_string(), Value::String(value.to_string()));
    }

    let mut renderer = AsyncTemplateRenderer::new().await?;
    renderer.merge_user_vars(json_vars);
    renderer.render_file(path).await
}

/// Render template with JSON variables asynchronously
pub async fn async_render_with_json(template: &str, vars: HashMap<&str, Value>) -> Result<String> {
    let mut json_vars = HashMap::new();
    for (key, value) in vars {
        json_vars.insert(key.to_string(), value);
    }

    let mut renderer = AsyncTemplateRenderer::new().await?;
    renderer.merge_user_vars(json_vars);
    renderer.render_str(template, "async_template").await
}

/// Async template builder for fluent configuration
///
/// Provides async versions of the template builder API
pub struct AsyncTemplateBuilder {
    template: Option<String>,
    variables: HashMap<String, Value>,
    format: OutputFormat,
    context: Option<TemplateContext>,
}

impl Default for AsyncTemplateBuilder {
    fn default() -> Self {
        Self {
            template: None,
            variables: HashMap::new(),
            format: OutputFormat::Toml,
            context: None,
        }
    }
}

impl AsyncTemplateBuilder {
    /// Create new async template builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set template content
    pub fn template<S: Into<String>>(mut self, template: S) -> Self {
        self.template = Some(template.into());
        self
    }

    /// Add string variable
    pub fn variable<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.variables.insert(key.into(), Value::String(value.into()));
        self
    }

    /// Add JSON variable
    pub fn json_variable<K: Into<String>>(mut self, key: K, value: Value) -> Self {
        self.variables.insert(key.into(), value);
        self
    }

    /// Set output format
    pub fn format(mut self, format: OutputFormat) -> Self {
        self.format = format;
        self
    }

    /// Set custom context
    pub fn context(mut self, context: TemplateContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Render template asynchronously
    pub async fn render(self) -> Result<String> {
        let template = self.template
            .ok_or_else(|| TemplateError::ValidationError("No template provided".to_string()))?;

        if let Some(context) = self.context {
            let mut renderer = AsyncTemplateRenderer::new().await?.with_context(context);
            let result = renderer.render_str(&template, "async_template").await?;

            match self.format {
                OutputFormat::Toml => Ok(result),
                OutputFormat::Json => crate::simple::convert_to_json(&result),
                OutputFormat::Yaml => crate::simple::convert_to_yaml(&result),
                OutputFormat::Plain => crate::simple::strip_template_syntax(&result),
            }
        } else {
            let mut json_vars = HashMap::new();
            for (key, value) in self.variables {
                json_vars.insert(key, value);
            }

            let mut renderer = AsyncTemplateRenderer::new().await?;
            renderer.merge_user_vars(json_vars);
            let result = renderer.render_str(&template, "async_template").await?;

            match self.format {
                OutputFormat::Toml => Ok(result),
                OutputFormat::Json => crate::simple::convert_to_json(&result),
                OutputFormat::Yaml => crate::simple::convert_to_yaml(&result),
                OutputFormat::Plain => crate::simple::strip_template_syntax(&result),
            }
        }
    }
}

/// Async TOML file operations for async applications
pub mod async_toml {
    use super::*;
    use crate::toml::{TomlFile, TomlLoader, TomlWriter};
    use std::collections::HashMap;

    /// Load TOML file asynchronously
    ///
    /// # Arguments
    /// * `path` - Path to TOML file
    pub async fn load_toml_file<P: AsRef<Path>>(path: P) -> Result<TomlFile> {
        let path = path.as_ref().to_path_buf();
        tokio::task::spawn_blocking(move || {
            let loader = TomlLoader::new();
            loader.load_file(path)
        })
        .await
        .map_err(|e| TemplateError::InternalError(format!("Async TOML loading failed: {}", e)))?
    }

    /// Load all TOML files from directory asynchronously
    ///
    /// # Arguments
    /// * `search_paths` - Directories to search
    pub async fn load_all_toml_files(search_paths: Vec<&Path>) -> Result<HashMap<PathBuf, TomlFile>> {
        let paths: Vec<PathBuf> = search_paths.iter().map(|p| p.to_path_buf()).collect();
        tokio::task::spawn_blocking(move || {
            let loader = TomlLoader::new().with_search_paths(paths);
            loader.load_all()
        })
        .await
        .map_err(|e| TemplateError::InternalError(format!("Async TOML directory loading failed: {}", e)))?
    }

    /// Write TOML file asynchronously
    ///
    /// # Arguments
    /// * `path` - Target file path
    /// * `content` - TOML content to write
    /// * `validator` - Optional validator
    pub async fn write_toml_file<P: AsRef<Path>>(
        path: P,
        content: &str,
        validator: Option<&crate::validation::TemplateValidator>
    ) -> Result<()> {
        let path = path.as_ref().to_path_buf();
        let content = content.to_string();
        let validator = validator.cloned();

        tokio::task::spawn_blocking(move || {
            let writer = TomlWriter::new();
            writer.write_file(path, &content, validator.as_ref())
        })
        .await
        .map_err(|e| TemplateError::InternalError(format!("Async TOML writing failed: {}", e)))?
    }
}

/// Async template discovery for large codebases
pub mod async_discovery {
    use super::*;
    use crate::discovery::{TemplateDiscovery, TemplateLoader};

    /// Discover templates asynchronously
    ///
    /// # Arguments
    /// * `search_paths` - Directories to search
    /// * `patterns` - Glob patterns to match
    pub async fn discover_templates(
        search_paths: Vec<&Path>,
        patterns: Vec<&str>
    ) -> Result<TemplateLoader> {
        let paths: Vec<PathBuf> = search_paths.iter().map(|p| p.to_path_buf()).collect();
        let patterns: Vec<String> = patterns.iter().map(|s| s.to_string()).collect();

        tokio::task::spawn_blocking(move || {
            let mut discovery = TemplateDiscovery::new();
            for path in paths {
                discovery = discovery.with_search_path(path);
            }
            for pattern in patterns {
                discovery = discovery.with_glob_pattern(&pattern);
            }
            discovery.load()
        })
        .await
        .map_err(|e| TemplateError::InternalError(format!("Async template discovery failed: {}", e)))?
    }
}

/// Async template validation for large templates
pub mod async_validation {
    use super::*;

    /// Validate template output asynchronously
    ///
    /// # Arguments
    /// * `output` - Rendered template content
    /// * `template_name` - Template name for error reporting
    /// * `validator` - Template validator
    pub async fn validate_async(
        output: &str,
        template_name: &str,
        validator: &crate::validation::TemplateValidator
    ) -> Result<()> {
        tokio::task::spawn_blocking(move || {
            validator.validate(output, template_name)
        })
        .await
        .map_err(|e| TemplateError::InternalError(format!("Async validation failed: {}", e)))?
    }
}

/// Async template caching for high-performance applications
pub mod async_cache {
    use super::*;
    use crate::cache::{TemplateCache, CachedRenderer};

    /// Create async cached renderer
    ///
    /// # Arguments
    /// * `context` - Template context
    /// * `hot_reload` - Enable hot-reload
    pub async fn create_async_cached_renderer(
        context: TemplateContext,
        hot_reload: bool
    ) -> Result<CachedRenderer> {
        tokio::task::spawn_blocking(move || {
            CachedRenderer::new(context, hot_reload)
        })
        .await
        .map_err(|e| TemplateError::InternalError(format!("Async cached renderer creation failed: {}", e)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_render() {
        let result = async_render("Hello {{ name }}!", [("name", "World")].iter().cloned().collect()).await.unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[tokio::test]
    async fn test_async_template_builder() {
        let result = AsyncTemplateBuilder::new()
            .template("Service: {{ service }}")
            .variable("service", "my-service")
            .render()
            .await
            .unwrap();

        assert_eq!(result, "Service: my-service");
    }

    #[tokio::test]
    async fn test_async_toml_loading() {
        use tempfile::tempdir;
        use std::fs;

        let temp_dir = tempdir().unwrap();
        let toml_file = temp_dir.path().join("test.toml");

        let content = r#"
[service]
name = "test-service"
        "#;

        fs::write(&toml_file, content).unwrap();

        let file = async_toml::load_toml_file(&toml_file).await.unwrap();
        assert_eq!(file.path, toml_file);
        assert!(file.parsed.get("service").is_some());
    }
}
