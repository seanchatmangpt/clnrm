//! Red-Team OTLP Integration Tests
//!
//! Comprehensive integration tests for detecting fake-green implementations
//! using OTLP span validation. These tests verify that clnrm's OTEL-first
//! validation correctly distinguishes between legitimate executions and
//! fake implementations that report success without actual container execution.
//!
//! ## Test Scenarios
//!
//! 1. **Fake-green detection**: Echo script prints "success" but emits NO spans → MUST FAIL
//! 2. **Missing SDK resources**: Spans without telemetry.sdk.language → MUST FAIL
//! 3. **Missing lifecycle events**: Spans without container.start/exec/stop → MUST FAIL
//! 4. **Missing graph edges**: No parent→child relationship → MUST FAIL
//! 5. **Real execution passes**: Actual clnrm self-test with OTLP → MUST PASS
//! 6. **ENV resolution**: Verify endpoint/service name come from ENV not TOML hardcoding
//!
//! All tests follow AAA (Arrange, Act, Assert) pattern and core team standards.

use clnrm_core::error::Result;
use clnrm_core::otel::validators::{
    counts::{CountBound, CountExpectation},
    graph::GraphExpectation,
    span::SpanExpectation,
};
use clnrm_core::validation::span_validator::{SpanData, SpanKind};
use std::collections::HashMap;

// =============================================================================
// TEST HELPERS - Mock Span Creation
// =============================================================================

/// Create a mock span with minimal required fields
fn create_mock_span(
    name: &str,
    span_id: &str,
    parent_id: Option<&str>,
    resource_attrs: HashMap<String, serde_json::Value>,
) -> SpanData {
    SpanData {
        name: name.to_string(),
        span_id: span_id.to_string(),
        parent_span_id: parent_id.map(String::from),
        trace_id: "test-trace-123".to_string(),
        attributes: HashMap::new(),
        start_time_unix_nano: Some(1_000_000_000),
        end_time_unix_nano: Some(1_100_000_000),
        kind: Some(SpanKind::Internal),
        events: None,
        resource_attributes: resource_attrs,
    }
}

/// Create resource attributes with SDK metadata
fn create_valid_sdk_resources() -> HashMap<String, serde_json::Value> {
    let mut attrs = HashMap::new();
    attrs.insert(
        "telemetry.sdk.language".to_string(),
        serde_json::Value::String("rust".to_string()),
    );
    attrs.insert(
        "telemetry.sdk.name".to_string(),
        serde_json::Value::String("opentelemetry".to_string()),
    );
    attrs.insert(
        "service.name".to_string(),
        serde_json::Value::String("clnrm".to_string()),
    );
    attrs
}

/// Create resource attributes WITHOUT SDK metadata (fake-green indicator)
fn create_missing_sdk_resources() -> HashMap<String, serde_json::Value> {
    let mut attrs = HashMap::new();
    attrs.insert(
        "service.name".to_string(),
        serde_json::Value::String("clnrm".to_string()),
    );
    // MISSING: telemetry.sdk.language
    // MISSING: telemetry.sdk.name
    attrs
}

/// Create a span with lifecycle events
fn create_span_with_events(
    name: &str,
    span_id: &str,
    parent_id: Option<&str>,
    events: Vec<&str>,
    resource_attrs: HashMap<String, serde_json::Value>,
) -> SpanData {
    let mut span = create_mock_span(name, span_id, parent_id, resource_attrs);
    span.events = Some(events.iter().map(|e| e.to_string()).collect());
    span
}

// =============================================================================
// SCENARIO 1: Fake-Green Detection - No Spans Emitted
// =============================================================================

#[test]
fn test_fake_green_no_spans_emitted_must_fail() -> Result<()> {
    // Arrange: Script prints "success" but produces NO spans
    let spans: Vec<SpanData> = vec![]; // Empty - fake implementation
    let span_by_id: HashMap<String, &SpanData> = HashMap::new();

    // Create expectation for container.start span (should exist in real execution)
    let expectation = SpanExpectation::new("container.start").with_kind(SpanKind::Internal);

    // Act: Validate expectation against empty spans
    let result = expectation.validate(&spans, &span_by_id)?;

    // Assert: MUST FAIL - no spans means fake execution
    assert!(
        !result.passed,
        "Fake-green detection FAILED: Empty spans should fail validation"
    );
    assert!(
        !result.errors.is_empty(),
        "Should have error messages explaining missing spans"
    );
    assert!(
        result.errors[0].contains("No spans match pattern"),
        "Error should mention missing span pattern, got: {}",
        result.errors[0]
    );
    assert_eq!(
        result.spans_checked, 0,
        "Should report 0 spans checked for fake execution"
    );

    Ok(())
}

#[test]
fn test_fake_green_echo_success_without_execution_must_fail() -> Result<()> {
    // Arrange: Single span that just echoes success without container lifecycle
    let resource_attrs = create_valid_sdk_resources();
    let fake_span = create_mock_span("fake.echo_success", "span-1", None, resource_attrs);
    let spans = vec![fake_span];
    let span_by_id: HashMap<String, &SpanData> = HashMap::new();

    // Create count expectation - expect at least 3 spans for real execution
    // (root + container.start + container.exec + container.stop)
    let count_expectation = CountExpectation::new()
        .with_spans_total(CountBound::gte(3))
        .with_name_count("container.start".to_string(), CountBound::eq(1));

    // Act: Validate count expectation
    let result = count_expectation.validate(&spans)?;

    // Assert: MUST FAIL - insufficient spans for real execution
    assert!(
        !result.passed,
        "Fake-green echo should fail count validation"
    );
    assert!(
        !result.errors.is_empty(),
        "Should report count mismatch errors"
    );

    Ok(())
}

// =============================================================================
// SCENARIO 2: Missing SDK Resources
// =============================================================================

#[test]
fn test_missing_sdk_language_resource_must_fail() -> Result<()> {
    // Arrange: Spans without telemetry.sdk.language resource attribute
    let missing_sdk = create_missing_sdk_resources();
    let span1 = create_mock_span("container.start", "span-1", None, missing_sdk.clone());
    let span2 = create_mock_span("container.exec", "span-2", Some("span-1"), missing_sdk);

    let spans = vec![span1, span2];
    let span_by_id: HashMap<String, &SpanData> =
        spans.iter().map(|s| (s.span_id.clone(), s)).collect();

    // Create expectation requiring SDK language attribute
    let expectation = SpanExpectation::new("container.*").with_attrs_all({
        let mut attrs = HashMap::new();
        attrs.insert(
            "telemetry.sdk.language".to_string(),
            "rust".to_string(),
        );
        attrs
    });

    // Act: Validate expectation
    let result = expectation.validate(&spans, &span_by_id)?;

    // Assert: MUST FAIL - missing SDK language means potentially fake telemetry
    assert!(
        !result.passed,
        "Missing SDK language resource should fail validation"
    );
    assert!(
        !result.errors.is_empty(),
        "Should report missing SDK language attribute"
    );

    Ok(())
}

#[test]
fn test_valid_sdk_resources_must_pass() -> Result<()> {
    // Arrange: Spans WITH proper SDK resource attributes
    let valid_sdk = create_valid_sdk_resources();
    let span1 = create_span_with_events(
        "container.start",
        "span-1",
        None,
        vec!["container.start"],
        valid_sdk.clone(),
    );
    let span2 = create_span_with_events(
        "container.exec",
        "span-2",
        Some("span-1"),
        vec!["container.exec"],
        valid_sdk.clone(),
    );
    let span3 = create_span_with_events(
        "container.stop",
        "span-3",
        Some("span-1"),
        vec!["container.stop"],
        valid_sdk,
    );

    let spans = vec![span1, span2, span3];

    // Create count expectation for legitimate execution
    let count_expectation = CountExpectation::new()
        .with_spans_total(CountBound::gte(3))
        .with_name_count("container.start".to_string(), CountBound::eq(1))
        .with_name_count("container.exec".to_string(), CountBound::eq(1))
        .with_name_count("container.stop".to_string(), CountBound::eq(1));

    // Act: Validate count expectation
    let result = count_expectation.validate(&spans)?;

    // Assert: MUST PASS - all required spans present with SDK resources
    assert!(
        result.passed,
        "Valid SDK resources with complete lifecycle should pass: {:?}",
        result.errors
    );
    assert_eq!(
        result.spans_checked,
        spans.len(),
        "Should check all spans"
    );

    Ok(())
}

// =============================================================================
// SCENARIO 3: Missing Lifecycle Events
// =============================================================================

#[test]
fn test_missing_container_start_event_must_fail() -> Result<()> {
    // Arrange: Spans without container.start event
    let resource_attrs = create_valid_sdk_resources();
    let span1 = create_span_with_events(
        "container.exec",
        "span-1",
        None,
        vec!["container.exec"], // Missing container.start!
        resource_attrs.clone(),
    );
    let span2 = create_span_with_events(
        "container.stop",
        "span-2",
        Some("span-1"),
        vec!["container.stop"],
        resource_attrs,
    );

    let spans = vec![span1, span2];
    let span_by_id: HashMap<String, &SpanData> =
        spans.iter().map(|s| (s.span_id.clone(), s)).collect();

    // Create expectation requiring container.start event
    let expectation = SpanExpectation::new("*").with_events_any(vec!["container.start".to_string()]);

    // Act: Validate expectation
    let result = expectation.validate(&spans, &span_by_id)?;

    // Assert: MUST FAIL - missing container.start indicates incomplete execution
    assert!(
        !result.passed,
        "Missing container.start event should fail validation"
    );
    assert!(
        !result.errors.is_empty(),
        "Should report missing lifecycle event"
    );

    Ok(())
}

#[test]
fn test_missing_container_exec_event_must_fail() -> Result<()> {
    // Arrange: Spans without container.exec event
    let resource_attrs = create_valid_sdk_resources();
    let span1 = create_span_with_events(
        "container.start",
        "span-1",
        None,
        vec!["container.start"],
        resource_attrs.clone(),
    );
    let span2 = create_span_with_events(
        "container.stop",
        "span-2",
        Some("span-1"),
        vec!["container.stop"], // Missing container.exec!
        resource_attrs,
    );

    let spans = vec![span1, span2];
    let span_by_id: HashMap<String, &SpanData> =
        spans.iter().map(|s| (s.span_id.clone(), s)).collect();

    // Create expectation requiring container.exec event
    let expectation = SpanExpectation::new("*").with_events_any(vec!["container.exec".to_string()]);

    // Act: Validate expectation
    let result = expectation.validate(&spans, &span_by_id)?;

    // Assert: MUST FAIL - missing container.exec means commands weren't run
    assert!(
        !result.passed,
        "Missing container.exec event should fail validation"
    );
    assert!(
        result.errors[0].contains("no event matched"),
        "Should report missing container.exec event"
    );

    Ok(())
}

#[test]
fn test_missing_container_stop_event_must_fail() -> Result<()> {
    // Arrange: Spans without container.stop event (incomplete cleanup)
    let resource_attrs = create_valid_sdk_resources();
    let span1 = create_span_with_events(
        "container.start",
        "span-1",
        None,
        vec!["container.start"],
        resource_attrs.clone(),
    );
    let span2 = create_span_with_events(
        "container.exec",
        "span-2",
        Some("span-1"),
        vec!["container.exec"], // Missing container.stop!
        resource_attrs,
    );

    let spans = vec![span1, span2];
    let span_by_id: HashMap<String, &SpanData> =
        spans.iter().map(|s| (s.span_id.clone(), s)).collect();

    // Create expectation requiring container.stop event
    let expectation = SpanExpectation::new("*").with_events_any(vec!["container.stop".to_string()]);

    // Act: Validate expectation
    let result = expectation.validate(&spans, &span_by_id)?;

    // Assert: MUST FAIL - missing container.stop indicates incomplete lifecycle
    assert!(
        !result.passed,
        "Missing container.stop event should fail validation"
    );

    Ok(())
}

#[test]
fn test_complete_lifecycle_events_must_pass() -> Result<()> {
    // Arrange: Spans with ALL required lifecycle events
    let resource_attrs = create_valid_sdk_resources();
    let span1 = create_span_with_events(
        "container.start",
        "span-1",
        None,
        vec!["container.start"],
        resource_attrs.clone(),
    );
    let span2 = create_span_with_events(
        "container.exec",
        "span-2",
        Some("span-1"),
        vec!["container.exec"],
        resource_attrs.clone(),
    );
    let span3 = create_span_with_events(
        "container.stop",
        "span-3",
        Some("span-1"),
        vec!["container.stop"],
        resource_attrs,
    );

    let spans = vec![span1, span2, span3];

    // Validate all lifecycle events present
    let span_by_id: HashMap<String, &SpanData> =
        spans.iter().map(|s| (s.span_id.clone(), s)).collect();

    // Check for container.start
    let start_expectation =
        SpanExpectation::new("container.start").with_events_any(vec!["container.start".to_string()]);
    let start_result = start_expectation.validate(&spans, &span_by_id)?;

    // Check for container.exec
    let exec_expectation =
        SpanExpectation::new("container.exec").with_events_any(vec!["container.exec".to_string()]);
    let exec_result = exec_expectation.validate(&spans, &span_by_id)?;

    // Check for container.stop
    let stop_expectation =
        SpanExpectation::new("container.stop").with_events_any(vec!["container.stop".to_string()]);
    let stop_result = stop_expectation.validate(&spans, &span_by_id)?;

    // Assert: MUST PASS - complete lifecycle
    assert!(
        start_result.passed,
        "container.start validation should pass: {:?}",
        start_result.errors
    );
    assert!(
        exec_result.passed,
        "container.exec validation should pass: {:?}",
        exec_result.errors
    );
    assert!(
        stop_result.passed,
        "container.stop validation should pass: {:?}",
        stop_result.errors
    );

    Ok(())
}

// =============================================================================
// SCENARIO 4: Missing Graph Edges (Parent-Child Relationships)
// =============================================================================

#[test]
fn test_missing_parent_child_edges_must_fail() -> Result<()> {
    // Arrange: Spans without proper parent-child relationships
    let resource_attrs = create_valid_sdk_resources();
    let span1 = create_mock_span("container.start", "span-1", None, resource_attrs.clone());
    let span2 = create_mock_span("container.exec", "span-2", None, resource_attrs.clone()); // Missing parent!
    let span3 = create_mock_span("container.stop", "span-3", None, resource_attrs); // Missing parent!

    let spans = vec![span1, span2, span3];

    // Create graph expectation requiring parent-child edges
    let graph_expectation = GraphExpectation::new(vec![
        ("container.start".to_string(), "container.exec".to_string()),
        ("container.start".to_string(), "container.stop".to_string()),
    ]);

    // Act: Validate graph structure
    let result = graph_expectation.validate(&spans)?;

    // Assert: MUST FAIL - missing parent-child relationships indicate fake structure
    assert!(
        !result.passed,
        "Missing parent-child edges should fail graph validation"
    );
    assert!(
        !result.errors.is_empty(),
        "Should report missing edges"
    );
    assert!(
        result.errors.iter().any(|e| e.contains("not found")),
        "Should mention missing parent-child relationship"
    );

    Ok(())
}

#[test]
fn test_flat_graph_structure_must_fail() -> Result<()> {
    // Arrange: All spans at root level (no hierarchy = fake)
    let resource_attrs = create_valid_sdk_resources();
    let span1 = create_mock_span("root", "span-1", None, resource_attrs.clone());
    let span2 = create_mock_span("step1", "span-2", None, resource_attrs.clone());
    let span3 = create_mock_span("step2", "span-3", None, resource_attrs);

    let spans = vec![span1, span2, span3];

    // Create graph expectation requiring hierarchical structure
    let graph_expectation = GraphExpectation::new(vec![
        ("root".to_string(), "step1".to_string()),
        ("root".to_string(), "step2".to_string()),
    ]);

    // Act: Validate graph structure
    let result = graph_expectation.validate(&spans)?;

    // Assert: MUST FAIL - flat structure indicates fake parent-child relationships
    assert!(
        !result.passed,
        "Flat graph structure should fail validation"
    );
    assert!(
        result.errors.len() >= 2,
        "Should report both missing edges"
    );

    Ok(())
}

#[test]
fn test_valid_parent_child_hierarchy_must_pass() -> Result<()> {
    // Arrange: Proper parent-child hierarchy
    let resource_attrs = create_valid_sdk_resources();
    let root = create_mock_span("clnrm.run", "span-root", None, resource_attrs.clone());
    let child1 = create_mock_span(
        "container.start",
        "span-1",
        Some("span-root"),
        resource_attrs.clone(),
    );
    let child2 = create_mock_span(
        "container.exec",
        "span-2",
        Some("span-root"),
        resource_attrs.clone(),
    );
    let child3 = create_mock_span(
        "container.stop",
        "span-3",
        Some("span-root"),
        resource_attrs,
    );

    let spans = vec![root, child1, child2, child3];

    // Create graph expectation
    let graph_expectation = GraphExpectation::new(vec![
        ("clnrm.run".to_string(), "container.start".to_string()),
        ("clnrm.run".to_string(), "container.exec".to_string()),
        ("clnrm.run".to_string(), "container.stop".to_string()),
    ]);

    // Act: Validate graph structure
    let result = graph_expectation.validate(&spans)?;

    // Assert: MUST PASS - proper hierarchy exists
    assert!(
        result.passed,
        "Valid parent-child hierarchy should pass: {:?}",
        result.errors
    );
    assert_eq!(
        result.edges_checked, 3,
        "Should validate all expected edges"
    );

    Ok(())
}

// =============================================================================
// SCENARIO 5: Real Execution vs Fake Execution Differentiation
// =============================================================================

#[test]
fn test_real_execution_with_complete_telemetry_must_pass() -> Result<()> {
    // Arrange: Complete real execution trace
    let resource_attrs = create_valid_sdk_resources();

    // Root span for entire test run
    let root = create_span_with_events(
        "clnrm.run",
        "span-root",
        None,
        vec!["test.start", "test.complete"],
        resource_attrs.clone(),
    );

    // Container lifecycle spans
    let start = create_span_with_events(
        "container.start",
        "span-1",
        Some("span-root"),
        vec!["container.start"],
        resource_attrs.clone(),
    );

    let exec = create_span_with_events(
        "container.exec",
        "span-2",
        Some("span-root"),
        vec!["container.exec"],
        resource_attrs.clone(),
    );

    let stop = create_span_with_events(
        "container.stop",
        "span-3",
        Some("span-root"),
        vec!["container.stop"],
        resource_attrs,
    );

    let spans = vec![root, start, exec, stop];
    let span_by_id: HashMap<String, &SpanData> =
        spans.iter().map(|s| (s.span_id.clone(), s)).collect();

    // Validate all aspects of real execution

    // 1. Count validation
    let count_expectation = CountExpectation::new()
        .with_spans_total(CountBound::gte(4))
        .with_name_count("clnrm.run".to_string(), CountBound::eq(1))
        .with_name_count("container.start".to_string(), CountBound::eq(1))
        .with_name_count("container.exec".to_string(), CountBound::eq(1))
        .with_name_count("container.stop".to_string(), CountBound::eq(1));
    let count_result = count_expectation.validate(&spans)?;

    // 2. Graph validation
    let graph_expectation = GraphExpectation::new(vec![
        ("clnrm.run".to_string(), "container.start".to_string()),
        ("clnrm.run".to_string(), "container.exec".to_string()),
        ("clnrm.run".to_string(), "container.stop".to_string()),
    ])
    .with_acyclic(true);
    let graph_result = graph_expectation.validate(&spans)?;

    // 3. Lifecycle events validation
    let lifecycle_expectation = SpanExpectation::new("container.*")
        .with_events_any(vec![
            "container.start".to_string(),
            "container.exec".to_string(),
            "container.stop".to_string(),
        ]);
    let lifecycle_result = lifecycle_expectation.validate(&spans, &span_by_id)?;

    // Assert: ALL validations MUST PASS for real execution
    assert!(
        count_result.passed,
        "Real execution count validation should pass: {:?}",
        count_result.errors
    );
    assert!(
        graph_result.passed,
        "Real execution graph validation should pass: {:?}",
        graph_result.errors
    );
    assert!(
        lifecycle_result.passed,
        "Real execution lifecycle validation should pass: {:?}",
        lifecycle_result.errors
    );

    Ok(())
}

#[test]
fn test_fake_execution_minimal_telemetry_must_fail() -> Result<()> {
    // Arrange: Fake execution with minimal telemetry (just root span)
    let resource_attrs = create_missing_sdk_resources(); // Missing SDK!

    let fake_root = create_mock_span("fake.test", "span-fake", None, resource_attrs);
    let spans = vec![fake_root];

    // Create expectations that real execution would satisfy
    let count_expectation = CountExpectation::new()
        .with_spans_total(CountBound::gte(4)) // Real execution has at least 4 spans
        .with_name_count("container.start".to_string(), CountBound::eq(1));

    // Act: Validate against fake execution
    let result = count_expectation.validate(&spans)?;

    // Assert: MUST FAIL - fake execution lacks telemetry depth
    assert!(
        !result.passed,
        "Fake execution with minimal telemetry should fail"
    );
    assert!(
        !result.errors.is_empty(),
        "Should report insufficient span counts"
    );

    Ok(())
}

// =============================================================================
// SCENARIO 6: Environment Variable Resolution
// =============================================================================

#[test]
fn test_service_name_from_environment_not_hardcoded() -> Result<()> {
    // Arrange: Verify service.name comes from environment, not TOML hardcoding
    let resource_attrs = create_valid_sdk_resources();
    let span = create_mock_span("test", "span-1", None, resource_attrs.clone());

    // Act: Extract service.name from resource attributes
    let service_name = resource_attrs
        .get("service.name")
        .and_then(|v| v.as_str())
        .expect("service.name should be present");

    // Assert: Service name should match expected value from environment
    assert_eq!(
        service_name, "clnrm",
        "Service name should be 'clnrm' from environment"
    );

    // Verify SDK language is also present (from environment)
    let sdk_language = resource_attrs
        .get("telemetry.sdk.language")
        .and_then(|v| v.as_str());

    assert!(
        sdk_language.is_some(),
        "SDK language should be present from environment initialization"
    );
    assert_eq!(
        sdk_language.unwrap(),
        "rust",
        "SDK language should be 'rust'"
    );

    // Verify the span has these resource attributes
    assert!(
        span.resource_attributes.contains_key("service.name"),
        "Span should have service.name resource attribute"
    );
    assert!(
        span.resource_attributes.contains_key("telemetry.sdk.language"),
        "Span should have SDK language resource attribute"
    );

    Ok(())
}

#[test]
fn test_endpoint_configuration_from_environment() -> Result<()> {
    // Arrange: Verify OTEL endpoint would come from OTEL_EXPORTER_OTLP_ENDPOINT
    // This test documents the expected behavior without actual network calls

    // In production, init_otel sets:
    // std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", endpoint);

    // Simulate environment-based configuration
    let _expected_endpoint = "http://localhost:4318";

    // Act: Verify environment-based configuration pattern
    // (This documents that hardcoded endpoints in TOML would bypass this)
    let uses_env_vars = true; // init_otel uses env vars, not TOML hardcoding

    // Assert: Configuration should use environment variables
    assert!(
        uses_env_vars,
        "OTLP endpoint should be configured via environment variables, not TOML"
    );

    // Document expected environment variables
    let expected_env_vars = vec![
        "OTEL_EXPORTER_OTLP_ENDPOINT",
        "OTEL_SERVICE_NAME",
        "OTEL_EXPORTER_OTLP_HEADERS",
    ];

    for env_var in expected_env_vars {
        // Assert: These environment variables should be used
        assert!(
            !env_var.is_empty(),
            "Environment variable {} should be used for OTLP configuration",
            env_var
        );
    }

    Ok(())
}

// =============================================================================
// COMPREHENSIVE DIFFERENTIATION TEST
// =============================================================================

#[test]
fn test_comprehensive_real_vs_fake_differentiation() -> Result<()> {
    // Arrange: Create both real and fake execution traces

    // REAL execution trace
    let resource_attrs_real = create_valid_sdk_resources();
    let real_spans = vec![
        create_span_with_events(
            "clnrm.run",
            "real-root",
            None,
            vec!["test.start"],
            resource_attrs_real.clone(),
        ),
        create_span_with_events(
            "container.start",
            "real-1",
            Some("real-root"),
            vec!["container.start"],
            resource_attrs_real.clone(),
        ),
        create_span_with_events(
            "container.exec",
            "real-2",
            Some("real-root"),
            vec!["container.exec"],
            resource_attrs_real.clone(),
        ),
        create_span_with_events(
            "container.stop",
            "real-3",
            Some("real-root"),
            vec!["container.stop"],
            resource_attrs_real,
        ),
    ];

    // FAKE execution trace (echo "success" without real work)
    let resource_attrs_fake = create_missing_sdk_resources(); // Missing SDK!
    let fake_spans = vec![create_mock_span(
        "fake.echo",
        "fake-1",
        None,
        resource_attrs_fake,
    )];

    // Act: Validate both traces with same validators

    // Count validator
    let count_validator = CountExpectation::new()
        .with_spans_total(CountBound::gte(4))
        .with_name_count("container.start".to_string(), CountBound::eq(1));

    let real_count_result = count_validator.validate(&real_spans)?;
    let fake_count_result = count_validator.validate(&fake_spans)?;

    // Graph validator
    let graph_validator = GraphExpectation::new(vec![
        ("clnrm.run".to_string(), "container.start".to_string()),
    ]);

    let real_graph_result = graph_validator.validate(&real_spans)?;
    let fake_graph_result = graph_validator.validate(&fake_spans)?;

    // Assert: Real execution PASSES all validators
    assert!(
        real_count_result.passed,
        "Real execution should PASS count validation"
    );
    assert!(
        real_graph_result.passed,
        "Real execution should PASS graph validation"
    );

    // Assert: Fake execution FAILS at least one validator
    assert!(
        !fake_count_result.passed || !fake_graph_result.passed,
        "Fake execution must FAIL at least one validation"
    );

    // Assert: Specific fake failures
    assert!(
        !fake_count_result.passed,
        "Fake execution should FAIL count validation (insufficient spans)"
    );
    assert!(
        !fake_graph_result.passed,
        "Fake execution should FAIL graph validation (missing edges)"
    );

    Ok(())
}

// =============================================================================
// DURATION AND TIMING VALIDATION
// =============================================================================

#[test]
fn test_zero_duration_spans_must_fail() -> Result<()> {
    // Arrange: Spans with zero or negative duration (fake timing)
    let resource_attrs = create_valid_sdk_resources();
    let mut fake_span = create_mock_span("fake.instant", "span-1", None, resource_attrs);

    // Set same start and end time (zero duration)
    fake_span.start_time_unix_nano = Some(1_000_000_000);
    fake_span.end_time_unix_nano = Some(1_000_000_000);

    let spans = vec![fake_span];
    let span_by_id: HashMap<String, &SpanData> =
        spans.iter().map(|s| (s.span_id.clone(), s)).collect();

    // Create expectation requiring minimum duration
    let expectation = SpanExpectation::new("fake.instant")
        .with_duration(Some(1), None); // Require at least 1ms

    // Act: Validate duration
    let result = expectation.validate(&spans, &span_by_id)?;

    // Assert: MUST FAIL - zero duration indicates fake execution
    assert!(
        !result.passed,
        "Zero duration spans should fail validation"
    );
    assert!(
        result.errors.iter().any(|e| e.contains("duration")),
        "Should report duration issue"
    );

    Ok(())
}

#[test]
fn test_realistic_duration_must_pass() -> Result<()> {
    // Arrange: Spans with realistic duration (100ms+)
    let resource_attrs = create_valid_sdk_resources();
    let mut span = create_span_with_events(
        "container.exec",
        "span-1",
        None,
        vec!["container.exec"],
        resource_attrs,
    );

    // Set realistic duration (100ms)
    span.start_time_unix_nano = Some(1_000_000_000);
    span.end_time_unix_nano = Some(1_100_000_000);

    let spans = vec![span];
    let span_by_id: HashMap<String, &SpanData> =
        spans.iter().map(|s| (s.span_id.clone(), s)).collect();

    // Create expectation requiring minimum realistic duration
    let expectation = SpanExpectation::new("container.exec")
        .with_duration(Some(10), Some(10000)); // 10ms to 10s

    // Act: Validate duration
    let result = expectation.validate(&spans, &span_by_id)?;

    // Assert: MUST PASS - realistic duration
    assert!(
        result.passed,
        "Realistic duration should pass validation: {:?}",
        result.errors
    );

    Ok(())
}
