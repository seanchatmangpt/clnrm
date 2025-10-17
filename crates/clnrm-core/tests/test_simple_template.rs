//! Simple template test to diagnose Tera parsing issues

use clnrm_core::template::TemplateRenderer;
use clnrm_core::error::Result;

#[test]
fn test_simple_var_substitution() -> Result<()> {
    let mut renderer = TemplateRenderer::new()?;
    renderer.set_var("test_name", "simple_test");

    let template = r#"name = "{{ vars.test_name }}""#;
    let result = renderer.render_str(template, "simple")?;

    assert_eq!(result.trim(), r#"name = "simple_test""#);
    Ok(())
}

#[test]
fn test_now_rfc3339_function() -> Result<()> {
    let mut renderer = TemplateRenderer::new()?;

    let template = r#"timestamp = "{{ now_rfc3339() }}""#;
    let result = renderer.render_str(template, "time")?;

    assert!(result.contains("timestamp ="));
    assert!(result.contains("T"));
    assert!(result.contains("Z"));
    Ok(())
}

#[test]
fn test_sha256_function() -> Result<()> {
    let mut renderer = TemplateRenderer::new()?;

    let template = r#"hash = "{{ sha256(s="test") }}""#;
    let result = renderer.render_str(template, "hash")?;

    assert!(result.contains("hash ="));
    Ok(())
}

#[test]
fn test_env_function_with_default() -> Result<()> {
    let mut renderer = TemplateRenderer::new()?;

    let template = r#"exporter = "{{ env(name="OTEL_EXPORTER") | default(value="stdout") }}""#;
    let result = renderer.render_str(template, "env")?;

    // Should use default since OTEL_EXPORTER is not set
    assert!(result.contains(r#"exporter = "stdout""#) || result.contains("OTEL_EXPORTER value"));
    Ok(())
}
