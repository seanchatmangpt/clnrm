//! Configuration shape validator for dry-run mode
//!
//! Validates TOML configuration structure without spinning up containers.
//! Performs fast, static validation of configuration shape and relationships.

use crate::config::{
    ExpectationsConfig, OrderExpectationConfig, OtelConfig, ScenarioConfig, ServiceConfig,
    SpanExpectationConfig, TestConfig, VolumeConfig, WindowExpectationConfig,
};
use crate::error::{CleanroomError, Result};
use glob::Pattern as GlobBuilder;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Result of shape validation
#[derive(Debug, Clone)]
pub struct ShapeValidationResult {
    /// Whether validation passed
    pub passed: bool,
    /// List of validation errors
    pub errors: Vec<ShapeValidationError>,
    /// File path that was validated
    pub file_path: String,
}

/// Shape validation error with file context
#[derive(Debug, Clone)]
pub struct ShapeValidationError {
    /// Error message
    pub message: String,
    /// Line number (if available)
    pub line: Option<usize>,
    /// Error category
    pub category: ErrorCategory,
}

/// Error categories for validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Missing required block
    MissingRequired,
    /// Invalid structure
    InvalidStructure,
    /// Orphan reference
    OrphanReference,
    /// Invalid duration
    InvalidDuration,
    /// Circular ordering
    CircularOrdering,
    /// Invalid glob pattern
    InvalidGlob,
    /// OTEL configuration error
    OtelError,
}

impl ShapeValidationError {
    /// Create error with category
    pub fn new(category: ErrorCategory, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            line: None,
            category,
        }
    }

    /// Set line number
    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }
}

/// Shape validator for configuration files
pub struct ShapeValidator {
    /// Errors collected during validation
    errors: Vec<ShapeValidationError>,
}

impl ShapeValidator {
    /// Create new shape validator
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Validate a configuration file
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be read or parsed
    pub fn validate_file(&mut self, path: &Path) -> Result<ShapeValidationResult> {
        // Read and parse file
        let content = std::fs::read_to_string(path).map_err(|e| {
            CleanroomError::config_error(format!("Failed to read config file: {}", e))
        })?;

        // Check if template rendering is needed
        let toml_content = if crate::template::is_template(&content) {
            // Render as Tera template
            let mut renderer = crate::template::TemplateRenderer::new()?;
            let path_str = path
                .to_str()
                .ok_or_else(|| CleanroomError::validation_error("Invalid file path encoding"))?;
            renderer.render_str(&content, path_str)?
        } else {
            content
        };

        // Parse TOML
        let config = toml::from_str::<TestConfig>(&toml_content)
            .map_err(|e| CleanroomError::config_error(format!("TOML parse error: {}", e)))?;

        // Validate shape
        self.validate_config(&config)?;

        // Build result
        let result = ShapeValidationResult {
            passed: self.errors.is_empty(),
            errors: self.errors.clone(),
            file_path: path.to_string_lossy().into_owned(),
        };

        Ok(result)
    }

    /// Validate a parsed configuration
    ///
    /// # Errors
    ///
    /// Returns error if validation logic fails unexpectedly
    pub fn validate_config(&mut self, config: &TestConfig) -> Result<()> {
        // Clear previous errors
        self.errors.clear();

        // 1. Validate required blocks
        self.validate_required_blocks(config);

        // 2. Validate OTEL configuration
        self.validate_otel_config(config);

        // 3. Validate scenarios
        self.validate_scenarios(config);

        // 4. Validate service references
        self.validate_service_references(config);

        // 5. Validate duration constraints
        self.validate_duration_constraints(config);

        // 6. Validate temporal ordering
        self.validate_temporal_ordering(config);

        // 7. Validate glob patterns
        self.validate_glob_patterns(config);

        // 8. Validate container images (ENHANCED)
        self.validate_container_images(config);

        // 9. Validate port bindings (ENHANCED)
        self.validate_port_bindings(config);

        // 10. Validate volume mounts (ENHANCED)
        self.validate_volume_mounts(config);

        // 11. Validate environment variables (ENHANCED)
        self.validate_environment_variables(config);

        // 12. Validate service dependencies (ENHANCED)
        self.validate_service_dependencies(config);

        Ok(())
    }

    /// Validate required configuration blocks
    fn validate_required_blocks(&mut self, config: &TestConfig) {
        // Check [meta] or [test.metadata] exists
        if config.meta.is_none() && config.test.is_none() {
            self.errors.push(ShapeValidationError::new(
                ErrorCategory::MissingRequired,
                "Configuration must have either [meta] or [test.metadata] section",
            ));
        }

        // Check meta has name and version (for v0.6.0 format)
        if let Some(ref meta) = config.meta {
            if meta.name.trim().is_empty() {
                self.errors.push(ShapeValidationError::new(
                    ErrorCategory::InvalidStructure,
                    "[meta] section missing required 'name' field",
                ));
            }
            if meta.version.trim().is_empty() {
                self.errors.push(ShapeValidationError::new(
                    ErrorCategory::InvalidStructure,
                    "[meta] section missing required 'version' field",
                ));
            }
        }

        // Check at least one scenario exists
        if config.scenario.is_empty() && config.steps.is_empty() {
            self.errors.push(ShapeValidationError::new(
                ErrorCategory::MissingRequired,
                "Configuration must have at least one [[scenario]] or [[steps]]",
            ));
        }
    }

    /// Validate OTEL configuration
    fn validate_otel_config(&mut self, config: &TestConfig) {
        if let Some(ref otel) = config.otel {
            self.validate_otel_exporter(otel);
        }
    }

    /// Validate OTEL exporter configuration
    fn validate_otel_exporter(&mut self, otel: &OtelConfig) {
        let valid_exporters = [
            "jaeger",
            "otlp",
            "otlp-http",
            "otlp-grpc",
            "datadog",
            "newrelic",
        ];

        if !valid_exporters.contains(&otel.exporter.as_str()) {
            self.errors.push(ShapeValidationError::new(
                ErrorCategory::OtelError,
                format!(
                    "Invalid OTEL exporter '{}'. Valid options: {}",
                    otel.exporter,
                    valid_exporters.join(", ")
                ),
            ));
        }

        // Validate sample ratio
        if let Some(ratio) = otel.sample_ratio {
            if !(0.0..=1.0).contains(&ratio) {
                self.errors.push(ShapeValidationError::new(
                    ErrorCategory::OtelError,
                    format!(
                        "OTEL sample_ratio must be between 0.0 and 1.0, got {}",
                        ratio
                    ),
                ));
            }
        }
    }

    /// Validate scenario configurations
    fn validate_scenarios(&mut self, config: &TestConfig) {
        for (idx, scenario) in config.scenario.iter().enumerate() {
            if scenario.name.trim().is_empty() {
                self.errors.push(ShapeValidationError::new(
                    ErrorCategory::InvalidStructure,
                    format!("Scenario {} missing required 'name' field", idx),
                ));
            }

            if scenario.steps.is_empty() {
                self.errors.push(ShapeValidationError::new(
                    ErrorCategory::InvalidStructure,
                    format!("Scenario '{}' must have at least one step", scenario.name),
                ));
            }
        }
    }

    /// Validate service references
    fn validate_service_references(&mut self, config: &TestConfig) {
        // Build set of defined services
        let mut defined_services = HashSet::new();

        if let Some(ref services) = config.services {
            for service_name in services.keys() {
                defined_services.insert(service_name.clone());
            }
        }

        if let Some(ref service) = config.service {
            for service_name in service.keys() {
                defined_services.insert(service_name.clone());
            }
        }

        // Check scenario step references
        for scenario in &config.scenario {
            self.validate_scenario_service_refs(scenario, &defined_services);
        }

        // Check top-level step references
        for step in &config.steps {
            if let Some(ref service_name) = step.service {
                if !defined_services.contains(service_name) {
                    self.errors.push(ShapeValidationError::new(
                        ErrorCategory::OrphanReference,
                        format!(
                            "Step '{}' references undefined service '{}'",
                            step.name, service_name
                        ),
                    ));
                }
            }
        }
    }

    /// Validate service references in a scenario
    fn validate_scenario_service_refs(
        &mut self,
        scenario: &ScenarioConfig,
        defined_services: &HashSet<String>,
    ) {
        for step in &scenario.steps {
            if let Some(ref service_name) = step.service {
                if !defined_services.contains(service_name) {
                    self.errors.push(ShapeValidationError::new(
                        ErrorCategory::OrphanReference,
                        format!(
                            "Scenario '{}' step '{}' references undefined service '{}'",
                            scenario.name, step.name, service_name
                        ),
                    ));
                }
            }
        }
    }

    /// Validate duration constraints
    fn validate_duration_constraints(&mut self, config: &TestConfig) {
        // Check span expectations
        if let Some(ref expect) = config.expect {
            for span in &expect.span {
                self.validate_span_duration(span);
            }

            // Check window expectations
            for window in &expect.window {
                self.validate_window_duration(window);
            }
        }

        // Check legacy OTEL validation
        if let Some(ref otel_val) = config.otel_validation {
            if let Some(ref spans) = otel_val.expected_spans {
                for span in spans {
                    if let (Some(min), Some(max)) = (span.min_duration_ms, span.max_duration_ms) {
                        if min > max {
                            self.errors.push(ShapeValidationError::new(
                                ErrorCategory::InvalidDuration,
                                format!(
                                    "Span '{}' has invalid duration: min ({}) > max ({})",
                                    span.name, min, max
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }

    /// Validate span duration constraints
    fn validate_span_duration(&mut self, _span: &SpanExpectationConfig) {
        // Duration validation is implicit in the structure
        // Real duration validation would happen at runtime
    }

    /// Validate window duration constraints
    fn validate_window_duration(&mut self, _window: &WindowExpectationConfig) {
        // Window duration validation is implicit
        // Real validation happens at runtime
    }

    /// Validate temporal ordering for cycles
    fn validate_temporal_ordering(&mut self, config: &TestConfig) {
        if let Some(ref expect) = config.expect {
            if let Some(ref order) = expect.order {
                self.check_ordering_cycles(order);
            }
        }

        // Check legacy OTEL validation
        if let Some(ref otel_val) = config.otel_validation {
            if let Some(ref order) = otel_val.expect_order {
                self.check_ordering_cycles(order);
            }
        }
    }

    /// Check for cycles in ordering constraints
    fn check_ordering_cycles(&mut self, order: &OrderExpectationConfig) {
        // Build adjacency graph
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();

        // Add must_precede edges (A -> B means A must come before B)
        if let Some(ref must_precede) = order.must_precede {
            for edge in must_precede {
                if edge.len() == 2 {
                    let first = &edge[0];
                    let second = &edge[1];
                    graph.entry(first.clone()).or_default().push(second.clone());
                }
            }
        }

        // Add must_follow edges (A follows B means B -> A)
        if let Some(ref must_follow) = order.must_follow {
            for edge in must_follow {
                if edge.len() == 2 {
                    let first = &edge[0];
                    let second = &edge[1];
                    graph.entry(second.clone()).or_default().push(first.clone());
                }
            }
        }

        // Detect cycles using DFS
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for node in graph.keys() {
            if !visited.contains(node)
                && Self::has_cycle_dfs(node, &graph, &mut visited, &mut rec_stack)
            {
                self.errors.push(ShapeValidationError::new(
                    ErrorCategory::CircularOrdering,
                    format!(
                        "Circular temporal ordering detected involving span '{}'",
                        node
                    ),
                ));
                break;
            }
        }
    }

    /// DFS cycle detection helper
    fn has_cycle_dfs(
        node: &str,
        graph: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if Self::has_cycle_dfs(neighbor, graph, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(neighbor) {
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        false
    }

    /// Validate glob patterns
    fn validate_glob_patterns(&mut self, config: &TestConfig) {
        // Check span expectations
        if let Some(ref expect) = config.expect {
            self.validate_expectation_globs(expect);
        }

        // Check legacy OTEL validation
        if let Some(ref otel_val) = config.otel_validation {
            if let Some(ref spans) = otel_val.expected_spans {
                for span in spans {
                    if let Err(e) = self.validate_glob_pattern(&span.name) {
                        self.errors.push(ShapeValidationError::new(
                            ErrorCategory::InvalidGlob,
                            format!("Invalid glob pattern in span '{}': {}", span.name, e),
                        ));
                    }
                }
            }
        }
    }

    /// Validate expectations globs
    fn validate_expectation_globs(&mut self, expect: &ExpectationsConfig) {
        for span in &expect.span {
            if let Err(e) = self.validate_glob_pattern(&span.name) {
                self.errors.push(ShapeValidationError::new(
                    ErrorCategory::InvalidGlob,
                    format!(
                        "Invalid glob pattern in span expectation '{}': {}",
                        span.name, e
                    ),
                ));
            }
        }
    }

    /// Validate a single glob pattern
    fn validate_glob_pattern(&self, pattern: &str) -> Result<()> {
        GlobBuilder::new(pattern).map_err(|e| {
            CleanroomError::validation_error(format!("Invalid pattern '{}': {}", pattern, e))
        })?;
        Ok(())
    }

    /// Get validation errors
    pub fn errors(&self) -> &[ShapeValidationError] {
        &self.errors
    }

    /// Check if validation passed
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    // ========================================================================
    // ENHANCED VALIDATION METHODS (v0.7.0)
    // ========================================================================

    /// Validate container image configurations
    fn validate_container_images(&mut self, config: &TestConfig) {
        let services = self.collect_all_services(config);

        for (service_name, service) in services {
            // Skip network services that don't require images
            if service.r#type == "network_service" || service.r#type == "ollama" {
                continue;
            }

            // Check image exists and is not empty
            if let Some(ref image) = service.image {
                if image.trim().is_empty() {
                    self.errors.push(ShapeValidationError::new(
                        ErrorCategory::InvalidStructure,
                        format!(
                            "Service '{}' has empty image. Suggestion: Use a valid image like 'alpine:latest' or 'ubuntu:20.04'",
                            service_name
                        ),
                    ));
                    continue;
                }

                // Validate image format
                self.validate_image_format(&service_name, image);
            }
        }
    }

    /// Validate Docker image format
    fn validate_image_format(&mut self, service_name: &str, image: &str) {
        // Docker image format: [registry/][namespace/]repository[:tag|@digest]
        // Examples:
        // - alpine:latest
        // - ubuntu:20.04
        // - docker.io/library/postgres:14
        // - ghcr.io/owner/repo:v1.0.0
        // - localhost:5000/myimage:tag

        // Check for invalid characters
        if image.contains(' ') {
            self.errors.push(ShapeValidationError::new(
                ErrorCategory::InvalidStructure,
                format!(
                    "Service '{}' has invalid image format '{}'. Images cannot contain spaces. Example: 'alpine:latest'",
                    service_name, image
                ),
            ));
            return;
        }

        // Check for exclamation marks or other special characters
        if image.contains('!') || image.contains('?') || image.contains('*') {
            self.errors.push(ShapeValidationError::new(
                ErrorCategory::InvalidStructure,
                format!(
                    "Service '{}' has invalid image format '{}'. Invalid characters detected. Example: 'alpine:latest'",
                    service_name, image
                ),
            ));
            return;
        }

        // Validate basic structure
        let parts: Vec<&str> = image.split('/').collect();
        if parts.len() > 3 {
            self.errors.push(ShapeValidationError::new(
                ErrorCategory::InvalidStructure,
                format!(
                    "Service '{}' has invalid image format '{}'. Too many path segments. Example: 'registry/namespace/repo:tag'",
                    service_name, image
                ),
            ));
        }
    }

    /// Validate port bindings across all services
    fn validate_port_bindings(&mut self, config: &TestConfig) {
        let mut port_usage: HashMap<u16, Vec<String>> = HashMap::new();
        let services = self.collect_all_services(config);

        for (service_name, service) in services {
            if let Some(ref ports) = service.ports {
                for &port in ports {
                    // Check for reserved/system ports (1-1023)
                    if port < 1024 {
                        self.errors.push(ShapeValidationError::new(
                            ErrorCategory::InvalidStructure,
                            format!(
                                "Service '{}' uses reserved port {}. Suggestion: Use ports >= 1024 (e.g., 8080, 9000, 3000)",
                                service_name, port
                            ),
                        ));
                    }

                    // Track port usage for conflict detection
                    port_usage
                        .entry(port)
                        .or_default()
                        .push(service_name.clone());
                }
            }
        }

        // Check for port conflicts
        for (port, services) in port_usage {
            if services.len() > 1 {
                self.errors.push(ShapeValidationError::new(
                    ErrorCategory::InvalidStructure,
                    format!(
                        "Port conflict detected: port {} is used by multiple services: {}. Each service must use unique ports.",
                        port,
                        services.join(", ")
                    ),
                ));
            }
        }
    }

    /// Validate volume mount configurations
    fn validate_volume_mounts(&mut self, config: &TestConfig) {
        let services = self.collect_all_services(config);

        for (service_name, service) in services {
            if let Some(ref volumes) = service.volumes {
                for (idx, volume) in volumes.iter().enumerate() {
                    self.validate_single_volume(&service_name, idx, volume);
                }
            }
        }
    }

    /// Validate a single volume configuration
    fn validate_single_volume(&mut self, service_name: &str, idx: usize, volume: &VolumeConfig) {
        // Check host path is absolute
        if !volume.host_path.starts_with('/') {
            self.errors.push(ShapeValidationError::new(
                ErrorCategory::InvalidStructure,
                format!(
                    "Service '{}' volume {}: host path '{}' must be absolute. Suggestion: Use '/tmp/data' or '/home/user/project'",
                    service_name, idx, volume.host_path
                ),
            ));
        }

        // Check container path is absolute
        if !volume.container_path.starts_with('/') {
            self.errors.push(ShapeValidationError::new(
                ErrorCategory::InvalidStructure,
                format!(
                    "Service '{}' volume {}: container path '{}' must be absolute. Suggestion: Use '/app/data' or '/var/lib/app'",
                    service_name, idx, volume.container_path
                ),
            ));
        }

        // Warn about dangerous system paths
        let dangerous_paths = [
            "/etc",
            "/var",
            "/proc",
            "/sys",
            "/dev",
            "/boot",
            "/root",
            "/bin",
            "/sbin",
            "/lib",
            "/lib64",
            "/usr/bin",
            "/usr/sbin",
        ];

        for dangerous in &dangerous_paths {
            if volume.container_path.starts_with(dangerous)
                && (volume.container_path == *dangerous
                    || volume
                        .container_path
                        .starts_with(&format!("{}/", dangerous)))
            {
                self.errors.push(ShapeValidationError::new(
                    ErrorCategory::InvalidStructure,
                    format!(
                        "Service '{}' volume {}: mounting to system path '{}' is dangerous. Suggestion: Use application paths like '/app/data'",
                        service_name, idx, volume.container_path
                    ),
                ));
            }
        }
    }

    /// Validate environment variable configurations
    fn validate_environment_variables(&mut self, config: &TestConfig) {
        let services = self.collect_all_services(config);

        for (service_name, service) in services {
            if let Some(ref env) = service.env {
                for (key, value) in env {
                    self.validate_env_var(&service_name, key, value);
                }
            }
        }

        // Also validate step-level env vars
        for step in &config.steps {
            if let Some(ref env) = step.env {
                for (key, value) in env {
                    self.validate_env_var(&format!("step '{}'", step.name), key, value);
                }
            }
        }

        for scenario in &config.scenario {
            for step in &scenario.steps {
                if let Some(ref env) = step.env {
                    for (key, value) in env {
                        self.validate_env_var(
                            &format!("scenario '{}' step '{}'", scenario.name, step.name),
                            key,
                            value,
                        );
                    }
                }
            }
        }
    }

    /// Validate a single environment variable
    fn validate_env_var(&mut self, context: &str, key: &str, value: &str) {
        // Check key is not empty
        if key.is_empty() {
            self.errors.push(ShapeValidationError::new(
                ErrorCategory::InvalidStructure,
                format!(
                    "{}: environment variable name cannot be empty. Use uppercase names like 'APP_ENV' or 'DATABASE_URL'",
                    context
                ),
            ));
            return;
        }

        // Validate key format (must start with letter or underscore, contain only alphanumeric and underscore)
        // Regex pattern is static and known to be valid, but handle error gracefully
        match Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*$") {
            Ok(env_var_regex) => {
                if !env_var_regex.is_match(key) {
                    self.errors.push(ShapeValidationError::new(
                        ErrorCategory::InvalidStructure,
                        format!(
                            "{}: invalid environment variable name '{}'. Names must start with a letter or underscore and contain only alphanumeric characters and underscores. Example: 'DATABASE_URL'",
                            context, key
                        ),
                    ));
                }
            }
            Err(e) => {
                // This should never happen with a static pattern, but handle gracefully
                self.errors.push(ShapeValidationError::new(
                    ErrorCategory::InvalidStructure,
                    format!("{}: internal error compiling regex: {}", context, e),
                ));
            }
        }

        // Warn about potential hardcoded secrets
        let sensitive_keys = [
            "API_KEY",
            "PASSWORD",
            "SECRET",
            "TOKEN",
            "PRIVATE_KEY",
            "CREDENTIALS",
            "AUTH_TOKEN",
            "ACCESS_KEY",
            "SECRET_KEY",
        ];

        for sensitive in &sensitive_keys {
            if key.to_uppercase().contains(sensitive)
                && !value.is_empty()
                && !value.starts_with('$')
            {
                self.errors.push(ShapeValidationError::new(
                    ErrorCategory::InvalidStructure,
                    format!(
                        "{}: potential hardcoded sensitive value in '{}'. Suggestion: Use environment variable references like '${{ENV_VAR}}' or template variables",
                        context, key
                    ),
                ));
                break;
            }
        }
    }

    /// Validate service dependencies
    fn validate_service_dependencies(&mut self, config: &TestConfig) {
        let services = self.collect_all_services(config);

        // Build dependency graph from health check commands
        let mut dep_graph: HashMap<String, Vec<String>> = HashMap::new();

        for (service_name, service) in &services {
            if let Some(ref health_check) = service.health_check {
                // Extract service dependencies from health check commands
                let deps = self.extract_service_deps_from_command(&health_check.cmd, &services);
                if !deps.is_empty() {
                    dep_graph.insert(service_name.clone(), deps);
                }
            }
        }

        // Detect circular dependencies
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for service_name in dep_graph.keys() {
            if !visited.contains(service_name)
                && Self::has_circular_dep(service_name, &dep_graph, &mut visited, &mut rec_stack)
            {
                self.errors.push(ShapeValidationError::new(
                    ErrorCategory::CircularOrdering,
                    format!(
                        "Circular service dependency detected involving '{}'. Services cannot depend on each other in a cycle.",
                        service_name
                    ),
                ));
                break;
            }
        }
    }

    /// Extract service dependencies from command
    fn extract_service_deps_from_command(
        &self,
        command: &[String],
        services: &HashMap<String, &ServiceConfig>,
    ) -> Vec<String> {
        let mut deps = Vec::new();
        let cmd_str = command.join(" ");

        for service_name in services.keys() {
            if cmd_str.contains(service_name) {
                deps.push(service_name.clone());
            }
        }

        deps
    }

    /// Check for circular dependencies
    fn has_circular_dep(
        service: &str,
        graph: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(service.to_string());
        rec_stack.insert(service.to_string());

        if let Some(deps) = graph.get(service) {
            for dep in deps {
                if !visited.contains(dep) {
                    if Self::has_circular_dep(dep, graph, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(dep) {
                    return true;
                }
            }
        }

        rec_stack.remove(service);
        false
    }

    /// Collect all services from config (both [services] and [service] tables)
    fn collect_all_services<'a>(
        &self,
        config: &'a TestConfig,
    ) -> HashMap<String, &'a ServiceConfig> {
        let mut all_services = HashMap::new();

        if let Some(ref services) = config.services {
            for (name, service) in services {
                all_services.insert(name.clone(), service);
            }
        }

        if let Some(ref service) = config.service {
            for (name, svc) in service {
                all_services.insert(name.clone(), svc);
            }
        }

        all_services
    }
}

impl Default for ShapeValidator {
    fn default() -> Self {
        Self::new()
    }
}
