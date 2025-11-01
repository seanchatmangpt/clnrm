//! TDD London School Test: Template Variable Substitution
//!
//! **Problem:** Template variables in [vars] section are not substituted before TOML parsing
//! **Root Cause:** Current implementation parses TOML first, then tries to substitute templates
//! **Solution:** Render templates BEFORE parsing TOML
//!
//! **TDD Cycle Documentation:**
//! - RED: Test proves variables don't work (fails with TOML parse error)
//! - GREEN: Fix loader.rs to substitute before parsing (test passes)
//! - REFACTOR: Clean up implementation
//!
//! **London School Approach:**
//! - Mock the template renderer behavior
//! - Verify collaboration between loader and renderer
//! - Focus on interaction patterns, not implementation details

use clnrm_core::config::loader::load_config_from_file;
use std::fs;

/// RED TEST: Proves template variables fail with current implementation
///
/// **Expected behavior:**
/// - Config with {{ port }} in [vars] should be substituted
/// - TOML parser should see "ports = [8080]" not "ports = [{{ port }}]"
/// - Current implementation fails because toml::from_str runs before substitution
#[test]
#[ignore] // RED: This test should fail with current implementation
fn test_red_template_variables_not_substituted_before_parsing() {
    // GIVEN: Config file with template variables
    let temp_dir = std::env::temp_dir();
    let config_path = temp_dir.join("test-template-vars-red.clnrm.toml");

    let toml_with_vars = r#"
[meta]
name = "template_var_test"
version = "1.0.0"

[vars]
port = 8080
image_name = "nginx:latest"

[service.api]
type = "generic_container"
image = "{{ image_name }}"
ports = [{{ port }}]

[[scenario]]
name = "health_check"
service = "api"
run = "curl localhost:{{ port }}/health"
"#;

    fs::write(&config_path, toml_with_vars).expect("Failed to write test config");

    // WHEN: Loading config with current implementation
    let result = load_config_from_file(&config_path);

    // THEN: Should FAIL because {{ port }} is not valid TOML
    // Current implementation: toml::from_str runs BEFORE template rendering
    // Result: TOML parse error on "ports = [{{ port }}]"
    assert!(
        result.is_err(),
        "RED TEST: Should fail because template vars are not substituted before TOML parsing. \
         Current order: parse_toml_config(content) → then template rendering. \
         Expected error: TOML parse error on '{{ port }}'"
    );

    // Clean up
    let _ = fs::remove_file(&config_path);
}

/// GREEN TEST: Proves fix works - template variables substituted before parsing
///
/// **Expected behavior:**
/// - Template rendering happens BEFORE toml::from_str
/// - Variables in [vars] section are available during rendering
/// - TOML parser sees clean substituted values
#[test]
fn test_green_template_variables_substituted_before_parsing() {
    // GIVEN: Config file with template variables
    let temp_dir = std::env::temp_dir();
    let config_path = temp_dir.join("test-template-vars-green.clnrm.toml");

    let toml_with_vars = r#"
[meta]
name = "template_var_test"
version = "1.0.0"

[vars]
port = 8080
image_name = "nginx:latest"

[service.api]
type = "generic_container"
image = "{{ image_name }}"
ports = [{{ port }}]

[[scenario]]
name = "health_check"
service = "api"
run = "curl localhost:{{ port }}/health"
"#;

    fs::write(&config_path, toml_with_vars).expect("Failed to write test config");

    // WHEN: Loading config with fixed implementation
    let result = load_config_from_file(&config_path);

    // THEN: Should succeed because templates are rendered BEFORE TOML parsing
    assert!(
        result.is_ok(),
        "GREEN TEST: Should succeed because template vars are substituted before parsing. \
         Fixed order: render_template(content) → then parse_toml_config(rendered). \
         Error: {:?}",
        result.err()
    );

    let config = result.unwrap();

    // Verify variables were substituted correctly
    let services = config.service.expect("Should have service section");
    let api_service = services.get("api").expect("Should have api service");

    assert_eq!(
        api_service.image.as_ref().unwrap(),
        "nginx:latest",
        "Image name should be substituted"
    );

    assert_eq!(
        api_service.ports.as_ref().unwrap(),
        &vec![8080],
        "Port should be substituted as integer"
    );

    // Verify scenario has substituted variables
    let scenarios = &config.scenario;
    assert!(!scenarios.is_empty(), "Should have scenarios");

    let health_check = &scenarios[0];
    assert_eq!(health_check.name, "health_check");
    assert_eq!(
        health_check.run.as_ref().unwrap(),
        "curl localhost:8080/health",
        "Port in scenario run command should be substituted"
    );

    // Clean up
    let _ = fs::remove_file(&config_path);
}

/// London School: Test interaction between loader and template renderer
///
/// **Behavioral Contract:**
/// - Loader MUST extract [vars] section before rendering
/// - Loader MUST call is_template() before parsing
/// - Loader MUST call render_template() with extracted vars
/// - Loader MUST pass rendered content to parse_toml_config()
#[test]
fn test_london_school_loader_renderer_interaction() {
    // GIVEN: Template content with variables
    let temp_dir = std::env::temp_dir();
    let config_path = temp_dir.join("test-london-school.clnrm.toml");

    let toml_with_vars = r#"
[meta]
name = "interaction_test"
version = "1.0.0"

[vars]
timeout = 5000

[service.db]
type = "generic_container"
image = "postgres:15"
env = { DATABASE_TIMEOUT = "{{ timeout }}" }

[[scenario]]
name = "test"
service = "db"
run = "echo test"
"#;

    fs::write(&config_path, toml_with_vars).expect("Failed to write test config");

    // WHEN: Loading config (tests the full interaction pattern)
    let config_result = load_config_from_file(&config_path);

    // THEN: Should succeed because loader extracts vars before rendering
    assert!(
        config_result.is_ok(),
        "Config loading should succeed with proper var extraction: {:?}",
        config_result.err()
    );

    let config = config_result.unwrap();
    let services = config.service.expect("Should have service section");
    let db_service = services.get("db").expect("Should have db service");

    assert_eq!(
        db_service
            .env
            .as_ref()
            .unwrap()
            .get("DATABASE_TIMEOUT")
            .unwrap(),
        "5000",
        "Environment variable should have substituted timeout value"
    );

    // Clean up
    let _ = fs::remove_file(&config_path);
}

/// REFACTOR TEST: Verify edge cases after implementation
///
/// **Edge cases:**
/// - Variables in multiple sections
/// - Variables in scenario commands
/// - Multiple services using different variables
#[test]
fn test_refactor_edge_cases() {
    // Test 1: Variables in multiple sections
    let temp_dir = std::env::temp_dir();
    let config_path = temp_dir.join("test-edge-cases.clnrm.toml");

    let toml = r#"
[meta]
name = "edge_case_test"
version = "1.0.0"

[vars]
base_port = 3000
db_port = 5432

[service.app]
type = "generic_container"
image = "node:20"
ports = [{{ base_port }}]

[service.db]
type = "generic_container"
image = "postgres:15"
ports = [{{ db_port }}]

[[scenario]]
name = "connect"
service = "app"
run = "nc -zv db {{ db_port }}"
"#;

    fs::write(&config_path, toml).expect("Failed to write test config");

    // Load and verify
    let config = load_config_from_file(&config_path).expect("Should load config");

    let services = config.service.expect("Should have services");
    assert_eq!(
        services.get("app").unwrap().ports.as_ref().unwrap(),
        &vec![3000]
    );
    assert_eq!(
        services.get("db").unwrap().ports.as_ref().unwrap(),
        &vec![5432]
    );

    // Test 2: Variables in scenario commands
    let scenario = &config.scenario[0];
    assert_eq!(scenario.run.as_ref().unwrap(), "nc -zv db 5432");

    // Clean up
    let _ = fs::remove_file(&config_path);
}

/// Integration test: Verify the complete flow works end-to-end
#[test]
fn test_integration_complete_template_flow() {
    // GIVEN: Complex config with variables throughout
    let temp_dir = std::env::temp_dir();
    let config_path = temp_dir.join("test-integration-template.clnrm.toml");

    let complex_toml = r#"
[meta]
name = "integration_test"
version = "1.0.0"
description = "Testing complete template variable flow"

[vars]
app_port = 8080
db_port = 5432
cache_port = 6379
app_image = "myapp:latest"

[service.app]
type = "generic_container"
image = "{{ app_image }}"
ports = [{{ app_port }}]
env = { DATABASE_URL = "postgres://db:{{ db_port }}/mydb", REDIS_URL = "redis://cache:{{ cache_port }}" }

[service.db]
type = "generic_container"
image = "postgres:15"
ports = [{{ db_port }}]

[service.cache]
type = "generic_container"
image = "redis:7"
ports = [{{ cache_port }}]

[[scenario]]
name = "health_checks"
service = "app"
run = "curl localhost:{{ app_port }}/health"

[[scenario]]
name = "db_connection"
service = "app"
run = "psql postgres://db:{{ db_port }}/mydb -c 'SELECT 1'"
"#;

    fs::write(&config_path, complex_toml).expect("Failed to write config");

    // WHEN: Loading the complete config
    let result = load_config_from_file(&config_path);

    // THEN: Everything should work
    assert!(
        result.is_ok(),
        "Integration test should pass: {:?}",
        result.err()
    );

    let config = result.unwrap();
    let services = config.service.expect("Should have services");

    // Verify all services have correct ports
    assert_eq!(
        services.get("app").unwrap().ports.as_ref().unwrap(),
        &vec![8080]
    );
    assert_eq!(
        services.get("db").unwrap().ports.as_ref().unwrap(),
        &vec![5432]
    );
    assert_eq!(
        services.get("cache").unwrap().ports.as_ref().unwrap(),
        &vec![6379]
    );

    // Verify environment variables have substituted ports
    let app_env = services.get("app").unwrap().env.as_ref().unwrap();
    assert_eq!(
        app_env.get("DATABASE_URL").unwrap(),
        "postgres://db:5432/mydb"
    );
    assert_eq!(app_env.get("REDIS_URL").unwrap(), "redis://cache:6379");

    // Verify scenarios have substituted variables
    assert_eq!(
        config.scenario[0].run.as_ref().unwrap(),
        "curl localhost:8080/health"
    );
    assert_eq!(
        config.scenario[1].run.as_ref().unwrap(),
        "psql postgres://db:5432/mydb -c 'SELECT 1'"
    );

    // Clean up
    let _ = fs::remove_file(&config_path);
}
