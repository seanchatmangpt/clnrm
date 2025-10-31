//! OTEL configuration tests
//!
//! Tests OpenTelemetry configuration parsing (high-value feature)

use clnrm_core::config::parse_toml_config;
use clnrm_core::error::Result;

#[test]
fn test_otel_exporter_configuration() -> Result<()> {
    // Arrange - Create TOML with OTEL exporter
    let toml_content = r#"
[meta]
name = "test_otel"
version = "1.0.0"

[otel]
exporter = "stdout"
sample_ratio = 1.0

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

    // Assert - Verify OTEL config parsed
    assert!(
        config.otel.is_some(),
        "BEHAVIOR: [otel] section should be parsed"
    );
    let otel = config.otel.as_ref().unwrap();
    assert_eq!(otel.exporter, Some("stdout".to_string()));
    assert_eq!(otel.sample_ratio, Some(1.0));

    Ok(())
}

#[test]
fn test_otel_resources_configuration() -> Result<()> {
    // Arrange - Create TOML with OTEL resources
    let toml_content = r#"
[meta]
name = "test_otel_resources"
version = "1.0.0"

[otel]
exporter = "otlp-http"
resources = {
  "service.name" = "test_service",
  "service.version" = "1.0.0",
  "deployment.environment" = "test"
}

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

    // Assert - Verify OTEL resources parsed
    assert!(config.otel.is_some());
    let otel = config.otel.as_ref().unwrap();
    assert!(
        otel.resources.is_some(),
        "BEHAVIOR: OTEL resources should be parsed"
    );
    let resources = otel.resources.as_ref().unwrap();
    assert!(
        resources.contains_key("service.name"),
        "BEHAVIOR: Resource attributes should be stored"
    );

    Ok(())
}

#[test]
fn test_otel_endpoint_configuration() -> Result<()> {
    // Arrange - Create TOML with OTEL endpoint
    let toml_content = r#"
[meta]
name = "test_otel_endpoint"
version = "1.0.0"

[otel]
exporter = "otlp-http"
endpoint = "http://localhost:4318"
protocol = "http/protobuf"

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

    // Assert - Verify OTEL endpoint parsed
    assert!(config.otel.is_some());
    let otel = config.otel.as_ref().unwrap();
    assert_eq!(
        otel.endpoint.as_ref().unwrap(),
        "http://localhost:4318"
    );
    assert_eq!(
        otel.protocol.as_ref().unwrap(),
        "http/protobuf"
    );

    Ok(())
}

