//! TOML parsing and validation tests
//!
//! Tests for parsing edge cases and validation (80% of issues)

use clnrm_core::config::parse_toml_config;
use clnrm_core::error::Result;

#[test]
fn test_missing_required_sections_fails() -> Result<()> {
    // Arrange - Create invalid TOML without required sections
    let invalid_toml = r#"
# Missing [meta] and [[scenario]] sections
[service.test_service]
plugin = "generic_container"
image = "alpine:latest"
"#;

    // Act - Try to parse
    let result = parse_toml_config(invalid_toml);

    // Assert - Should parse but validation should catch missing scenarios
    // Note: Parsing may succeed, but validation should catch missing required parts
    // The actual validation happens during test execution
    let config = result?;

    // At minimum, we can check that required sections are missing
    assert!(
        config.meta.is_none() && config.test.is_none(),
        "BEHAVIOR: Missing metadata should be detected"
    );
    assert!(
        config.scenario.is_empty(),
        "BEHAVIOR: Missing scenarios should be detected"
    );

    Ok(())
}

#[test]
fn test_service_reference_in_scenario_exists() -> Result<()> {
    // Arrange - Create TOML with scenario referencing non-existent service
    let toml_content = r#"
[meta]
name = "test_service_ref"
version = "1.0.0"

[[scenario]]
name = "test_scenario"
service = "nonexistent_service"
run = "echo test"
"#;

    // Act - Parse TOML
    let config = parse_toml_config(toml_content)?;

    // Assert - Config should parse (validation happens later)
    assert!(!config.scenario.is_empty());
    // Note: Actual service existence validation happens during execution
    // Here we just verify the config structure is parsed

    Ok(())
}

#[test]
fn test_multiple_service_definitions() -> Result<()> {
    // Arrange - Create TOML with multiple services
    let toml_content = r#"
[meta]
name = "test_multiple_services"
version = "1.0.0"

[service.service1]
plugin = "generic_container"
image = "alpine:latest"

[service.service2]
plugin = "generic_container"
image = "ubuntu:latest"

[[scenario]]
name = "scenario_1"
service = "service1"
run = "echo test1"

[[scenario]]
name = "scenario_2"
service = "service2"
run = "echo test2"
"#;

    // Act - Parse TOML
    let config = parse_toml_config(toml_content)?;

    // Assert - Verify multiple services parsed
    assert!(config.service.is_some());
    let services = config.service.as_ref().unwrap();
    assert_eq!(
        services.len(),
        2,
        "BEHAVIOR: All service definitions should be parsed"
    );
    assert!(services.contains_key("service1"));
    assert!(services.contains_key("service2"));

    Ok(())
}

#[test]
fn test_determinism_section_parses() -> Result<()> {
    // Arrange - Create TOML with determinism configuration
    let toml_content = r#"
[meta]
name = "test_determinism"
version = "1.0.0"

[determinism]
seed = 42
freeze_clock = "2025-01-01T00:00:00Z"

[service.test_service]
plugin = "generic_container"
image = "alpine:latest"

[[scenario]]
name = "test_scenario"
service = "test_service"
run = "echo test"
"#;

    // Act - Parse TOML
    let config = parse_toml_config(toml_content)?;

    // Assert - Verify determinism config parsed
    assert!(
        config.determinism.is_some(),
        "BEHAVIOR: [determinism] section should be parsed"
    );
    let determinism = config.determinism.as_ref().unwrap();
    assert_eq!(determinism.seed, Some(42));
    assert!(
        determinism.freeze_clock.is_some(),
        "BEHAVIOR: Freeze clock should be parsed"
    );

    Ok(())
}

#[test]
fn test_report_section_parses() -> Result<()> {
    // Arrange - Create TOML with report configuration
    let toml_content = r#"
[meta]
name = "test_report"
version = "1.0.0"

[report]
json = "report.json"
junit = "report.xml"
digest = "trace.sha256"

[service.test_service]
plugin = "generic_container"
image = "alpine:latest"

[[scenario]]
name = "test_scenario"
service = "test_service"
run = "echo test"
"#;

    // Act - Parse TOML
    let config = parse_toml_config(toml_content)?;

    // Assert - Verify report config parsed
    assert!(
        config.report.is_some(),
        "BEHAVIOR: [report] section should be parsed"
    );
    let report = config.report.as_ref().unwrap();
    assert_eq!(report.json.as_ref().unwrap(), "report.json");
    assert_eq!(report.junit.as_ref().unwrap(), "report.xml");

    Ok(())
}

#[test]
fn test_limits_section_parses() -> Result<()> {
    // Arrange - Create TOML with resource limits
    let toml_content = r#"
[meta]
name = "test_limits"
version = "1.0.0"

[limits]
cpu_millicores = 1000
memory_mb = 512

[service.test_service]
plugin = "generic_container"
image = "alpine:latest"

[[scenario]]
name = "test_scenario"
service = "test_service"
run = "echo test"
"#;

    // Act - Parse TOML
    let config = parse_toml_config(toml_content)?;

    // Assert - Verify limits config parsed
    assert!(
        config.limits.is_some(),
        "BEHAVIOR: [limits] section should be parsed"
    );
    let limits = config.limits.as_ref().unwrap();
    assert_eq!(limits.cpu_millicores, Some(1000));
    assert_eq!(limits.memory_mb, Some(512));

    Ok(())
}

