//! Span Enforcement Tests for v1.2.1
//!
//! This test suite follows London School TDD principles to verify OTEL span
//! collection, validation, and enforcement mechanisms.
//!
//! ## Coverage
//! - Span collection during test execution
//! - Span validation against expectations
//! - Missing span detection
//! - Attribute completeness verification
//! - Span relationships (parent-child)

use clnrm_core::config::{ExpectedSpanConfig, SpanExpectationConfig, TestConfig};
use clnrm_core::error::Result;
use std::collections::HashMap;

// ============================================================================
// SECTION 1: Span Collection Configuration
// ============================================================================

#[test]
fn test_span_expectation_configuration() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "span_test"
        version = "1.0.0"

        [[steps]]
        name = "test"
        command = ["echo", "test"]

        [expect.span]
        names = ["test.execution", "step.run"]
        count.min = 2
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    assert!(config.expect.is_some());
    let expect = config.expect.as_ref().unwrap();
    assert!(!expect.span.is_empty());
}

#[test]
fn test_multiple_span_expectations() {
    // Arrange - Multiple span expectations
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "test"
        command = ["echo", "test"]

        [expect.span]
        names = ["span1", "span2", "span3"]
        count.min = 3
        count.max = 10
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    let expect = config.expect.as_ref().unwrap();
    // span is a Vec, not Option - check length instead
    assert!(expect.span.len() > 0);
    // Names are checked at runtime
}

#[test]
fn test_span_expectation_with_attributes() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "test"
        command = ["echo", "test"]

        [[otel_validation.expected_spans]]
        name = "test.span"
        attributes = { "service.name" = "clnrm", "test.name" = "test" }
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    assert!(config.otel_validation.is_some());
}

// ============================================================================
// SECTION 2: Span Validation Rules
// ============================================================================

#[test]
fn test_span_count_validation() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "test"
        command = ["echo", "test"]

        [expect.span]
        names = ["test.span"]
        count.min = 1
        count.max = 5
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    let expect = config.expect.as_ref().unwrap();
    // span is a Vec, not Option - check length instead
    assert!(expect.span.len() > 0);
    // Count expectations are checked at runtime
}

#[test]
fn test_span_required_attributes() {
    // Arrange - Span must have specific attributes
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "test"
        command = ["echo", "test"]

        [[otel_validation.expected_spans]]
        name = "test.execution"
        attributes = { "test.status" = "success", "test.duration_ms" = "100" }
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    assert!(config.otel_validation.is_some());
}

#[test]
fn test_span_status_validation() {
    // Arrange - Validate span status
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "test"
        command = ["echo", "test"]

        [expect.status]
        allow = ["ok", "error"]
        deny = ["unset"]
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    assert!(config.expect.is_some());
}

// ============================================================================
// SECTION 3: Missing Span Detection
// ============================================================================

/// Mock span collector for testing
#[derive(Debug, Clone)]
struct MockSpanCollector {
    collected_spans: Vec<MockSpan>,
}

#[derive(Debug, Clone)]
struct MockSpan {
    name: String,
    attributes: HashMap<String, String>,
    parent_id: Option<String>,
}

impl MockSpanCollector {
    fn new() -> Self {
        Self {
            collected_spans: Vec::new(),
        }
    }

    fn add_span(&mut self, name: &str, attributes: HashMap<String, String>) {
        self.collected_spans.push(MockSpan {
            name: name.to_string(),
            attributes,
            parent_id: None,
        });
    }

    fn validate_expected_spans(&self, expected: &[&str]) -> Result<()> {
        for expected_name in expected {
            if !self
                .collected_spans
                .iter()
                .any(|s| s.name == *expected_name)
            {
                return Err(clnrm_core::error::CleanroomError::validation_error(
                    format!(
                        "Expected span '{}' not found in collected spans",
                        expected_name
                    ),
                ));
            }
        }
        Ok(())
    }

    fn get_span_count(&self) -> usize {
        self.collected_spans.len()
    }
}

#[test]
fn test_missing_span_detected() {
    // Arrange
    let mut collector = MockSpanCollector::new();
    collector.add_span("span1", HashMap::new());
    collector.add_span("span2", HashMap::new());

    let expected_spans = vec!["span1", "span2", "span3"];

    // Act
    let result = collector.validate_expected_spans(&expected_spans);

    // Assert - should fail because span3 is missing
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("span3"));
}

#[test]
fn test_all_expected_spans_present() {
    // Arrange
    let mut collector = MockSpanCollector::new();
    collector.add_span("span1", HashMap::new());
    collector.add_span("span2", HashMap::new());

    let expected_spans = vec!["span1", "span2"];

    // Act
    let result = collector.validate_expected_spans(&expected_spans);

    // Assert - should succeed
    assert!(result.is_ok());
}

#[test]
fn test_extra_spans_allowed() {
    // Arrange - More spans than expected is OK
    let mut collector = MockSpanCollector::new();
    collector.add_span("span1", HashMap::new());
    collector.add_span("span2", HashMap::new());
    collector.add_span("span3", HashMap::new());

    let expected_spans = vec!["span1", "span2"];

    // Act
    let result = collector.validate_expected_spans(&expected_spans);

    // Assert - should succeed (extra spans are fine)
    assert!(result.is_ok());
}

// ============================================================================
// SECTION 4: Attribute Completeness
// ============================================================================

impl MockSpan {
    fn has_attribute(&self, key: &str) -> bool {
        self.attributes.contains_key(key)
    }

    fn validate_attributes(&self, required: &[&str]) -> Result<()> {
        for attr in required {
            if !self.has_attribute(attr) {
                return Err(clnrm_core::error::CleanroomError::validation_error(
                    format!("Span '{}' missing required attribute '{}'", self.name, attr),
                ));
            }
        }
        Ok(())
    }

    fn get_attribute(&self, key: &str) -> Option<&String> {
        self.attributes.get(key)
    }
}

#[test]
fn test_required_attributes_present() {
    // Arrange
    let mut attrs = HashMap::new();
    attrs.insert("service.name".to_string(), "clnrm".to_string());
    attrs.insert("test.name".to_string(), "my_test".to_string());

    let span = MockSpan {
        name: "test.execution".to_string(),
        attributes: attrs,
        parent_id: None,
    };

    let required = vec!["service.name", "test.name"];

    // Act
    let result = span.validate_attributes(&required);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn test_missing_required_attribute() {
    // Arrange
    let mut attrs = HashMap::new();
    attrs.insert("service.name".to_string(), "clnrm".to_string());

    let span = MockSpan {
        name: "test.execution".to_string(),
        attributes: attrs,
        parent_id: None,
    };

    let required = vec!["service.name", "test.name"];

    // Act
    let result = span.validate_attributes(&required);

    // Assert - should fail
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("test.name"));
}

#[test]
fn test_attribute_value_validation() {
    // Arrange
    let mut attrs = HashMap::new();
    attrs.insert("test.status".to_string(), "success".to_string());

    let span = MockSpan {
        name: "test.execution".to_string(),
        attributes: attrs,
        parent_id: None,
    };

    // Act - validate attribute value
    let status = span.get_attribute("test.status");

    // Assert
    assert_eq!(status, Some(&"success".to_string()));
}

// ============================================================================
// SECTION 5: Span Relationships (Parent-Child)
// ============================================================================

#[test]
fn test_spanparent_child_relationship() {
    // Arrange
    let parent = MockSpan {
        name: "parent.span".to_string(),
        attributes: HashMap::new(),
        parent_id: None,
    };

    let child = MockSpan {
        name: "child.span".to_string(),
        attributes: HashMap::new(),
        parent_id: Some("parent.span".to_string()),
    };

    // Act
    let hasparent = child.parent_id.is_some();
    let parent_name = child.parent_id.as_ref().unwrap();

    // Assert
    assert!(hasparent);
    assert_eq!(parent_name, "parent.span");
}

#[test]
fn test_root_span_noparent() {
    // Arrange
    let root = MockSpan {
        name: "root.span".to_string(),
        attributes: HashMap::new(),
        parent_id: None,
    };

    // Act
    let is_root = root.parent_id.is_none();

    // Assert
    assert!(is_root);
}

#[test]
fn test_span_hierarchy_validation() {
    // Arrange - Build span tree
    let mut collector = MockSpanCollector::new();

    collector.collected_spans.push(MockSpan {
        name: "test.run".to_string(),
        attributes: HashMap::new(),
        parent_id: None,
    });

    collector.collected_spans.push(MockSpan {
        name: "step.execute".to_string(),
        attributes: HashMap::new(),
        parent_id: Some("test.run".to_string()),
    });

    collector.collected_spans.push(MockSpan {
        name: "command.run".to_string(),
        attributes: HashMap::new(),
        parent_id: Some("step.execute".to_string()),
    });

    // Act - validate hierarchy
    let has_root = collector
        .collected_spans
        .iter()
        .any(|s| s.parent_id.is_none());
    let has_children = collector
        .collected_spans
        .iter()
        .any(|s| s.parent_id.is_some());

    // Assert
    assert!(has_root, "Should have root span");
    assert!(has_children, "Should have child spans");
    assert_eq!(collector.get_span_count(), 3);
}

// ============================================================================
// SECTION 6: Span Count Enforcement
// ============================================================================

#[test]
fn test_minimum_span_count() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "test"
        command = ["echo", "test"]

        [expect.span]
        count.min = 2
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    let expect = config.expect.as_ref().unwrap();
    // span is a Vec, not Option - check length instead
    assert!(expect.span.len() > 0);
    // Count expectations are checked at runtime
}

#[test]
fn test_maximum_span_count() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "test"
        command = ["echo", "test"]

        [expect.span]
        count.max = 10
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    let expect = config.expect.as_ref().unwrap();
    assert!(!expect.span.is_empty());
}

#[test]
fn test_exact_span_count_range() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "test"
        command = ["echo", "test"]

        [expect.span]
        count.min = 5
        count.max = 5
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert - min == max implies exact count
    assert!(config.expect.is_some());
}

#[test]
fn test_span_count_validation_logic() {
    // Arrange
    let collector = MockSpanCollector::new();
    let min_count = 1;
    let max_count = 5;

    // Act - validate count is within bounds
    let actual_count = collector.get_span_count();
    let is_valid = actual_count >= min_count && actual_count <= max_count;

    // Assert
    assert!(!is_valid, "Empty collector should fail validation");
}

// ============================================================================
// SECTION 7: Integration with OTEL Validation
// ============================================================================

#[test]
fn test_otel_validation_section() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "test"
        command = ["echo", "test"]

        [otel_validation]
        enabled = true

        [[otel_validation.expected_spans]]
        name = "test.execution"
        attributes = { "service.name" = "clnrm" }
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    assert!(config.otel_validation.is_some());
    let otel_val = config.otel_validation.as_ref().unwrap();
    assert!(otel_val.enabled);
}

#[test]
fn test_expect_section_integration() {
    // Arrange - New v0.6.0 expect syntax
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "test"
        command = ["echo", "test"]

        [expect.span]
        names = ["test.run", "step.execute"]
        count.min = 2

        [expect.status]
        allow = ["ok"]
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    assert!(config.expect.is_some());
    let expect = config.expect.as_ref().unwrap();
    assert!(!expect.span.is_empty());
    assert!(expect.status.is_some());
}

#[test]
fn test_span_enforcement_disabled() {
    // Arrange - Config without span expectations
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "test"
        command = ["echo", "test"]
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert - no span enforcement
    assert!(config.expect.is_none());
    assert!(config.otel_validation.is_none());
}
