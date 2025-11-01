//! TOML Schema Compatibility Tests
//!
//! Validates that both v1.3.0 ([test.metadata]) and v1.4.0 ([test]) schemas
//! are properly supported and backward compatible.

use clnrm_core::config::{parse_toml_config, TestConfig};
use clnrm_core::error::Result;

#[test]
fn test_old_schema_test_metadata_parses() -> Result<()> {
    let toml = r#"
        [test.metadata]
        name = "old_schema_test"
        description = "Test with old v1.3.0 schema"

        [[steps]]
        name = "test_step"
        command = ["echo", "hello"]
    "#;

    let config = parse_toml_config(toml)?;

    // Old schema uses test.metadata, but TestConfig has been refactored
    // The test field is now Option<TestMetadataSection>
    // This test validates backward compatibility
    assert!(config.test.is_some() || config.meta.is_some());
    assert!(!config.steps.is_empty());
    assert_eq!(config.steps[0].name, "test_step");

    Ok(())
}

#[test]
fn test_new_schema_test_section_parses() -> Result<()> {
    let toml = r#"
        [test]
        name = "new_schema_test"
        description = "Test with new v1.4.0 schema"

        [[steps]]
        name = "test_step"
        command = ["echo", "hello"]
    "#;

    let config = parse_toml_config(toml)?;

    assert!(config.test.is_some());
    assert!(!config.steps.is_empty());
    assert_eq!(config.steps[0].name, "test_step");

    Ok(())
}

#[test]
fn test_meta_section_parses() -> Result<()> {
    let toml = r#"
        [meta]
        name = "meta_schema_test"
        description = "Test with meta section"

        [[steps]]
        name = "test_step"
        command = ["echo", "hello"]
    "#;

    let config = parse_toml_config(toml)?;

    assert!(config.meta.is_some());
    assert!(!config.steps.is_empty());

    Ok(())
}

#[test]
fn test_service_configuration_compatibility() -> Result<()> {
    let toml = r#"
        [test]
        name = "service_test"
        description = "Test service configuration"

        [services.test_container]
        type = "generic_container"
        image = "alpine:latest"

        [[steps]]
        name = "test_step"
        command = ["echo", "hello"]
        service = "test_container"
    "#;

    let config = parse_toml_config(toml)?;

    assert!(config.test.is_some());
    assert!(config.services.is_some());
    assert!(config
        .services
        .as_ref()
        .unwrap()
        .contains_key("test_container"));

    Ok(())
}

#[test]
fn test_service_vs_services_sections() -> Result<()> {
    // Test [service.name] syntax (v0.6.0+)
    let toml = r#"
        [test]
        name = "service_syntax_test"
        description = "Test [service.name] syntax"

        [service.my_service]
        type = "generic_container"
        image = "alpine:latest"

        [[steps]]
        name = "test_step"
        command = ["echo", "hello"]
    "#;

    let config = parse_toml_config(toml)?;

    assert!(config.service.is_some());
    assert!(config.service.as_ref().unwrap().contains_key("my_service"));

    Ok(())
}

#[test]
fn test_weaver_configuration_compatibility() -> Result<()> {
    let toml = r#"
        [test]
        name = "weaver_test"
        description = "Test Weaver configuration"

        [weaver]
        enabled = true
        registry_path = "registry"
        otlp_port = 4317

        [[steps]]
        name = "test_step"
        command = ["echo", "hello"]
    "#;

    let config = parse_toml_config(toml)?;

    assert!(config.weaver.is_some());
    let weaver = config.weaver.unwrap();
    assert!(weaver.enabled);
    assert_eq!(weaver.registry_path, "registry");
    assert_eq!(weaver.otlp_port, 4317);

    Ok(())
}

#[test]
fn test_otel_validation_section() -> Result<()> {
    let toml = r#"
        [test]
        name = "otel_test"
        description = "Test OTEL validation"

        [otel_validation]
        enabled = true

        [[steps]]
        name = "test_step"
        command = ["echo", "hello"]
    "#;

    let config = parse_toml_config(toml)?;

    assert!(config.otel_validation.is_some());

    Ok(())
}

#[test]
fn test_template_variables_section() -> Result<()> {
    let toml = r#"
        [test]
        name = "vars_test"
        description = "Test template variables"

        [vars]
        port = 8080
        image = "alpine:latest"

        [[steps]]
        name = "test_step"
        command = ["echo", "{{ port }}"]
    "#;

    let config = parse_toml_config(toml)?;

    assert!(config.vars.is_some());
    let vars = config.vars.unwrap();
    assert!(vars.contains_key("port"));
    assert!(vars.contains_key("image"));

    Ok(())
}

#[test]
fn test_chaos_configuration() -> Result<()> {
    let toml = r#"
        [test]
        name = "chaos_test"
        description = "Test chaos configuration"

        [chaos]
        enabled = true

        [[chaos.experiments]]
        type = "container_kill"
        target_service = "test_service"

        [[steps]]
        name = "test_step"
        command = ["echo", "hello"]
    "#;

    let config = parse_toml_config(toml)?;

    assert!(config.chaos.is_some());
    let chaos = config.chaos.unwrap();
    assert!(chaos.enabled);
    assert!(!chaos.experiments.is_empty());

    Ok(())
}

#[test]
fn test_complex_real_world_example() -> Result<()> {
    let toml = r#"
        [test]
        name = "complex_integration_test"
        description = "Real-world integration test with all features"

        [vars]
        db_port = 5432
        api_port = 8080

        [services.database]
        type = "generic_container"
        image = "postgres:15"

        [services.api]
        type = "generic_container"
        image = "myapi:latest"

        [weaver]
        enabled = true
        registry_path = "registry"
        fail_fast = true

        [weaver.validation]
        mode = "strict"
        fail_on_violation = true

        [[steps]]
        name = "start_database"
        command = ["pg_isready"]
        service = "database"

        [[steps]]
        name = "run_api_tests"
        command = ["curl", "http://localhost:{{ api_port }}/health"]
        service = "api"

        [assertions]
        container_should_have_executed_commands = 2
        execution_should_be_hermetic = true
    "#;

    let config = parse_toml_config(toml)?;

    // Validate all sections parsed correctly
    assert!(config.test.is_some());
    assert!(config.vars.is_some());
    assert!(config.services.is_some());
    assert!(config.weaver.is_some());
    assert_eq!(config.steps.len(), 2);
    assert!(config.assertions.is_some());

    Ok(())
}

#[test]
fn test_backward_compatibility_comprehensive() -> Result<()> {
    // Test that old schema files can still be parsed
    let old_schemas = vec![
        r#"
            [test.metadata]
            name = "test1"
            description = "Old schema variant 1"

            [[steps]]
            name = "step1"
            command = ["echo", "test"]
        "#,
        r#"
            [meta]
            name = "test2"
            description = "Old schema variant 2"

            [[steps]]
            name = "step1"
            command = ["echo", "test"]
        "#,
    ];

    for (i, toml) in old_schemas.iter().enumerate() {
        let config = parse_toml_config(toml)
            .map_err(|e| format!("Failed to parse old schema variant {}: {}", i + 1, e))
            .unwrap();

        assert!(
            !config.steps.is_empty(),
            "Schema variant {} should have steps",
            i + 1
        );
    }

    Ok(())
}
