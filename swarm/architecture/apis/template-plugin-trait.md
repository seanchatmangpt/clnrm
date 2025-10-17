# Template Plugin Trait Interface - v0.6.0

**API Architect Report - Architecture Sub-Coordinator**

## Overview

Template plugins extend Tera's capabilities for .clnrm.toml rendering with custom functions, filters, and context providers. All trait methods are **dyn-compatible** (sync methods only) following core team standards.

## Core Trait: `TemplatePlugin`

```rust
//! Template plugin trait for extending Tera rendering capabilities
//!
//! Provides extension points for:
//! - Custom Tera functions ({{ custom_fn() }})
//! - Custom Tera filters ({{ value | custom_filter }})
//! - Context providers (dynamic variable injection)
//! - Template validators (pre-render validation)

use crate::error::{CleanroomError, Result};
use std::collections::HashMap;
use tera::{Tera, Value, Function, Filter};

/// Template plugin for extending template rendering
///
/// All methods are sync (dyn-compatible) and return Result<T, CleanroomError>
pub trait TemplatePlugin: Send + Sync + std::fmt::Debug {
    /// Plugin identifier (unique name)
    fn name(&self) -> &str;

    /// Register custom Tera functions
    ///
    /// # Example
    /// ```rust
    /// fn register_functions(&self, tera: &mut Tera) -> Result<()> {
    ///     tera.register_function("custom_fn", MyCustomFunction);
    ///     Ok(())
    /// }
    /// ```
    fn register_functions(&self, tera: &mut Tera) -> Result<()> {
        // Default: no custom functions
        let _ = tera; // Prevent unused variable warning
        Ok(())
    }

    /// Register custom Tera filters
    ///
    /// # Example
    /// ```rust
    /// fn register_filters(&self, tera: &mut Tera) -> Result<()> {
    ///     tera.register_filter("uppercase", UppercaseFilter);
    ///     Ok(())
    /// }
    /// ```
    fn register_filters(&self, tera: &mut Tera) -> Result<()> {
        // Default: no custom filters
        let _ = tera;
        Ok(())
    }

    /// Provide context variables
    ///
    /// Returns key-value pairs to inject into template context
    ///
    /// # Example
    /// ```rust
    /// fn provide_context(&self) -> Result<HashMap<String, Value>> {
    ///     let mut ctx = HashMap::new();
    ///     ctx.insert("timestamp".to_string(),
    ///                Value::String(chrono::Utc::now().to_rfc3339()));
    ///     Ok(ctx)
    /// }
    /// ```
    fn provide_context(&self) -> Result<HashMap<String, Value>> {
        // Default: no context variables
        Ok(HashMap::new())
    }

    /// Validate template before rendering
    ///
    /// Returns validation errors if template is invalid
    ///
    /// # Example
    /// ```rust
    /// fn validate_template(&self, template_content: &str) -> Result<()> {
    ///     if !template_content.contains("{{ required_var }}") {
    ///         return Err(CleanroomError::validation_error(
    ///             "Template missing required_var variable"
    ///         ));
    ///     }
    ///     Ok(())
    /// }
    /// ```
    fn validate_template(&self, _template_content: &str) -> Result<()> {
        // Default: no validation
        Ok(())
    }

    /// Get plugin metadata
    ///
    /// Returns plugin version, description, and other metadata
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: self.name().to_string(),
            version: "1.0.0".to_string(),
            description: "Template plugin".to_string(),
            author: None,
            provides_functions: Vec::new(),
            provides_filters: Vec::new(),
            provides_context_keys: Vec::new(),
        }
    }
}

/// Plugin metadata structure
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    /// Plugin name
    pub name: String,
    /// Plugin version (semver)
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Plugin author
    pub author: Option<String>,
    /// List of Tera functions this plugin provides
    pub provides_functions: Vec<String>,
    /// List of Tera filters this plugin provides
    pub provides_filters: Vec<String>,
    /// List of context keys this plugin provides
    pub provides_context_keys: Vec<String>,
}
```

## Plugin Registry

```rust
/// Template plugin registry for managing and applying plugins
#[derive(Debug, Default)]
pub struct TemplatePluginRegistry {
    /// Registered plugins by name
    plugins: HashMap<String, Box<dyn TemplatePlugin>>,
}

impl TemplatePluginRegistry {
    /// Create new plugin registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a template plugin
    ///
    /// # Errors
    /// Returns error if plugin with same name already registered
    pub fn register(&mut self, plugin: Box<dyn TemplatePlugin>) -> Result<()> {
        let name = plugin.name().to_string();

        if self.plugins.contains_key(&name) {
            return Err(CleanroomError::configuration_error(
                format!("Template plugin '{}' already registered", name)
            ));
        }

        self.plugins.insert(name, plugin);
        Ok(())
    }

    /// Apply all plugins to Tera instance
    ///
    /// Registers functions, filters from all plugins
    pub fn apply_to_tera(&self, tera: &mut Tera) -> Result<()> {
        for plugin in self.plugins.values() {
            plugin.register_functions(tera)?;
            plugin.register_filters(tera)?;
        }
        Ok(())
    }

    /// Get combined context from all plugins
    pub fn collect_context(&self) -> Result<HashMap<String, Value>> {
        let mut combined = HashMap::new();

        for plugin in self.plugins.values() {
            let plugin_ctx = plugin.provide_context()?;
            for (key, value) in plugin_ctx {
                if combined.contains_key(&key) {
                    tracing::warn!(
                        "Context key '{}' from plugin '{}' overrides existing value",
                        key, plugin.name()
                    );
                }
                combined.insert(key, value);
            }
        }

        Ok(combined)
    }

    /// Validate template with all plugins
    pub fn validate_template(&self, template_content: &str) -> Result<()> {
        for plugin in self.plugins.values() {
            plugin.validate_template(template_content)?;
        }
        Ok(())
    }

    /// List all registered plugins
    pub fn list_plugins(&self) -> Vec<PluginMetadata> {
        self.plugins.values()
            .map(|p| p.metadata())
            .collect()
    }

    /// Get plugin by name
    pub fn get(&self, name: &str) -> Option<&dyn TemplatePlugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }
}
```

## Built-in Plugins

### 1. Environment Plugin

```rust
/// Environment variable access plugin
#[derive(Debug, Clone, Default)]
pub struct EnvPlugin;

impl EnvPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl TemplatePlugin for EnvPlugin {
    fn name(&self) -> &str {
        "env"
    }

    fn register_functions(&self, tera: &mut Tera) -> Result<()> {
        tera.register_function("env", EnvFunction);
        Ok(())
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "env".to_string(),
            version: "1.0.0".to_string(),
            description: "Environment variable access via env(name=\"VAR\")".to_string(),
            author: Some("clnrm-core".to_string()),
            provides_functions: vec!["env".to_string()],
            provides_filters: Vec::new(),
            provides_context_keys: Vec::new(),
        }
    }
}
```

### 2. Determinism Plugin

```rust
/// Deterministic timestamp and hashing plugin
#[derive(Debug, Clone)]
pub struct DeterminismPlugin {
    config: DeterminismConfig,
}

impl DeterminismPlugin {
    pub fn new(config: DeterminismConfig) -> Self {
        Self { config }
    }
}

impl TemplatePlugin for DeterminismPlugin {
    fn name(&self) -> &str {
        "determinism"
    }

    fn register_functions(&self, tera: &mut Tera) -> Result<()> {
        let now_fn = NowRfc3339Function::new();

        // Freeze clock if configured
        if let Some(ref frozen_time) = self.config.freeze_clock {
            now_fn.freeze(frozen_time.clone());
        }

        tera.register_function("now_rfc3339", now_fn);
        tera.register_function("sha256", Sha256Function);
        Ok(())
    }

    fn provide_context(&self) -> Result<HashMap<String, Value>> {
        let mut ctx = HashMap::new();

        if let Some(seed) = self.config.seed {
            ctx.insert("determinism_seed".to_string(),
                      Value::Number(seed.into()));
        }

        if let Some(ref frozen) = self.config.freeze_clock {
            ctx.insert("frozen_time".to_string(),
                      Value::String(frozen.clone()));
        }

        Ok(ctx)
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "determinism".to_string(),
            version: "1.0.0".to_string(),
            description: "Deterministic timestamps and hashing".to_string(),
            author: Some("clnrm-core".to_string()),
            provides_functions: vec!["now_rfc3339".to_string(), "sha256".to_string()],
            provides_filters: Vec::new(),
            provides_context_keys: vec![
                "determinism_seed".to_string(),
                "frozen_time".to_string(),
            ],
        }
    }
}
```

### 3. TOML Encoding Plugin

```rust
/// TOML encoding plugin
#[derive(Debug, Clone, Default)]
pub struct TomlPlugin;

impl TomlPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl TemplatePlugin for TomlPlugin {
    fn name(&self) -> &str {
        "toml"
    }

    fn register_functions(&self, tera: &mut Tera) -> Result<()> {
        tera.register_function("toml_encode", TomlEncodeFunction);
        Ok(())
    }

    fn register_filters(&self, tera: &mut Tera) -> Result<()> {
        tera.register_filter("toml", TomlEncodeFilter);
        Ok(())
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "toml".to_string(),
            version: "1.0.0".to_string(),
            description: "TOML encoding for template values".to_string(),
            author: Some("clnrm-core".to_string()),
            provides_functions: vec!["toml_encode".to_string()],
            provides_filters: vec!["toml".to_string()],
            provides_context_keys: Vec::new(),
        }
    }
}
```

## Usage Examples

### Example 1: Registering Built-in Plugins

```rust
use clnrm_core::template::{TemplateRenderer, TemplatePluginRegistry};
use clnrm_core::template::plugins::{EnvPlugin, DeterminismPlugin, TomlPlugin};

// Create plugin registry
let mut registry = TemplatePluginRegistry::new();

// Register built-in plugins
registry.register(Box::new(EnvPlugin::new()))?;
registry.register(Box::new(DeterminismPlugin::new(determinism_config)))?;
registry.register(Box::new(TomlPlugin::new()))?;

// Apply to renderer
let mut renderer = TemplateRenderer::new()?;
registry.apply_to_tera(renderer.tera_mut())?;

// Collect and merge context
let plugin_context = registry.collect_context()?;
renderer.add_context_vars(plugin_context)?;
```

### Example 2: Custom Plugin Implementation

```rust
/// Custom plugin for CI/CD variable injection
#[derive(Debug, Clone)]
pub struct CiCdPlugin {
    ci_vars: HashMap<String, String>,
}

impl TemplatePlugin for CiCdPlugin {
    fn name(&self) -> &str {
        "cicd"
    }

    fn provide_context(&self) -> Result<HashMap<String, Value>> {
        let mut ctx = HashMap::new();

        for (key, value) in &self.ci_vars {
            ctx.insert(
                format!("ci_{}", key.to_lowercase()),
                Value::String(value.clone())
            );
        }

        Ok(ctx)
    }

    fn validate_template(&self, template_content: &str) -> Result<()> {
        // Ensure CI variables are used correctly
        if template_content.contains("{{ ci_") && !template_content.contains("cicd") {
            return Err(CleanroomError::validation_error(
                "CI variables must use 'cicd' plugin context"
            ));
        }
        Ok(())
    }
}
```

## API Contract Guarantees

1. **Dyn Compatibility**: All trait methods are sync (no async)
2. **Error Handling**: All methods return `Result<T, CleanroomError>`
3. **No Panics**: No `.unwrap()` or `.expect()` in production paths
4. **Thread Safety**: All plugins are `Send + Sync`
5. **Composability**: Multiple plugins can be registered and composed
6. **Namespace Isolation**: Plugins should namespace their context keys

## Testing Requirements

All plugin implementations MUST include:

1. Unit tests for each trait method
2. Integration tests with Tera renderer
3. Error case coverage (invalid inputs)
4. Thread safety tests (concurrent access)
5. AAA pattern (Arrange, Act, Assert)

## Migration Path

Existing `TemplateRenderer` will be enhanced:

```rust
// Before (v0.5.x)
let renderer = TemplateRenderer::new()?;

// After (v0.6.0)
let mut renderer = TemplateRenderer::new()?;
let mut registry = TemplatePluginRegistry::new();
registry.register(Box::new(EnvPlugin::new()))?;
registry.apply_to_tera(renderer.tera_mut())?;
```

Built-in functions remain registered by default for backwards compatibility.
