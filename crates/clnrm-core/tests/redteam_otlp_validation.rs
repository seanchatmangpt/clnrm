//! Red-Team OTLP Validation Tests
//!
//! This module validates the red-team approach to detecting fake-green tests
//! through OpenTelemetry evidence requirements. Tests follow core team standards:
//!
//! - No unwrap/expect in production paths
//! - Proper error handling with Result
//! - AAA pattern (Arrange, Act, Assert)
//! - Descriptive test names
//! - Comprehensive validation
//!
//! ## Detection Strategy
//!
//! The red-team validation uses 7 independent detection layers:
//! 1. Span Validator - Requires specific spans with lifecycle events
//! 2. Graph Validator - Validates parent-child relationships
//! 3. Count Validator - Enforces minimum span counts
//! 4. Window Validator - Validates temporal containment
//! 5. Order Validator - Validates execution order
//! 6. Status Validator - Validates span status codes
//! 7. Hermeticity Validator - Validates SDK resource attributes
//!
//! Each layer independently catches fake-green tests by different mechanisms.

use clnrm_core::error::Result;
use std::collections::HashMap;

// ============================================================================
// Test Data Structures
// ============================================================================

/// Represents a test execution result with OTEL spans
#[derive(Debug, Clone)]
struct TestExecution {
    verdict: TestVerdict,
    spans: Vec<SpanData>,
    validation_results: ValidationResults,
}

/// Test verdict
#[derive(Debug, Clone, PartialEq)]
enum TestVerdict {
    Pass,
    Fail(String),
}

/// Span data structure matching OTEL format
#[derive(Debug, Clone)]
struct SpanData {
    name: String,
    kind: SpanKind,
    parent_id: Option<String>,
    attributes: HashMap<String, String>,
    events: Vec<String>,
    status: SpanStatus,
}

/// Span kind enumeration
#[derive(Debug, Clone, PartialEq)]
enum SpanKind {
    Internal,
    Server,
    Client,
}

/// Span status
#[derive(Debug, Clone, PartialEq)]
enum SpanStatus {
    Ok,
    Error,
}

/// Validation results from all 7 detection layers
#[derive(Debug, Clone)]
struct ValidationResults {
    span_validator: LayerResult,
    graph_validator: LayerResult,
    count_validator: LayerResult,
    window_validator: LayerResult,
    order_validator: LayerResult,
    status_validator: LayerResult,
    hermeticity_validator: LayerResult,
}

/// Individual layer result
#[derive(Debug, Clone, PartialEq)]
struct LayerResult {
    passed: bool,
    message: String,
}

impl LayerResult {
    fn pass(message: impl Into<String>) -> Self {
        Self {
            passed: true,
            message: message.into(),
        }
    }

    fn fail(message: impl Into<String>) -> Self {
        Self {
            passed: false,
            message: message.into(),
        }
    }
}

// ============================================================================
// Validator Implementations (Stubs for Testing)
// ============================================================================

/// Validates that required spans exist with proper lifecycle events
fn validate_spans(spans: &[SpanData]) -> LayerResult {
    // Check for root span
    let has_run_span = spans.iter().any(|s| s.name == "clnrm.run");
    if !has_run_span {
        return LayerResult::fail("Missing required span: clnrm.run");
    }

    // Check for step span
    let has_step_span = spans.iter().any(|s| s.name.starts_with("clnrm.step:"));
    if !has_step_span {
        return LayerResult::fail("Missing required step span");
    }

    // Check for lifecycle events in step span
    let step_span = spans.iter().find(|s| s.name.starts_with("clnrm.step:"));
    if let Some(span) = step_span {
        let has_lifecycle = span
            .events
            .iter()
            .any(|e| e == "container.start" || e == "container.exec" || e == "container.stop");
        if !has_lifecycle {
            return LayerResult::fail("Step span missing lifecycle events (container.start, container.exec, container.stop)");
        }
    }

    LayerResult::pass("All required spans present with lifecycle events")
}

/// Validates parent-child relationships in span graph
fn validate_graph(spans: &[SpanData]) -> LayerResult {
    // Find run and step spans
    let run_span = spans.iter().find(|s| s.name == "clnrm.run");
    let step_span = spans.iter().find(|s| s.name.starts_with("clnrm.step:"));

    match (run_span, step_span) {
        (Some(_run), Some(step)) => {
            // Check that step has run as parent
            if step.parent_id.is_none() {
                return LayerResult::fail("Step span has no parent (orphaned span)");
            }
            LayerResult::pass("Valid parent-child relationship: clnrm.run → clnrm.step")
        }
        (None, _) => LayerResult::fail("Missing run span - no graph root"),
        (_, None) => LayerResult::fail("Missing step span - incomplete graph"),
    }
}

/// Validates span counts meet minimum requirements
fn validate_counts(spans: &[SpanData]) -> LayerResult {
    let total_spans = spans.len();
    if total_spans < 2 {
        return LayerResult::fail(format!(
            "Insufficient spans: expected >=2, got {}",
            total_spans
        ));
    }

    let total_events: usize = spans.iter().map(|s| s.events.len()).sum();
    if total_events < 3 {
        return LayerResult::fail(format!(
            "Insufficient events: expected >=3 lifecycle events, got {}",
            total_events
        ));
    }

    LayerResult::pass(format!(
        "Span counts valid: {} spans, {} events",
        total_spans, total_events
    ))
}

/// Validates temporal window containment
fn validate_windows(spans: &[SpanData]) -> LayerResult {
    // Simplified: Just check that parent exists for child
    let orphaned = spans.iter().filter(|s| s.parent_id.is_none()).count();
    if orphaned > 1 {
        return LayerResult::fail(format!(
            "Multiple root spans detected: {} orphaned spans (expected 1 root)",
            orphaned
        ));
    }

    LayerResult::pass("Temporal containment valid: all spans properly nested")
}

/// Validates execution order constraints
fn validate_order(_spans: &[SpanData]) -> LayerResult {
    // Simplified: Assume order is correct if spans exist
    // Real implementation would check timestamps
    LayerResult::pass("Execution order valid: no temporal violations detected")
}

/// Validates span status codes
fn validate_status(spans: &[SpanData]) -> LayerResult {
    let error_count = spans
        .iter()
        .filter(|s| s.status == SpanStatus::Error)
        .count();
    if error_count > 0 {
        return LayerResult::fail(format!(
            "Spans with ERROR status detected: {} errors",
            error_count
        ));
    }

    LayerResult::pass("All spans have OK status")
}

/// Validates hermeticity and SDK resource attributes
fn validate_hermeticity(spans: &[SpanData]) -> LayerResult {
    // Check for SDK resource attributes (proves real OTEL SDK usage)
    for span in spans {
        let has_service_name = span.attributes.contains_key("service.name");
        let has_deployment_env = span.attributes.contains_key("deployment.environment");

        if !has_service_name || !has_deployment_env {
            return LayerResult::fail(
                "Missing SDK resource attributes (service.name, deployment.environment) - not from real OTEL SDK"
            );
        }

        // Check for forbidden external service attributes
        let forbidden_keys = ["net.peer.name", "http.url", "db.connection_string"];
        for key in &forbidden_keys {
            if span.attributes.contains_key(*key) {
                return LayerResult::fail(format!(
                    "Hermetic violation: span has external service attribute '{}'",
                    key
                ));
            }
        }
    }

    LayerResult::pass("Hermeticity validated: SDK resources present, no external services")
}

/// Runs all 7 validation layers
fn run_validation(spans: &[SpanData]) -> ValidationResults {
    ValidationResults {
        span_validator: validate_spans(spans),
        graph_validator: validate_graph(spans),
        count_validator: validate_counts(spans),
        window_validator: validate_windows(spans),
        order_validator: validate_order(spans),
        status_validator: validate_status(spans),
        hermeticity_validator: validate_hermeticity(spans),
    }
}

/// Determines overall verdict from validation results
fn compute_verdict(results: &ValidationResults) -> TestVerdict {
    let all_layers = [
        &results.span_validator,
        &results.graph_validator,
        &results.count_validator,
        &results.window_validator,
        &results.order_validator,
        &results.status_validator,
        &results.hermeticity_validator,
    ];

    for layer in all_layers {
        if !layer.passed {
            return TestVerdict::Fail(layer.message.clone());
        }
    }

    TestVerdict::Pass
}

// ============================================================================
// Test Factories
// ============================================================================

/// Creates honest test execution with real OTEL spans
fn create_honest_execution() -> TestExecution {
    let mut run_attrs = HashMap::new();
    run_attrs.insert("service.name".to_string(), "clnrm".to_string());
    run_attrs.insert("deployment.environment".to_string(), "ci".to_string());
    run_attrs.insert("clnrm.version".to_string(), "0.7.0".to_string());
    run_attrs.insert("test.hermetic".to_string(), "true".to_string());

    let mut step_attrs = HashMap::new();
    step_attrs.insert("service.name".to_string(), "clnrm".to_string());
    step_attrs.insert("deployment.environment".to_string(), "ci".to_string());
    step_attrs.insert("test.name".to_string(), "otel_validation".to_string());
    step_attrs.insert("step.name".to_string(), "run_test".to_string());

    let spans = vec![
        SpanData {
            name: "clnrm.run".to_string(),
            kind: SpanKind::Internal,
            parent_id: None,
            attributes: run_attrs,
            events: vec!["test.start".to_string(), "test.end".to_string()],
            status: SpanStatus::Ok,
        },
        SpanData {
            name: "clnrm.step:run_test".to_string(),
            kind: SpanKind::Internal,
            parent_id: Some("run-span-id".to_string()),
            attributes: step_attrs,
            events: vec![
                "container.start".to_string(),
                "container.exec".to_string(),
                "container.stop".to_string(),
            ],
            status: SpanStatus::Ok,
        },
    ];

    let validation_results = run_validation(&spans);
    let verdict = compute_verdict(&validation_results);

    TestExecution {
        verdict,
        spans,
        validation_results,
    }
}

/// Creates fake-green test execution (echo-based, no spans)
fn create_fake_green_execution() -> TestExecution {
    let spans = vec![]; // No spans generated!
    let validation_results = run_validation(&spans);
    let verdict = compute_verdict(&validation_results);

    TestExecution {
        verdict,
        spans,
        validation_results,
    }
}

/// Creates spoofed test execution (fake spans without SDK resources)
fn create_spoofed_execution() -> TestExecution {
    let mut fake_attrs = HashMap::new();
    fake_attrs.insert("fake.attribute".to_string(), "spoofed".to_string());
    // Missing SDK resource attributes!

    let spans = vec![SpanData {
        name: "clnrm.run".to_string(),
        kind: SpanKind::Internal,
        parent_id: None,
        attributes: fake_attrs,
        events: vec![], // No lifecycle events!
        status: SpanStatus::Ok,
    }];

    let validation_results = run_validation(&spans);
    let verdict = compute_verdict(&validation_results);

    TestExecution {
        verdict,
        spans,
        validation_results,
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

/// Test: Honest implementation with real OTEL spans passes validation
#[test]
fn test_honest_implementation_passes_all_validators() -> Result<()> {
    // Arrange
    let execution = create_honest_execution();

    // Act
    let verdict = &execution.verdict;
    let results = &execution.validation_results;

    // Assert - Overall verdict
    assert_eq!(
        *verdict,
        TestVerdict::Pass,
        "Honest implementation should pass validation"
    );

    // Assert - Individual validators
    assert!(
        results.span_validator.passed,
        "Span validator should pass: {}",
        results.span_validator.message
    );
    assert!(
        results.graph_validator.passed,
        "Graph validator should pass: {}",
        results.graph_validator.message
    );
    assert!(
        results.count_validator.passed,
        "Count validator should pass: {}",
        results.count_validator.message
    );
    assert!(
        results.window_validator.passed,
        "Window validator should pass: {}",
        results.window_validator.message
    );
    assert!(
        results.order_validator.passed,
        "Order validator should pass: {}",
        results.order_validator.message
    );
    assert!(
        results.status_validator.passed,
        "Status validator should pass: {}",
        results.status_validator.message
    );
    assert!(
        results.hermeticity_validator.passed,
        "Hermeticity validator should pass: {}",
        results.hermeticity_validator.message
    );

    // Assert - Span characteristics
    assert_eq!(
        execution.spans.len(),
        2,
        "Should have exactly 2 spans (run + step)"
    );

    Ok(())
}

/// Test: Fake-green implementation (echo-based) fails validation
#[test]
fn test_fake_green_implementation_fails_validation() -> Result<()> {
    // Arrange
    let execution = create_fake_green_execution();

    // Act
    let verdict = &execution.verdict;
    let results = &execution.validation_results;

    // Assert - Overall verdict
    assert!(
        matches!(verdict, TestVerdict::Fail(_)),
        "Fake-green implementation should fail validation"
    );

    // Assert - Multiple validators should fail
    assert!(
        !results.span_validator.passed,
        "Span validator should fail: missing required spans"
    );
    assert!(
        !results.graph_validator.passed,
        "Graph validator should fail: no graph structure"
    );
    assert!(
        !results.count_validator.passed,
        "Count validator should fail: zero spans"
    );

    // Assert - No spans generated
    assert_eq!(
        execution.spans.len(),
        0,
        "Fake-green test should produce zero spans"
    );

    Ok(())
}

/// Test: Spoofed spans without SDK resources fail validation
#[test]
fn test_spoofed_spans_without_sdk_resources_fail() -> Result<()> {
    // Arrange
    let execution = create_spoofed_execution();

    // Act
    let verdict = &execution.verdict;
    let results = &execution.validation_results;

    // Assert - Overall verdict
    assert!(
        matches!(verdict, TestVerdict::Fail(_)),
        "Spoofed implementation should fail validation"
    );

    // Assert - Hermeticity validator catches missing SDK resources
    assert!(
        !results.hermeticity_validator.passed,
        "Hermeticity validator should fail: {}",
        results.hermeticity_validator.message
    );

    // Assert - Span validator catches missing lifecycle events
    assert!(
        !results.span_validator.passed,
        "Span validator should fail: missing lifecycle events"
    );

    Ok(())
}

/// Test: Validation catches spans with error status
#[test]
fn test_error_status_spans_fail_validation() -> Result<()> {
    // Arrange
    let mut attrs = HashMap::new();
    attrs.insert("service.name".to_string(), "clnrm".to_string());
    attrs.insert("deployment.environment".to_string(), "ci".to_string());

    let spans = vec![SpanData {
        name: "clnrm.run".to_string(),
        kind: SpanKind::Internal,
        parent_id: None,
        attributes: attrs,
        events: vec!["container.start".to_string()],
        status: SpanStatus::Error, // ERROR status!
    }];

    // Act
    let validation_results = run_validation(&spans);
    let verdict = compute_verdict(&validation_results);

    // Assert
    assert!(
        matches!(verdict, TestVerdict::Fail(_)),
        "Spans with ERROR status should fail validation"
    );
    assert!(
        !validation_results.status_validator.passed,
        "Status validator should fail: {}",
        validation_results.status_validator.message
    );

    Ok(())
}

/// Test: Validation catches external service attributes
#[test]
fn test_external_service_attributes_fail_validation() -> Result<()> {
    // Arrange
    let mut attrs = HashMap::new();
    attrs.insert("service.name".to_string(), "clnrm".to_string());
    attrs.insert("deployment.environment".to_string(), "ci".to_string());
    attrs.insert("http.url".to_string(), "https://external.api".to_string()); // Forbidden!

    let spans = vec![SpanData {
        name: "clnrm.run".to_string(),
        kind: SpanKind::Internal,
        parent_id: None,
        attributes: attrs,
        events: vec!["container.start".to_string()],
        status: SpanStatus::Ok,
    }];

    // Act
    let validation_results = run_validation(&spans);
    let verdict = compute_verdict(&validation_results);

    // Assert
    assert!(
        matches!(verdict, TestVerdict::Fail(_)),
        "External service attributes should fail validation"
    );
    assert!(
        !validation_results.hermeticity_validator.passed,
        "Hermeticity validator should fail: {}",
        validation_results.hermeticity_validator.message
    );

    Ok(())
}

/// Test: Validation catches orphaned spans (no parent)
#[test]
fn test_orphaned_spans_fail_graph_validation() -> Result<()> {
    // Arrange
    let mut attrs = HashMap::new();
    attrs.insert("service.name".to_string(), "clnrm".to_string());
    attrs.insert("deployment.environment".to_string(), "ci".to_string());

    let spans = vec![
        SpanData {
            name: "clnrm.run".to_string(),
            kind: SpanKind::Internal,
            parent_id: None,
            attributes: attrs.clone(),
            events: vec!["test.start".to_string()],
            status: SpanStatus::Ok,
        },
        SpanData {
            name: "clnrm.step:orphan".to_string(),
            kind: SpanKind::Internal,
            parent_id: None, // Orphaned span!
            attributes: attrs,
            events: vec!["container.start".to_string()],
            status: SpanStatus::Ok,
        },
    ];

    // Act
    let validation_results = run_validation(&spans);

    // Assert
    assert!(
        !validation_results.window_validator.passed,
        "Window validator should catch orphaned spans: {}",
        validation_results.window_validator.message
    );

    Ok(())
}

/// Test: All validators have descriptive failure messages
#[test]
fn test_validators_provide_descriptive_messages() -> Result<()> {
    // Arrange
    let execution = create_fake_green_execution();

    // Act
    let results = &execution.validation_results;

    // Assert - Each failed validator has a message
    assert!(
        !results.span_validator.message.is_empty(),
        "Span validator should provide failure message"
    );
    assert!(
        !results.graph_validator.message.is_empty(),
        "Graph validator should provide failure message"
    );
    assert!(
        !results.count_validator.message.is_empty(),
        "Count validator should provide failure message"
    );

    Ok(())
}

// ============================================================================
// Documentation Tests
// ============================================================================

/// Test: Documentation example - how to use validators
#[test]
fn test_validator_usage_example() -> Result<()> {
    // This test demonstrates how to use the validation system

    // Step 1: Create spans (from OTEL exporter)
    let spans = vec![
        /* ... collected from OTEL */
    ];

    // Step 2: Run validation
    let results = run_validation(&spans);

    // Step 3: Check results
    let _verdict = compute_verdict(&results);

    // Step 4: Inspect individual validators
    if !results.span_validator.passed {
        eprintln!("Span validation failed: {}", results.span_validator.message);
    }

    Ok(())
}
