//! Example demonstrating GraphAnalyzer (GraphValidator) usage
//!
//! This example shows how to validate span graph topology using the
//! GraphValidator to ensure correct parent-child relationships,
//! forbidden edges, and acyclic graphs.

use clnrm_core::validation::{GraphExpectation, GraphValidator, SpanData};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Graph Validation Example ===\n");

    // Example 1: Valid Linear Chain
    println!("Example 1: Valid Linear Chain");
    example_valid_linear_chain()?;
    println!();

    // Example 2: Required Edge Missing
    println!("Example 2: Required Edge Missing (Error)");
    example_missing_required_edge();
    println!();

    // Example 3: Forbidden Edge Present
    println!("Example 3: Forbidden Edge Present (Error)");
    example_forbidden_edge_present();
    println!();

    // Example 4: Cycle Detection
    println!("Example 4: Cycle Detection (Error)");
    example_cycle_detection();
    println!();

    // Example 5: Complex Multi-Service Architecture
    println!("Example 5: Complex Multi-Service Architecture");
    example_complex_architecture()?;
    println!();

    Ok(())
}

fn create_span(name: &str, span_id: &str, parent_id: Option<&str>) -> SpanData {
    SpanData {
        name: name.to_string(),
        span_id: span_id.to_string(),
        parent_span_id: parent_id.map(|s| s.to_string()),
        trace_id: "trace123".to_string(),
        attributes: HashMap::new(),
        start_time_unix_nano: Some(1000000),
        end_time_unix_nano: Some(2000000),
        kind: None,
        events: None,
        resource_attributes: HashMap::new(),
    }
}

fn example_valid_linear_chain() -> Result<(), Box<dyn std::error::Error>> {
    // Create a linear chain: clnrm.run -> clnrm.test -> clnrm.step
    let spans = vec![
        create_span("clnrm.run", "span1", None),
        create_span("clnrm.test", "span2", Some("span1")),
        create_span("clnrm.step", "span3", Some("span2")),
    ];

    let expectation = GraphExpectation::new(vec![
        ("clnrm.run".to_string(), "clnrm.test".to_string()),
        ("clnrm.test".to_string(), "clnrm.step".to_string()),
    ])
    .with_acyclic(true);

    expectation.validate(&spans)?;
    println!("✅ Linear chain validated successfully");
    println!("   clnrm.run -> clnrm.test -> clnrm.step");

    Ok(())
}

fn example_missing_required_edge() {
    // Create spans without required edge
    let spans = vec![
        create_span("clnrm.run", "span1", None),
        create_span("clnrm.test", "span2", None), // Missing parent relationship
    ];

    let expectation =
        GraphExpectation::new(vec![("clnrm.run".to_string(), "clnrm.test".to_string())]);

    match expectation.validate(&spans) {
        Ok(_) => println!("❌ Validation should have failed"),
        Err(e) => println!("✅ Expected error: {}", e),
    }
}

fn example_forbidden_edge_present() {
    // Create spans with forbidden edge
    let spans = vec![
        create_span("clnrm.step", "span1", None),
        create_span("clnrm.plugin.registry", "span2", Some("span1")), // Forbidden!
    ];

    let expectation = GraphExpectation::new(vec![]).with_must_not_cross(vec![(
        "clnrm.step".to_string(),
        "clnrm.plugin.registry".to_string(),
    )]);

    match expectation.validate(&spans) {
        Ok(_) => println!("❌ Validation should have failed"),
        Err(e) => println!("✅ Expected error: {}", e),
    }
}

fn example_cycle_detection() {
    // Create a cycle: a -> b -> c -> a
    let spans = vec![
        create_span("span_a", "a", Some("c")), // Points back to c
        create_span("span_b", "b", Some("a")),
        create_span("span_c", "c", Some("b")),
    ];

    let expectation = GraphExpectation::new(vec![]).with_acyclic(true);

    match expectation.validate(&spans) {
        Ok(_) => println!("❌ Validation should have failed"),
        Err(e) => println!("✅ Expected error: {}", e),
    }
}

fn example_complex_architecture() -> Result<(), Box<dyn std::error::Error>> {
    // Simulate a complex microservice architecture
    let spans = vec![
        // Entry point
        create_span("http.server.request", "span1", None),
        // Authentication
        create_span("auth.verify", "span2", Some("span1")),
        create_span("auth.jwt.decode", "span3", Some("span2")),
        // Business logic
        create_span("order.process", "span4", Some("span1")),
        create_span("inventory.check", "span5", Some("span4")),
        create_span("payment.charge", "span6", Some("span4")),
        // Database operations
        create_span("db.query.inventory", "span7", Some("span5")),
        create_span("db.query.payment", "span8", Some("span6")),
        // External API
        create_span("http.client.stripe", "span9", Some("span6")),
    ];

    let expectation = GraphExpectation::new(vec![
        // Must have these edges
        (
            "http.server.request".to_string(),
            "auth.verify".to_string(),
        ),
        ("http.server.request".to_string(), "order.process".to_string()),
        ("order.process".to_string(), "inventory.check".to_string()),
        ("order.process".to_string(), "payment.charge".to_string()),
    ])
    .with_must_not_cross(vec![
        // Business logic should not directly access database
        ("order.process".to_string(), "db.query.inventory".to_string()),
        ("order.process".to_string(), "db.query.payment".to_string()),
    ])
    .with_acyclic(true);

    expectation.validate(&spans)?;

    // Demonstrate edge enumeration
    let validator = GraphValidator::new(&spans);
    let edges = validator.get_all_edges();

    println!("✅ Complex architecture validated successfully");
    println!("   Total spans: {}", spans.len());
    println!("   Total edges: {}", edges.len());
    println!("   Architecture:");
    println!("   - HTTP Request");
    println!("     ├─ Authentication");
    println!("     │  └─ JWT Decode");
    println!("     └─ Order Processing");
    println!("        ├─ Inventory Check → DB Query");
    println!("        └─ Payment Charge → DB Query + Stripe API");

    Ok(())
}
