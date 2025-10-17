# OTEL Weaver Integration Implementation

## Overview

This document provides a concrete, optional implementation of OTEL Weaver integration that enhances Cleanroom's telemetry validation capabilities while maintaining backward compatibility and following plugin architecture patterns.

## Core Design Principles

### Optional Integration
- **Feature-gated**: Behind `weaver-integration` feature flag
- **Zero impact**: No performance overhead when disabled
- **Gradual adoption**: Teams can opt-in incrementally
- **Backward compatible**: Existing OTEL validation continues to work

### Plugin Architecture
- **Modular design**: Weaver integration as optional plugin
- **Loose coupling**: Core validation logic remains independent
- **Extensible**: Easy to add/remove Weaver features
- **Configuration-driven**: Enable/disable via `.clnrm.toml`

## Implementation Architecture

### Feature Flag Structure

```toml
# Cargo.toml
[features]
default = ["otel-traces"]
otel-traces = ["opentelemetry", "tracing-opentelemetry"]
weaver-integration = ["weaver", "weaver-live-check", "serde_yaml"]
```

### Plugin Module Structure

```
crates/clnrm-core/src/
├── validation/
│   ├── otel.rs                    # Existing OTEL validation (unchanged)
│   └── weaver/                    # New Weaver integration plugin
│       ├── mod.rs                 # Plugin entry point
│       ├── registry.rs            # Schema registry management
│       ├── codegen.rs             # Code generation utilities
│       ├── live_check.rs          # Runtime validation
│       └── config.rs              # Weaver-specific configuration
```

## Phase 1: Schema Registry Plugin (Week 1-2)

### 1.1 Core Plugin Interface

```rust
// crates/clnrm-core/src/validation/weaver/mod.rs
use crate::error::{CleanroomError, Result};

pub trait WeaverPlugin: Send + Sync {
    /// Initialize the Weaver plugin with configuration
    fn initialize(&mut self, config: WeaverConfig) -> Result<()>;

    /// Validate telemetry schema at build time
    fn validate_schema(&self, registry_path: &str) -> Result<ValidationReport>;

    /// Generate validation code from schema
    fn generate_validation_code(&self, registry_path: &str, output_dir: &str) -> Result<()>;

    /// Check if plugin is enabled
    fn is_enabled(&self) -> bool;
}

pub struct WeaverRegistryPlugin {
    config: Option<WeaverConfig>,
    registry: Option<Registry>,
}

impl WeaverPlugin for WeaverRegistryPlugin {
    fn initialize(&mut self, config: WeaverConfig) -> Result<()> {
        // Only initialize if explicitly enabled
        if !config.enabled {
            return Ok(());
        }

        // Load and validate schema registry
        let registry = Registry::from_config(&config.registry_config)
            .map_err(|e| CleanroomError::validation_error(format!("Failed to load Weaver registry: {}", e)))?;

        // Validate registry using Weaver's checker
        use weaver::registry::check;
        check::validate_registry(&registry)
            .map_err(|e| CleanroomError::validation_error(format!("Schema validation failed: {}", e)))?;

        self.registry = Some(registry);
        self.config = Some(config);
        Ok(())
    }

    fn validate_schema(&self, _registry_path: &str) -> Result<ValidationReport> {
        if !self.is_enabled() {
            return Err(CleanroomError::validation_error("Weaver plugin is disabled"));
        }

        // Schema already validated during initialization
        Ok(ValidationReport {
            passed: true,
            message: "Schema validation passed".to_string(),
            details: vec![],
        })
    }

    fn generate_validation_code(&self, _registry_path: &str, _output_dir: &str) -> Result<()> {
        if !self.is_enabled() {
            return Ok(()); // No-op when disabled
        }

        // Generate validation code using Weaver templates
        unimplemented!("Code generation from Weaver schemas")
    }

    fn is_enabled(&self) -> bool {
        self.config.as_ref().map(|c| c.enabled).unwrap_or(false)
    }
}
```

### 1.2 Configuration Structure

```rust
// crates/clnrm-core/src/validation/weaver/config.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaverConfig {
    /// Enable Weaver integration
    pub enabled: bool,

    /// Path to Weaver registry configuration
    pub registry_path: String,

    /// Enable code generation
    pub enable_codegen: bool,

    /// Enable runtime live checking
    pub enable_live_check: bool,

    /// Weaver configuration file
    pub registry_config: std::collections::HashMap<String, serde_yaml::Value>,
}

impl Default for WeaverConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default
            registry_path: "telemetry/registry".to_string(),
            enable_codegen: false,
            enable_live_check: false,
            registry_config: std::collections::HashMap::new(),
        }
    }
}
```

### 1.3 Integration with Core Config

```rust
// crates/clnrm-core/src/config.rs - Enhanced OtelValidationSection
use super::validation::weaver::config::WeaverConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelValidationSection {
    /// Enable OTEL validation
    pub enabled: bool,

    /// Validate span creation and attributes
    pub validate_spans: bool,

    /// Validate trace completeness
    pub validate_traces: bool,

    /// Validate export functionality
    pub validate_exports: bool,

    /// Validate performance overhead
    pub validate_performance: bool,

    /// Maximum allowed overhead in milliseconds
    pub max_overhead_ms: f64,

    /// Expected spans configuration
    pub expected_spans: Vec<ExpectedSpan>,

    /// Weaver integration configuration (optional)
    #[serde(default)]
    pub weaver: WeaverConfig,
}
```

## Phase 2: Runtime Live Checking (Week 3-4)

### 2.1 Live Checker Plugin

```rust
// crates/clnrm-core/src/validation/weaver/live_check.rs
use weaver::live_check::{LiveChecker, ConformanceLevel};

pub struct WeaverLiveChecker {
    checker: Option<LiveChecker>,
    config: Option<LiveCheckConfig>,
}

impl WeaverPlugin for WeaverLiveChecker {
    fn initialize(&mut self, config: WeaverConfig) -> Result<()> {
        if !config.enabled || !config.enable_live_check {
            return Ok(());
        }

        // Initialize Weaver live checker
        let checker = LiveChecker::new(config.registry_config)
            .map_err(|e| CleanroomError::validation_error(format!("Failed to initialize live checker: {}", e)))?;

        self.checker = Some(checker);
        self.config = Some(LiveCheckConfig::from_weaver_config(config));
        Ok(())
    }

    fn validate_schema(&self, _registry_path: &str) -> Result<ValidationReport> {
        // Runtime validation using live checker
        if !self.is_enabled() {
            return Ok(ValidationReport::disabled());
        }

        // This would integrate with existing OTEL span collection
        unimplemented!("Live checking against collected spans")
    }

    fn generate_validation_code(&self, _registry_path: &str, _output_dir: &str) -> Result<()> {
        Ok(()) // Not applicable for live checking
    }

    fn is_enabled(&self) -> bool {
        self.config.as_ref().map(|c| c.enabled).unwrap_or(false)
    }
}
```

## Phase 3: Code Generation Plugin (Week 5-6)

### 3.1 Code Generation Plugin

```rust
// crates/clnrm-core/src/validation/weaver/codegen.rs
use weaver::registry::generate;

pub struct WeaverCodeGenerator {
    config: Option<CodeGenConfig>,
}

impl WeaverPlugin for WeaverCodeGenerator {
    fn initialize(&mut self, config: WeaverConfig) -> Result<()> {
        if !config.enabled || !config.enable_codegen {
            return Ok(());
        }

        // Validate that template directories exist
        let template_dir = std::path::Path::new(&config.registry_path).join("templates");
        if !template_dir.exists() {
            return Err(CleanroomError::validation_error(
                format!("Template directory not found: {}", template_dir.display())
            ));
        }

        self.config = Some(CodeGenConfig::from_weaver_config(config));
        Ok(())
    }

    fn validate_schema(&self, _registry_path: &str) -> Result<ValidationReport> {
        // Schema validation is handled by registry plugin
        Ok(ValidationReport::passed("Schema validation delegated to registry plugin"))
    }

    fn generate_validation_code(&self, registry_path: &str, output_dir: &str) -> Result<()> {
        if !self.is_enabled() {
            return Ok(()); // No-op when disabled
        }

        // Generate validation code using Weaver
        let registry = Registry::from_file(registry_path)
            .map_err(|e| CleanroomError::validation_error(format!("Failed to load registry: {}", e)))?;

        // Generate code using configured templates
        generate::generate_from_registry(&registry, output_dir, &self.config.templates)
            .map_err(|e| CleanroomError::validation_error(format!("Code generation failed: {}", e)))?;

        Ok(())
    }

    fn is_enabled(&self) -> bool {
        self.config.as_ref().map(|c| c.enabled).unwrap_or(false)
    }
}
```

## Configuration Enhancement

### Enhanced `.clnrm.toml` Schema

```toml
[meta]
name = "weaver_integration_example"
version = "1.0"
description = "Example with optional Weaver integration"

[otel]
exporter = "otlp"
endpoint = "http://collector:4318"
sample_ratio = 1.0
resources = { "service.name" = "cleanroom", "service.version" = "0.6.0" }

# Optional Weaver integration - disabled by default
[otel_validation.weaver]
enabled = false  # Opt-in only
registry_path = "telemetry/registry"
enable_codegen = false
enable_live_check = false

# Weaver registry configuration (loaded when enabled)
[otel_validation.weaver.registry_config]
registry:
  version: "1.0"
  schema_url: "https://opentelemetry.io/schemas/1.0.0"
  schemas:
    - id: "spans"
      file: "spans.yaml"
    - id: "metrics"
      file: "metrics.yaml"
```

### Template Structure

```
telemetry/registry/
├── weaver.yaml              # Weaver configuration
├── spans.yaml              # Span semantic conventions
├── metrics.yaml            # Metric semantic conventions
└── templates/
    └── validation/
        ├── otel_validation.rs.j2    # Rust validation code template
        ├── test_cases.rs.j2         # Test case generation template
        └── docs/
            └── validation_guide.md.j2 # Documentation template
```

## Usage Examples

### Basic Usage (No Weaver)

```bash
# Works exactly as before - no Weaver integration
cargo run -- run my-test.clnrm.toml --features otel-traces
```

### With Weaver Integration

```toml
# Enable Weaver in .clnrm.toml
[otel_validation.weaver]
enabled = true
enable_codegen = true
```

```bash
# Build with Weaver integration
cargo build --features otel-traces,weaver-integration

# Run with schema validation and code generation
cargo run -- run my-test.clnrm.toml --features otel-traces,weaver-integration
```

### CLI Commands

```bash
# Validate Weaver schema only
cargo run -- weaver validate --registry-path telemetry/registry

# Generate validation code only
cargo run -- weaver codegen --registry-path telemetry/registry --output src/generated/

# Full integration (validate + generate + check)
cargo run -- weaver integrate --registry-path telemetry/registry
```

## Migration Path

### For Existing Projects

1. **Zero Changes Required**: Existing `.clnrm.toml` files continue to work
2. **Opt-in Integration**: Add `[otel_validation.weaver]` section to enable features
3. **Gradual Enhancement**: Enable codegen, then live checking incrementally

### For New Projects

1. **Template Generation**: Use Weaver to generate initial schema and templates
2. **Schema-First Development**: Define telemetry requirements in Weaver registry
3. **Automated Validation**: Code generation ensures consistency

## Testing Strategy

### Unit Tests

```rust
#[cfg(feature = "weaver-integration")]
mod weaver_plugin_tests {
    use super::*;

    #[tokio::test]
    async fn test_weaver_plugin_disabled_by_default() {
        let plugin = WeaverRegistryPlugin::default();
        assert!(!plugin.is_enabled());
    }

    #[tokio::test]
    async fn test_weaver_plugin_initialization() -> Result<()> {
        let mut plugin = WeaverRegistryPlugin::new();
        let config = WeaverConfig {
            enabled: true,
            registry_path: "test-registry".to_string(),
            enable_codegen: false,
            enable_live_check: false,
            registry_config: test_registry_config(),
        };

        plugin.initialize(config)?;
        assert!(plugin.is_enabled());
        Ok(())
    }
}
```

### Integration Tests

```rust
#[cfg(all(feature = "otel-traces", feature = "weaver-integration"))]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_weaver_enhanced_otel_validation() -> Result<()> {
        // Test that Weaver integration enhances existing OTEL validation
        // without breaking existing functionality
        let config = TestConfig::with_weaver_integration();
        let result = run_test_with_config(config).await?;

        assert!(result.passed);
        assert!(result.weaver_report.is_some()); // Enhanced reporting
        Ok(())
    }
}
```

## Benefits Summary

### For Framework Maintainers

1. **Zero Breaking Changes**: Existing code continues to work unchanged
2. **Optional Enhancement**: Teams can adopt Weaver features incrementally
3. **Plugin Architecture**: Easy to maintain and extend
4. **Feature Gated**: No runtime overhead when disabled

### For End Users

1. **Gradual Adoption**: Can start with basic OTEL validation, add Weaver later
2. **Enhanced Validation**: Better semantic validation when enabled
3. **Code Generation**: Reduced boilerplate for validation code
4. **Documentation**: Auto-generated validation guides

### For Ecosystem

1. **Standards Compliance**: Better alignment with OTEL semantic conventions
2. **Interoperability**: Compatible with existing OTEL tooling
3. **Extensibility**: Plugin architecture allows other enhancements
4. **Best Practices**: Demonstrates schema-first telemetry validation

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Backward compatibility | 100% | Zero existing tests fail with Weaver enabled |
| Optional overhead | <1% | Performance impact when Weaver disabled |
| Adoption rate | >30% | Projects opting into Weaver integration |
| Schema compliance | >90% | Telemetry following semantic conventions |
| Code generation usage | >50% | Validation code using generated templates |

## Conclusion

This optional, plugin-based implementation provides a clean path for integrating OTEL Weaver's powerful schema validation and code generation capabilities into Cleanroom while maintaining full backward compatibility and following the framework's core principles of reliability and extensibility.

The implementation follows the 80/20 principle by focusing on the most valuable Weaver features (registry validation and code generation) while keeping the integration lightweight and optional.

