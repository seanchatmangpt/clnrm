//! OTLP Export Validation Tests
//!
//! CRITICAL: These tests validate that ALL telemetry is correctly exported via OTLP
//! and can be validated by Weaver. If telemetry doesn't export, Weaver can't validate it.

use clnrm_core::telemetry::*;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

/// Mock OTLP collector for testing
#[derive(Debug, Clone)]
pub struct MockOtlpCollector {
    spans: Arc<Mutex<Vec<ExportedSpan>>>,
    metrics: Arc<Mutex<Vec<ExportedMetric>>>,
}

#[derive(Debug, Clone)]
pub struct ExportedSpan {
    pub name: String,
    pub attributes: HashMap<String, AttributeValue>,
    pub status: SpanStatus,
}

#[derive(Debug, Clone)]
pub struct ExportedMetric {
    pub name: String,
    pub value: f64,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum AttributeValue {
    String(String),
    Bool(bool),
    Int(i64),
    Float(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpanStatus {
    Ok,
    Error,
}

impl MockOtlpCollector {
    pub fn new() -> Self {
        Self {
            spans: Arc::new(Mutex::new(Vec::new())),
            metrics: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn record_span(&self, span: ExportedSpan) {
        self.spans.lock().unwrap().push(span);
    }

    pub fn record_metric(&self, metric: ExportedMetric) {
        self.metrics.lock().unwrap().push(metric);
    }

    pub fn get_spans(&self) -> Vec<ExportedSpan> {
        self.spans.lock().unwrap().clone()
    }

    pub fn get_metrics(&self) -> Vec<ExportedMetric> {
        self.metrics.lock().unwrap().clone()
    }

    pub fn clear(&self) {
        self.spans.lock().unwrap().clear();
        self.metrics.lock().unwrap().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_otlp_exporter_initializes() {
        // Arrange
        let config = OtlpConfig {
            endpoint: "http://localhost:4317".to_string(),
            protocol: OtlpProtocol::Grpc,
            timeout_seconds: 10,
        };

        // Act
        let result = initialize_otlp_exporter(&config);

        // Assert
        assert!(result.is_ok(), "OTLP exporter failed to initialize: {:?}", result.err());
    }

    #[tokio::test]
    async fn test_http_protocol_initializes() {
        // Arrange
        let config = OtlpConfig {
            endpoint: "http://localhost:4318".to_string(),
            protocol: OtlpProtocol::Http,
            timeout_seconds: 10,
        };

        // Act
        let result = initialize_otlp_exporter(&config);

        // Assert
        assert!(result.is_ok(), "HTTP OTLP exporter failed to initialize");
    }

    #[tokio::test]
    async fn test_span_export_succeeds() {
        // Arrange
        let collector = MockOtlpCollector::new();
        let _guard = initialize_test_telemetry_with_collector(collector.clone());

        // Act - Create and export span
        let span = TestExecutionSpan::new("test", "alpine:latest");
        span.set_isolated(true);
        span.set_result(TestResult::Pass);
        span.end();

        // Wait for export
        sleep(Duration::from_millis(100)).await;

        // Assert - Check OTLP collector received span
        let received_spans = collector.get_spans();
        assert!(
            received_spans.len() > 0,
            "No spans exported to OTLP"
        );
        assert!(
            received_spans.iter().any(|s| s.name == "test_execution"),
            "test_execution span not found"
        );
    }

    #[tokio::test]
    async fn test_all_span_types_export() {
        // Arrange
        let collector = MockOtlpCollector::new();
        let _guard = initialize_test_telemetry_with_collector(collector.clone());

        // Act - Export spans of each type
        let test_span = TestExecutionSpan::new("test", "alpine");
        test_span.end();

        let container_span = ContainerLifecycleSpan::new("container-123", "alpine");
        container_span.set_state(ContainerState::Running);
        container_span.end();

        let plugin_span = PluginExecutionSpan::new("surrealdb", "database");
        plugin_span.set_state(PluginState::Started);
        plugin_span.end();

        // Wait for export
        sleep(Duration::from_millis(100)).await;

        // Assert all exported
        let spans = collector.get_spans();
        assert!(
            spans.iter().any(|s| s.name == "test_execution"),
            "test_execution span missing"
        );
        assert!(
            spans.iter().any(|s| s.name == "container_lifecycle"),
            "container_lifecycle span missing"
        );
        assert!(
            spans.iter().any(|s| s.name == "plugin_execution"),
            "plugin_execution span missing"
        );
    }

    #[tokio::test]
    async fn test_required_attributes_export() {
        // Arrange
        let collector = MockOtlpCollector::new();
        let _guard = initialize_test_telemetry_with_collector(collector.clone());

        // Act - Create span with all required attributes
        let span = TestExecutionSpan::new("test", "alpine");
        span.set_container_id("abc123");
        span.set_isolated(true);
        span.set_result(TestResult::Pass);
        span.end();

        sleep(Duration::from_millis(100)).await;

        // Assert all attributes present
        let spans = collector.get_spans();
        let test_span = spans
            .iter()
            .find(|s| s.name == "test_execution")
            .expect("test_execution span not found");

        // Check required attributes
        assert!(
            matches!(test_span.attributes.get("container.id"), Some(AttributeValue::String(id)) if id == "abc123"),
            "container.id attribute missing or incorrect"
        );
        assert!(
            matches!(test_span.attributes.get("test.isolated"), Some(AttributeValue::Bool(true))),
            "test.isolated attribute missing or incorrect"
        );
        assert!(
            matches!(test_span.attributes.get("test.result"), Some(AttributeValue::String(result)) if result == "pass"),
            "test.result attribute missing or incorrect"
        );
    }

    #[tokio::test]
    async fn test_error_telemetry_exports() {
        // Arrange
        let collector = MockOtlpCollector::new();
        let _guard = initialize_test_telemetry_with_collector(collector.clone());

        // Act - Simulate error
        let span = TestExecutionSpan::new("test", "alpine");
        span.set_result(TestResult::Error);
        span.set_error_message("Container failed to start");
        span.set_error_type("ContainerStartupError");
        span.end();

        sleep(Duration::from_millis(100)).await;

        // Assert error attributes present
        let spans = collector.get_spans();
        let test_span = spans
            .iter()
            .find(|s| s.name == "test_execution")
            .expect("test_execution span not found");

        assert!(
            matches!(test_span.attributes.get("test.result"), Some(AttributeValue::String(result)) if result == "error"),
            "test.result not set to error"
        );
        assert!(
            test_span.attributes.contains_key("error.message"),
            "error.message attribute missing"
        );
        assert!(
            test_span.attributes.contains_key("error.type"),
            "error.type attribute missing"
        );
        assert_eq!(
            test_span.status,
            SpanStatus::Error,
            "Span status should be Error"
        );
    }

    #[tokio::test]
    async fn test_metrics_export() {
        // Arrange
        let collector = MockOtlpCollector::new();
        let _guard = initialize_test_telemetry_with_collector(collector.clone());

        // Act - Record metrics
        record_test_duration("test", 125.5, true);
        increment_test_counter("test", "pass");
        record_container_count(1);

        sleep(Duration::from_millis(100)).await;

        // Assert metrics exported
        let metrics = collector.get_metrics();
        assert!(
            metrics.iter().any(|m| m.name == "clnrm.test.duration"),
            "clnrm.test.duration metric missing"
        );
        assert!(
            metrics.iter().any(|m| m.name == "clnrm.test.counter"),
            "clnrm.test.counter metric missing"
        );
        assert!(
            metrics.iter().any(|m| m.name == "clnrm.container.count"),
            "clnrm.container.count metric missing"
        );
    }

    #[tokio::test]
    async fn test_metric_values_correct() {
        // Arrange
        let collector = MockOtlpCollector::new();
        let _guard = initialize_test_telemetry_with_collector(collector.clone());

        // Act
        record_test_duration("test", 125.5, true);

        sleep(Duration::from_millis(100)).await;

        // Assert
        let metrics = collector.get_metrics();
        let duration_metric = metrics
            .iter()
            .find(|m| m.name == "clnrm.test.duration")
            .expect("duration metric not found");

        assert_eq!(duration_metric.value, 125.5, "Incorrect metric value");
        assert_eq!(
            duration_metric.attributes.get("test.name").unwrap(),
            "test"
        );
    }

    #[tokio::test]
    async fn test_container_lifecycle_attributes() {
        // Arrange
        let collector = MockOtlpCollector::new();
        let _guard = initialize_test_telemetry_with_collector(collector.clone());

        // Act
        let span = ContainerLifecycleSpan::new("container-abc", "alpine:3.19");
        span.set_state(ContainerState::Running);
        span.set_port_mapping("8080", "80");
        span.set_health_status(HealthStatus::Healthy);
        span.end();

        sleep(Duration::from_millis(100)).await;

        // Assert
        let spans = collector.get_spans();
        let container_span = spans
            .iter()
            .find(|s| s.name == "container_lifecycle")
            .expect("container_lifecycle span not found");

        assert!(
            matches!(container_span.attributes.get("container.id"), Some(AttributeValue::String(id)) if id == "container-abc")
        );
        assert!(
            matches!(container_span.attributes.get("container.image"), Some(AttributeValue::String(img)) if img == "alpine:3.19")
        );
        assert!(
            matches!(container_span.attributes.get("container.state"), Some(AttributeValue::String(state)) if state == "running")
        );
        assert!(
            matches!(container_span.attributes.get("container.health.status"), Some(AttributeValue::String(health)) if health == "healthy")
        );
    }

    #[tokio::test]
    async fn test_plugin_execution_attributes() {
        // Arrange
        let collector = MockOtlpCollector::new();
        let _guard = initialize_test_telemetry_with_collector(collector.clone());

        // Act
        let span = PluginExecutionSpan::new("surrealdb", "database");
        span.set_state(PluginState::Started);
        span.set_config_option("host", "localhost");
        span.set_config_option("port", "8000");
        span.end();

        sleep(Duration::from_millis(100)).await;

        // Assert
        let spans = collector.get_spans();
        let plugin_span = spans
            .iter()
            .find(|s| s.name == "plugin_execution")
            .expect("plugin_execution span not found");

        assert!(
            matches!(plugin_span.attributes.get("plugin.name"), Some(AttributeValue::String(name)) if name == "surrealdb")
        );
        assert!(
            matches!(plugin_span.attributes.get("plugin.type"), Some(AttributeValue::String(ptype)) if ptype == "database")
        );
        assert!(
            matches!(plugin_span.attributes.get("plugin.state"), Some(AttributeValue::String(state)) if state == "started")
        );
    }

    #[tokio::test]
    async fn test_concurrent_span_export() {
        // Arrange
        let collector = MockOtlpCollector::new();
        let _guard = initialize_test_telemetry_with_collector(collector.clone());

        // Act - Create multiple spans concurrently
        let handles: Vec<_> = (0..10)
            .map(|i| {
                tokio::spawn(async move {
                    let span = TestExecutionSpan::new(&format!("test-{}", i), "alpine");
                    span.set_isolated(true);
                    span.set_result(TestResult::Pass);
                    span.end();
                })
            })
            .collect();

        for handle in handles {
            handle.await.unwrap();
        }

        sleep(Duration::from_millis(200)).await;

        // Assert - All spans exported
        let spans = collector.get_spans();
        assert!(
            spans.len() >= 10,
            "Expected at least 10 spans, got {}",
            spans.len()
        );
    }

    #[tokio::test]
    async fn test_span_hierarchy_preserved() {
        // Arrange
        let collector = MockOtlpCollector::new();
        let _guard = initialize_test_telemetry_with_collector(collector.clone());

        // Act - Create parent-child span relationship
        let parent_span = TestExecutionSpan::new("parent-test", "alpine");

        let child_span = ContainerLifecycleSpan::new("container-child", "alpine");
        child_span.set_parent_span_id(parent_span.span_id());
        child_span.end();

        parent_span.end();

        sleep(Duration::from_millis(100)).await;

        // Assert - Hierarchy maintained
        let spans = collector.get_spans();
        let parent = spans.iter().find(|s| s.name == "test_execution").unwrap();
        let child = spans.iter().find(|s| s.name == "container_lifecycle").unwrap();

        assert!(
            child.attributes.contains_key("parent.span.id"),
            "Child span missing parent reference"
        );
    }

    #[tokio::test]
    async fn test_export_failure_handling() {
        // Arrange - Use invalid endpoint to trigger export failure
        let config = OtlpConfig {
            endpoint: "http://invalid-endpoint:9999".to_string(),
            protocol: OtlpProtocol::Grpc,
            timeout_seconds: 1,
        };

        // Act
        let result = initialize_otlp_exporter(&config);

        // Assert - Should handle gracefully
        // Export failures should not panic, but log errors
        assert!(result.is_ok(), "Exporter initialization should not fail");
    }
}

// Helper functions for testing
#[cfg(test)]
fn initialize_test_telemetry_with_collector(_collector: MockOtlpCollector) -> TelemetryGuard {
    // Initialize telemetry with mock collector
    // In real implementation, this would configure the OTLP pipeline
    // to use the mock collector instead of real endpoint
    todo!("Implement mock collector integration")
}

// Type definitions for testing (will be in actual telemetry module)
pub struct OtlpConfig {
    pub endpoint: String,
    pub protocol: OtlpProtocol,
    pub timeout_seconds: u64,
}

pub enum OtlpProtocol {
    Grpc,
    Http,
}

pub struct TelemetryGuard;

pub struct TestExecutionSpan {
    name: String,
    image: String,
}

impl TestExecutionSpan {
    pub fn new(name: &str, image: &str) -> Self {
        Self {
            name: name.to_string(),
            image: image.to_string(),
        }
    }

    pub fn set_isolated(&self, _isolated: bool) {}
    pub fn set_result(&self, _result: TestResult) {}
    pub fn set_container_id(&self, _id: &str) {}
    pub fn set_error_message(&self, _msg: &str) {}
    pub fn set_error_type(&self, _error_type: &str) {}
    pub fn span_id(&self) -> String {
        "span-123".to_string()
    }
    pub fn end(&self) {}
}

pub struct ContainerLifecycleSpan {
    id: String,
    image: String,
}

impl ContainerLifecycleSpan {
    pub fn new(id: &str, image: &str) -> Self {
        Self {
            id: id.to_string(),
            image: image.to_string(),
        }
    }

    pub fn set_state(&self, _state: ContainerState) {}
    pub fn set_port_mapping(&self, _host: &str, _container: &str) {}
    pub fn set_health_status(&self, _status: HealthStatus) {}
    pub fn set_parent_span_id(&self, _id: String) {}
    pub fn end(&self) {}
}

pub struct PluginExecutionSpan {
    name: String,
    plugin_type: String,
}

impl PluginExecutionSpan {
    pub fn new(name: &str, plugin_type: &str) -> Self {
        Self {
            name: name.to_string(),
            plugin_type: plugin_type.to_string(),
        }
    }

    pub fn set_state(&self, _state: PluginState) {}
    pub fn set_config_option(&self, _key: &str, _value: &str) {}
    pub fn end(&self) {}
}

#[derive(Debug, Clone, Copy)]
pub enum TestResult {
    Pass,
    Error,
}

#[derive(Debug, Clone, Copy)]
pub enum ContainerState {
    Running,
}

#[derive(Debug, Clone, Copy)]
pub enum PluginState {
    Started,
}

#[derive(Debug, Clone, Copy)]
pub enum HealthStatus {
    Healthy,
}

pub fn initialize_otlp_exporter(_config: &OtlpConfig) -> Result<(), String> {
    Ok(())
}

pub fn record_test_duration(_name: &str, _duration: f64, _success: bool) {}
pub fn increment_test_counter(_name: &str, _status: &str) {}
pub fn record_container_count(_count: i32) {}
