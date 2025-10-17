//! Test for v0.6.0 Tera template generators
//!
//! Validates that all template generator functions produce valid output

use clnrm_core::cli::commands::{
    generate_deterministic_template, generate_full_validation_template, generate_macro_library,
    generate_matrix_template, generate_otel_template,
};
use clnrm_core::error::Result;

#[test]
fn test_otel_template_generation() -> Result<()> {
    // Act
    let template = generate_otel_template()?;

    // Assert
    assert!(template.contains("[meta]"));
    assert!(template.contains("[otel]"));
    assert!(template.contains("{{"));
    assert!(template.contains("vars."));

    Ok(())
}

#[test]
fn test_matrix_template_generation() -> Result<()> {
    // Act
    let template = generate_matrix_template()?;

    // Assert
    assert!(template.contains("[matrix]"));
    assert!(template.contains("{% for"));
    assert!(template.contains("matrix.os"));
    assert!(template.contains("matrix.version"));

    Ok(())
}

#[test]
fn test_macro_library_generation() -> Result<()> {
    // Act
    let template = generate_macro_library()?;

    // Assert
    assert!(template.contains("{% macro"));
    assert!(template.contains("container_lifecycle_events"));
    assert!(template.contains("otel_standard_resources"));
    assert!(template.contains("span_assertions"));

    Ok(())
}

#[test]
fn test_full_validation_template_generation() -> Result<()> {
    // Act
    let template = generate_full_validation_template()?;

    // Assert
    assert!(template.contains("[expect.order]"));
    assert!(template.contains("[expect.status]"));
    assert!(template.contains("[expect.counts]"));
    assert!(template.contains("[expect.window]"));
    assert!(template.contains("[expect.graph]"));
    assert!(template.contains("[expect.hermeticity]"));
    assert!(template.contains("[determinism]"));
    assert!(template.contains("[limits]"));
    assert!(template.contains("[report]"));

    Ok(())
}

#[test]
fn test_deterministic_template_generation() -> Result<()> {
    // Act
    let template = generate_deterministic_template()?;

    // Assert
    assert!(template.contains("[determinism]"));
    assert!(template.contains("seed"));
    assert!(template.contains("freeze_clock"));
    assert!(template.contains("[report]"));
    assert!(template.contains("digest"));

    Ok(())
}

#[test]
fn test_all_templates_have_version_0_6_0() -> Result<()> {
    // Arrange & Act
    let templates = vec![
        generate_otel_template()?,
        generate_matrix_template()?,
        generate_full_validation_template()?,
        generate_deterministic_template()?,
    ];

    // Assert - all templates should reference v0.6.0
    for template in templates {
        assert!(
            template.contains("0.6.0") || template.contains("v0.6.0"),
            "Template should reference v0.6.0"
        );
    }

    Ok(())
}
