//! Template engine builder for comprehensive configuration
//!
//! Provides a fluent API for configuring template engines with all available features:
//! - Template discovery and loading
//! - Context configuration
//! - Validation rules
//! - Custom functions and filters
//! - Caching and performance options
//! - Output format configuration

use crate::error::{TemplateError, Result};
use crate::context::{TemplateContext, TemplateContextBuilder};
use crate::renderer::{TemplateRenderer, OutputFormat};
use crate::discovery::{TemplateDiscovery, TemplateLoader, TemplateOrganization};
use crate::validation::{TemplateValidator, ValidationRule};
use crate::cache::{TemplateCache, CachedRenderer};
use crate::custom::{CustomFunction, CustomFilter, FunctionRegistry};
use crate::toml::{TomlLoader, TomlWriter, TomlMerger};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use serde_json::Value;

/// Comprehensive template engine builder
///
/// Provides a fluent API for configuring all aspects of the template engine:
///
/// ```rust
/// use clnrm_template::TemplateEngineBuilder;
///
/// let engine = TemplateEngineBuilder::new()
///     .with_search_paths(vec!["./templates", "./configs"])
///     .with_context_defaults()
///     .with_validation_rules(vec![
///         ValidationRule::ServiceName,
///         ValidationRule::Semver,
///     ])
///     .with_custom_function("my_func", |args| Ok(Value::String("custom".to_string())))
///     .with_cache(Duration::from_secs(3600))
///     .with_hot_reload(true)
///     .build()
///     .unwrap();
/// ```
pub struct TemplateEngineBuilder {
    /// Template discovery configuration
    discovery: TemplateDiscovery,
    /// Context configuration
    context_builder: TemplateContextBuilder,
    /// Validation configuration
    validator: TemplateValidator,
    /// Custom function registry
    function_registry: FunctionRegistry,
    /// TOML operations configuration
    toml_loader: TomlLoader,
    toml_writer: TomlWriter,
    toml_merger: TomlMerger,
    /// Cache configuration
    cache_config: Option<(bool, Duration)>, // (hot_reload, ttl)
    /// Output format
    output_format: OutputFormat,
    /// Debug configuration
    debug_enabled: bool,
}

impl Default for TemplateEngineBuilder {
    fn default() -> Self {
        Self {
            discovery: TemplateDiscovery::new(),
            context_builder: TemplateContextBuilder::new(),
            validator: TemplateValidator::new(),
            function_registry: FunctionRegistry::new(),
            toml_loader: TomlLoader::new(),
            toml_writer: TomlWriter::new(),
            toml_merger: TomlMerger::new(),
            cache_config: None,
            output_format: OutputFormat::Toml,
            debug_enabled: false,
        }
    }
}

impl TemplateEngineBuilder {
    /// Create new template engine builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure template discovery
    pub fn with_discovery<F>(mut self, f: F) -> Self
    where
        F: FnOnce(TemplateDiscovery) -> TemplateDiscovery,
    {
        self.discovery = f(self.discovery);
        self
    }

    /// Add search paths for template discovery
    pub fn with_search_paths<I, P>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        for path in paths {
            self.discovery = self.discovery.with_search_path(path);
        }
        self
    }

    /// Add glob patterns for template discovery
    pub fn with_glob_patterns<I, S>(mut self, patterns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for pattern in patterns {
            self.discovery = self.discovery.with_glob_pattern(pattern.as_ref());
        }
        self
    }

    /// Set template organization strategy
    pub fn with_organization(mut self, organization: TemplateOrganization) -> Self {
        self.discovery = self.discovery.with_organization(organization);
        self
    }

    /// Configure template context
    pub fn with_context<F>(mut self, f: F) -> Self
    where
        F: FnOnce(TemplateContextBuilder) -> TemplateContextBuilder,
    {
        self.context_builder = f(self.context_builder);
        self
    }

    /// Use default PRD v1.0 context variables
    pub fn with_context_defaults(mut self) -> Self {
        self.context_builder = TemplateContextBuilder::with_defaults();
        self
    }

    /// Add context variables
    pub fn with_variables<I, K, V>(mut self, variables: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<Value>,
    {
        for (key, value) in variables {
            self.context_builder = self.context_builder.var(key, value);
        }
        self
    }

    /// Load context from file
    pub fn with_context_from_file<P: AsRef<Path>>(mut self, path: P) -> Result<Self> {
        self.context_builder = self.context_builder.load_vars_from_file(path)?;
        Ok(self)
    }

    /// Configure validation
    pub fn with_validation<F>(mut self, f: F) -> Self
    where
        F: FnOnce(TemplateValidator) -> TemplateValidator,
    {
        self.validator = f(self.validator);
        self
    }

    /// Add validation rules
    pub fn with_validation_rules<I>(mut self, rules: I) -> Self
    where
        I: IntoIterator<Item = ValidationRule>,
    {
        for rule in rules {
            self.validator = self.validator.with_rule(rule);
        }
        self
    }

    /// Set validation format
    pub fn with_validation_format(mut self, format: OutputFormat) -> Self {
        self.validator = self.validator.format(format);
        self
    }

    /// Add custom function
    pub fn with_custom_function<F>(mut self, func: CustomFunction<F>) -> Self
    where
        F: Fn(&HashMap<String, Value>) -> Result<Value> + Send + Sync + 'static,
    {
        self.function_registry = self.function_registry.add_function(func);
        self
    }

    /// Add custom filter
    pub fn with_custom_filter<F>(mut self, filter: CustomFilter<F>) -> Self
    where
        F: Fn(&Value, &HashMap<String, Value>) -> Result<Value> + Send + Sync + 'static,
    {
        self.function_registry = self.function_registry.add_filter(filter);
        self
    }

    /// Configure TOML loading
    pub fn with_toml_loader<F>(mut self, f: F) -> Self
    where
        F: FnOnce(TomlLoader) -> TomlLoader,
    {
        self.toml_loader = f(self.toml_loader);
        self
    }

    /// Configure TOML writing
    pub fn with_toml_writer<F>(mut self, f: F) -> Self
    where
        F: FnOnce(TomlWriter) -> TomlWriter,
    {
        self.toml_writer = f(self.toml_writer);
        self
    }

    /// Configure TOML merging
    pub fn with_toml_merger<F>(mut self, f: F) -> Self
    where
        F: FnOnce(TomlMerger) -> TomlMerger,
    {
        self.toml_merger = f(self.toml_merger);
        self
    }

    /// Configure caching
    pub fn with_cache(mut self, ttl: Duration) -> Self {
        self.cache_config = Some((true, ttl));
        self
    }

    /// Configure caching with hot-reload
    pub fn with_cache_and_reload(mut self, ttl: Duration, hot_reload: bool) -> Self {
        self.cache_config = Some((hot_reload, ttl));
        self
    }

    /// Disable caching
    pub fn without_cache(mut self) -> Self {
        self.cache_config = None;
        self
    }

    /// Set output format
    pub fn with_output_format(mut self, format: OutputFormat) -> Self {
        self.output_format = format;
        self
    }

    /// Enable debug mode
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug_enabled = debug;
        self
    }

    /// Build the template engine configuration
    ///
    /// Returns a configured template loader that can be used for rendering
    pub fn build(self) -> Result<TemplateLoader> {
        // Load templates using discovery configuration
        let loader = self.discovery.load()?;

        // Build context from configuration
        let context = self.context_builder.build();

        // Apply validation rules to templates
        for (name, content) in &loader.templates {
            self.validator.validate(content, name)?;
        }

        Ok(loader)
    }

    /// Build cached renderer for performance
    pub fn build_cached(self) -> Result<CachedRenderer> {
        let context = self.context_builder.build();
        let loader = self.build()?;

        let (hot_reload, ttl) = self.cache_config.unwrap_or((true, Duration::from_secs(3600)));
        CachedRenderer::new(context, hot_reload)
    }

    /// Build async cached renderer (if async feature is enabled)
    #[cfg(feature = "async")]
    pub async fn build_async_cached(self) -> Result<crate::r#async::AsyncTemplateRenderer> {
        let context = self.context_builder.build();
        crate::r#async::AsyncTemplateRenderer::with_defaults().await?.with_context(context)
    }

    /// Build complete template engine with all components
    ///
    /// Returns a struct containing all configured components for advanced usage
    pub fn build_complete(self) -> Result<TemplateEngine> {
        let loader = self.build()?;
        let context = self.context_builder.build();

        let (hot_reload, ttl) = self.cache_config.unwrap_or((true, Duration::from_secs(3600)));
        let cached_renderer = CachedRenderer::new(context.clone(), hot_reload)?;

        Ok(TemplateEngine {
            loader,
            context,
            validator: self.validator,
            function_registry: self.function_registry,
            toml_loader: self.toml_loader,
            toml_writer: self.toml_writer,
            toml_merger: self.toml_merger,
            cache: cached_renderer,
            output_format: self.output_format,
            debug_enabled: self.debug_enabled,
        })
    }
}

/// Complete template engine with all configured components
///
/// Provides access to all template engine components for advanced usage scenarios
pub struct TemplateEngine {
    /// Template loader for template discovery and loading
    pub loader: TemplateLoader,
    /// Template context for variable resolution
    pub context: TemplateContext,
    /// Template validator for output validation
    pub validator: TemplateValidator,
    /// Custom function registry
    pub function_registry: FunctionRegistry,
    /// TOML file loader
    pub toml_loader: TomlLoader,
    /// TOML file writer
    pub toml_writer: TomlWriter,
    /// TOML merger for combining files
    pub toml_merger: TomlMerger,
    /// Cached renderer for performance
    pub cache: CachedRenderer,
    /// Default output format
    pub output_format: OutputFormat,
    /// Debug mode enabled
    pub debug_enabled: bool,
}

impl TemplateEngine {
    /// Render template by name
    ///
    /// # Arguments
    /// * `name` - Template name
    pub fn render(&mut self, name: &str) -> Result<String> {
        self.loader.render(name, self.context.clone())
    }

    /// Render template with custom context
    ///
    /// # Arguments
    /// * `name` - Template name
    /// * `context` - Custom context for rendering
    pub fn render_with_context(&mut self, name: &str, context: TemplateContext) -> Result<String> {
        self.loader.render(name, context)
    }

    /// Render template to specific format
    ///
    /// # Arguments
    /// * `name` - Template name
    /// * `format` - Output format
    pub fn render_to_format(&mut self, name: &str, format: OutputFormat) -> Result<String> {
        let rendered = self.render(name)?;
        match format {
            OutputFormat::Toml => Ok(rendered),
            OutputFormat::Json => crate::simple::convert_to_json(&rendered),
            OutputFormat::Yaml => crate::simple::convert_to_yaml(&rendered),
            OutputFormat::Plain => crate::simple::strip_template_syntax(&rendered),
        }
    }

    /// Validate template output
    ///
    /// # Arguments
    /// * `name` - Template name
    pub fn validate_template(&self, name: &str) -> Result<()> {
        if let Some(content) = self.loader.get_template(name) {
            self.validator.validate(content, name)
        } else {
            Err(TemplateError::ValidationError(format!("Template '{}' not found", name)))
        }
    }

    /// Load TOML file
    ///
    /// # Arguments
    /// * `path` - Path to TOML file
    pub fn load_toml_file<P: AsRef<Path>>(&self, path: P) -> Result<crate::toml::TomlFile> {
        self.toml_loader.load_file(path)
    }

    /// Write TOML file
    ///
    /// # Arguments
    /// * `path` - Target file path
    /// * `content` - TOML content to write
    pub fn write_toml_file<P: AsRef<Path>>(&self, path: P, content: &str) -> Result<()> {
        self.toml_writer.write_file(path, content, Some(&self.validator))
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> crate::cache::CacheStats {
        self.cache.cache_stats()
    }

    /// Clear cache
    pub fn clear_cache(&self) {
        self.cache.clear_cache();
    }
}

/// Preset configurations for common use cases

/// Configuration for web application templates
pub fn web_app_config() -> TemplateEngineBuilder {
    TemplateEngineBuilder::new()
        .with_search_paths(vec!["./templates", "./configs"])
        .with_glob_patterns(vec!["**/*.toml", "**/*.json"])
        .with_context_defaults()
        .with_validation_rules(vec![
            ValidationRule::ServiceName,
            ValidationRule::OtelConfig,
        ])
        .with_output_format(OutputFormat::Json)
        .with_cache(Duration::from_secs(300)) // 5 minutes for web apps
}

/// Configuration for CLI tool templates
pub fn cli_tool_config() -> TemplateEngineBuilder {
    TemplateEngineBuilder::new()
        .with_search_paths(vec!["./templates"])
        .with_context_defaults()
        .with_validation_rules(vec![
            ValidationRule::ServiceName,
        ])
        .with_output_format(OutputFormat::Toml)
        .with_cache(Duration::from_secs(60)) // 1 minute for CLI tools
}

/// Configuration for development workflows
pub fn development_config() -> TemplateEngineBuilder {
    TemplateEngineBuilder::new()
        .with_search_paths(vec!["./templates", "./test-templates"])
        .with_glob_patterns(vec!["**/*.toml", "**/*.tera"])
        .with_context_defaults()
        .with_validation_rules(vec![
            ValidationRule::ServiceName,
            ValidationRule::Semver,
        ])
        .with_debug(true)
        .with_cache_and_reload(Duration::from_secs(30), true) // Hot-reload for development
}

/// Configuration for production deployments
pub fn production_config() -> TemplateEngineBuilder {
    TemplateEngineBuilder::new()
        .with_search_paths(vec!["./templates", "./configs"])
        .with_context_defaults()
        .with_validation_rules(vec![
            ValidationRule::ServiceName,
            ValidationRule::Semver,
            ValidationRule::OtelConfig,
        ])
        .with_cache(Duration::from_secs(3600)) // 1 hour for production
        .with_debug(false)
}

/// Configuration for CI/CD pipelines
pub fn ci_config() -> TemplateEngineBuilder {
    TemplateEngineBuilder::new()
        .with_search_paths(vec!["./.github/templates", "./templates"])
        .with_context_defaults()
        .with_validation_rules(vec![
            ValidationRule::ServiceName,
            ValidationRule::Environment { allowed: vec!["ci".to_string(), "staging".to_string()] },
        ])
        .with_cache(Duration::from_secs(1800)) // 30 minutes for CI
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::rules;

    #[test]
    fn test_template_engine_builder() {
        let builder = TemplateEngineBuilder::new()
            .with_search_paths(vec!["./templates"])
            .with_context_defaults()
            .with_validation_rules(vec![
                rules::service_name(),
                rules::semver(),
            ])
            .with_output_format(OutputFormat::Json)
            .with_cache(Duration::from_secs(300));

        // Build should not fail (would need actual template files to test fully)
        let result = builder.build();
        assert!(result.is_err()); // Expected to fail without actual template files
    }

    #[test]
    fn test_preset_configurations() {
        let web_config = web_app_config();
        assert_eq!(web_config.output_format, OutputFormat::Json);

        let cli_config = cli_tool_config();
        assert_eq!(cli_config.output_format, OutputFormat::Toml);

        let dev_config = development_config();
        assert!(dev_config.debug_enabled);

        let prod_config = production_config();
        assert!(!prod_config.debug_enabled);
    }

    #[test]
    fn test_template_engine_components() {
        let engine = TemplateEngineBuilder::new()
            .with_context_defaults()
            .with_validation_rules(vec![rules::service_name()])
            .build_complete()
            .unwrap();

        // Test that all components are properly configured
        assert!(engine.debug_enabled == false);
        assert!(engine.validator.rules.len() > 0);
        assert!(engine.context.vars.contains_key("svc"));
    }
}
