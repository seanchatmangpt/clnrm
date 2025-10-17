//! Standalone test for Issue #7 - Advanced Macro Library
//!
//! This test validates all 8 advanced macros work correctly without
//! depending on other parts of the codebase that may have compilation issues.

use tera::Tera;

#[test]
fn test_all_8_advanced_macros_are_available() {
    // Arrange - Load macro library
    let macro_content = include_str!("../crates/clnrm-core/src/template/_macros.toml.tera");
    let mut tera = Tera::default();
    tera.add_raw_template("_macros.toml.tera", macro_content)
        .expect("Failed to load macro library");

    // Act & Assert - Test each macro individually

    // 1. span_exists
    let template1 = r#"{% import "_macros.toml.tera" as m %}{{ m::span_exists("test.span") }}"#;
    let result1 = tera.render_str(template1, &tera::Context::new()).unwrap();
    assert!(result1.contains("[[expect.span]]"));
    assert!(result1.contains("name = \"test.span\""));
    assert!(result1.contains("exists = true"));

    // 2. graph_relationship
    let template2 = r#"{% import "_macros.toml.tera" as m %}{{ m::graph_relationship("parent", "child") }}"#;
    let result2 = tera.render_str(template2, &tera::Context::new()).unwrap();
    assert!(result2.contains("[[expect.graph]]"));
    assert!(result2.contains("parent = \"parent\""));
    assert!(result2.contains("child = \"child\""));
    assert!(result2.contains("relationship = \"calls\""));

    // 3. temporal_ordering
    let template3 = r#"{% import "_macros.toml.tera" as m %}{{ m::temporal_ordering("first", "second") }}"#;
    let result3 = tera.render_str(template3, &tera::Context::new()).unwrap();
    assert!(result3.contains("[[expect.temporal]]"));
    assert!(result3.contains("before = \"first\""));
    assert!(result3.contains("after = \"second\""));

    // 4. error_propagation
    let template4 = r#"{% import "_macros.toml.tera" as m %}{{ m::error_propagation("source", "target") }}"#;
    let result4 = tera.render_str(template4, &tera::Context::new()).unwrap();
    assert_eq!(result4.matches("[[expect.span]]").count(), 2);
    assert!(result4.contains("name = \"source\""));
    assert!(result4.contains("name = \"target\""));
    assert!(result4.contains("error.source"));

    // 5. service_interaction
    let template5 = r#"{% import "_macros.toml.tera" as m %}{{ m::service_interaction("client", "server") }}"#;
    let result5 = tera.render_str(template5, &tera::Context::new()).unwrap();
    assert!(result5.contains("[[expect.graph]]"));
    assert!(result5.contains("http.method"));
    assert!(result5.contains("POST"));

    // 6. attribute_validation
    let template6 = r#"{% import "_macros.toml.tera" as m %}{{ m::attribute_validation("span", "key", "value") }}"#;
    let result6 = tera.render_str(template6, &tera::Context::new()).unwrap();
    assert!(result6.contains("[[expect.span]]"));
    assert!(result6.contains("name = \"span\""));
    assert!(result6.contains("attrs.all = { \"key\" = \"value\" }"));

    // 7. resource_check
    let template7 = r#"{% import "_macros.toml.tera" as m %}{{ m::resource_check("container", "test_container") }}"#;
    let result7 = tera.render_str(template7, &tera::Context::new()).unwrap();
    assert!(result7.contains("[[expect.resource]]"));
    assert!(result7.contains("type = \"container\""));
    assert!(result7.contains("name = \"test_container\""));
    assert!(result7.contains("exists = true"));

    // 8. batch_validation
    let template8 = r#"{% import "_macros.toml.tera" as m %}{{ m::batch_validation(["s1", "s2"], "exists = true") }}"#;
    let result8 = tera.render_str(template8, &tera::Context::new()).unwrap();
    assert_eq!(result8.matches("[[expect.span]]").count(), 2);
    assert!(result8.contains("name = \"s1\""));
    assert!(result8.contains("name = \"s2\""));
}

#[test]
fn test_comprehensive_template_with_all_advanced_macros() {
    // Arrange
    let macro_content = include_str!("../crates/clnrm-core/src/template/_macros.toml.tera");
    let mut tera = Tera::default();
    tera.add_raw_template("_macros.toml.tera", macro_content)
        .expect("Failed to load macro library");

    let comprehensive_template = r#"
{% import "_macros.toml.tera" as m %}
[test.metadata]
name = "comprehensive-test"

{{ m::service("postgres", "postgres:15") }}
{{ m::service("api", "nginx:alpine") }}

{{ m::scenario("test", "api", "echo test") }}

{{ m::span_exists("http.server") }}
{{ m::graph_relationship("api", "db") }}
{{ m::temporal_ordering("step1", "step2") }}
{{ m::error_propagation("source", "target") }}
{{ m::service_interaction("client", "server", method="GET") }}
{{ m::attribute_validation("span", "key", "value") }}
{{ m::resource_check("container", "postgres_db") }}
{{ m::batch_validation(["span1", "span2"], "exists = true") }}
"#;

    // Act
    let result = tera.render_str(comprehensive_template, &tera::Context::new());

    // Assert
    assert!(result.is_ok(), "Template rendering failed: {:?}", result.err());
    let output = result.unwrap();

    // Verify all sections present
    assert!(output.contains("[test.metadata]"));
    assert!(output.contains("[service.postgres]"));
    assert!(output.contains("[service.api]"));
    assert!(output.contains("[[scenario]]"));

    // Verify all 8 advanced macros produced output
    assert!(output.contains("[[expect.span]]"));
    assert!(output.contains("[[expect.graph]]"));
    assert!(output.contains("[[expect.temporal]]"));
    assert!(output.contains("error.source"));
    assert!(output.contains("http.method"));
    assert!(output.contains("attrs.all"));
    assert!(output.contains("[[expect.resource]]"));

    // Count key patterns
    let span_count = output.matches("[[expect.span]]").count();
    assert!(span_count >= 6, "Expected at least 6 span blocks, got {}", span_count);

    let graph_count = output.matches("[[expect.graph]]").count();
    assert!(graph_count >= 2, "Expected at least 2 graph blocks, got {}", graph_count);

    println!("✓ All 8 advanced macros validated successfully!");
    println!("✓ Generated {} span expectations", span_count);
    println!("✓ Generated {} graph relationships", graph_count);
}

#[test]
fn test_macro_library_backwards_compatibility() {
    // Ensure original 3 macros still work
    let macro_content = include_str!("../crates/clnrm-core/src/template/_macros.toml.tera");
    let mut tera = Tera::default();
    tera.add_raw_template("_macros.toml.tera", macro_content).unwrap();

    // Test original span macro
    let template = r#"
{% import "_macros.toml.tera" as m %}
{{ m::span("test.span") }}
{{ m::service("test", "alpine:latest") }}
{{ m::scenario("test", "svc", "cmd") }}
"#;

    let result = tera.render_str(template, &tera::Context::new());
    assert!(result.is_ok(), "Original macros broken: {:?}", result.err());

    let output = result.unwrap();
    assert!(output.contains("[[expect.span]]"));
    assert!(output.contains("[service.test]"));
    assert!(output.contains("[[scenario]]"));

    println!("✓ Original 3 macros remain fully functional");
}
