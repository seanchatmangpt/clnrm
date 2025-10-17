# Hyper-Advanced Plugin Architecture

**Version**: 2.0.0
**Status**: Design Document
**Foundation**: clnrm v1.0.1 Plugin System
**Date**: 2025-10-17

---

## Overview

The hyper-advanced plugin architecture extends clnrm's existing plugin system to support complex validation types, multi-dimensional orchestration, and intelligent automation. This document defines the plugin contract, lifecycle, and integration patterns.

---

## Core Principles

### 1. dyn-Compatible Design
All plugin traits MUST be object-safe (dyn-compatible):
- No async trait methods (use `tokio::task::block_in_place` internally)
- No generic methods with type parameters
- No associated types (use trait bounds instead)

### 2. Zero unwrap() Policy
All plugins MUST follow core team error handling standards:
- Return `Result<T, CleanroomError>` from all fallible operations
- Use proper error context with `.map_err()`
- Never use `.unwrap()` or `.expect()` in production code

### 3. OTEL-First Instrumentation
All plugins MUST emit spans for observability:
- Use `telemetry::spans` helpers for span creation
- Record key events with `telemetry::events`
- Track metrics with `telemetry::metrics` when applicable

---

## Plugin Trait Hierarchy

```rust
use crate::error::{CleanroomError, Result};
use std::collections::HashMap;

/// Base trait for all hyper-advanced plugins
pub trait HyperAdvancedPlugin: Send + Sync + std::fmt::Debug {
    /// Unique plugin identifier
    fn id(&self) -> &str;

    /// Human-readable plugin name
    fn name(&self) -> &str;

    /// Plugin type classification
    fn plugin_type(&self) -> PluginType;

    /// Initialize plugin with configuration
    fn initialize(&mut self, config: PluginConfig) -> Result<()>;

    /// Validate plugin configuration
    fn validate_config(&self, config: &PluginConfig) -> Result<()> {
        // Default implementation: no validation
        Ok(())
    }

    /// Plugin version (semver)
    fn version(&self) -> &str {
        "1.0.0"
    }

    /// Plugin dependencies (other plugin IDs)
    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }

    /// Shutdown and cleanup
    fn shutdown(&mut self) -> Result<()>;
}

/// Plugin lifecycle hooks
pub trait PluginLifecycle: HyperAdvancedPlugin {
    /// Called before test suite starts
    fn on_suite_start(&self, context: &SuiteContext) -> Result<()> {
        Ok(())
    }

    /// Called after test suite ends
    fn on_suite_end(&self, context: &SuiteContext, result: &SuiteResult) -> Result<()> {
        Ok(())
    }

    /// Called before each test starts
    fn on_test_start(&self, context: &TestContext) -> Result<()> {
        Ok(())
    }

    /// Called after each test ends
    fn on_test_end(&self, context: &TestContext, result: &TestResult) -> Result<()> {
        Ok(())
    }

    /// Called before each step executes
    fn on_step_start(&self, context: &StepContext) -> Result<()> {
        Ok(())
    }

    /// Called after each step completes
    fn on_step_end(&self, context: &StepContext, result: &StepResult) -> Result<()> {
        Ok(())
    }
}

/// Telemetry hooks for span/trace processing
pub trait TelemetryPlugin: HyperAdvancedPlugin {
    /// Called when a span is created
    fn on_span_created(&self, span: &SpanData) -> Result<()> {
        Ok(())
    }

    /// Called when a span ends
    fn on_span_ended(&self, span: &SpanData) -> Result<()> {
        Ok(())
    }

    /// Called when a trace is completed
    fn on_trace_completed(&self, trace: &TraceData) -> Result<()> {
        Ok(())
    }

    /// Called to validate trace graph
    fn validate_trace(&self, trace: &TraceData) -> Result<ValidationResult> {
        Ok(ValidationResult::default())
    }
}

/// Container lifecycle hooks
pub trait ContainerPlugin: HyperAdvancedPlugin {
    /// Called before container starts
    fn on_container_start(&self, context: &ContainerContext) -> Result<()> {
        Ok(())
    }

    /// Called after container stops
    fn on_container_stop(&self, context: &ContainerContext, exit_code: i32) -> Result<()> {
        Ok(())
    }

    /// Called when container fails
    fn on_container_failure(&self, context: &ContainerContext, error: &CleanroomError) -> Result<()> {
        Ok(())
    }
}

/// Plugin type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluginType {
    /// Multi-service orchestration
    Orchestration,
    /// Intelligent test generation
    Intelligence,
    /// Chaos engineering
    Chaos,
    /// Performance benchmarking
    Performance,
    /// Self-healing automation
    SelfHealing,
    /// Custom user-defined plugin
    Custom,
}

/// Plugin configuration
#[derive(Debug, Clone)]
pub struct PluginConfig {
    /// Plugin-specific configuration values
    pub values: HashMap<String, ConfigValue>,
}

/// Configuration value types
#[derive(Debug, Clone)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
    Object(HashMap<String, ConfigValue>),
}

/// Context provided to plugin hooks
#[derive(Debug, Clone)]
pub struct SuiteContext {
    pub suite_name: String,
    pub test_count: usize,
    pub parallel: bool,
}

#[derive(Debug, Clone)]
pub struct TestContext {
    pub test_name: String,
    pub suite_name: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct StepContext {
    pub step_name: String,
    pub test_name: String,
    pub step_index: usize,
}

#[derive(Debug, Clone)]
pub struct ContainerContext {
    pub container_id: String,
    pub image: String,
    pub service_name: String,
}

/// Test result passed to plugins
#[derive(Debug, Clone)]
pub struct TestResult {
    pub passed: bool,
    pub duration_ms: f64,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SuiteResult {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub duration_ms: f64,
}

#[derive(Debug, Clone)]
pub struct StepResult {
    pub passed: bool,
    pub output: String,
    pub exit_code: i32,
}

/// Validation result from telemetry plugin
#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Span data (simplified OTEL span)
#[derive(Debug, Clone)]
pub struct SpanData {
    pub span_id: String,
    pub trace_id: String,
    pub parent_span_id: Option<String>,
    pub name: String,
    pub kind: String,
    pub start_time_ns: u64,
    pub end_time_ns: Option<u64>,
    pub attributes: HashMap<String, String>,
    pub events: Vec<SpanEvent>,
    pub status: SpanStatus,
}

#[derive(Debug, Clone)]
pub struct SpanEvent {
    pub name: String,
    pub timestamp_ns: u64,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpanStatus {
    Unset,
    Ok,
    Error { message: String },
}

/// Trace data (collection of spans)
#[derive(Debug, Clone)]
pub struct TraceData {
    pub trace_id: String,
    pub spans: Vec<SpanData>,
    pub root_span_id: String,
}
```

---

## Plugin Registry

### Design

The plugin registry manages plugin lifecycle and dependency resolution:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

/// Plugin registry manages all loaded plugins
#[derive(Debug)]
pub struct PluginRegistry {
    /// Registered plugins by ID
    plugins: HashMap<String, Arc<RwLock<Box<dyn HyperAdvancedPlugin>>>>,
    /// Plugin initialization order (resolved from dependencies)
    init_order: Vec<String>,
    /// Plugin type index for fast lookup
    type_index: HashMap<PluginType, Vec<String>>,
}

impl PluginRegistry {
    /// Create new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            init_order: Vec::new(),
            type_index: HashMap::new(),
        }
    }

    /// Register a plugin
    pub fn register(&mut self, plugin: Box<dyn HyperAdvancedPlugin>) -> Result<()> {
        let id = plugin.id().to_string();
        let plugin_type = plugin.plugin_type();

        // Check for duplicate ID
        if self.plugins.contains_key(&id) {
            return Err(CleanroomError::validation_error(
                format!("Plugin with ID '{}' already registered", id)
            ));
        }

        // Add to type index
        self.type_index
            .entry(plugin_type)
            .or_insert_with(Vec::new)
            .push(id.clone());

        // Store plugin
        self.plugins.insert(id, Arc::new(RwLock::new(plugin)));

        Ok(())
    }

    /// Initialize all plugins in dependency order
    pub async fn initialize_all(&mut self, configs: HashMap<String, PluginConfig>) -> Result<()> {
        // Resolve initialization order from dependencies
        self.resolve_dependencies()?;

        // Initialize plugins in order
        for plugin_id in &self.init_order {
            let plugin = self.plugins.get(plugin_id)
                .ok_or_else(|| CleanroomError::internal_error(
                    format!("Plugin '{}' not found", plugin_id)
                ))?;

            let config = configs.get(plugin_id)
                .cloned()
                .unwrap_or_else(|| PluginConfig { values: HashMap::new() });

            // Validate config
            {
                let plugin_guard = plugin.read().await;
                plugin_guard.validate_config(&config).map_err(|e| {
                    CleanroomError::validation_error(
                        format!("Plugin '{}' config validation failed: {}", plugin_id, e)
                    )
                })?;
            }

            // Initialize
            {
                let mut plugin_guard = plugin.write().await;
                plugin_guard.initialize(config).map_err(|e| {
                    CleanroomError::internal_error(
                        format!("Plugin '{}' initialization failed: {}", plugin_id, e)
                    )
                })?;
            }

            tracing::info!("Initialized plugin: {}", plugin_id);
        }

        Ok(())
    }

    /// Resolve plugin dependencies and determine initialization order
    fn resolve_dependencies(&mut self) -> Result<()> {
        // Build dependency graph
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();

        for (plugin_id, plugin_arc) in &self.plugins {
            let plugin = plugin_arc.blocking_read();
            let deps = plugin.dependencies();

            graph.insert(plugin_id.clone(), deps.iter().map(|s| s.to_string()).collect());
            in_degree.insert(plugin_id.clone(), 0);
        }

        // Calculate in-degrees
        for deps in graph.values() {
            for dep in deps {
                *in_degree.get_mut(dep).unwrap() += 1;
            }
        }

        // Topological sort (Kahn's algorithm)
        let mut queue: Vec<String> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut order = Vec::new();

        while let Some(plugin_id) = queue.pop() {
            order.push(plugin_id.clone());

            if let Some(deps) = graph.get(&plugin_id) {
                for dep in deps {
                    if let Some(degree) = in_degree.get_mut(dep) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push(dep.clone());
                        }
                    }
                }
            }
        }

        // Check for cycles
        if order.len() != self.plugins.len() {
            return Err(CleanroomError::validation_error(
                "Plugin dependency cycle detected"
            ));
        }

        self.init_order = order;
        Ok(())
    }

    /// Get plugins by type
    pub fn get_by_type(&self, plugin_type: PluginType) -> Vec<Arc<RwLock<Box<dyn HyperAdvancedPlugin>>>> {
        self.type_index
            .get(&plugin_type)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.plugins.get(id))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Shutdown all plugins
    pub async fn shutdown_all(&mut self) -> Result<()> {
        // Shutdown in reverse order
        for plugin_id in self.init_order.iter().rev() {
            if let Some(plugin_arc) = self.plugins.get(plugin_id) {
                let mut plugin = plugin_arc.write().await;
                plugin.shutdown().map_err(|e| {
                    CleanroomError::internal_error(
                        format!("Plugin '{}' shutdown failed: {}", plugin_id, e)
                    )
                })?;
                tracing::info!("Shutdown plugin: {}", plugin_id);
            }
        }

        Ok(())
    }
}
```

---

## Example Plugin Implementations

### 1. Orchestration Plugin

```rust
/// Multi-dimensional orchestration plugin
#[derive(Debug)]
pub struct OrchestrationPlugin {
    id: String,
    name: String,
    dependency_graph: DependencyGraph,
    temporal_engine: TemporalConstraintEngine,
    initialized: bool,
}

impl OrchestrationPlugin {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            name: "Multi-Dimensional Orchestration".to_string(),
            dependency_graph: DependencyGraph::new(),
            temporal_engine: TemporalConstraintEngine::new(),
            initialized: false,
        }
    }
}

impl HyperAdvancedPlugin for OrchestrationPlugin {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn plugin_type(&self) -> PluginType {
        PluginType::Orchestration
    }

    fn initialize(&mut self, config: PluginConfig) -> Result<()> {
        #[cfg(feature = "otel-traces")]
        let _span = crate::telemetry::spans::plugin_registry_span(1);

        // Extract topology from config
        let topology = config.values.get("topology")
            .and_then(|v| match v {
                ConfigValue::String(s) => Some(s.as_str()),
                _ => None,
            })
            .unwrap_or("mesh");

        // Initialize dependency graph
        self.dependency_graph.set_topology(topology)?;

        // Initialize temporal engine
        self.temporal_engine.initialize()?;

        self.initialized = true;
        tracing::info!("OrchestrationPlugin initialized with topology: {}", topology);
        Ok(())
    }

    fn validate_config(&self, config: &PluginConfig) -> Result<()> {
        // Validate topology value
        if let Some(ConfigValue::String(topology)) = config.values.get("topology") {
            match topology.as_str() {
                "mesh" | "hierarchical" | "pipeline" => Ok(()),
                _ => Err(CleanroomError::validation_error(
                    format!("Invalid topology: '{}'. Must be mesh, hierarchical, or pipeline", topology)
                )),
            }
        } else {
            Ok(())  // Optional config
        }
    }

    fn shutdown(&mut self) -> Result<()> {
        self.dependency_graph.clear()?;
        self.temporal_engine.shutdown()?;
        self.initialized = false;
        Ok(())
    }
}

impl PluginLifecycle for OrchestrationPlugin {
    fn on_test_start(&self, context: &TestContext) -> Result<()> {
        // Build dependency graph for test services
        let services = context.metadata.get("services")
            .map(|s| s.split(',').collect::<Vec<_>>())
            .unwrap_or_default();

        self.dependency_graph.add_services(&services)?;
        Ok(())
    }

    fn on_test_end(&self, context: &TestContext, _result: &TestResult) -> Result<()> {
        // Clear test-specific state
        self.dependency_graph.clear_test_state()?;
        Ok(())
    }
}

impl TelemetryPlugin for OrchestrationPlugin {
    fn validate_trace(&self, trace: &TraceData) -> Result<ValidationResult> {
        let mut result = ValidationResult::default();

        // Validate service startup order from spans
        let startup_order = self.extract_startup_order(trace)?;
        let expected_order = self.dependency_graph.compute_startup_order()?;

        if startup_order != expected_order {
            result.valid = false;
            result.errors.push(format!(
                "Service startup order violation. Expected: {:?}, Got: {:?}",
                expected_order, startup_order
            ));
        } else {
            result.valid = true;
        }

        // Validate temporal constraints
        let temporal_violations = self.temporal_engine.validate(trace)?;
        if !temporal_violations.is_empty() {
            result.valid = false;
            result.errors.extend(temporal_violations);
        }

        Ok(result)
    }
}
```

### 2. Chaos Plugin

```rust
/// Chaos engineering plugin
#[derive(Debug)]
pub struct ChaosPlugin {
    id: String,
    name: String,
    fault_catalog: FaultCatalog,
    injection_scheduler: InjectionScheduler,
    active_faults: Vec<ActiveFault>,
}

impl ChaosPlugin {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            name: "Chaos Engineering".to_string(),
            fault_catalog: FaultCatalog::default(),
            injection_scheduler: InjectionScheduler::new(),
            active_faults: Vec::new(),
        }
    }
}

impl HyperAdvancedPlugin for ChaosPlugin {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn plugin_type(&self) -> PluginType {
        PluginType::Chaos
    }

    fn initialize(&mut self, config: PluginConfig) -> Result<()> {
        // Load fault catalog from config
        if let Some(ConfigValue::String(catalog_path)) = config.values.get("catalog") {
            self.fault_catalog = FaultCatalog::load_from_file(catalog_path)?;
        }

        tracing::info!("ChaosPlugin initialized with {} faults", self.fault_catalog.len());
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        // Clean up any active fault injections
        for fault in self.active_faults.drain(..) {
            fault.cleanup()?;
        }
        Ok(())
    }
}

impl PluginLifecycle for ChaosPlugin {
    fn on_test_start(&self, context: &TestContext) -> Result<()> {
        // Check if chaos is enabled for this test
        if let Some(chaos_enabled) = context.metadata.get("chaos_enabled") {
            if chaos_enabled == "true" {
                // Schedule fault injections
                self.injection_scheduler.schedule_for_test(context)?;
            }
        }
        Ok(())
    }

    fn on_test_end(&self, _context: &TestContext, _result: &TestResult) -> Result<()> {
        // Clean up any remaining active faults
        for fault in &self.active_faults {
            fault.cleanup()?;
        }
        Ok(())
    }
}

impl ContainerPlugin for ChaosPlugin {
    fn on_container_start(&self, context: &ContainerContext) -> Result<()> {
        // Potentially inject fault on container start
        if self.injection_scheduler.should_inject_on_start(context)? {
            let fault = self.injection_scheduler.get_next_fault()?;
            fault.inject(context)?;
        }
        Ok(())
    }
}
```

### 3. Performance Plugin

```rust
/// Performance benchmarking plugin
#[derive(Debug)]
pub struct PerformancePlugin {
    id: String,
    name: String,
    baseline_db: BaselineDatabase,
    metrics_collector: MetricsCollector,
    regression_detector: RegressionDetector,
}

impl HyperAdvancedPlugin for PerformancePlugin {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn plugin_type(&self) -> PluginType {
        PluginType::Performance
    }

    fn initialize(&mut self, config: PluginConfig) -> Result<()> {
        // Initialize baseline database
        if let Some(ConfigValue::String(db_path)) = config.values.get("baseline_db") {
            self.baseline_db = BaselineDatabase::open(db_path)?;
        }

        // Initialize metrics collector
        self.metrics_collector.start()?;

        tracing::info!("PerformancePlugin initialized");
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        self.metrics_collector.stop()?;
        self.baseline_db.close()?;
        Ok(())
    }
}

impl TelemetryPlugin for PerformancePlugin {
    fn on_span_ended(&self, span: &SpanData) -> Result<()> {
        // Collect performance metrics from span
        if let Some(duration_ns) = span.end_time_ns {
            let duration_ms = (duration_ns - span.start_time_ns) as f64 / 1_000_000.0;

            // Record metric
            self.metrics_collector.record_latency(&span.name, duration_ms)?;

            // Check for regression
            if let Some(baseline) = self.baseline_db.get_baseline(&span.name)? {
                let regression = self.regression_detector.check(duration_ms, &baseline)?;
                if regression.detected {
                    tracing::warn!(
                        "Performance regression detected for '{}': {}ms vs baseline {}ms",
                        span.name, duration_ms, baseline.p95_latency_ms
                    );
                }
            }
        }

        Ok(())
    }
}
```

---

## Plugin Configuration in TOML

```toml
# Configure hyper-advanced plugins
[plugins]
enabled = true

# Orchestration plugin
[[plugins.register]]
id = "orchestration_mesh"
type = "orchestration"
library = "libclnrm_orchestration.so"  # Dynamic library path
config = {
    topology = "mesh",
    max_services = 10,
    parallel_startup = true
}

# Chaos plugin
[[plugins.register]]
id = "chaos_injector"
type = "chaos"
library = "libclnrm_chaos.so"
config = {
    catalog = "faults/catalog.toml",
    enabled_faults = ["network_latency", "container_kill", "cpu_pressure"]
}

# Performance plugin
[[plugins.register]]
id = "perf_tracker"
type = "performance"
library = "libclnrm_performance.so"
config = {
    baseline_db = "baselines/performance.db",
    regression_threshold_percent = 15,
    track_percentiles = [50, 95, 99]
}

# Intelligence plugin
[[plugins.register]]
id = "test_generator"
type = "intelligence"
library = "libclnrm_intelligence.so"
config = {
    trace_source = "traces/production.json",
    pattern_min_frequency = 0.05,
    generate_on_suite_end = true
}

# Self-healing plugin
[[plugins.register]]
id = "auto_remediation"
type = "self_healing"
library = "libclnrm_self_healing.so"
config = {
    failure_db = "learning/failures.db",
    max_retries = 5,
    auto_tune_resources = true
}
```

---

## Plugin Development Guide

### Creating a New Plugin

1. **Define the plugin struct**:
```rust
#[derive(Debug)]
pub struct MyPlugin {
    id: String,
    name: String,
    // Plugin-specific fields
}
```

2. **Implement HyperAdvancedPlugin**:
```rust
impl HyperAdvancedPlugin for MyPlugin {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { &self.name }
    fn plugin_type(&self) -> PluginType { PluginType::Custom }

    fn initialize(&mut self, config: PluginConfig) -> Result<()> {
        // Initialization logic
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        // Cleanup logic
        Ok(())
    }
}
```

3. **Implement optional traits** (PluginLifecycle, TelemetryPlugin, ContainerPlugin)

4. **Add OTEL instrumentation**:
```rust
fn on_test_start(&self, context: &TestContext) -> Result<()> {
    #[cfg(feature = "otel-traces")]
    let _span = crate::telemetry::spans::plugin_registry_span(1);

    // Plugin logic
    Ok(())
}
```

5. **Write tests**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin_initialization() -> Result<()> {
        let mut plugin = MyPlugin::new("test");
        let config = PluginConfig { values: HashMap::new() };
        plugin.initialize(config)?;
        assert!(plugin.is_initialized());
        Ok(())
    }
}
```

---

## Best Practices

1. **Always use proper error handling** - No unwrap()!
2. **Emit spans for observability** - OTEL-first approach
3. **Keep traits dyn-compatible** - No async trait methods
4. **Validate configuration** - Fail fast with clear messages
5. **Clean up resources** - Implement shutdown properly
6. **Write comprehensive tests** - Test initialization, lifecycle, and error cases
7. **Document configuration** - Provide clear examples
8. **Version your plugins** - Use semver for compatibility

---

## Conclusion

The hyper-advanced plugin architecture provides a robust foundation for extending clnrm with sophisticated validation types. By following core team standards (zero unwrap(), dyn-compatible, OTEL-first), plugins integrate seamlessly with the existing framework while enabling powerful new capabilities.

**Status**: Ready for Implementation
**Foundation**: clnrm v1.0.1 Plugin System
**Target**: v2.0.0 Plugin Framework
