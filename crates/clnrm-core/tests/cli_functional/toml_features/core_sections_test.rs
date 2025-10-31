//! Core TOML sections tests (80% of usage)
//!
//! Tests the most commonly used TOML sections:
//! - [meta] / [test.metadata] - Test identification
//! - [service.<name>] / [services.<name>] - Service definitions
//! - [[scenario]] - Test scenarios (CRITICAL - most important section)
//! - Basic variable substitution

use clnrm_core::config::parse_toml_config;
use clnrm_core::error::Result;

// Helper module
mod helpers {
    use clnrm_core::error::{CleanroomError, Result};
    use tempfile::NamedTempFile;

    pub fn create_temp_file(contents: &str) -> Result<NamedTempFile> {
        let mut file = tempfile::NamedTempFile::new().map_err(|e| {
            CleanroomError::io_error(format!("Failed to create temp file: {}", e))
        })?;
        
        std::io::Write::write_all(&mut file, contents.as_bytes()).map_err(|e| {
            CleanroomError::io_error(format!("Failed to write temp file: {}", e))
        })?;
        
        file.flush().map_err(|e| {
            CleanroomError::io_error(format!("Failed to flush temp file: {}", e))
        })?;
        
        Ok(file)
    }
}

#[test]
fn test_meta_section_parses_correctly() -> Result<()> {
    // Arrange - Create TOML with [meta] section
    let toml_content = r#"
[meta]
name = "test_meta"
version = "1.0.0"
description = "Test metadata parsing"

[[scenario]]
name = "test_scenario"
service = "test_service"
run = "echo test"

[service.test_service]
plugin = "generic_container"
image = "alpine:latest"
"#;

    // Act - Parse TOML config
    let config = parse_toml_config(toml_content)?;

    // Assert - Verify meta section parsed
    assert!(
        config.meta.is_some(),
        "BEHAVIOR: [meta] section should be parsed"
    );
    let meta = config.meta.as_ref().unwrap();
    assert_eq!(meta.name, "test_meta");
    assert_eq!(meta.version, "1.0.0");
    assert_eq!(
        meta.description.as_ref().unwrap(),
        "Test metadata parsing"
    );

    Ok(())
}

#[test]
fn test_legacy_test_metadata_section_parses() -> Result<()> {
    // Arrange - Create TOML with legacy [test.metadata] section
    let toml_content = r#"
[test.metadata]
name = "legacy_test"
description = "Legacy format test"

[[scenario]]
name = "test_scenario"
service = "test_service"
run = "echo test"

[service.test_service]
plugin = "generic_container"
image = "alpine:latest"
"#;

    // Act - Parse TOML config
    let config = parse_toml_config(toml_content)?;

    // Assert - Verify legacy format still works
    assert!(
        config.test.is_some(),
        "BEHAVIOR: Legacy [test.metadata] section should be parsed"
    );
    let test_meta = config.test.as_ref().unwrap();
    assert_eq!(test_meta.metadata.name, "legacy_test");

    Ok(())
}

#[test]
fn test_service_section_parses_with_plugin_and_image() -> Result<()> {
    // Arrange - Create TOML with service definition
    let toml_content = r#"
[meta]
name = "test_service_parsing"
version = "1.0.0"

[service.my_service]
plugin = "generic_container"
image = "alpine:latest"
args = ["echo", "hello"]

[[scenario]]
name = "test_scenario"
service = "my_service"
run = "echo test"
"#;

    // Act - Parse TOML config
    let config = parse_toml_config(toml_content)?;

    // Assert - Verify service section parsed
    assert!(
        config.service.is_some(),
        "BEHAVIOR: [service.<name>] section should be parsed"
    );
    let services = config.service.as_ref().unwrap();
    assert!(
        services.contains_key("my_service"),
        "BEHAVIOR: Service should be stored with correct key"
    );
    let service = &services["my_service"];
    assert_eq!(service.plugin, "generic_container");
    assert_eq!(service.image, "alpine:latest");
    assert!(!service.args.as_ref().unwrap_or(&vec![]).is_empty());

    Ok(())
}

#[test]
fn test_scenario_section_parses_multiple_scenarios() -> Result<()> {
    // Arrange - Create TOML with multiple scenarios
    let toml_content = r#"
[meta]
name = "test_scenarios"
version = "1.0.0"

[service.test_service]
plugin = "generic_container"
image = "alpine:latest"

[[scenario]]
name = "scenario_1"
service = "test_service"
run = "echo first"

[[scenario]]
name = "scenario_2"
service = "test_service"
run = "echo second"
"#;

    // Act - Parse TOML config
    let config = parse_toml_config(toml_content)?;

    // Assert - Verify multiple scenarios parsed
    assert_eq!(
        config.scenario.len(),
        2,
        "BEHAVIOR: All scenarios should be parsed"
    );
    assert_eq!(config.scenario[0].name, "scenario_1");
    assert_eq!(config.scenario[1].name, "scenario_2");
    assert_eq!(config.scenario[0].service.as_ref().unwrap(), "test_service");

    Ok(())
}

#[test]
fn test_scenario_with_artifacts_collection() -> Result<()> {
    // Arrange - Create TOML with artifact collection
    let toml_content = r#"
[meta]
name = "test_artifacts"
version = "1.0.0"

[service.test_service]
plugin = "generic_container"
image = "alpine:latest"

[[scenario]]
name = "test_scenario"
service = "test_service"
run = "echo test"
artifacts.collect = ["spans:default"]
"#;

    // Act - Parse TOML config
    let config = parse_toml_config(toml_content)?;

    // Assert - Verify artifacts configuration parsed
    assert!(!config.scenario.is_empty());
    let scenario = &config.scenario[0];
    assert!(
        scenario.artifacts.is_some(),
        "BEHAVIOR: Artifacts configuration should be parsed"
    );
    let artifacts = scenario.artifacts.as_ref().unwrap();
    assert!(
        artifacts.collect.contains(&"spans:default".to_string()),
        "BEHAVIOR: Artifact collection types should be parsed"
    );

    Ok(())
}

#[test]
fn test_vars_section_parses_variables() -> Result<()> {
    // Arrange - Create TOML with variables section
    let toml_content = r#"
[meta]
name = "test_vars"
version = "1.0.0"

[vars]
image = "alpine:latest"
endpoint = "http://localhost:8080"
env = "test"

[service.test_service]
plugin = "generic_container"
image = "alpine:latest"

[[scenario]]
name = "test_scenario"
service = "test_service"
run = "echo test"
"#;

    // Act - Parse TOML config
    let config = parse_toml_config(toml_content)?;

    // Assert - Verify vars section parsed
    assert!(
        config.vars.is_some(),
        "BEHAVIOR: [vars] section should be parsed"
    );
    let vars = config.vars.as_ref().unwrap();
    assert!(
        vars.contains_key("image"),
        "BEHAVIOR: Variables should be stored correctly"
    );

    Ok(())
}

