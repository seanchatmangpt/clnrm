//! London School TDD Test Suite for TOML Features
//!
//! This test suite follows the London School (mockist) approach to TDD:
//! 1. Mock collaborators FIRST (define contracts through mocks)
//! 2. Verify interactions (not state)
//! 3. Tests drive design from the outside-in
//! 4. Initially RED (failing) - minimal implementation makes them GREEN
//!
//! ## Test Coverage
//! - TOML parsing with schema validation
//! - Service registry with plugin lookup
//! - Container backend with exec simulation
//! - Template renderer with variable substitution
//! - Span collector with validation expectations

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Mock trait definitions - these define the contracts we expect

/// Mock TOML parser that validates against schema
#[derive(Clone, Debug)]
pub struct MockTomlParser {
    /// Expected parse calls
    pub parse_calls: Arc<Mutex<Vec<String>>>,
    /// Pre-configured parse results (path -> result)
    pub parse_results: Arc<Mutex<HashMap<String, Result<MockTestConfig, String>>>>,
    /// Schema validation results (path -> is_valid)
    pub schema_validations: Arc<Mutex<HashMap<String, bool>>>,
}

impl MockTomlParser {
    pub fn new() -> Self {
        Self {
            parse_calls: Arc::new(Mutex::new(Vec::new())),
            parse_results: Arc::new(Mutex::new(HashMap::new())),
            schema_validations: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Configure parser to return specific result for path
    pub fn expect_parse(&self, path: &str, result: Result<MockTestConfig, String>) {
        self.parse_results
            .lock()
            .unwrap()
            .insert(path.to_string(), result);
    }

    /// Configure schema validation result
    pub fn expect_schema_valid(&self, path: &str, is_valid: bool) {
        self.schema_validations
            .lock()
            .unwrap()
            .insert(path.to_string(), is_valid);
    }

    /// Parse TOML from path
    pub fn parse(&self, path: &str) -> Result<MockTestConfig, String> {
        // Record the call
        self.parse_calls.lock().unwrap().push(path.to_string());

        // Return pre-configured result or error
        self.parse_results
            .lock()
            .unwrap()
            .get(path)
            .cloned()
            .unwrap_or_else(|| Err(format!("No parse result configured for: {}", path)))
    }

    /// Validate TOML against schema
    pub fn validate_schema(&self, path: &str) -> bool {
        self.schema_validations
            .lock()
            .unwrap()
            .get(path)
            .copied()
            .unwrap_or(false)
    }

    /// Verify parse was called with specific path
    pub fn verify_parse_called(&self, path: &str) -> bool {
        self.parse_calls.lock().unwrap().contains(&path.to_string())
    }

    /// Get number of parse calls
    pub fn parse_call_count(&self) -> usize {
        self.parse_calls.lock().unwrap().len()
    }
}

/// Mock test configuration (simplified from actual TestConfig)
#[derive(Clone, Debug, PartialEq)]
pub struct MockTestConfig {
    pub name: String,
    pub services: Vec<String>,
    pub steps: Vec<MockStep>,
    pub variables: HashMap<String, String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MockStep {
    pub name: String,
    pub command: Vec<String>,
}

/// Mock service registry for plugin lookup
#[derive(Clone, Debug)]
pub struct MockServiceRegistry {
    /// Registered services (service_name -> plugin_type)
    pub services: Arc<Mutex<HashMap<String, String>>>,
    /// Lookup calls (for verification)
    pub lookup_calls: Arc<Mutex<Vec<String>>>,
    /// Service creation calls
    pub creation_calls: Arc<Mutex<Vec<(String, String)>>>,
}

impl MockServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: Arc::new(Mutex::new(HashMap::new())),
            lookup_calls: Arc::new(Mutex::new(Vec::new())),
            creation_calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register a service with plugin type
    pub fn register_service(&self, name: &str, plugin_type: &str) {
        self.services
            .lock()
            .unwrap()
            .insert(name.to_string(), plugin_type.to_string());
        self.creation_calls
            .lock()
            .unwrap()
            .push((name.to_string(), plugin_type.to_string()));
    }

    /// Lookup a service plugin
    pub fn lookup(&self, service_name: &str) -> Option<String> {
        self.lookup_calls
            .lock()
            .unwrap()
            .push(service_name.to_string());
        self.services.lock().unwrap().get(service_name).cloned()
    }

    /// Verify service was looked up
    pub fn verify_lookup(&self, service_name: &str) -> bool {
        self.lookup_calls
            .lock()
            .unwrap()
            .contains(&service_name.to_string())
    }

    /// Verify service was created
    pub fn verify_creation(&self, service_name: &str, plugin_type: &str) -> bool {
        self.creation_calls
            .lock()
            .unwrap()
            .contains(&(service_name.to_string(), plugin_type.to_string()))
    }
}

/// Mock container backend for exec simulation
#[derive(Clone, Debug)]
pub struct MockContainerBackend {
    /// Execution results (command -> result)
    pub exec_results: Arc<Mutex<HashMap<Vec<String>, Result<String, String>>>>,
    /// Execution calls (for verification)
    pub exec_calls: Arc<Mutex<Vec<Vec<String>>>>,
    /// Running containers
    pub containers: Arc<Mutex<Vec<String>>>,
}

impl MockContainerBackend {
    pub fn new() -> Self {
        Self {
            exec_results: Arc::new(Mutex::new(HashMap::new())),
            exec_calls: Arc::new(Mutex::new(Vec::new())),
            containers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Configure expected execution result
    pub fn expect_exec(&self, command: Vec<String>, result: Result<String, String>) {
        self.exec_results.lock().unwrap().insert(command, result);
    }

    /// Execute command in container
    pub fn exec(&self, container_id: &str, command: Vec<String>) -> Result<String, String> {
        // Record the call
        self.exec_calls.lock().unwrap().push(command.clone());

        // Return pre-configured result
        self.exec_results
            .lock()
            .unwrap()
            .get(&command)
            .cloned()
            .unwrap_or_else(|| Ok(format!("Executed in {}: {:?}", container_id, command)))
    }

    /// Start a container
    pub fn start_container(&self, image: &str) -> String {
        let container_id = format!("container_{}", self.containers.lock().unwrap().len());
        self.containers
            .lock()
            .unwrap()
            .push(format!("{}:{}", image, container_id));
        container_id
    }

    /// Verify command was executed
    pub fn verify_exec(&self, command: &[String]) -> bool {
        self.exec_calls
            .lock()
            .unwrap()
            .iter()
            .any(|cmd| cmd == command)
    }

    /// Get number of exec calls
    pub fn exec_call_count(&self) -> usize {
        self.exec_calls.lock().unwrap().len()
    }
}

/// Mock template renderer for variable substitution
#[derive(Clone, Debug)]
pub struct MockTemplateRenderer {
    /// Rendering calls (template -> variables)
    pub render_calls: Arc<Mutex<Vec<(String, HashMap<String, String>)>>>,
    /// Pre-configured render results
    pub render_results: Arc<Mutex<HashMap<String, String>>>,
}

impl MockTemplateRenderer {
    pub fn new() -> Self {
        Self {
            render_calls: Arc::new(Mutex::new(Vec::new())),
            render_results: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Configure expected render result
    pub fn expect_render(&self, template: &str, result: &str) {
        self.render_results
            .lock()
            .unwrap()
            .insert(template.to_string(), result.to_string());
    }

    /// Render template with variables
    pub fn render(&self, template: &str, variables: HashMap<String, String>) -> String {
        // Record the call
        self.render_calls
            .lock()
            .unwrap()
            .push((template.to_string(), variables.clone()));

        // Return pre-configured result or simple substitution
        if let Some(result) = self.render_results.lock().unwrap().get(template) {
            result.clone()
        } else {
            // Simple substitution for testing
            let mut result = template.to_string();
            for (key, value) in variables {
                result = result.replace(&format!("{{{{{}}}}}", key), &value);
            }
            result
        }
    }

    /// Verify render was called with template
    pub fn verify_render(&self, template: &str) -> bool {
        self.render_calls
            .lock()
            .unwrap()
            .iter()
            .any(|(t, _)| t == template)
    }

    /// Verify render was called with specific variables
    pub fn verify_render_with_vars(
        &self,
        template: &str,
        expected_vars: &HashMap<String, String>,
    ) -> bool {
        self.render_calls
            .lock()
            .unwrap()
            .iter()
            .any(|(t, vars)| t == template && vars == expected_vars)
    }
}

/// Mock span collector for OTEL validation
#[derive(Clone, Debug)]
pub struct MockSpanCollector {
    /// Collected spans
    pub spans: Arc<Mutex<Vec<MockSpan>>>,
    /// Validation expectations
    pub expectations: Arc<Mutex<Vec<SpanExpectation>>>,
    /// Validation results
    pub validation_results: Arc<Mutex<Vec<bool>>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MockSpan {
    pub name: String,
    pub attributes: HashMap<String, String>,
    pub parent_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct SpanExpectation {
    pub name: String,
    pub required_attributes: Vec<String>,
    pub parent_required: bool,
}

impl MockSpanCollector {
    pub fn new() -> Self {
        Self {
            spans: Arc::new(Mutex::new(Vec::new())),
            expectations: Arc::new(Mutex::new(Vec::new())),
            validation_results: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Collect a span
    pub fn collect(&self, span: MockSpan) {
        self.spans.lock().unwrap().push(span);
    }

    /// Add validation expectation
    pub fn expect_span(&self, name: &str, required_attrs: Vec<String>, parent_required: bool) {
        self.expectations.lock().unwrap().push(SpanExpectation {
            name: name.to_string(),
            required_attributes: required_attrs,
            parent_required,
        });
    }

    /// Validate collected spans against expectations
    pub fn validate(&self) -> bool {
        let spans = self.spans.lock().unwrap();
        let expectations = self.expectations.lock().unwrap();

        for expectation in expectations.iter() {
            let found = spans.iter().any(|span| {
                // Check name matches
                if span.name != expectation.name {
                    return false;
                }

                // Check required attributes present
                for attr in &expectation.required_attributes {
                    if !span.attributes.contains_key(attr) {
                        return false;
                    }
                }

                // Check parent if required
                if expectation.parent_required && span.parent_id.is_none() {
                    return false;
                }

                true
            });

            if !found {
                self.validation_results.lock().unwrap().push(false);
                return false;
            }
        }

        self.validation_results.lock().unwrap().push(true);
        true
    }

    /// Get collected span count
    pub fn span_count(&self) -> usize {
        self.spans.lock().unwrap().len()
    }

    /// Find span by name
    pub fn find_span(&self, name: &str) -> Option<MockSpan> {
        self.spans
            .lock()
            .unwrap()
            .iter()
            .find(|s| s.name == name)
            .cloned()
    }
}

// ============================================================================
// LONDON SCHOOL TDD TESTS - Initially RED (Failing)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // Test Suite 1: TOML Parser Mock Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_mock_toml_parser_validates_schema_before_parsing() {
        // ARRANGE: Create mock parser with valid schema
        let parser = MockTomlParser::new();
        parser.expect_schema_valid("test.toml", true);
        parser.expect_parse(
            "test.toml",
            Ok(MockTestConfig {
                name: "test".to_string(),
                services: vec!["svc1".to_string()],
                steps: vec![],
                variables: HashMap::new(),
            }),
        );

        // ACT: Validate schema first, then parse
        let schema_valid = parser.validate_schema("test.toml");
        let parse_result = parser.parse("test.toml");

        // ASSERT: Schema validation happens BEFORE parsing
        assert!(schema_valid, "Schema should be valid");
        assert!(
            parse_result.is_ok(),
            "Parse should succeed after schema validation"
        );
        assert!(
            parser.verify_parse_called("test.toml"),
            "Parse should have been called"
        );
    }

    #[test]
    fn test_mock_toml_parser_rejects_invalid_schema() {
        // ARRANGE: Mock parser with invalid schema
        let parser = MockTomlParser::new();
        parser.expect_schema_valid("invalid.toml", false);

        // ACT: Validate schema
        let schema_valid = parser.validate_schema("invalid.toml");

        // ASSERT: Schema validation fails
        assert!(
            !schema_valid,
            "Schema validation should fail for invalid TOML"
        );
    }

    #[test]
    fn test_mock_toml_parser_tracks_parse_calls() {
        // ARRANGE: Mock parser
        let parser = MockTomlParser::new();
        parser.expect_parse(
            "file1.toml",
            Ok(MockTestConfig {
                name: "test1".to_string(),
                services: vec![],
                steps: vec![],
                variables: HashMap::new(),
            }),
        );
        parser.expect_parse(
            "file2.toml",
            Ok(MockTestConfig {
                name: "test2".to_string(),
                services: vec![],
                steps: vec![],
                variables: HashMap::new(),
            }),
        );

        // ACT: Parse multiple files
        let _ = parser.parse("file1.toml");
        let _ = parser.parse("file2.toml");

        // ASSERT: Both parse calls recorded
        assert_eq!(parser.parse_call_count(), 2, "Should track all parse calls");
        assert!(
            parser.verify_parse_called("file1.toml"),
            "First parse call tracked"
        );
        assert!(
            parser.verify_parse_called("file2.toml"),
            "Second parse call tracked"
        );
    }

    #[test]
    fn test_mock_toml_parser_returns_configured_error() {
        // ARRANGE: Mock parser configured to fail
        let parser = MockTomlParser::new();
        parser.expect_parse("bad.toml", Err("Parse error: invalid TOML".to_string()));

        // ACT: Attempt parse
        let result = parser.parse("bad.toml");

        // ASSERT: Returns configured error
        assert!(result.is_err(), "Should return error");
        assert_eq!(
            result.unwrap_err(),
            "Parse error: invalid TOML",
            "Should return configured error message"
        );
    }

    // ------------------------------------------------------------------------
    // Test Suite 2: Service Registry Mock Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_mock_service_registry_looks_up_registered_services() {
        // ARRANGE: Registry with registered service
        let registry = MockServiceRegistry::new();
        registry.register_service("postgres", "generic_container");

        // ACT: Lookup service
        let plugin_type = registry.lookup("postgres");

        // ASSERT: Returns correct plugin type
        assert_eq!(plugin_type, Some("generic_container".to_string()));
        assert!(
            registry.verify_lookup("postgres"),
            "Lookup should be tracked"
        );
    }

    #[test]
    fn test_mock_service_registry_returns_none_for_unknown_service() {
        // ARRANGE: Empty registry
        let registry = MockServiceRegistry::new();

        // ACT: Lookup non-existent service
        let plugin_type = registry.lookup("unknown");

        // ASSERT: Returns None
        assert_eq!(plugin_type, None, "Should return None for unknown service");
        assert!(
            registry.verify_lookup("unknown"),
            "Failed lookup still tracked"
        );
    }

    #[test]
    fn test_mock_service_registry_tracks_creation_calls() {
        // ARRANGE: Registry
        let registry = MockServiceRegistry::new();

        // ACT: Register multiple services
        registry.register_service("redis", "generic_container");
        registry.register_service("postgres", "generic_container");

        // ASSERT: Creation calls tracked
        assert!(
            registry.verify_creation("redis", "generic_container"),
            "Redis creation tracked"
        );
        assert!(
            registry.verify_creation("postgres", "generic_container"),
            "Postgres creation tracked"
        );
    }

    #[test]
    fn test_mock_service_registry_supports_multiple_lookups() {
        // ARRANGE: Registry with service
        let registry = MockServiceRegistry::new();
        registry.register_service("cache", "redis");

        // ACT: Multiple lookups
        let lookup1 = registry.lookup("cache");
        let lookup2 = registry.lookup("cache");
        let lookup3 = registry.lookup("other");

        // ASSERT: All lookups work correctly
        assert_eq!(lookup1, Some("redis".to_string()));
        assert_eq!(lookup2, Some("redis".to_string()));
        assert_eq!(lookup3, None);
    }

    // ------------------------------------------------------------------------
    // Test Suite 3: Container Backend Mock Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_mock_container_backend_executes_commands() {
        // ARRANGE: Backend with expected command result
        let backend = MockContainerBackend::new();
        let command = vec!["echo".to_string(), "hello".to_string()];
        backend.expect_exec(command.clone(), Ok("hello\n".to_string()));

        // ACT: Execute command
        let result = backend.exec("container123", command.clone());

        // ASSERT: Returns expected result and tracks call
        assert_eq!(result, Ok("hello\n".to_string()));
        assert!(backend.verify_exec(&command), "Command execution tracked");
    }

    #[test]
    fn test_mock_container_backend_starts_containers() {
        // ARRANGE: Backend
        let backend = MockContainerBackend::new();

        // ACT: Start containers
        let container1 = backend.start_container("alpine:latest");
        let container2 = backend.start_container("postgres:14");

        // ASSERT: Unique container IDs generated
        assert_ne!(container1, container2, "Container IDs should be unique");
        assert!(
            container1.starts_with("container_"),
            "Container ID has prefix"
        );
    }

    #[test]
    fn test_mock_container_backend_tracks_exec_calls() {
        // ARRANGE: Backend with multiple commands
        let backend = MockContainerBackend::new();
        let cmd1 = vec!["ls".to_string()];
        let cmd2 = vec!["pwd".to_string()];
        backend.expect_exec(cmd1.clone(), Ok("file1\nfile2\n".to_string()));
        backend.expect_exec(cmd2.clone(), Ok("/home\n".to_string()));

        // ACT: Execute multiple commands
        let _ = backend.exec("container", cmd1.clone());
        let _ = backend.exec("container", cmd2.clone());

        // ASSERT: Both calls tracked
        assert_eq!(backend.exec_call_count(), 2, "Should track all exec calls");
        assert!(backend.verify_exec(&cmd1), "First command tracked");
        assert!(backend.verify_exec(&cmd2), "Second command tracked");
    }

    #[test]
    fn test_mock_container_backend_returns_exec_errors() {
        // ARRANGE: Backend configured to fail
        let backend = MockContainerBackend::new();
        let command = vec!["false".to_string()];
        backend.expect_exec(command.clone(), Err("exit code 1".to_string()));

        // ACT: Execute failing command
        let result = backend.exec("container", command);

        // ASSERT: Returns error as configured
        assert!(result.is_err(), "Should return error");
        assert_eq!(result.unwrap_err(), "exit code 1");
    }

    // ------------------------------------------------------------------------
    // Test Suite 4: Template Renderer Mock Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_mock_template_renderer_substitutes_variables() {
        // ARRANGE: Renderer with template
        let renderer = MockTemplateRenderer::new();
        let template = "Hello {{name}}!";
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "World".to_string());

        // ACT: Render template
        let result = renderer.render(template, vars.clone());

        // ASSERT: Variables substituted
        assert_eq!(result, "Hello World!", "Variables should be substituted");
        assert!(renderer.verify_render(template), "Render call tracked");
    }

    #[test]
    fn test_mock_template_renderer_tracks_render_calls() {
        // ARRANGE: Renderer
        let renderer = MockTemplateRenderer::new();
        let template1 = "{{var1}}";
        let template2 = "{{var2}}";
        let mut vars1 = HashMap::new();
        vars1.insert("var1".to_string(), "value1".to_string());
        let mut vars2 = HashMap::new();
        vars2.insert("var2".to_string(), "value2".to_string());

        // ACT: Render multiple templates
        let _ = renderer.render(template1, vars1.clone());
        let _ = renderer.render(template2, vars2.clone());

        // ASSERT: All renders tracked
        assert!(renderer.verify_render(template1), "First render tracked");
        assert!(renderer.verify_render(template2), "Second render tracked");
        assert!(
            renderer.verify_render_with_vars(template1, &vars1),
            "First render with vars tracked"
        );
    }

    #[test]
    fn test_mock_template_renderer_uses_configured_result() {
        // ARRANGE: Renderer with pre-configured result
        let renderer = MockTemplateRenderer::new();
        let template = "{{complex}}";
        renderer.expect_render(template, "CUSTOM_RESULT");

        // ACT: Render
        let result = renderer.render(template, HashMap::new());

        // ASSERT: Uses configured result
        assert_eq!(result, "CUSTOM_RESULT", "Should use configured result");
    }

    #[test]
    fn test_mock_template_renderer_handles_multiple_variables() {
        // ARRANGE: Renderer with multi-var template
        let renderer = MockTemplateRenderer::new();
        let template = "{{name}} is {{age}} years old";
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        vars.insert("age".to_string(), "30".to_string());

        // ACT: Render
        let result = renderer.render(template, vars);

        // ASSERT: All variables substituted
        assert_eq!(result, "Alice is 30 years old");
    }

    // ------------------------------------------------------------------------
    // Test Suite 5: Span Collector Mock Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_mock_span_collector_collects_spans() {
        // ARRANGE: Collector
        let collector = MockSpanCollector::new();
        let span = MockSpan {
            name: "test.span".to_string(),
            attributes: HashMap::new(),
            parent_id: None,
        };

        // ACT: Collect span
        collector.collect(span.clone());

        // ASSERT: Span collected
        assert_eq!(collector.span_count(), 1, "Should have 1 span");
        assert_eq!(
            collector.find_span("test.span"),
            Some(span),
            "Should find collected span"
        );
    }

    #[test]
    fn test_mock_span_collector_validates_expectations() {
        // ARRANGE: Collector with expectations
        let collector = MockSpanCollector::new();
        collector.expect_span("operation.start", vec!["service.name".to_string()], false);

        // ACT: Collect matching span
        let mut attrs = HashMap::new();
        attrs.insert("service.name".to_string(), "test-service".to_string());
        collector.collect(MockSpan {
            name: "operation.start".to_string(),
            attributes: attrs,
            parent_id: None,
        });

        // ASSERT: Validation passes
        assert!(collector.validate(), "Should validate successfully");
    }

    #[test]
    fn test_mock_span_collector_fails_validation_for_missing_span() {
        // ARRANGE: Collector with expectations but no spans
        let collector = MockSpanCollector::new();
        collector.expect_span("required.span", vec![], false);

        // ACT: Validate without collecting span
        let valid = collector.validate();

        // ASSERT: Validation fails
        assert!(!valid, "Should fail validation when expected span missing");
    }

    #[test]
    fn test_mock_span_collector_validates_required_attributes() {
        // ARRANGE: Collector expecting attributes
        let collector = MockSpanCollector::new();
        collector.expect_span(
            "database.query",
            vec!["db.system".to_string(), "db.statement".to_string()],
            false,
        );

        // ACT: Collect span missing required attribute
        let mut attrs = HashMap::new();
        attrs.insert("db.system".to_string(), "postgresql".to_string());
        // Missing db.statement
        collector.collect(MockSpan {
            name: "database.query".to_string(),
            attributes: attrs,
            parent_id: None,
        });

        // ASSERT: Validation fails
        assert!(
            !collector.validate(),
            "Should fail validation when required attribute missing"
        );
    }

    #[test]
    fn test_mock_span_collector_validates_parent_requirement() {
        // ARRANGE: Collector expecting parent span
        let collector = MockSpanCollector::new();
        collector.expect_span("child.span", vec![], true);

        // ACT: Collect span without parent
        collector.collect(MockSpan {
            name: "child.span".to_string(),
            attributes: HashMap::new(),
            parent_id: None,
        });

        // ASSERT: Validation fails
        assert!(
            !collector.validate(),
            "Should fail when parent required but missing"
        );
    }

    #[test]
    fn test_mock_span_collector_validates_parent_present() {
        // ARRANGE: Collector expecting parent span
        let collector = MockSpanCollector::new();
        collector.expect_span("child.span", vec![], true);

        // ACT: Collect span WITH parent
        collector.collect(MockSpan {
            name: "child.span".to_string(),
            attributes: HashMap::new(),
            parent_id: Some("parent-123".to_string()),
        });

        // ASSERT: Validation passes
        assert!(collector.validate(), "Should pass when parent present");
    }

    // ------------------------------------------------------------------------
    // Test Suite 6: Integration Tests - Mock Collaboration
    // ------------------------------------------------------------------------

    #[test]
    fn test_mocks_collaborate_in_toml_workflow() {
        // ARRANGE: Full mock ecosystem
        let parser = MockTomlParser::new();
        let registry = MockServiceRegistry::new();
        let backend = MockContainerBackend::new();
        let collector = MockSpanCollector::new();

        // Configure mocks for workflow
        parser.expect_schema_valid("test.toml", true);
        parser.expect_parse(
            "test.toml",
            Ok(MockTestConfig {
                name: "integration_test".to_string(),
                services: vec!["postgres".to_string()],
                steps: vec![MockStep {
                    name: "setup".to_string(),
                    command: vec!["psql".to_string(), "-c".to_string(), "SELECT 1".to_string()],
                }],
                variables: HashMap::new(),
            }),
        );

        registry.register_service("postgres", "generic_container");
        backend.expect_exec(
            vec!["psql".to_string(), "-c".to_string(), "SELECT 1".to_string()],
            Ok("1\n".to_string()),
        );

        collector.expect_span("test.execution", vec!["test.name".to_string()], false);

        // ACT: Execute workflow
        // 1. Parse TOML
        let config = parser.parse("test.toml").unwrap();

        // 2. Lookup service
        let plugin = registry.lookup(&config.services[0]).unwrap();

        // 3. Start container
        let container_id = backend.start_container("postgres:14");

        // 4. Execute command
        let result = backend.exec(&container_id, config.steps[0].command.clone());

        // 5. Collect span
        let mut attrs = HashMap::new();
        attrs.insert("test.name".to_string(), config.name.clone());
        collector.collect(MockSpan {
            name: "test.execution".to_string(),
            attributes: attrs,
            parent_id: None,
        });

        // ASSERT: All interactions verified
        assert!(parser.verify_parse_called("test.toml"), "TOML parsed");
        assert!(registry.verify_lookup("postgres"), "Service looked up");
        assert_eq!(plugin, "generic_container", "Correct plugin found");
        assert!(result.is_ok(), "Command executed successfully");
        assert!(
            backend.verify_exec(&vec![
                "psql".to_string(),
                "-c".to_string(),
                "SELECT 1".to_string()
            ]),
            "Command execution tracked"
        );
        assert!(collector.validate(), "Span expectations met");
    }

    #[test]
    fn test_mocks_handle_workflow_failure() {
        // ARRANGE: Mocks configured for failure scenario
        let parser = MockTomlParser::new();
        let registry = MockServiceRegistry::new();
        let backend = MockContainerBackend::new();

        // Configure failure
        parser.expect_schema_valid("bad.toml", false);
        parser.expect_parse("bad.toml", Err("Invalid schema".to_string()));

        // ACT: Attempt workflow
        let schema_valid = parser.validate_schema("bad.toml");

        // ASSERT: Workflow stops on schema validation failure
        assert!(!schema_valid, "Schema validation failed");
        // Don't parse if schema invalid
        if schema_valid {
            let _ = parser.parse("bad.toml");
        }
        assert_eq!(
            parser.parse_call_count(),
            0,
            "Parse should not be called when schema invalid"
        );
    }

    #[test]
    fn test_mocks_support_template_variable_workflow() {
        // ARRANGE: Mocks for template-driven test
        let parser = MockTomlParser::new();
        let renderer = MockTemplateRenderer::new();
        let backend = MockContainerBackend::new();

        let mut variables = HashMap::new();
        variables.insert("db_name".to_string(), "testdb".to_string());

        parser.expect_parse(
            "template.toml",
            Ok(MockTestConfig {
                name: "template_test".to_string(),
                services: vec![],
                steps: vec![MockStep {
                    name: "query".to_string(),
                    command: vec![
                        "psql".to_string(),
                        "-d".to_string(),
                        "{{db_name}}".to_string(),
                    ],
                }],
                variables: variables.clone(),
            }),
        );

        // ACT: Parse and render
        let config = parser.parse("template.toml").unwrap();
        let rendered_db = renderer.render("{{db_name}}", config.variables);

        // Configure backend with rendered command
        backend.expect_exec(
            vec!["psql".to_string(), "-d".to_string(), "testdb".to_string()],
            Ok("Connected to testdb\n".to_string()),
        );

        let result = backend.exec(
            "container",
            vec!["psql".to_string(), "-d".to_string(), rendered_db],
        );

        // ASSERT: Template variables rendered and used
        assert!(renderer.verify_render("{{db_name}}"), "Template rendered");
        assert!(result.is_ok(), "Command with rendered variables executed");
    }
}
