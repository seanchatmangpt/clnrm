// Integration tests for fake-green detection using OTEL validation
// Tests London School TDD: Mock telemetry data, verify validator behavior
//
// These tests validate that clnrm's OTEL-first validation correctly:
// 1. PASSES honest implementations (with full OTEL evidence)
// 2. FAILS fake-green implementations (missing OTEL evidence)

use std::collections::HashMap;

// =============================================================================
// MOCK OTEL STRUCTURES FOR TESTING
// =============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct MockSpan {
    pub name: String,
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub status: SpanStatus,
    pub events: Vec<SpanEvent>,
    pub start_time: u64,
    pub end_time: u64,
    pub attributes: HashMap<String, String>,
    pub resource_attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpanStatus {
    Ok,
    Error,
    Unset,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpanEvent {
    pub name: String,
    pub timestamp: u64,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct MockTraceData {
    pub spans: Vec<MockSpan>,
    pub test_name: String,
    pub reported_status: String,
}

// =============================================================================
// OTEL VALIDATORS - London School TDD: Test behavior, not implementation
// =============================================================================

pub trait OtelValidator {
    fn validate(&self, trace_data: &MockTraceData) -> ValidationResult;
    fn name(&self) -> &str;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
    pub passed: bool,
    pub validator_name: String,
    pub failures: Vec<String>,
}

impl ValidationResult {
    fn success(validator_name: String) -> Self {
        Self {
            passed: true,
            validator_name,
            failures: vec![],
        }
    }

    fn failure(validator_name: String, failures: Vec<String>) -> Self {
        Self {
            passed: false,
            validator_name,
            failures,
        }
    }
}

// =============================================================================
// SPAN VALIDATOR - Checks for required lifecycle events
// =============================================================================

pub struct SpanValidator {
    required_events: Vec<String>,
}

impl SpanValidator {
    pub fn new(required_events: Vec<String>) -> Self {
        Self { required_events }
    }
}

impl OtelValidator for SpanValidator {
    fn validate(&self, trace_data: &MockTraceData) -> ValidationResult {
        let mut failures = Vec::new();

        // Collect all events from all spans
        let all_events: Vec<String> = trace_data
            .spans
            .iter()
            .flat_map(|span| span.events.iter().map(|e| e.name.clone()))
            .collect();

        // Check for required events
        for required_event in &self.required_events {
            if !all_events.contains(required_event) {
                failures.push(format!(
                    "Missing required lifecycle event: {}",
                    required_event
                ));
            }
        }

        if failures.is_empty() {
            ValidationResult::success("span_validator".to_string())
        } else {
            ValidationResult::failure("span_validator".to_string(), failures)
        }
    }

    fn name(&self) -> &str {
        "span_validator"
    }
}

// =============================================================================
// GRAPH VALIDATOR - Checks for parent-child relationships
// =============================================================================

pub struct GraphValidator {
    require_edges: bool,
}

impl GraphValidator {
    pub fn new(require_edges: bool) -> Self {
        Self { require_edges }
    }
}

impl OtelValidator for GraphValidator {
    fn validate(&self, trace_data: &MockTraceData) -> ValidationResult {
        let mut failures = Vec::new();

        if !self.require_edges {
            return ValidationResult::success("graph_validator".to_string());
        }

        // Find root span (no parent)
        let root_spans: Vec<_> = trace_data
            .spans
            .iter()
            .filter(|s| s.parent_span_id.is_none())
            .collect();

        if root_spans.is_empty() {
            failures.push("No root span found".to_string());
            return ValidationResult::failure("graph_validator".to_string(), failures);
        }

        if root_spans.len() > 1 {
            failures.push(format!("Multiple root spans found: {}", root_spans.len()));
        }

        // Check for child spans
        let child_spans: Vec<_> = trace_data
            .spans
            .iter()
            .filter(|s| s.parent_span_id.is_some())
            .collect();

        // Check if we expect child spans:
        // 1. If there are multiple spans, some should be children
        // 2. If there are multiple lifecycle events in a single span, child spans are expected
        let has_multiple_lifecycle_events =
            trace_data.spans.iter().any(|span| span.events.len() > 1);

        let should_have_children = trace_data.spans.len() > 1 || has_multiple_lifecycle_events;

        if child_spans.is_empty() && should_have_children {
            failures.push(
                "Graph validator detects missing parent→child edges: no child spans found"
                    .to_string(),
            );
        }

        if failures.is_empty() {
            ValidationResult::success("graph_validator".to_string())
        } else {
            ValidationResult::failure("graph_validator".to_string(), failures)
        }
    }

    fn name(&self) -> &str {
        "graph_validator"
    }
}

// =============================================================================
// COUNTS VALIDATOR - Verifies span counts match expectations
// =============================================================================

pub struct CountsValidator {
    minimum_spans: usize,
}

impl CountsValidator {
    pub fn new(minimum_spans: usize) -> Self {
        Self { minimum_spans }
    }
}

impl OtelValidator for CountsValidator {
    fn validate(&self, trace_data: &MockTraceData) -> ValidationResult {
        let mut failures = Vec::new();
        let span_count = trace_data.spans.len();

        if span_count < self.minimum_spans {
            failures.push(format!(
                "Counts validator detects span count mismatch (expected >= {}, got {})",
                self.minimum_spans, span_count
            ));
        }

        if failures.is_empty() {
            ValidationResult::success("counts_validator".to_string())
        } else {
            ValidationResult::failure("counts_validator".to_string(), failures)
        }
    }

    fn name(&self) -> &str {
        "counts_validator"
    }
}

// =============================================================================
// STATUS VALIDATOR - Checks for status mismatches
// =============================================================================

pub struct StatusValidator;

impl StatusValidator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StatusValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl OtelValidator for StatusValidator {
    fn validate(&self, trace_data: &MockTraceData) -> ValidationResult {
        let mut failures = Vec::new();

        // Check if reported status is OK but we have ERROR spans
        if trace_data.reported_status == "OK" {
            let error_spans: Vec<_> = trace_data
                .spans
                .iter()
                .filter(|s| s.status == SpanStatus::Error)
                .collect();

            if !error_spans.is_empty() {
                failures.push(format!(
                    "Status validator detects mismatch (reported OK but has {} ERROR spans)",
                    error_spans.len()
                ));
            }
        }

        if failures.is_empty() {
            ValidationResult::success("status_validator".to_string())
        } else {
            ValidationResult::failure("status_validator".to_string(), failures)
        }
    }

    fn name(&self) -> &str {
        "status_validator"
    }
}

// =============================================================================
// VALIDATION ORCHESTRATOR
// =============================================================================

pub fn validate_trace(trace_data: &MockTraceData) -> Vec<ValidationResult> {
    let validators: Vec<Box<dyn OtelValidator>> = vec![
        Box::new(SpanValidator::new(vec![
            "container.start".to_string(),
            "container.exec".to_string(),
            "container.stop".to_string(),
        ])),
        Box::new(GraphValidator::new(true)),
        Box::new(CountsValidator::new(4)), // root + 3 steps minimum
        Box::new(StatusValidator::new()),
    ];

    validators.iter().map(|v| v.validate(trace_data)).collect()
}

// =============================================================================
// MOCK REPORT CREATORS - Following AAA pattern
// =============================================================================

fn create_legitimate_trace() -> MockTraceData {
    // Arrange: Create proper trace with all lifecycle events
    let mut resource_attrs = HashMap::new();
    resource_attrs.insert("service.name".to_string(), "clnrm".to_string());
    resource_attrs.insert("deployment.environment".to_string(), "test".to_string());

    let root_span = MockSpan {
        name: "legitimate_test".to_string(),
        trace_id: "trace-123".to_string(),
        span_id: "span-root".to_string(),
        parent_span_id: None,
        status: SpanStatus::Ok,
        events: vec![SpanEvent {
            name: "container.start".to_string(),
            timestamp: 1000,
            attributes: HashMap::new(),
        }],
        start_time: 1000,
        end_time: 5000,
        attributes: HashMap::new(),
        resource_attributes: resource_attrs.clone(),
    };

    let step1_span = MockSpan {
        name: "container_start".to_string(),
        trace_id: "trace-123".to_string(),
        span_id: "span-1".to_string(),
        parent_span_id: Some("span-root".to_string()),
        status: SpanStatus::Ok,
        events: vec![SpanEvent {
            name: "container.exec".to_string(),
            timestamp: 1500,
            attributes: HashMap::new(),
        }],
        start_time: 1500,
        end_time: 2500,
        attributes: HashMap::new(),
        resource_attributes: resource_attrs.clone(),
    };

    let step2_span = MockSpan {
        name: "execute_command".to_string(),
        trace_id: "trace-123".to_string(),
        span_id: "span-2".to_string(),
        parent_span_id: Some("span-root".to_string()),
        status: SpanStatus::Ok,
        events: vec![SpanEvent {
            name: "container.exec".to_string(),
            timestamp: 2600,
            attributes: HashMap::new(),
        }],
        start_time: 2600,
        end_time: 3500,
        attributes: HashMap::new(),
        resource_attributes: resource_attrs.clone(),
    };

    let step3_span = MockSpan {
        name: "verify_execution".to_string(),
        trace_id: "trace-123".to_string(),
        span_id: "span-3".to_string(),
        parent_span_id: Some("span-root".to_string()),
        status: SpanStatus::Ok,
        events: vec![SpanEvent {
            name: "container.stop".to_string(),
            timestamp: 4500,
            attributes: HashMap::new(),
        }],
        start_time: 3600,
        end_time: 4500,
        attributes: HashMap::new(),
        resource_attributes: resource_attrs,
    };

    MockTraceData {
        spans: vec![root_span, step1_span, step2_span, step3_span],
        test_name: "legitimate_test".to_string(),
        reported_status: "OK".to_string(),
    }
}

fn create_no_execution_trace() -> MockTraceData {
    // Arrange: Fake trace with no lifecycle events
    let root_span = MockSpan {
        name: "fake_no_execution".to_string(),
        trace_id: "trace-456".to_string(),
        span_id: "span-root".to_string(),
        parent_span_id: None,
        status: SpanStatus::Ok,
        events: vec![], // NO EVENTS - this is the fake!
        start_time: 1000,
        end_time: 2000,
        attributes: HashMap::new(),
        resource_attributes: HashMap::new(),
    };

    MockTraceData {
        spans: vec![root_span],
        test_name: "fake_no_execution".to_string(),
        reported_status: "OK".to_string(),
    }
}

fn create_missing_edges_trace() -> MockTraceData {
    // Arrange: Fake trace with root but no children
    let mut resource_attrs = HashMap::new();
    resource_attrs.insert("service.name".to_string(), "clnrm".to_string());

    let root_span = MockSpan {
        name: "fake_missing_edges".to_string(),
        trace_id: "trace-789".to_string(),
        span_id: "span-root".to_string(),
        parent_span_id: None,
        status: SpanStatus::Ok,
        events: vec![
            SpanEvent {
                name: "container.start".to_string(),
                timestamp: 1000,
                attributes: HashMap::new(),
            },
            SpanEvent {
                name: "container.exec".to_string(),
                timestamp: 2000,
                attributes: HashMap::new(),
            },
            SpanEvent {
                name: "container.stop".to_string(),
                timestamp: 4500,
                attributes: HashMap::new(),
            },
        ],
        start_time: 1000,
        end_time: 5000,
        attributes: HashMap::new(),
        resource_attributes: resource_attrs,
    };

    // Note: Only root span, no child spans despite having multiple steps
    MockTraceData {
        spans: vec![root_span],
        test_name: "fake_missing_edges".to_string(),
        reported_status: "OK".to_string(),
    }
}

fn create_wrong_counts_trace() -> MockTraceData {
    // Arrange: Fake trace with zero spans (claims success but no telemetry)
    MockTraceData {
        spans: vec![],
        test_name: "fake_wrong_counts".to_string(),
        reported_status: "OK".to_string(),
    }
}

fn create_status_mismatch_trace() -> MockTraceData {
    // Arrange: Fake trace claiming OK but has ERROR span
    let mut resource_attrs = HashMap::new();
    resource_attrs.insert("service.name".to_string(), "clnrm".to_string());

    let root_span = MockSpan {
        name: "fake_status_mismatch".to_string(),
        trace_id: "trace-abc".to_string(),
        span_id: "span-root".to_string(),
        parent_span_id: None,
        status: SpanStatus::Ok,
        events: vec![SpanEvent {
            name: "container.start".to_string(),
            timestamp: 1000,
            attributes: HashMap::new(),
        }],
        start_time: 1000,
        end_time: 5000,
        attributes: HashMap::new(),
        resource_attributes: resource_attrs.clone(),
    };

    let error_step = MockSpan {
        name: "step_with_hidden_error".to_string(),
        trace_id: "trace-abc".to_string(),
        span_id: "span-1".to_string(),
        parent_span_id: Some("span-root".to_string()),
        status: SpanStatus::Error, // ERROR status
        events: vec![SpanEvent {
            name: "container.exec".to_string(),
            timestamp: 1500,
            attributes: HashMap::new(),
        }],
        start_time: 1500,
        end_time: 2500,
        attributes: HashMap::new(),
        resource_attributes: resource_attrs.clone(),
    };

    let masking_step = MockSpan {
        name: "step_masking_failure".to_string(),
        trace_id: "trace-abc".to_string(),
        span_id: "span-2".to_string(),
        parent_span_id: Some("span-root".to_string()),
        status: SpanStatus::Ok,
        events: vec![SpanEvent {
            name: "container.stop".to_string(),
            timestamp: 4500,
            attributes: HashMap::new(),
        }],
        start_time: 2600,
        end_time: 4500,
        attributes: HashMap::new(),
        resource_attributes: resource_attrs,
    };

    MockTraceData {
        spans: vec![root_span, error_step, masking_step],
        test_name: "fake_status_mismatch".to_string(),
        reported_status: "OK".to_string(), // Claims OK despite error
    }
}

// =============================================================================
// INTEGRATION TESTS - London School TDD: Test behavior via mocks
// =============================================================================

#[test]
fn test_legitimate_test_passes_all_validators() {
    // Arrange
    let trace_data = create_legitimate_trace();

    // Act
    let results = validate_trace(&trace_data);

    // Assert
    assert_eq!(results.len(), 4, "Should run all 4 validators");

    for result in &results {
        assert!(
            result.passed,
            "Validator {} should pass for legitimate test. Failures: {:?}",
            result.validator_name, result.failures
        );
    }
}

#[test]
fn test_no_execution_fails_span_validator() {
    // Arrange
    let trace_data = create_no_execution_trace();

    // Act
    let results = validate_trace(&trace_data);

    // Assert
    let span_validator_result = results
        .iter()
        .find(|r| r.validator_name == "span_validator")
        .expect("Should have span_validator result");

    assert!(
        !span_validator_result.passed,
        "Span validator should FAIL for no_execution fake"
    );

    assert_eq!(
        span_validator_result.failures.len(),
        3,
        "Should report 3 missing lifecycle events"
    );

    assert!(
        span_validator_result.failures[0].contains("container.start"),
        "Should report missing container.start event"
    );
}

#[test]
fn test_missing_edges_fails_graph_validator() {
    // Arrange
    let trace_data = create_missing_edges_trace();

    // Act
    let results = validate_trace(&trace_data);

    // Assert
    let graph_validator_result = results
        .iter()
        .find(|r| r.validator_name == "graph_validator")
        .expect("Should have graph_validator result");

    assert!(
        !graph_validator_result.passed,
        "Graph validator should FAIL for missing_edges fake"
    );

    assert!(
        graph_validator_result.failures[0].contains("missing parent→child edges"),
        "Should report missing parent-child edges. Got: {:?}",
        graph_validator_result.failures
    );
}

#[test]
fn test_wrong_counts_fails_counts_validator() {
    // Arrange
    let trace_data = create_wrong_counts_trace();

    // Act
    let results = validate_trace(&trace_data);

    // Assert
    let counts_validator_result = results
        .iter()
        .find(|r| r.validator_name == "counts_validator")
        .expect("Should have counts_validator result");

    assert!(
        !counts_validator_result.passed,
        "Counts validator should FAIL for wrong_counts fake"
    );

    assert!(
        counts_validator_result.failures[0].contains("expected >= 4, got 0"),
        "Should report span count mismatch. Got: {:?}",
        counts_validator_result.failures
    );
}

#[test]
fn test_status_mismatch_fails_status_validator() {
    // Arrange
    let trace_data = create_status_mismatch_trace();

    // Act
    let results = validate_trace(&trace_data);

    // Assert
    let status_validator_result = results
        .iter()
        .find(|r| r.validator_name == "status_validator")
        .expect("Should have status_validator result");

    assert!(
        !status_validator_result.passed,
        "Status validator should FAIL for status_mismatch fake"
    );

    assert!(
        status_validator_result.failures[0].contains("reported OK but has"),
        "Should report status mismatch. Got: {:?}",
        status_validator_result.failures
    );

    assert!(
        status_validator_result.failures[0].contains("ERROR spans"),
        "Should mention ERROR spans. Got: {:?}",
        status_validator_result.failures
    );
}

#[test]
fn test_all_fake_scenarios_detected() {
    // Arrange
    let scenarios = vec![
        ("no_execution", create_no_execution_trace()),
        ("missing_edges", create_missing_edges_trace()),
        ("wrong_counts", create_wrong_counts_trace()),
        ("status_mismatch", create_status_mismatch_trace()),
    ];

    // Act & Assert
    for (scenario_name, trace_data) in scenarios {
        let results = validate_trace(&trace_data);

        let any_failed = results.iter().any(|r| !r.passed);

        assert!(
            any_failed,
            "Scenario '{}' should fail at least one validator",
            scenario_name
        );
    }
}

#[test]
fn test_legitimate_vs_fake_differentiation() {
    // Arrange
    let legitimate = create_legitimate_trace();
    let fake = create_no_execution_trace();

    // Act
    let legit_results = validate_trace(&legitimate);
    let fake_results = validate_trace(&fake);

    // Assert - legitimate passes all
    assert!(
        legit_results.iter().all(|r| r.passed),
        "Legitimate test should pass all validators"
    );

    // Assert - fake fails at least one
    assert!(
        fake_results.iter().any(|r| !r.passed),
        "Fake test should fail at least one validator"
    );
}

#[test]
fn test_validator_failure_messages_are_descriptive() {
    // Arrange
    let scenarios = vec![
        create_no_execution_trace(),
        create_missing_edges_trace(),
        create_wrong_counts_trace(),
        create_status_mismatch_trace(),
    ];

    // Act & Assert
    for trace_data in scenarios {
        let results = validate_trace(&trace_data);

        for result in results {
            if !result.passed {
                assert!(
                    !result.failures.is_empty(),
                    "Failed validators must provide failure messages"
                );

                for failure_msg in &result.failures {
                    assert!(
                        failure_msg.len() > 10,
                        "Failure messages should be descriptive, got: '{}'",
                        failure_msg
                    );
                }
            }
        }
    }
}

#[test]
fn test_validation_results_contain_validator_names() {
    // Arrange
    let trace_data = create_legitimate_trace();

    // Act
    let results = validate_trace(&trace_data);

    // Assert
    let expected_validators = vec![
        "span_validator",
        "graph_validator",
        "counts_validator",
        "status_validator",
    ];

    for expected in expected_validators {
        assert!(
            results.iter().any(|r| r.validator_name == expected),
            "Should have result for validator: {}",
            expected
        );
    }
}

#[test]
fn test_each_validator_independently_catches_specific_fake() {
    // Arrange & Act & Assert

    // Test 1: Span validator catches no_execution
    let no_exec_trace = create_no_execution_trace();
    let span_val = SpanValidator::new(vec![
        "container.start".to_string(),
        "container.exec".to_string(),
        "container.stop".to_string(),
    ]);
    let result = span_val.validate(&no_exec_trace);
    assert!(
        !result.passed,
        "Span validator should catch no_execution fake"
    );

    // Test 2: Graph validator catches missing_edges
    let missing_edges_trace = create_missing_edges_trace();
    let graph_val = GraphValidator::new(true);
    let result = graph_val.validate(&missing_edges_trace);
    assert!(
        !result.passed,
        "Graph validator should catch missing_edges fake"
    );

    // Test 3: Counts validator catches wrong_counts
    let wrong_counts_trace = create_wrong_counts_trace();
    let counts_val = CountsValidator::new(4);
    let result = counts_val.validate(&wrong_counts_trace);
    assert!(
        !result.passed,
        "Counts validator should catch wrong_counts fake"
    );

    // Test 4: Status validator catches status_mismatch
    let status_mismatch_trace = create_status_mismatch_trace();
    let status_val = StatusValidator::new();
    let result = status_val.validate(&status_mismatch_trace);
    assert!(
        !result.passed,
        "Status validator should catch status_mismatch fake"
    );
}

#[test]
fn test_legitimate_passes_each_validator_independently() {
    // Arrange
    let legit_trace = create_legitimate_trace();

    // Act & Assert - Each validator should pass individually
    let span_val = SpanValidator::new(vec![
        "container.start".to_string(),
        "container.exec".to_string(),
        "container.stop".to_string(),
    ]);
    assert!(
        span_val.validate(&legit_trace).passed,
        "Legitimate should pass span validator"
    );

    let graph_val = GraphValidator::new(true);
    assert!(
        graph_val.validate(&legit_trace).passed,
        "Legitimate should pass graph validator"
    );

    let counts_val = CountsValidator::new(4);
    assert!(
        counts_val.validate(&legit_trace).passed,
        "Legitimate should pass counts validator"
    );

    let status_val = StatusValidator::new();
    assert!(
        status_val.validate(&legit_trace).passed,
        "Legitimate should pass status validator"
    );
}
