//! Weaver live-checking TOML configuration tests

use clnrm_core::config::parse_toml_config;
use clnrm_core::error::Result;

#[test]
fn test_weaver_section_parses_with_defaults() -> Result<()> {
    // Arrange - Create TOML with minimal Weaver config
    let toml_content = r#"
[meta]
name = "test_weaver_defaults"
version = "1.0.0"

[weaver]
enabled = true

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

    // Assert - Verify Weaver config parsed with defaults
    assert!(
        config.weaver.is_some(),
        "BEHAVIOR: [weaver] section should be parsed"
    );
    let weaver = config.weaver.as_ref().unwrap();
    assert!(weaver.enabled, "BEHAVIOR: enabled should default to true");
    assert_eq!(weaver.registry_path, "registry");
    assert_eq!(weaver.otlp_port, 0); // 0 = auto-discover
    assert_eq!(weaver.admin_port, 0); // 0 = auto-discover
    assert_eq!(weaver.output_dir, "./validation_output");
    assert!(!weaver.stream, "BEHAVIOR: stream should default to false");
    assert!(!weaver.fail_fast, "BEHAVIOR: fail_fast should default to false");

    Ok(())
}

#[test]
fn test_weaver_section_parses_with_custom_config() -> Result<()> {
    // Arrange - Create TOML with custom Weaver config
    let toml_content = r#"
[meta]
name = "test_weaver_custom"
version = "1.0.0"

[weaver]
enabled = true
registry_path = "/custom/registry/path"
otlp_port = 4317
admin_port = 8080
output_dir = "./custom_output"
stream = true
fail_fast = true

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

    // Assert - Verify custom Weaver config parsed
    assert!(config.weaver.is_some());
    let weaver = config.weaver.as_ref().unwrap();
    assert!(weaver.enabled);
    assert_eq!(weaver.registry_path, "/custom/registry/path");
    assert_eq!(weaver.otlp_port, 4317);
    assert_eq!(weaver.admin_port, 8080);
    assert_eq!(weaver.output_dir, "./custom_output");
    assert!(weaver.stream, "BEHAVIOR: stream should be true");
    assert!(weaver.fail_fast, "BEHAVIOR: fail_fast should be true");

    Ok(())
}

#[test]
fn test_weaver_section_with_auto_discover_ports() -> Result<()> {
    // Arrange - Create TOML with auto-discover ports (0)
    let toml_content = r#"
[meta]
name = "test_weaver_auto_ports"
version = "1.0.0"

[weaver]
enabled = true
otlp_port = 0
admin_port = 0

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

    // Assert - Verify auto-discover ports parsed correctly
    assert!(config.weaver.is_some());
    let weaver = config.weaver.as_ref().unwrap();
    assert_eq!(weaver.otlp_port, 0, "BEHAVIOR: 0 should enable auto-discovery");
    assert_eq!(weaver.admin_port, 0, "BEHAVIOR: 0 should enable auto-discovery");

    Ok(())
}

#[test]
fn test_weaver_section_disabled() -> Result<()> {
    // Arrange - Create TOML with Weaver disabled
    let toml_content = r#"
[meta]
name = "test_weaver_disabled"
version = "1.0.0"

[weaver]
enabled = false

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

    // Assert - Verify Weaver config parsed but disabled
    assert!(config.weaver.is_some());
    let weaver = config.weaver.as_ref().unwrap();
    assert!(!weaver.enabled, "BEHAVIOR: enabled should be false");

    Ok(())
}

#[test]
fn test_weaver_section_validation() -> Result<()> {
    // Arrange - Create TOML with invalid Weaver config (port < 1024)
    let toml_content = r#"
[meta]
name = "test_weaver_invalid"
version = "1.0.0"

[weaver]
enabled = true
otlp_port = 100

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

    // Assert - Config should parse, but validation should fail
    assert!(config.weaver.is_some());
    let weaver = config.weaver.as_ref().unwrap();
    
    // Validation should catch invalid port
    let validation_result = weaver.validate();
    assert!(
        validation_result.is_err(),
        "BEHAVIOR: Validation should reject ports < 1024"
    );

    Ok(())
}

#[test]
fn test_weaver_with_otel_integration() -> Result<()> {
    // Arrange - Create TOML with both Weaver and OTEL config
    let toml_content = r#"
[meta]
name = "test_weaver_otel"
version = "1.0.0"

[weaver]
enabled = true
registry_path = "registry"
stream = false

[otel]
exporter = "otlp-http"
endpoint = "http://localhost:4318"

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

    // Assert - Verify both Weaver and OTEL configs parsed
    assert!(
        config.weaver.is_some(),
        "BEHAVIOR: Weaver config should be parsed"
    );
    assert!(
        config.otel.is_some(),
        "BEHAVIOR: OTEL config should be parsed"
    );
    
    let weaver = config.weaver.as_ref().unwrap();
    let otel = config.otel.as_ref().unwrap();
    
    assert!(weaver.enabled);
    assert_eq!(otel.exporter, "otlp-http");

    Ok(())
}

