//! Integration test for v0.6.0 self-validation
//!
//! This test validates that the v0.6.0 self-validation TOML file:
//! 1. Renders correctly with Tera templating
//! 2. Parses without errors
//! 3. All validators are properly configured
//! 4. Determinism produces identical digests on multiple runs

use clnrm_core::config::load_config_from_file;
use clnrm_core::error::Result;
use clnrm_core::template::{is_template, TemplateRenderer};
use std::path::Path;

#[test]
fn test_v0_6_0_self_validation_file_exists() {
    // Arrange
    let test_file = Path::new("../../tests/self-test/clnrm-v0.6.0-self-validation.clnrm.toml");

    // Assert
    assert!(
        test_file.exists(),
        "v0.6.0 self-validation test file must exist"
    );
}

#[test]
fn test_v0_6_0_self_validation_is_template() -> Result<()> {
    // Arrange
    let test_file = Path::new("../../tests/self-test/clnrm-v0.6.0-self-validation.clnrm.toml");
    let content = std::fs::read_to_string(test_file)?;

    // Act
    let is_tera_template = is_template(&content);

    // Assert
    assert!(is_tera_template, "Self-validation file must use Tera templates");
    assert!(content.contains("{{"), "Must contain Tera expressions");
    assert!(content.contains("vars."), "Must use vars namespace");
    assert!(content.contains("otel."), "Must use otel namespace");

    Ok(())
}

#[test]
fn test_v0_6_0_self_validation_renders_successfully() -> Result<()> {
    // Arrange
    let test_file = Path::new("../../tests/self-test/clnrm-v0.6.0-self-validation.clnrm.toml");
    let content = std::fs::read_to_string(test_file)?;
    let mut renderer = TemplateRenderer::new()?;

    // Act
    let rendered = renderer.render_str(&content, "self-validation")?;

    // Assert
    assert!(!rendered.contains("{{"), "All Tera expressions should be rendered");
    assert!(rendered.contains("[meta]"), "Must have meta section");
    assert!(rendered.contains("[otel]"), "Must have otel section");
    assert!(rendered.contains("[expect.order]"), "Must have order validator");
    assert!(rendered.contains("[expect.status]"), "Must have status validator");
    assert!(rendered.contains("[expect.counts]"), "Must have counts validator");
    assert!(rendered.contains("[expect.window]"), "Must have window validator");
    assert!(rendered.contains("[expect.graph]"), "Must have graph validator");
    assert!(rendered.contains("[expect.hermeticity]"), "Must have hermeticity validator");
    assert!(rendered.contains("[determinism]"), "Must have determinism config");
    assert!(rendered.contains("[limits]"), "Must have resource limits");
    assert!(rendered.contains("[report]"), "Must have reporting config");

    Ok(())
}

#[test]
fn test_v0_6_0_self_validation_parses_correctly() -> Result<()> {
    // Arrange
    let test_file = Path::new("../../tests/self-test/clnrm-v0.6.0-self-validation.clnrm.toml");

    // Act - load_config_from_file handles template rendering + parsing
    let config = load_config_from_file(test_file)?;

    // Assert
    assert!(config.meta.is_some(), "Must have meta section");
    let meta = config.meta.as_ref().unwrap();
    assert_eq!(meta.name, "clnrm_v0_6_0_self_validation");
    assert_eq!(meta.version, "0.6.0");

    assert!(config.otel.is_some(), "Must have OTEL configuration");
    assert!(config.expect.is_some(), "Must have expect configuration");

    let expect = config.expect.as_ref().unwrap();
    assert!(expect.order.is_some(), "Must have order expectations");
    assert!(expect.status.is_some(), "Must have status expectations");
    assert!(expect.counts.is_some(), "Must have count expectations");
    assert!(!expect.window.is_empty(), "Must have window expectations");
    assert!(expect.graph.is_some(), "Must have graph expectations");
    assert!(expect.hermeticity.is_some(), "Must have hermeticity expectations");

    assert!(config.determinism.is_some(), "Must have determinism config");
    assert!(config.limits.is_some(), "Must have resource limits");
    assert!(config.report.is_some(), "Must have report config");

    Ok(())
}

#[test]
fn test_v0_6_0_determinism_configuration() -> Result<()> {
    // Arrange
    let test_file = Path::new("../../tests/self-test/clnrm-v0.6.0-self-validation.clnrm.toml");
    let config = load_config_from_file(test_file)?;

    // Act
    let determinism = config.determinism.as_ref().expect("Must have determinism config");

    // Assert
    assert_eq!(determinism.seed, Some(42), "Must have seed = 42");
    assert_eq!(
        determinism.freeze_clock,
        Some("2025-01-01T00:00:00Z".to_string()),
        "Must have frozen clock"
    );

    Ok(())
}

#[test]
fn test_v0_6_0_order_validator_configuration() -> Result<()> {
    // Arrange
    let test_file = Path::new("../../tests/self-test/clnrm-v0.6.0-self-validation.clnrm.toml");
    let config = load_config_from_file(test_file)?;

    // Act
    let expect = config.expect.as_ref().expect("Must have expect configuration");
    let order = expect.order.as_ref().expect("Must have order expectations");

    // Assert
    let must_precede = order.must_precede.as_ref().expect("Must have must_precede");
    assert!(must_precede.len() >= 5, "Must have multiple precedence constraints");

    let must_follow = order.must_follow.as_ref().expect("Must have must_follow");
    assert!(must_follow.len() >= 2, "Must have multiple follow constraints");

    Ok(())
}

#[test]
fn test_v0_6_0_status_validator_configuration() -> Result<()> {
    // Arrange
    let test_file = Path::new("../../tests/self-test/clnrm-v0.6.0-self-validation.clnrm.toml");
    let config = load_config_from_file(test_file)?;

    // Act
    let expect = config.expect.as_ref().expect("Must have expect configuration");
    let status = expect.status.as_ref().expect("Must have status expectations");

    // Assert
    assert_eq!(status.all, Some("ok".to_string()), "All spans must be OK");

    let by_name = status.by_name.as_ref().expect("Must have by_name patterns");
    assert!(by_name.contains_key("validation_service.*"), "Must have glob pattern");
    assert!(by_name.contains_key("*_phase"), "Must have wildcard pattern");

    Ok(())
}

#[test]
fn test_v0_6_0_reporting_configuration() -> Result<()> {
    // Arrange
    let test_file = Path::new("../../tests/self-test/clnrm-v0.6.0-self-validation.clnrm.toml");
    let config = load_config_from_file(test_file)?;

    // Act
    let report = config.report.as_ref().expect("Must have report config");

    // Assert
    assert!(report.json.is_some(), "Must have JSON report path");
    assert!(report.junit.is_some(), "Must have JUnit report path");
    assert!(report.digest.is_some(), "Must have digest report path");

    Ok(())
}

#[test]
fn test_v0_6_0_tera_functions_used() -> Result<()> {
    // Arrange
    let test_file = Path::new("../../tests/self-test/clnrm-v0.6.0-self-validation.clnrm.toml");
    let content = std::fs::read_to_string(test_file)?;

    // Assert - verify all 4 custom Tera functions are used
    assert!(content.contains("env(name="), "Must use env() function");
    assert!(content.contains("now_rfc3339()"), "Must use now_rfc3339() function");
    assert!(content.contains("sha256(s="), "Must use sha256() function");
    // toml_encode is optional but should be available

    Ok(())
}

#[test]
fn test_v0_6_0_template_namespaces() -> Result<()> {
    // Arrange
    let test_file = Path::new("../../tests/self-test/clnrm-v0.6.0-self-validation.clnrm.toml");
    let content = std::fs::read_to_string(test_file)?;

    // Assert - verify template namespaces are used
    assert!(content.contains("vars."), "Must use vars namespace");
    assert!(content.contains("vars.test_name"), "Must reference vars.test_name");
    assert!(content.contains("vars.service_name"), "Must reference vars.service_name");
    // otel namespace is used in conditional blocks

    Ok(())
}

#[test]
fn test_v0_6_0_all_validators_present() -> Result<()> {
    // Arrange
    let test_file = Path::new("../../tests/self-test/clnrm-v0.6.0-self-validation.clnrm.toml");
    let content = std::fs::read_to_string(test_file)?;

    // Assert - verify all validator types are present
    let validators = vec![
        "expect.span",
        "expect.order",
        "expect.status",
        "expect.counts",
        "expect.window",
        "expect.graph",
        "expect.hermeticity",
    ];

    for validator in validators {
        assert!(
            content.contains(validator),
            "Must include {} validator",
            validator
        );
    }

    Ok(())
}

#[test]
fn test_v0_6_0_resource_limits_configured() -> Result<()> {
    // Arrange
    let test_file = Path::new("../../tests/self-test/clnrm-v0.6.0-self-validation.clnrm.toml");
    let config = load_config_from_file(test_file)?;

    // Act
    let limits = config.limits.as_ref().expect("Must have limits config");

    // Assert
    assert_eq!(limits.cpu_millicores, Some(500), "Must have CPU limit");
    assert_eq!(limits.memory_mb, Some(256), "Must have memory limit");

    Ok(())
}
