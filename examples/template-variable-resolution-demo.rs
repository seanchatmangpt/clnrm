//! Template Variable Resolution Demo
//!
//! This example demonstrates the complete variable resolution precedence chain:
//! 1. User-provided variables (highest priority)
//! 2. Environment variables
//! 3. Default values (lowest priority)
//!
//! Run with:
//! ```bash
//! cargo run --example template-variable-resolution-demo
//! ```

use clnrm_core::error::Result;
use clnrm_core::template::{render_template, TemplateContext, TemplateRenderer};
use serde_json::json;
use std::collections::HashMap;

fn main() -> Result<()> {
    println!("=== Template Variable Resolution Demo ===\n");

    // Demo 1: Default Values
    demo_defaults()?;

    // Demo 2: Environment Variable Override
    demo_env_override()?;

    // Demo 3: User Variable Override (Highest Priority)
    demo_user_override()?;

    // Demo 4: Complete Precedence Chain
    demo_complete_chain()?;

    // Demo 5: Real-World Scenario
    demo_real_world()?;

    Ok(())
}

/// Demo 1: Using default values
fn demo_defaults() -> Result<()> {
    println!("📋 Demo 1: Default Values");
    println!("---------------------------");

    // Clear environment to ensure defaults are used
    std::env::remove_var("SERVICE_NAME");
    std::env::remove_var("ENV");

    let context = TemplateContext::with_defaults();

    println!("Standard variables with defaults:");
    println!("  svc      = {:?}", context.vars.get("svc"));
    println!("  env      = {:?}", context.vars.get("env"));
    println!("  endpoint = {:?}", context.vars.get("endpoint"));
    println!("  exporter = {:?}", context.vars.get("exporter"));
    println!();

    Ok(())
}

/// Demo 2: Environment variables override defaults
fn demo_env_override() -> Result<()> {
    println!("🌍 Demo 2: Environment Variable Override");
    println!("------------------------------------------");

    // Set environment variables
    std::env::set_var("SERVICE_NAME", "my-api-service");
    std::env::set_var("ENV", "production");
    std::env::set_var("OTEL_ENDPOINT", "https://otel.company.com:4318");

    let context = TemplateContext::with_defaults();

    println!("Variables resolved from ENV:");
    println!("  svc      = {:?} (from SERVICE_NAME)", context.vars.get("svc"));
    println!("  env      = {:?} (from ENV)", context.vars.get("env"));
    println!("  endpoint = {:?} (from OTEL_ENDPOINT)", context.vars.get("endpoint"));
    println!(
        "  exporter = {:?} (from defaults - no ENV set)",
        context.vars.get("exporter")
    );

    // Cleanup
    std::env::remove_var("SERVICE_NAME");
    std::env::remove_var("ENV");
    std::env::remove_var("OTEL_ENDPOINT");

    println!();
    Ok(())
}

/// Demo 3: User variables override everything
fn demo_user_override() -> Result<()> {
    println!("👤 Demo 3: User Variable Override (Highest Priority)");
    println!("-----------------------------------------------------");

    // Set ENV (will be overridden)
    std::env::set_var("SERVICE_NAME", "env-service");

    let template = r#"
[meta]
name = "{{ svc }}_{{ scenario }}"
env = "{{ env }}"
custom = "{{ custom_var }}"
"#;

    // User provides specific overrides
    let mut user_vars = HashMap::new();
    user_vars.insert("svc".to_string(), json!("user-override-service"));
    user_vars.insert("scenario".to_string(), json!("integration"));
    user_vars.insert("custom_var".to_string(), json!("special-value"));
    user_vars.insert("env".to_string(), json!("test"));

    let rendered = render_template(template, user_vars)?;

    println!("Template with user overrides:");
    println!("{}", rendered);

    // Cleanup
    std::env::remove_var("SERVICE_NAME");

    Ok(())
}

/// Demo 4: Complete precedence chain
fn demo_complete_chain() -> Result<()> {
    println!("🔗 Demo 4: Complete Precedence Chain");
    println!("-------------------------------------");

    // Setup: Default → ENV → User vars
    std::env::set_var("SERVICE_NAME", "from-env");

    let mut context = TemplateContext::new();

    // Step 1: Add with precedence (ENV wins over default)
    println!("Step 1: Apply precedence (ENV > default)");
    context.add_var_with_precedence("svc", "SERVICE_NAME", "from-default");
    println!("  svc = {:?} (from ENV)", context.vars.get("svc"));

    // Step 2: User vars win over everything
    println!("\nStep 2: User vars override ENV");
    let mut user_vars = HashMap::new();
    user_vars.insert("svc".to_string(), json!("from-user"));
    context.merge_user_vars(user_vars);
    println!("  svc = {:?} (from user vars)", context.vars.get("svc"));

    println!("\nFinal precedence: User > ENV > Default ✅");

    // Cleanup
    std::env::remove_var("SERVICE_NAME");

    println!();
    Ok(())
}

/// Demo 5: Real-world scenario
fn demo_real_world() -> Result<()> {
    println!("🌐 Demo 5: Real-World Scenario");
    println!("--------------------------------");
    println!("Scenario: Developer with local ENV + test-specific overrides\n");

    // 1. Developer's local environment
    std::env::set_var("SERVICE_NAME", "local-dev-api");
    std::env::set_var("OTEL_ENDPOINT", "http://localhost:4318");
    println!("1. Developer's ENV:");
    println!("     SERVICE_NAME = local-dev-api");
    println!("     OTEL_ENDPOINT = http://localhost:4318");

    // 2. Template
    let template = r#"
[meta]
name = "{{ svc }}_{{ test_type }}_test"
description = "Integration test for {{ env }}"

[otel]
endpoint = "{{ endpoint }}"
exporter = "{{ exporter }}"

[services.database]
image = "{{ db_image }}"
"#;

    println!("\n2. Template uses: svc, test_type, env, endpoint, exporter, db_image");

    // 3. Test-specific overrides
    let mut user_vars = HashMap::new();
    user_vars.insert("test_type".to_string(), json!("auth"));
    user_vars.insert("db_image".to_string(), json!("postgres:15"));

    println!("\n3. User provides test-specific vars:");
    println!("     test_type = auth");
    println!("     db_image = postgres:15");

    let rendered = render_template(template, user_vars)?;

    println!("\n4. Final rendered template:");
    println!("{}", rendered);

    println!("Resolution breakdown:");
    println!("  - svc:      'local-dev-api' (from ENV)");
    println!("  - test_type: 'auth' (from user)");
    println!("  - env:      'ci' (from defaults)");
    println!("  - endpoint: 'http://localhost:4318' (from ENV)");
    println!("  - exporter: 'otlp' (from defaults)");
    println!("  - db_image: 'postgres:15' (from user)");

    // Cleanup
    std::env::remove_var("SERVICE_NAME");
    std::env::remove_var("OTEL_ENDPOINT");

    println!();
    Ok(())
}
