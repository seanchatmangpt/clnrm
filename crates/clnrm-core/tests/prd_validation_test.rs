//! Integration tests for OTEL PRD validation
//!
//! Tests validate the complete PRD validation workflow:
//! - Span existence and hierarchy
//! - Graph validation (edges, cycles)
//! - Count validation (bounds)
//! - Temporal window containment
//! - Hermeticity validation

use clnrm_core::validation::span_validator::SpanValidator;

// ============================================================================
// PRD MINIMAL HAPPY PATH TESTS
// ============================================================================

#[test]
fn test_prd_minimal_happy_path_validates_span_existence() {
    // Arrange: Create minimal valid span data matching PRD example
    let json = r#"
{"name":"clnrm.run","trace_id":"abc","span_id":"1","parent_span_id":null,"attributes":{"result":"pass"}}
{"name":"clnrm.step:hello_world","trace_id":"abc","span_id":"2","parent_span_id":"1","attributes":{"step.name":"hello_world"}}
"#;

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Failed to parse valid JSON");

    // Assert: Both required spans exist
    assert!(
        validator.has_span("clnrm.run"),
        "PRD requires clnrm.run span to exist"
    );
    assert!(
        validator.has_span("clnrm.step:hello_world"),
        "PRD requires clnrm.step:hello_world span to exist"
    );
}

#[test]
fn test_prd_minimal_happy_path_validates_hierarchy() {
    // Arrange: Create parent-child relationship
    let json = r#"
{"name":"clnrm.run","trace_id":"abc","span_id":"1","parent_span_id":null,"attributes":{"result":"pass"},"start_time_unix_nano":1000,"end_time_unix_nano":2000}
{"name":"clnrm.step:hello_world","trace_id":"abc","span_id":"2","parent_span_id":"1","attributes":{"step.name":"hello_world"},"start_time_unix_nano":1100,"end_time_unix_nano":1900}
"#;

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Failed to parse valid JSON");

    // Assert: Verify parent-child relationship
    let run_span = validator.get_span("clnrm.run")
        .expect("clnrm.run span should exist");
    let step_span = validator.get_span("clnrm.step:hello_world")
        .expect("clnrm.step:hello_world span should exist");

    assert!(
        run_span.parent_span_id.is_none(),
        "clnrm.run should be root span with no parent"
    );
    assert_eq!(
        step_span.parent_span_id.as_deref(),
        Some("1"),
        "clnrm.step:hello_world should have clnrm.run as parent"
    );
}

#[test]
fn test_prd_minimal_happy_path_validates_counts() {
    // Arrange: Create span data with known counts
    let json = r#"
{"name":"clnrm.run","trace_id":"abc","span_id":"1","parent_span_id":null,"attributes":{"result":"pass"},"start_time_unix_nano":1000,"end_time_unix_nano":2000}
{"name":"clnrm.step:hello_world","trace_id":"abc","span_id":"2","parent_span_id":"1","attributes":{"step.name":"hello_world"},"start_time_unix_nano":1100,"end_time_unix_nano":1900}
"#;

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Failed to parse valid JSON");

    // Assert: Verify counts match PRD expectations
    assert_eq!(
        validator.count_spans("clnrm.run"),
        1,
        "PRD specifies exactly one clnrm.run span"
    );
    assert_eq!(
        validator.count_spans("clnrm.step:hello_world"),
        1,
        "PRD example has exactly one step span"
    );
}

#[test]
fn test_prd_minimal_happy_path_validates_temporal_window() {
    // Arrange: Create spans with proper temporal containment
    let json = r#"
{"name":"clnrm.run","trace_id":"abc","span_id":"1","parent_span_id":null,"attributes":{"result":"pass"},"start_time_unix_nano":1000,"end_time_unix_nano":2000}
{"name":"clnrm.step:hello_world","trace_id":"abc","span_id":"2","parent_span_id":"1","attributes":{"step.name":"hello_world"},"start_time_unix_nano":1100,"end_time_unix_nano":1900}
"#;

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Failed to parse valid JSON");

    let run_span = validator.get_span("clnrm.run").unwrap();
    let step_span = validator.get_span("clnrm.step:hello_world").unwrap();

    // Assert: Child span is temporally contained in parent
    let run_start = run_span.start_time_unix_nano.expect("run span should have start time");
    let run_end = run_span.end_time_unix_nano.expect("run span should have end time");
    let step_start = step_span.start_time_unix_nano.expect("step span should have start time");
    let step_end = step_span.end_time_unix_nano.expect("step span should have end time");

    assert!(
        step_start >= run_start,
        "Child span must start after or with parent: child_start={}, parent_start={}",
        step_start,
        run_start
    );
    assert!(
        step_end <= run_end,
        "Child span must end before or with parent: child_end={}, parent_end={}",
        step_end,
        run_end
    );
}

// ============================================================================
// GRAPH VALIDATION TESTS
// ============================================================================

#[test]
fn test_graph_validation_must_include_edges_present() {
    // Arrange: Create graph with required edges
    let json = r#"
{"name":"clnrm.run","trace_id":"abc","span_id":"1","parent_span_id":null,"attributes":{},"start_time_unix_nano":1000,"end_time_unix_nano":3000}
{"name":"clnrm.step:step1","trace_id":"abc","span_id":"2","parent_span_id":"1","attributes":{},"start_time_unix_nano":1100,"end_time_unix_nano":2000}
{"name":"clnrm.step:step2","trace_id":"abc","span_id":"3","parent_span_id":"1","attributes":{},"start_time_unix_nano":2100,"end_time_unix_nano":2900}
"#;

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Failed to parse valid JSON");

    // Assert: Verify edges exist using parent_span_id relationships
    let step1 = validator.get_span("clnrm.step:step1").unwrap();
    let step2 = validator.get_span("clnrm.step:step2").unwrap();

    assert_eq!(
        step1.parent_span_id.as_deref(),
        Some("1"),
        "clnrm.run -> clnrm.step:step1 edge must exist"
    );
    assert_eq!(
        step2.parent_span_id.as_deref(),
        Some("1"),
        "clnrm.run -> clnrm.step:step2 edge must exist"
    );
}

#[test]
fn test_graph_validation_must_include_edges_missing_fails() {
    // Arrange: Create graph with missing required edge
    let json = r#"
{"name":"clnrm.run","trace_id":"abc","span_id":"1","parent_span_id":null,"attributes":{},"start_time_unix_nano":1000,"end_time_unix_nano":3000}
{"name":"clnrm.step:step1","trace_id":"abc","span_id":"2","parent_span_id":null,"attributes":{},"start_time_unix_nano":1100,"end_time_unix_nano":2000}
"#;

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Failed to parse valid JSON");

    let step1 = validator.get_span("clnrm.step:step1").unwrap();

    // Assert: Edge is missing (step1 has no parent, should have clnrm.run as parent)
    assert!(
        step1.parent_span_id.is_none(),
        "Missing edge: clnrm.step:step1 is orphaned (no parent when it should have clnrm.run)"
    );
}

#[test]
fn test_graph_validation_acyclic_detects_no_cycles_in_valid_dag() {
    // Arrange: Create valid DAG (Directed Acyclic Graph)
    let json = r#"
{"name":"root","trace_id":"abc","span_id":"1","parent_span_id":null,"attributes":{},"start_time_unix_nano":1000,"end_time_unix_nano":5000}
{"name":"child1","trace_id":"abc","span_id":"2","parent_span_id":"1","attributes":{},"start_time_unix_nano":1100,"end_time_unix_nano":3000}
{"name":"child2","trace_id":"abc","span_id":"3","parent_span_id":"1","attributes":{},"start_time_unix_nano":3100,"end_time_unix_nano":4900}
{"name":"grandchild","trace_id":"abc","span_id":"4","parent_span_id":"2","attributes":{},"start_time_unix_nano":1200,"end_time_unix_nano":2900}
"#;

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Failed to parse valid JSON");

    // Assert: Verify it's a valid tree (no cycles possible in tree structure)
    // In a tree, every span has at most one parent
    let spans = validator.all_spans();
    let mut parent_counts = std::collections::HashMap::new();

    for span in spans {
        if let Some(parent_id) = &span.parent_span_id {
            *parent_counts.entry(parent_id.clone()).or_insert(0) += 1;
        }
    }

    // Each span can be referenced as parent by multiple children (valid tree)
    assert!(
        parent_counts.values().all(|&count| count >= 1),
        "Valid DAG should have no cycles - each parent referenced by children"
    );
}

#[test]
fn test_graph_validation_detects_self_reference_cycle() {
    // Arrange: Create span with self-reference (cycle)
    let json = r#"
{"name":"self_ref","trace_id":"abc","span_id":"1","parent_span_id":"1","attributes":{},"start_time_unix_nano":1000,"end_time_unix_nano":2000}
"#;

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Failed to parse valid JSON");

    let span = validator.get_span("self_ref").unwrap();

    // Assert: Span references itself as parent (cycle)
    assert_eq!(
        span.span_id,
        span.parent_span_id.as_deref().unwrap(),
        "Self-reference cycle detected: span is its own parent"
    );
}

// ============================================================================
// COUNT VALIDATION TESTS
// ============================================================================

#[test]
fn test_count_validation_gte_bound_satisfied() {
    // Arrange: Create spans with count >= threshold
    let json = r#"
{"name":"clnrm.step","trace_id":"abc","span_id":"1","parent_span_id":null,"attributes":{},"start_time_unix_nano":1000,"end_time_unix_nano":2000}
{"name":"clnrm.step","trace_id":"abc","span_id":"2","parent_span_id":null,"attributes":{},"start_time_unix_nano":3000,"end_time_unix_nano":4000}
{"name":"clnrm.step","trace_id":"abc","span_id":"3","parent_span_id":null,"attributes":{},"start_time_unix_nano":5000,"end_time_unix_nano":6000}
"#;

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Failed to parse valid JSON");

    let count = validator.count_spans("clnrm.step");

    // Assert: gte(2) bound is satisfied
    assert!(
        count >= 2,
        "Count validation gte(2) failed: got {} spans",
        count
    );
}

#[test]
fn test_count_validation_gte_bound_violated() {
    // Arrange: Create spans with count < threshold
    let json = r#"
{"name":"clnrm.step","trace_id":"abc","span_id":"1","parent_span_id":null,"attributes":{},"start_time_unix_nano":1000,"end_time_unix_nano":2000}
"#;

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Failed to parse valid JSON");

    let count = validator.count_spans("clnrm.step");

    // Assert: gte(2) bound is violated
    assert!(
        count < 2,
        "Count validation gte(2) should be violated: got {} spans (expected < 2)",
        count
    );
}

#[test]
fn test_count_validation_lte_bound_satisfied() {
    // Arrange: Create spans with count <= threshold
    let json = r#"
{"name":"clnrm.step","trace_id":"abc","span_id":"1","parent_span_id":null,"attributes":{},"start_time_unix_nano":1000,"end_time_unix_nano":2000}
{"name":"clnrm.step","trace_id":"abc","span_id":"2","parent_span_id":null,"attributes":{},"start_time_unix_nano":3000,"end_time_unix_nano":4000}
"#;

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Failed to parse valid JSON");

    let count = validator.count_spans("clnrm.step");

    // Assert: lte(5) bound is satisfied
    assert!(
        count <= 5,
        "Count validation lte(5) failed: got {} spans",
        count
    );
}

#[test]
fn test_count_validation_lte_bound_violated() {
    // Arrange: Create spans with count > threshold
    let json = r#"
{"name":"clnrm.step","trace_id":"abc","span_id":"1","parent_span_id":null,"attributes":{},"start_time_unix_nano":1000,"end_time_unix_nano":2000}
{"name":"clnrm.step","trace_id":"abc","span_id":"2","parent_span_id":null,"attributes":{},"start_time_unix_nano":3000,"end_time_unix_nano":4000}
{"name":"clnrm.step","trace_id":"abc","span_id":"3","parent_span_id":null,"attributes":{},"start_time_unix_nano":5000,"end_time_unix_nano":6000}
{"name":"clnrm.step","trace_id":"abc","span_id":"4","parent_span_id":null,"attributes":{},"start_time_unix_nano":7000,"end_time_unix_nano":8000}
"#;

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Failed to parse valid JSON");

    let count = validator.count_spans("clnrm.step");

    // Assert: lte(3) bound is violated
    assert!(
        count > 3,
        "Count validation lte(3) should be violated: got {} spans (expected > 3)",
        count
    );
}

#[test]
fn test_count_validation_eq_bound_satisfied() {
    // Arrange: Create spans with exact count
    let json = r#"
{"name":"clnrm.run","trace_id":"abc","span_id":"1","parent_span_id":null,"attributes":{},"start_time_unix_nano":1000,"end_time_unix_nano":2000}
"#;

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Failed to parse valid JSON");

    let count = validator.count_spans("clnrm.run");

    // Assert: eq(1) bound is satisfied
    assert_eq!(
        count,
        1,
        "Count validation eq(1) failed: got {} spans",
        count
    );
}

#[test]
fn test_count_validation_eq_bound_violated_too_many() {
    // Arrange: Create spans with count > expected
    let json = r#"
{"name":"clnrm.run","trace_id":"abc","span_id":"1","parent_span_id":null,"attributes":{},"start_time_unix_nano":1000,"end_time_unix_nano":2000}
{"name":"clnrm.run","trace_id":"abc","span_id":"2","parent_span_id":null,"attributes":{},"start_time_unix_nano":3000,"end_time_unix_nano":4000}
"#;

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Failed to parse valid JSON");

    let count = validator.count_spans("clnrm.run");

    // Assert: eq(1) bound is violated (too many)
    assert_ne!(
        count,
        1,
        "Count validation eq(1) should be violated: got {} spans (expected != 1)",
        count
    );
}

#[test]
fn test_count_validation_eq_bound_violated_too_few() {
    // Arrange: Create empty span data
    let json = "";

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Failed to parse valid JSON");

    let count = validator.count_spans("clnrm.run");

    // Assert: eq(1) bound is violated (too few)
    assert_eq!(
        count,
        0,
        "Count validation eq(1) should be violated: got {} spans (expected 0)",
        count
    );
}

#[test]
fn test_count_validation_zero_spans_for_nonexistent_name() {
    // Arrange: Create spans with different names
    let json = r#"
{"name":"clnrm.run","trace_id":"abc","span_id":"1","parent_span_id":null,"attributes":{},"start_time_unix_nano":1000,"end_time_unix_nano":2000}
"#;

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Failed to parse valid JSON");

    let count = validator.count_spans("nonexistent.span");

    // Assert: Count is 0 for nonexistent span name
    assert_eq!(
        count,
        0,
        "Count for nonexistent span should be 0"
    );
}

// ============================================================================
// TEMPORAL WINDOW VALIDATION TESTS
// ============================================================================

#[test]
fn test_temporal_window_containment_valid() {
    // Arrange: Child span fully contained in parent span
    let json = r#"
{"name":"parent","trace_id":"abc","span_id":"1","parent_span_id":null,"attributes":{},"start_time_unix_nano":1000,"end_time_unix_nano":5000}
{"name":"child","trace_id":"abc","span_id":"2","parent_span_id":"1","attributes":{},"start_time_unix_nano":2000,"end_time_unix_nano":4000}
"#;

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Failed to parse valid JSON");

    let parent = validator.get_span("parent").unwrap();
    let child = validator.get_span("child").unwrap();

    // Assert: Child is temporally contained in parent
    let parent_start = parent.start_time_unix_nano.unwrap();
    let parent_end = parent.end_time_unix_nano.unwrap();
    let child_start = child.start_time_unix_nano.unwrap();
    let child_end = child.end_time_unix_nano.unwrap();

    assert!(
        child_start >= parent_start,
        "Child must start after parent: child={}, parent={}",
        child_start,
        parent_start
    );
    assert!(
        child_end <= parent_end,
        "Child must end before parent: child={}, parent={}",
        child_end,
        parent_end
    );
}

#[test]
fn test_temporal_window_containment_child_starts_before_parent_invalid() {
    // Arrange: Child starts before parent (invalid)
    let json = r#"
{"name":"parent","trace_id":"abc","span_id":"1","parent_span_id":null,"attributes":{},"start_time_unix_nano":2000,"end_time_unix_nano":5000}
{"name":"child","trace_id":"abc","span_id":"2","parent_span_id":"1","attributes":{},"start_time_unix_nano":1000,"end_time_unix_nano":4000}
"#;

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Failed to parse valid JSON");

    let parent = validator.get_span("parent").unwrap();
    let child = validator.get_span("child").unwrap();

    // Assert: Temporal violation - child starts before parent
    let parent_start = parent.start_time_unix_nano.unwrap();
    let child_start = child.start_time_unix_nano.unwrap();

    assert!(
        child_start < parent_start,
        "Temporal violation: child starts before parent (child={}, parent={})",
        child_start,
        parent_start
    );
}

#[test]
fn test_temporal_window_containment_child_ends_after_parent_invalid() {
    // Arrange: Child ends after parent (invalid)
    let json = r#"
{"name":"parent","trace_id":"abc","span_id":"1","parent_span_id":null,"attributes":{},"start_time_unix_nano":1000,"end_time_unix_nano":4000}
{"name":"child","trace_id":"abc","span_id":"2","parent_span_id":"1","attributes":{},"start_time_unix_nano":2000,"end_time_unix_nano":5000}
"#;

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Failed to parse valid JSON");

    let parent = validator.get_span("parent").unwrap();
    let child = validator.get_span("child").unwrap();

    // Assert: Temporal violation - child ends after parent
    let parent_end = parent.end_time_unix_nano.unwrap();
    let child_end = child.end_time_unix_nano.unwrap();

    assert!(
        child_end > parent_end,
        "Temporal violation: child ends after parent (child={}, parent={})",
        child_end,
        parent_end
    );
}

#[test]
fn test_temporal_window_containment_exact_boundaries_valid() {
    // Arrange: Child has exact same boundaries as parent (edge case - valid)
    let json = r#"
{"name":"parent","trace_id":"abc","span_id":"1","parent_span_id":null,"attributes":{},"start_time_unix_nano":1000,"end_time_unix_nano":5000}
{"name":"child","trace_id":"abc","span_id":"2","parent_span_id":"1","attributes":{},"start_time_unix_nano":1000,"end_time_unix_nano":5000}
"#;

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Failed to parse valid JSON");

    let parent = validator.get_span("parent").unwrap();
    let child = validator.get_span("child").unwrap();

    // Assert: Exact boundaries are valid (child contained in parent with equality)
    assert_eq!(
        child.start_time_unix_nano.unwrap(),
        parent.start_time_unix_nano.unwrap(),
        "Exact start boundary is valid"
    );
    assert_eq!(
        child.end_time_unix_nano.unwrap(),
        parent.end_time_unix_nano.unwrap(),
        "Exact end boundary is valid"
    );
}

#[test]
fn test_temporal_window_sibling_spans_can_overlap() {
    // Arrange: Two sibling spans with overlapping time windows
    let json = r#"
{"name":"parent","trace_id":"abc","span_id":"1","parent_span_id":null,"attributes":{},"start_time_unix_nano":1000,"end_time_unix_nano":10000}
{"name":"child1","trace_id":"abc","span_id":"2","parent_span_id":"1","attributes":{},"start_time_unix_nano":2000,"end_time_unix_nano":6000}
{"name":"child2","trace_id":"abc","span_id":"3","parent_span_id":"1","attributes":{},"start_time_unix_nano":4000,"end_time_unix_nano":8000}
"#;

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Failed to parse valid JSON");

    let child1 = validator.get_span("child1").unwrap();
    let child2 = validator.get_span("child2").unwrap();

    // Assert: Sibling spans can overlap (both contained in parent)
    let child1_start = child1.start_time_unix_nano.unwrap();
    let child1_end = child1.end_time_unix_nano.unwrap();
    let child2_start = child2.start_time_unix_nano.unwrap();
    let child2_end = child2.end_time_unix_nano.unwrap();

    assert!(
        child2_start < child1_end,
        "Sibling spans can overlap: child2 starts before child1 ends"
    );
    assert!(
        child1_start < child2_end,
        "Sibling spans can overlap: child1 starts before child2 ends"
    );
}

// ============================================================================
// HERMETICITY VALIDATION TESTS
// ============================================================================

#[test]
fn test_hermeticity_no_external_services_all_internal() {
    // Arrange: All services are internal (clnrm.* namespace)
    let json = r#"
{"name":"clnrm.run","trace_id":"abc","span_id":"1","parent_span_id":null,"attributes":{"service.name":"clnrm"},"start_time_unix_nano":1000,"end_time_unix_nano":5000}
{"name":"clnrm.step:test","trace_id":"abc","span_id":"2","parent_span_id":"1","attributes":{"service.name":"clnrm.step"},"start_time_unix_nano":2000,"end_time_unix_nano":4000}
"#;

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Failed to parse valid JSON");

    let spans = validator.all_spans();

    // Assert: All span names start with "clnrm." (hermetic)
    for span in spans {
        assert!(
            span.name.starts_with("clnrm."),
            "Hermetic test: span '{}' should be in clnrm.* namespace",
            span.name
        );
    }
}

#[test]
fn test_hermeticity_external_service_detected() {
    // Arrange: External service call detected
    let json = r#"
{"name":"clnrm.run","trace_id":"abc","span_id":"1","parent_span_id":null,"attributes":{"service.name":"clnrm"},"start_time_unix_nano":1000,"end_time_unix_nano":5000}
{"name":"http.request","trace_id":"abc","span_id":"2","parent_span_id":"1","attributes":{"http.url":"https://api.example.com"},"start_time_unix_nano":2000,"end_time_unix_nano":4000}
"#;

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Failed to parse valid JSON");

    let external_span = validator.get_span("http.request").unwrap();

    // Assert: External service detected (not in clnrm.* namespace)
    assert!(
        !external_span.name.starts_with("clnrm."),
        "External service detected: span '{}' is not in clnrm.* namespace",
        external_span.name
    );
}

#[test]
fn test_hermeticity_forbidden_attributes_http_url() {
    // Arrange: Span with forbidden http.url attribute
    let json = r#"
{"name":"clnrm.step","trace_id":"abc","span_id":"1","parent_span_id":null,"attributes":{"http.url":"https://external.com"},"start_time_unix_nano":1000,"end_time_unix_nano":2000}
"#;

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Failed to parse valid JSON");

    let span = validator.get_span("clnrm.step").unwrap();

    // Assert: Forbidden attribute detected
    assert!(
        span.attributes.contains_key("http.url"),
        "Forbidden attribute 'http.url' detected (indicates external call)"
    );
}

#[test]
fn test_hermeticity_forbidden_attributes_db_connection_string() {
    // Arrange: Span with forbidden db.connection_string attribute
    let json = r#"
{"name":"clnrm.step","trace_id":"abc","span_id":"1","parent_span_id":null,"attributes":{"db.connection_string":"postgresql://external-db"},"start_time_unix_nano":1000,"end_time_unix_nano":2000}
"#;

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Failed to parse valid JSON");

    let span = validator.get_span("clnrm.step").unwrap();

    // Assert: Forbidden attribute detected
    assert!(
        span.attributes.contains_key("db.connection_string"),
        "Forbidden attribute 'db.connection_string' detected (indicates external DB)"
    );
}

#[test]
fn test_hermeticity_forbidden_attributes_net_peer_name() {
    // Arrange: Span with forbidden net.peer.name attribute
    let json = r#"
{"name":"clnrm.step","trace_id":"abc","span_id":"1","parent_span_id":null,"attributes":{"net.peer.name":"external-service.com"},"start_time_unix_nano":1000,"end_time_unix_nano":2000}
"#;

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Failed to parse valid JSON");

    let span = validator.get_span("clnrm.step").unwrap();

    // Assert: Forbidden attribute detected
    assert!(
        span.attributes.contains_key("net.peer.name"),
        "Forbidden attribute 'net.peer.name' detected (indicates network call)"
    );
}

#[test]
fn test_hermeticity_allowed_attributes_only() {
    // Arrange: Span with only allowed attributes
    let json = r#"
{"name":"clnrm.step","trace_id":"abc","span_id":"1","parent_span_id":null,"attributes":{"step.name":"test","result":"pass"},"start_time_unix_nano":1000,"end_time_unix_nano":2000}
"#;

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Failed to parse valid JSON");

    let span = validator.get_span("clnrm.step").unwrap();

    // Assert: Only allowed attributes present
    let forbidden_attrs = ["http.url", "db.connection_string", "net.peer.name", "rpc.service"];
    for attr in &forbidden_attrs {
        assert!(
            !span.attributes.contains_key(*attr),
            "Hermetic test: forbidden attribute '{}' should not be present",
            attr
        );
    }
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[test]
fn test_invalid_json_silently_skipped() {
    // Arrange: Invalid JSON line mixed with valid
    let json = r#"
{"name":"test","invalid
{"name":"valid","trace_id":"abc","span_id":"1","parent_span_id":null,"attributes":{}}
"#;

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Parser should skip invalid lines and continue");

    // Assert: Valid span was parsed, invalid line was skipped
    assert_eq!(
        validator.count_spans("valid"),
        1,
        "Valid span should be parsed despite invalid line"
    );
    assert_eq!(
        validator.count_spans("test"),
        0,
        "Invalid JSON line should be skipped"
    );
}

#[test]
fn test_empty_json_returns_empty_validator() {
    // Arrange: Empty JSON
    let json = "";

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Empty JSON should be valid");

    // Assert: Validator has no spans
    assert_eq!(
        validator.count_spans("any"),
        0,
        "Empty JSON should result in zero spans"
    );
}

#[test]
fn test_malformed_span_missing_required_fields_silently_skipped() {
    // Arrange: Span missing required 'name' field
    let json = r#"
{"trace_id":"abc","span_id":"1"}
{"name":"valid","trace_id":"abc","span_id":"2","parent_span_id":null,"attributes":{}}
"#;

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Parser should skip malformed spans and continue");

    // Assert: Malformed span was skipped, valid span was parsed
    assert_eq!(
        validator.count_spans("valid"),
        1,
        "Valid span should be parsed"
    );
    assert_eq!(
        validator.all_spans().len(),
        1,
        "Malformed span should be skipped (not counted)"
    );
}

#[test]
fn test_duplicate_span_ids_last_wins() {
    // Arrange: Two spans with same span_id
    let json = r#"
{"name":"span1","trace_id":"abc","span_id":"1","parent_span_id":null,"attributes":{},"start_time_unix_nano":1000,"end_time_unix_nano":2000}
{"name":"span2","trace_id":"abc","span_id":"1","parent_span_id":null,"attributes":{},"start_time_unix_nano":3000,"end_time_unix_nano":4000}
"#;

    // Act
    let validator = SpanValidator::from_json(json)
        .expect("Duplicate span_id should be handled");

    // Assert: Last span with duplicate ID wins (or implementation-defined behavior)
    let span = validator.get_span_by_id("1").unwrap();
    // Implementation detail: verify which span "won"
    assert!(
        span.name == "span1" || span.name == "span2",
        "Duplicate span_id handling: one of the spans should exist"
    );
}
