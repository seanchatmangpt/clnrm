//! Demonstration of PRD-aligned span validator functionality
//!
//! This example shows how to use the new span validation features
//! that align with the OTEL PRD schema.

use clnrm_core::validation::span_validator::{SpanAssertion, SpanKind, SpanValidator};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Span Validator PRD Extension Demo ===\n");

    // Example 1: SpanKind Validation
    println!("1. SpanKind Validation");
    let json_server_span = r#"{"name":"api.request","trace_id":"abc123","span_id":"span1","parent_span_id":null,"attributes":{},"kind":2}"#;
    let validator = SpanValidator::from_json(json_server_span)?;

    let assertion = SpanAssertion::SpanKind {
        name: "api.request".to_string(),
        kind: SpanKind::Server,
    };

    match validator.validate_assertion(&assertion) {
        Ok(_) => println!("   ✓ Server span kind validated successfully"),
        Err(e) => println!("   ✗ Validation failed: {}", e),
    }

    // Example 2: SpanAllAttributes Validation
    println!("\n2. SpanAllAttributes Validation (attrs.all)");
    let json_with_attrs = r#"{"name":"http.request","trace_id":"abc123","span_id":"span1","parent_span_id":null,"attributes":{"http.method":{"stringValue":"GET"},"http.status_code":{"stringValue":"200"},"service.name":{"stringValue":"api"}}}"#;
    let validator = SpanValidator::from_json(json_with_attrs)?;

    let mut required_attrs = HashMap::new();
    required_attrs.insert("http.method".to_string(), "GET".to_string());
    required_attrs.insert("http.status_code".to_string(), "200".to_string());

    let assertion = SpanAssertion::SpanAllAttributes {
        name: "http.request".to_string(),
        attributes: required_attrs,
    };

    match validator.validate_assertion(&assertion) {
        Ok(_) => println!("   ✓ All required attributes present"),
        Err(e) => println!("   ✗ Validation failed: {}", e),
    }

    // Example 3: SpanAnyAttributes Validation
    println!("\n3. SpanAnyAttributes Validation (attrs.any)");
    let json_any = r#"{"name":"database.query","trace_id":"abc123","span_id":"span1","parent_span_id":null,"attributes":{"db.operation":{"stringValue":"SELECT"},"db.name":{"stringValue":"users"}}}"#;
    let validator = SpanValidator::from_json(json_any)?;

    let patterns = vec![
        "db.operation=SELECT".to_string(),
        "db.operation=INSERT".to_string(),
    ];

    let assertion = SpanAssertion::SpanAnyAttributes {
        name: "database.query".to_string(),
        attribute_patterns: patterns,
    };

    match validator.validate_assertion(&assertion) {
        Ok(_) => println!("   ✓ At least one attribute pattern matched"),
        Err(e) => println!("   ✗ Validation failed: {}", e),
    }

    // Example 4: SpanEvents Validation
    println!("\n4. SpanEvents Validation (events.any)");
    let json_events = r#"{"name":"service.call","trace_id":"abc123","span_id":"span1","parent_span_id":null,"attributes":{},"events":[{"name":"exception"},{"name":"retry"}]}"#;
    let validator = SpanValidator::from_json(json_events)?;

    let assertion = SpanAssertion::SpanEvents {
        name: "service.call".to_string(),
        events: vec!["exception".to_string(), "timeout".to_string()],
    };

    match validator.validate_assertion(&assertion) {
        Ok(_) => println!("   ✓ Required event found in span"),
        Err(e) => println!("   ✗ Validation failed: {}", e),
    }

    // Example 5: SpanDuration Validation
    println!("\n5. SpanDuration Validation (duration_ms)");
    let json_duration = r#"{"name":"api.response","trace_id":"abc123","span_id":"span1","parent_span_id":null,"attributes":{},"start_time_unix_nano":"1000000000","end_time_unix_nano":"1150000000"}"#;
    let validator = SpanValidator::from_json(json_duration)?;

    let assertion = SpanAssertion::SpanDuration {
        name: "api.response".to_string(),
        min_ms: Some(100),
        max_ms: Some(200),
    };

    match validator.validate_assertion(&assertion) {
        Ok(_) => println!("   ✓ Span duration within acceptable bounds (100-200ms)"),
        Err(e) => println!("   ✗ Validation failed: {}", e),
    }

    // Example 6: SpanKind Enum Parsing
    println!("\n6. SpanKind Enum Features");
    let kinds = vec!["internal", "server", "client", "producer", "consumer"];
    for kind_str in kinds {
        match SpanKind::from_str(kind_str) {
            Ok(kind) => {
                let otel_int = kind.to_otel_int();
                println!("   ✓ '{}' → SpanKind::{:?} → OTEL int: {}", kind_str, kind, otel_int);
            }
            Err(e) => println!("   ✗ Failed to parse '{}': {}", kind_str, e),
        }
    }

    println!("\n=== Demo Complete ===");
    println!("\nAll PRD-aligned span validation features demonstrated successfully!");
    println!("\nFeatures:");
    println!("  • SpanKind - Validate span type (internal/server/client/producer/consumer)");
    println!("  • SpanAllAttributes - All specified attributes must be present (attrs.all)");
    println!("  • SpanAnyAttributes - At least one attribute must match (attrs.any)");
    println!("  • SpanEvents - At least one event must be present (events.any)");
    println!("  • SpanDuration - Validate span duration bounds (duration_ms)");

    Ok(())
}
