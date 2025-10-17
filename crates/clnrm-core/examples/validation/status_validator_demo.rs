//! Status Validator Demo
//!
//! Demonstrates span status validation with glob pattern matching.
//!
//! Run with:
//! ```
//! cargo run --example status_validator_demo
//! ```

use clnrm_core::error::Result;
use clnrm_core::validation::{SpanData, StatusCode, StatusExpectation};
use serde_json::json;
use std::collections::HashMap;

fn create_span(name: &str, status: &str) -> SpanData {
    let mut attrs = HashMap::new();
    attrs.insert("otel.status_code".to_string(), json!(status));

    SpanData {
        name: name.to_string(),
        trace_id: "demo_trace".to_string(),
        span_id: format!("span_{}", name),
        parent_span_id: None,
        start_time_unix_nano: Some(1000),
        end_time_unix_nano: Some(2000),
        attributes: attrs,
        kind: None,
        events: None,
        resource_attributes: HashMap::new(),
    }
}

fn main() -> Result<()> {
    println!("=== Status Validator Demo ===\n");

    // Example 1: Validate all spans have OK status
    println!("Example 1: All spans OK");
    let spans = vec![
        create_span("clnrm.init", "OK"),
        create_span("clnrm.run", "OK"),
        create_span("clnrm.cleanup", "OK"),
    ];

    let expectation = StatusExpectation::new().with_all(StatusCode::Ok);
    match expectation.validate(&spans) {
        Ok(_) => println!("✓ All spans have OK status"),
        Err(e) => println!("✗ Validation failed: {}", e),
    }

    // Example 2: Glob pattern matching
    println!("\nExample 2: Glob pattern matching");
    let spans = vec![
        create_span("clnrm.test.unit", "OK"),
        create_span("clnrm.test.integration", "OK"),
        create_span("clnrm.cleanup", "OK"),
        create_span("external.api.call", "ERROR"),
    ];

    let expectation = StatusExpectation::new()
        .with_name_pattern("clnrm.test.*".to_string(), StatusCode::Ok)
        .with_name_pattern("external.*".to_string(), StatusCode::Error);

    match expectation.validate(&spans) {
        Ok(_) => println!("✓ All patterns matched expected status"),
        Err(e) => println!("✗ Validation failed: {}", e),
    }

    // Example 3: Wildcard patterns
    println!("\nExample 3: Wildcard patterns with ?");
    let spans = vec![
        create_span("test_1", "OK"),
        create_span("test_2", "OK"),
        create_span("test_3", "OK"),
    ];

    let expectation =
        StatusExpectation::new().with_name_pattern("test_?".to_string(), StatusCode::Ok);

    match expectation.validate(&spans) {
        Ok(_) => println!("✓ All wildcard matches have OK status"),
        Err(e) => println!("✗ Validation failed: {}", e),
    }

    // Example 4: Multiple pattern validation
    println!("\nExample 4: Multiple patterns");
    let spans = vec![
        create_span("backend.database.query", "OK"),
        create_span("backend.cache.set", "OK"),
        create_span("frontend.render", "OK"),
        create_span("error.handler", "ERROR"),
    ];

    let expectation = StatusExpectation::new()
        .with_name_pattern("backend.*".to_string(), StatusCode::Ok)
        .with_name_pattern("frontend.*".to_string(), StatusCode::Ok)
        .with_name_pattern("error.*".to_string(), StatusCode::Error);

    match expectation.validate(&spans) {
        Ok(_) => println!("✓ All multi-pattern validations passed"),
        Err(e) => println!("✗ Validation failed: {}", e),
    }

    // Example 5: Error case - status mismatch
    println!("\nExample 5: Error case - status mismatch");
    let spans = vec![
        create_span("clnrm.test", "OK"),
        create_span("clnrm.fail", "ERROR"), // This will fail
    ];

    let expectation = StatusExpectation::new().with_all(StatusCode::Ok);
    match expectation.validate(&spans) {
        Ok(_) => println!("✓ Validation passed"),
        Err(e) => println!("✗ Expected error: {}", e),
    }

    // Example 6: Error case - no matches for pattern
    println!("\nExample 6: Error case - no matches for pattern");
    let spans = vec![
        create_span("other.test", "OK"),
        create_span("another.test", "OK"),
    ];

    let expectation =
        StatusExpectation::new().with_name_pattern("clnrm.*".to_string(), StatusCode::Ok);
    match expectation.validate(&spans) {
        Ok(_) => println!("✓ Validation passed"),
        Err(e) => println!("✗ Expected error: {}", e),
    }

    // Example 7: Complex real-world scenario
    println!("\nExample 7: Complex real-world scenario");
    let spans = vec![
        // Successful operations
        create_span("clnrm.container.create", "OK"),
        create_span("clnrm.container.start", "OK"),
        create_span("clnrm.test.execute", "OK"),
        create_span("clnrm.container.stop", "OK"),
        // Expected errors (chaos testing)
        create_span("chaos.network.latency", "ERROR"),
        create_span("chaos.disk.full", "ERROR"),
        // Monitoring (unset status)
        create_span("monitor.health.check", "UNSET"),
    ];

    let expectation = StatusExpectation::new()
        .with_name_pattern("clnrm.container.*".to_string(), StatusCode::Ok)
        .with_name_pattern("clnrm.test.*".to_string(), StatusCode::Ok)
        .with_name_pattern("chaos.*".to_string(), StatusCode::Error)
        .with_name_pattern("monitor.*".to_string(), StatusCode::Unset);

    match expectation.validate(&spans) {
        Ok(_) => println!("✓ Complex scenario validated successfully"),
        Err(e) => println!("✗ Validation failed: {}", e),
    }

    println!("\n=== Demo Complete ===");
    Ok(())
}
