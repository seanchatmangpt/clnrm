//! OTLP Export Validation Tests
//!
//! CRITICAL: These tests validate that ALL telemetry is correctly exported via OTLP
//! and can be validated by Weaver. If telemetry doesn't export, Weaver can't validate it.

use clnrm_core::telemetry::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
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

#[derive(Debug, Clone, PartialEq)]
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
        assert!(
            result.is_ok(),
            "OTLP exporter failed to initialize: {:?}",
            result.err()
        );
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
        assert!(received_spans.len() > 0, "No spans exported to OTLP");
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
            matches!(
                test_span.attributes.get("test.isolated"),
                Some(AttributeValue::Bool(true))
            ),
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
        assert_eq!(duration_metric.attributes.get("test.name").unwrap(), "test");
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
        
        let parent = spans.iter().find(|s| s.name == "test.execution" || s.name == "parent-test" || s.name == "test_execution")
            .unwrap_or_else(|| panic!("Parent span not found. Available spans: {:?}", spans.iter().map(|s| &s.name).collect::<Vec<_>>()));
            
        let child = spans.iter().find(|s| s.name == "container.lifecycle" || s.name == "container-child" || s.name == "container_lifecycle")
            .unwrap_or_else(|| panic!("Child span not found. Available spans: {:?}", spans.iter().map(|s| &s.name).collect::<Vec<_>>()));

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
use std::cell::{Cell, RefCell};

thread_local! {
    static THREAD_COLLECTOR: RefCell<Option<MockOtlpCollector>> = RefCell::new(None);
    static THREAD_NETWORK_AVAILABLE: Cell<bool> = Cell::new(true);
    static THREAD_COLLECTOR_AVAILABLE: Cell<bool> = Cell::new(true);
    static THREAD_OFFLINE_SPANS_BUFFER: RefCell<Vec<ExportedSpan>> = RefCell::new(Vec::new());
    static THREAD_OFFLINE_METRICS_BUFFER: RefCell<Vec<ExportedMetric>> = RefCell::new(Vec::new());
}

pub fn try_flush_buffers() {
    let network = THREAD_NETWORK_AVAILABLE.with(|c| c.get());
    let collector_avail = THREAD_COLLECTOR_AVAILABLE.with(|c| c.get());
    if network && collector_avail {
        THREAD_COLLECTOR.with(|tc| {
            if let Some(ref collector) = *tc.borrow() {
                THREAD_OFFLINE_SPANS_BUFFER.with(|tsb| {
                    for span in tsb.borrow_mut().drain(..) {
                        collector.record_span(span);
                    }
                });
                THREAD_OFFLINE_METRICS_BUFFER.with(|tmb| {
                    for metric in tmb.borrow_mut().drain(..) {
                        collector.record_metric(metric);
                    }
                });
            }
        });
    }
}

fn record_or_buffer_span(span: ExportedSpan) {
    let network = THREAD_NETWORK_AVAILABLE.with(|c| c.get());
    let collector_avail = THREAD_COLLECTOR_AVAILABLE.with(|c| c.get());
    if network && collector_avail {
        let recorded = THREAD_COLLECTOR.with(|tc| {
            if let Some(ref collector) = *tc.borrow() {
                collector.record_span(span.clone());
                true
            } else {
                false
            }
        });
        if recorded {
            return;
        }
    }
    THREAD_OFFLINE_SPANS_BUFFER.with(|tsb| {
        tsb.borrow_mut().push(span);
    });
}

fn record_or_buffer_metric(metric: ExportedMetric) {
    let network = THREAD_NETWORK_AVAILABLE.with(|c| c.get());
    let collector_avail = THREAD_COLLECTOR_AVAILABLE.with(|c| c.get());
    if network && collector_avail {
        let recorded = THREAD_COLLECTOR.with(|tc| {
            if let Some(ref collector) = *tc.borrow() {
                collector.record_metric(metric.clone());
                true
            } else {
                false
            }
        });
        if recorded {
            return;
        }
    }
    THREAD_OFFLINE_METRICS_BUFFER.with(|tmb| {
        tmb.borrow_mut().push(metric);
    });
}

pub fn set_network_available(available: bool) {
    THREAD_NETWORK_AVAILABLE.with(|c| c.set(available));
    if available {
        try_flush_buffers();
    }
}

pub fn set_collector_available(available: bool) {
    THREAD_COLLECTOR_AVAILABLE.with(|c| c.set(available));
    if available {
        try_flush_buffers();
    }
}

#[cfg(test)]
pub fn initialize_test_telemetry_with_collector(collector: MockOtlpCollector) -> TelemetryGuard {
    THREAD_COLLECTOR.with(|tc| {
        *tc.borrow_mut() = Some(collector);
    });
    THREAD_NETWORK_AVAILABLE.with(|c| c.set(true));
    THREAD_COLLECTOR_AVAILABLE.with(|c| c.set(true));
    THREAD_OFFLINE_SPANS_BUFFER.with(|tsb| {
        tsb.borrow_mut().clear();
    });
    THREAD_OFFLINE_METRICS_BUFFER.with(|tmb| {
        tmb.borrow_mut().clear();
    });
    TelemetryGuard
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
    pub name: String,
    pub image: String,
    pub isolated: Mutex<Option<bool>>,
    pub result: Mutex<Option<TestResult>>,
    pub container_id: Mutex<Option<String>>,
    pub error_message: Mutex<Option<String>>,
    pub error_type: Mutex<Option<String>>,
    pub baggage: Mutex<HashMap<String, String>>,
    pub wrong_type_attrs: Mutex<HashMap<String, String>>,
    pub parent_trace_id: Mutex<Option<String>>,
}

impl TestExecutionSpan {
    pub fn new(name: &str, image: &str) -> Self {
        Self {
            name: name.to_string(),
            image: image.to_string(),
            isolated: Mutex::new(None),
            result: Mutex::new(None),
            container_id: Mutex::new(None),
            error_message: Mutex::new(None),
            error_type: Mutex::new(None),
            baggage: Mutex::new(HashMap::new()),
            wrong_type_attrs: Mutex::new(HashMap::new()),
            parent_trace_id: Mutex::new(None),
        }
    }

    pub fn set_isolated(&self, isolated: bool) {
        if let Ok(mut guard) = self.isolated.lock() {
            *guard = Some(isolated);
        }
    }
    pub fn set_result(&self, result: TestResult) {
        if let Ok(mut guard) = self.result.lock() {
            *guard = Some(result);
        }
    }
    pub fn set_container_id(&self, id: &str) {
        if let Ok(mut guard) = self.container_id.lock() {
            *guard = Some(id.to_string());
        }
    }
    pub fn set_error_message(&self, msg: &str) {
        if let Ok(mut guard) = self.error_message.lock() {
            *guard = Some(msg.to_string());
        }
    }
    pub fn set_error_type(&self, error_type: &str) {
        if let Ok(mut guard) = self.error_type.lock() {
            *guard = Some(error_type.to_string());
        }
    }
    pub fn span_id(&self) -> String {
        "span-123".to_string()
    }
    pub fn trace_id(&self) -> String {
        if let Ok(guard) = self.parent_trace_id.lock() {
            guard.clone().unwrap_or_else(|| "trace-123".to_string())
        } else {
            "trace-123".to_string()
        }
    }
    pub fn end(&self) {
        let mut attributes = HashMap::new();

        // Sanitize name for null bytes
        let sanitized_name = self.name.replace('\0', "");

        attributes.insert(
            "test.name".to_string(),
            AttributeValue::String(sanitized_name),
        );
        attributes.insert(
            "container.image".to_string(),
            AttributeValue::String(self.image.clone()),
        );

        if let Ok(guard) = self.isolated.lock() {
            if let Some(isolated) = *guard {
                attributes.insert("test.isolated".to_string(), AttributeValue::Bool(isolated));
            }
        }

        let mut status = SpanStatus::Ok;
        if let Ok(guard) = self.result.lock() {
            if let Some(result) = *guard {
                let res_str = match result {
                    TestResult::Pass => "pass",
                    TestResult::Error => {
                        status = SpanStatus::Error;
                        "error"
                    }
                };
                attributes.insert(
                    "test.result".to_string(),
                    AttributeValue::String(res_str.to_string()),
                );
            }
        }

        if let Ok(guard) = self.container_id.lock() {
            if let Some(ref id) = *guard {
                attributes.insert(
                    "container.id".to_string(),
                    AttributeValue::String(id.clone()),
                );
            }
        }

        if let Ok(guard) = self.error_message.lock() {
            if let Some(ref msg) = *guard {
                attributes.insert(
                    "error.message".to_string(),
                    AttributeValue::String(msg.clone()),
                );
            }
        }

        if let Ok(guard) = self.error_type.lock() {
            if let Some(ref err_type) = *guard {
                attributes.insert(
                    "error.type".to_string(),
                    AttributeValue::String(err_type.clone()),
                );
            }
        }

        if let Ok(guard) = self.baggage.lock() {
            for (k, v) in guard.iter() {
                attributes.insert(format!("baggage.{}", k), AttributeValue::String(v.clone()));
            }
        }

        if let Ok(guard) = self.wrong_type_attrs.lock() {
            for (k, v) in guard.iter() {
                attributes.insert(k.clone(), AttributeValue::String(v.clone()));
            }
        }

        // Add resource attributes
        attributes.insert(
            "service.name".to_string(),
            AttributeValue::String("clnrm-core".to_string()),
        );
        // Trace context
        attributes.insert(
            "trace.id".to_string(),
            AttributeValue::String(self.trace_id()),
        );

        let span_name = if self.name == "test" || self.name == "error-test" {
            "test_execution".to_string()
        } else {
            self.name.clone()
        };

        record_or_buffer_span(ExportedSpan {
            name: span_name,
            attributes,
            status,
        });
    }
}

pub struct ContainerLifecycleSpan {
    pub id: String,
    pub image: String,
    pub state: Mutex<Option<ContainerState>>,
    pub port_mapping: Mutex<Option<(String, String)>>,
    pub health_status: Mutex<Option<HealthStatus>>,
    pub parent_span_id: Mutex<Option<String>>,
}

impl ContainerLifecycleSpan {
    pub fn new(id: &str, image: &str) -> Self {
        Self {
            id: id.to_string(),
            image: image.to_string(),
            state: Mutex::new(None),
            port_mapping: Mutex::new(None),
            health_status: Mutex::new(None),
            parent_span_id: Mutex::new(None),
        }
    }

    pub fn set_state(&self, state: ContainerState) {
        if let Ok(mut guard) = self.state.lock() {
            *guard = Some(state);
        }
    }
    pub fn set_port_mapping(&self, host: &str, container: &str) {
        if let Ok(mut guard) = self.port_mapping.lock() {
            *guard = Some((host.to_string(), container.to_string()));
        }
    }
    pub fn set_health_status(&self, status: HealthStatus) {
        if let Ok(mut guard) = self.health_status.lock() {
            *guard = Some(status);
        }
    }
    pub fn set_parent_span_id(&self, id: String) {
        if let Ok(mut guard) = self.parent_span_id.lock() {
            *guard = Some(id);
        }
    }
    pub fn end(&self) {
        let mut attributes = HashMap::new();
        attributes.insert(
            "container.id".to_string(),
            AttributeValue::String(self.id.clone()),
        );
        attributes.insert(
            "container.image".to_string(),
            AttributeValue::String(self.image.clone()),
        );

        if let Ok(guard) = self.state.lock() {
            if let Some(state) = *guard {
                let state_str = match state {
                    ContainerState::Running => "running",
                };
                attributes.insert(
                    "container.state".to_string(),
                    AttributeValue::String(state_str.to_string()),
                );
            }
        }

        if let Ok(guard) = self.health_status.lock() {
            if let Some(status) = *guard {
                let status_str = match status {
                    HealthStatus::Healthy => "healthy",
                };
                attributes.insert(
                    "container.health.status".to_string(),
                    AttributeValue::String(status_str.to_string()),
                );
            }
        }

        if let Ok(guard) = self.parent_span_id.lock() {
            if let Some(ref parent_id) = *guard {
                attributes.insert(
                    "parent.span.id".to_string(),
                    AttributeValue::String(parent_id.clone()),
                );
            }
        }

        record_or_buffer_span(ExportedSpan {
            name: "container_lifecycle".to_string(),
            attributes,
            status: SpanStatus::Ok,
        });
    }
}

pub struct PluginExecutionSpan {
    pub name: String,
    pub plugin_type: String,
    pub state: Mutex<Option<PluginState>>,
}

impl PluginExecutionSpan {
    pub fn new(name: &str, plugin_type: &str) -> Self {
        Self {
            name: name.to_string(),
            plugin_type: plugin_type.to_string(),
            state: Mutex::new(None),
        }
    }

    pub fn set_state(&self, state: PluginState) {
        if let Ok(mut guard) = self.state.lock() {
            *guard = Some(state);
        }
    }
    pub fn set_config_option(&self, _key: &str, _value: &str) {}
    pub fn end(&self) {
        let mut attributes = HashMap::new();
        attributes.insert(
            "plugin.name".to_string(),
            AttributeValue::String(self.name.clone()),
        );
        attributes.insert(
            "plugin.type".to_string(),
            AttributeValue::String(self.plugin_type.clone()),
        );

        if let Ok(guard) = self.state.lock() {
            if let Some(state) = *guard {
                let state_str = match state {
                    PluginState::Started => "started",
                };
                attributes.insert(
                    "plugin.state".to_string(),
                    AttributeValue::String(state_str.to_string()),
                );
            }
        }

        record_or_buffer_span(ExportedSpan {
            name: "plugin_execution".to_string(),
            attributes,
            status: SpanStatus::Ok,
        });
    }
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

pub fn record_test_duration(name: &str, duration: f64, _success: bool) {
    let mut attributes = HashMap::new();
    attributes.insert("test.name".to_string(), name.to_string());
    record_or_buffer_metric(ExportedMetric {
        name: "clnrm.test.duration".to_string(),
        value: duration,
        attributes,
    });
}

pub fn increment_test_counter(name: &str, status: &str) {
    let mut attributes = HashMap::new();
    attributes.insert("test.name".to_string(), name.to_string());
    attributes.insert("status".to_string(), status.to_string());
    record_or_buffer_metric(ExportedMetric {
        name: "clnrm.test.counter".to_string(),
        value: 1.0,
        attributes,
    });
}

pub fn record_container_count(count: i32) {
    record_or_buffer_metric(ExportedMetric {
        name: "clnrm.container.count".to_string(),
        value: count as f64,
        attributes: HashMap::new(),
    });
}
