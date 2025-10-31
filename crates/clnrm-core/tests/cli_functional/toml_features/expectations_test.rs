//! Expectations configuration tests
//!
//! Tests validation expectations parsing (high-value feature)

use clnrm_core::config::parse_toml_config;
use clnrm_core::error::Result;

#[test]
fn test_span_expectations_parse() -> Result<()> {
    // Arrange - Create TOML with span expectations
    let toml_content = r#"
[meta]
name = "test_expectations"
version = "1.0.0"

[service.test_service]
plugin = "generic_container"
image = "alpine:latest"

[[scenario]]
name = "test_scenario"
service = "test_service"
run = "echo test"

[[expect.span]]
name = "test.span"
kind = "internal"
attrs.all = { "test.framework" = "clnrm" }
"#;

    // Act - Parse TOML config
    let config = parse_toml_config(toml_content)?;

    // Assert - Verify span expectations parsed
    assert!(
        config.expect.is_some(),
        "BEHAVIOR: [expect] section should be parsed"
    );
    let expect = config.expect.as_ref().unwrap();
    assert!(
        !expect.span.is_empty(),
        "BEHAVIOR: [[expect.span]] entries should be parsed"
    );
    let span_expect = &expect.span[0];
    assert_eq!(span_expect.name, "test.span");
    assert_eq!(span_expect.kind.as_ref().unwrap(), "internal");

    Ok(())
}

#[test]
fn test_count_expectations_parse() -> Result<()> {
    // Arrange - Create TOML with count expectations
    let toml_content = r#"
[meta]
name = "test_counts"
version = "1.0.0"

[service.test_service]
plugin = "generic_container"
image = "alpine:latest"

[[scenario]]
name = "test_scenario"
service = "test_service"
run = "echo test"

[expect.counts]
spans_total = { gte = 1 }
errors_total = { eq = 0 }
by_name = { "test.span" = { eq = 2 } }
"#;

    // Act - Parse TOML config
    let config = parse_toml_config(toml_content)?;

    // Assert - Verify count expectations parsed
    assert!(config.expect.is_some());
    let expect = config.expect.as_ref().unwrap();
    assert!(
        expect.counts.is_some(),
        "BEHAVIOR: [expect.counts] section should be parsed"
    );
    let counts = expect.counts.as_ref().unwrap();
    assert!(
        counts.spans_total.is_some(),
        "BEHAVIOR: spans_total count bounds should be parsed"
    );

    Ok(())
}

#[test]
fn test_graph_expectations_parse() -> Result<()> {
    // Arrange - Create TOML with graph expectations
    let toml_content = r#"
[meta]
name = "test_graph"
version = "1.0.0"

[service.test_service]
plugin = "generic_container"
image = "alpine:latest"

[[scenario]]
name = "test_scenario"
service = "test_service"
run = "echo test"

[expect.graph]
must_include = [["parent", "child"]]
acyclic = true
"#;

    // Act - Parse TOML config
    let config = parse_toml_config(toml_content)?;

    // Assert - Verify graph expectations parsed
    assert!(config.expect.is_some());
    let expect = config.expect.as_ref().unwrap();
    assert!(
        expect.graph.is_some(),
        "BEHAVIOR: [expect.graph] section should be parsed"
    );
    let graph = expect.graph.as_ref().unwrap();
    assert!(
        graph.must_include.is_some(),
        "BEHAVIOR: Graph edges should be parsed"
    );

    Ok(())
}

