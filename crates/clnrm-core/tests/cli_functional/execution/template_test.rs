//! Template command tests
//!
//! Tests verify template generation using AAA pattern.

use clnrm_core::cli::commands::template::generate_otel_template;
use clnrm_core::error::Result;

#[test]
fn test_generate_otel_template_produces_valid_tera_syntax() -> Result<()> {
    // Arrange - No setup needed (pure function)

    // Act - Generate OTEL template
    let template = generate_otel_template()?;

    // Assert - Verify template contains expected Tera syntax
    assert!(
        template.contains("{{"),
        "BEHAVIOR: Template should contain Tera variable syntax"
    );
    assert!(
        template.contains("}}"),
        "BEHAVIOR: Template should contain closing Tera syntax"
    );
    assert!(
        template.contains("[meta]"),
        "BEHAVIOR: Template should contain TOML configuration"
    );
    assert!(
        template.contains("otel"),
        "BEHAVIOR: Template should contain OTEL configuration"
    );

    // Verify it's not empty
    assert!(
        !template.is_empty(),
        "BEHAVIOR: Generated template should not be empty"
    );

    Ok(())
}

