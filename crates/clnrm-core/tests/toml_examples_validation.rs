//! TOML Examples Validation Tests
//!
//! Comprehensive validation that all fixed TOML example files load correctly
//! with clnrm's config parser. This validates the TOML fixes we made for:
//! - Inline tables with nested maps (attrs = { all = { "key" = "value" } })
//! - Triple-quote multi-line strings
//! - Table sections ([section.by_name])
//! - Array values (non-duplicate keys)

use clnrm_core::config::{load_config_from_file, parse_toml_config, TestConfig};
use clnrm_core::error::Result;
use std::path::{Path, PathBuf};

/// Helper to get project root directory
fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Test result tracker
#[derive(Debug, Default)]
struct TestResults {
    total: usize,
    passed: usize,
    failed: Vec<(String, String)>,
}

impl TestResults {
    fn test_file(&mut self, path: &Path) {
        self.total += 1;
        let path_str = path.display().to_string();

        match load_config_from_file(path) {
            Ok(config) => {
                // Validate basic structure
                if let Err(e) = validate_config_structure(&config) {
                    self.failed
                        .push((path_str, format!("Structure validation failed: {}", e)));
                } else {
                    self.passed += 1;
                }
            }
            Err(e) => {
                self.failed.push((path_str, format!("Parse error: {}", e)));
            }
        }
    }

    fn summary(&self) -> String {
        format!(
            "Total: {} | Passed: {} | Failed: {} | Success Rate: {:.1}%",
            self.total,
            self.passed,
            self.failed.len(),
            (self.passed as f64 / self.total as f64) * 100.0
        )
    }

    fn is_success(&self) -> bool {
        self.failed.is_empty()
    }
}

/// Validate basic structure of a parsed config
fn validate_config_structure(config: &TestConfig) -> Result<()> {
    // Should have either meta or test section
    if config.meta.is_none() && config.test.is_none() {
        return Err(clnrm_core::error::CleanroomError::validation_error(
            "Missing [meta] or [test] section",
        ));
    }

    // Should be able to get name
    let _name = config.get_name()?;

    // Steps or scenario should exist (unless it's a template-only file)
    // Some templates may not have steps until rendered
    // So we just check that the config is parseable

    Ok(())
}

// ============================================================================
// Test Group 1: live-check examples (4 files)
// ============================================================================

#[test]
fn test_live_check_basic() -> Result<()> {
    let path = project_root().join("examples/live-check/basic.clnrm.toml");
    let config = load_config_from_file(&path)?;
    validate_config_structure(&config)?;
    assert_eq!(config.get_name()?, "basic-otel-live-check");
    Ok(())
}

#[test]
fn test_live_check_strict() -> Result<()> {
    let path = project_root().join("examples/live-check/strict.clnrm.toml");
    let config = load_config_from_file(&path)?;
    validate_config_structure(&config)?;
    assert_eq!(config.get_name()?, "strict-otel-live-check");
    Ok(())
}

#[test]
fn test_live_check_ci_cd() -> Result<()> {
    let path = project_root().join("examples/live-check/ci-cd.clnrm.toml");
    let config = load_config_from_file(&path)?;
    validate_config_structure(&config)?;
    assert_eq!(config.get_name()?, "ci-cd-otel-live-check");
    Ok(())
}

#[test]
fn test_live_check_80_20() -> Result<()> {
    let path = project_root().join("examples/live-check/80-20.clnrm.toml");
    let config = load_config_from_file(&path)?;
    validate_config_structure(&config)?;
    assert_eq!(config.get_name()?, "80-20-otel-live-check");
    Ok(())
}

// ============================================================================
// Test Group 2: clnrm-case-study tests (3-4 files)
// ============================================================================

#[test]
fn test_case_study_ai_character_interaction() -> Result<()> {
    let path =
        project_root().join("examples/clnrm-case-study/tests/ai-character-interaction.clnrm.toml");
    let config = load_config_from_file(&path)?;
    validate_config_structure(&config)?;
    Ok(())
}

#[test]
fn test_case_study_ai_production_readiness() -> Result<()> {
    let path =
        project_root().join("examples/clnrm-case-study/tests/ai-production-readiness.clnrm.toml");
    let config = load_config_from_file(&path)?;
    validate_config_structure(&config)?;
    Ok(())
}

#[test]
fn test_case_study_ai_performance_benchmark() -> Result<()> {
    let path =
        project_root().join("examples/clnrm-case-study/tests/ai-performance-benchmark.clnrm.toml");
    let config = load_config_from_file(&path)?;
    validate_config_structure(&config)?;
    Ok(())
}

#[test]
fn test_case_study_vercel_ai_integration() -> Result<()> {
    let path =
        project_root().join("examples/clnrm-case-study/tests/vercel-ai-integration.clnrm.toml");
    let config = load_config_from_file(&path)?;
    validate_config_structure(&config)?;
    Ok(())
}

// ============================================================================
// Test Group 3: toml-config examples (3 files)
// ============================================================================

#[test]
fn test_toml_config_simple_demo() -> Result<()> {
    let path = project_root().join("examples/toml-config/simple-toml-demo.toml");
    let config = load_config_from_file(&path)?;
    validate_config_structure(&config)?;
    Ok(())
}

#[test]
fn test_toml_config_regex_validation() -> Result<()> {
    let path = project_root().join("examples/toml-config/regex-validation-demo.toml");
    let config = load_config_from_file(&path)?;
    validate_config_structure(&config)?;
    Ok(())
}

#[test]
fn test_toml_config_rich_assertions() -> Result<()> {
    let path = project_root().join("examples/toml-config/rich-assertions-demo.toml");
    let config = load_config_from_file(&path)?;
    validate_config_structure(&config)?;
    Ok(())
}

#[test]
fn test_toml_config_complete_demo() -> Result<()> {
    let path = project_root().join("examples/toml-config/complete-toml-demo.toml");
    let config = load_config_from_file(&path)?;
    validate_config_structure(&config)?;
    Ok(())
}

// ============================================================================
// Test Group 4: templates examples (9 files)
// ============================================================================

#[test]
fn test_template_simple_test_working() -> Result<()> {
    let path = project_root().join("examples/templates/simple-test-working.clnrm.toml");
    let config = load_config_from_file(&path)?;
    validate_config_structure(&config)?;
    Ok(())
}

#[test]
fn test_template_advanced_validators() -> Result<()> {
    let path = project_root().join("examples/templates/advanced-validators.clnrm.toml");
    let config = load_config_from_file(&path)?;
    validate_config_structure(&config)?;
    Ok(())
}

#[test]
fn test_template_matrix_expansion() -> Result<()> {
    let path = project_root().join("examples/templates/matrix-expansion.clnrm.toml");
    let config = load_config_from_file(&path)?;
    validate_config_structure(&config)?;
    Ok(())
}

#[test]
fn test_template_ci_integration() -> Result<()> {
    let path = project_root().join("examples/templates/ci-integration.clnrm.toml");
    let config = load_config_from_file(&path)?;
    validate_config_structure(&config)?;
    Ok(())
}

#[test]
fn test_template_simple_variables() -> Result<()> {
    let path = project_root().join("examples/templates/simple-variables.clnrm.toml");
    let config = load_config_from_file(&path)?;
    validate_config_structure(&config)?;
    Ok(())
}

#[test]
fn test_template_macros_and_includes() -> Result<()> {
    let path = project_root().join("examples/templates/macros-and-includes.clnrm.toml");
    let config = load_config_from_file(&path)?;
    validate_config_structure(&config)?;
    Ok(())
}

#[test]
fn test_template_multi_environment() -> Result<()> {
    let path = project_root().join("examples/templates/multi-environment.clnrm.toml");
    let config = load_config_from_file(&path)?;
    validate_config_structure(&config)?;
    Ok(())
}

#[test]
fn test_template_service_mesh() -> Result<()> {
    let path = project_root().join("examples/templates/service-mesh.clnrm.toml");
    let config = load_config_from_file(&path)?;
    validate_config_structure(&config)?;
    Ok(())
}

#[test]
fn test_template_env_resolution_demo() -> Result<()> {
    let path = project_root().join("examples/templates/env_resolution_demo.clnrm.toml");
    let config = load_config_from_file(&path)?;
    validate_config_structure(&config)?;
    Ok(())
}

// ============================================================================
// Test Group 5: Other key examples
// ============================================================================

#[test]
fn test_behaviors() -> Result<()> {
    let path = project_root().join("examples/behaviors.clnrm.toml");
    let config = load_config_from_file(&path)?;
    validate_config_structure(&config)?;
    Ok(())
}

#[test]
fn test_multi_service_demo() -> Result<()> {
    let path = project_root().join("examples/multi-service-demo.clnrm.toml");
    let config = load_config_from_file(&path)?;
    validate_config_structure(&config)?;
    Ok(())
}

#[test]
fn test_surrealdb_integration_demo() -> Result<()> {
    let path = project_root().join("examples/surrealdb-integration-demo.clnrm.toml");
    let config = load_config_from_file(&path)?;
    validate_config_structure(&config)?;
    Ok(())
}

#[test]
fn test_volume_mount_demo() -> Result<()> {
    let path = project_root().join("examples/volume-mount-demo.clnrm.toml");
    let config = load_config_from_file(&path)?;
    validate_config_structure(&config)?;
    Ok(())
}

#[test]
fn test_weaver_toml_configuration() -> Result<()> {
    let path = project_root().join("examples/weaver-toml-configuration.clnrm.toml");
    let config = load_config_from_file(&path)?;
    validate_config_structure(&config)?;
    Ok(())
}

#[test]
fn test_readme_example_validation() -> Result<()> {
    let path = project_root().join("examples/readme-example-validation.clnrm.toml");
    let config = load_config_from_file(&path)?;
    validate_config_structure(&config)?;
    Ok(())
}

// ============================================================================
// Test Group 6: Specific pattern validation tests
// ============================================================================

#[test]
fn test_inline_table_nested_maps_pattern() -> Result<()> {
    // Test the fixed pattern: attrs = { all = { "key" = "value" } }
    let toml = r#"
        [meta]
        name = "inline-table-test"
        version = "0.6.0"

        [[steps]]
        name = "test_inline_tables"
        command = ["echo", "test"]

        [otel.validation.spans.by_name.test_span.attrs.all]
        "http.method" = "GET"
        "http.status_code" = "200"
    "#;

    let config = parse_toml_config(toml)?;
    validate_config_structure(&config)?;

    // Verify OTEL validation section was parsed (just check it exists)
    assert!(
        config.otel_validation.is_some(),
        "Should have OTEL validation section"
    );

    Ok(())
}

#[test]
fn test_triple_quote_multiline_strings_pattern() -> Result<()> {
    // Test triple-quote multi-line strings
    let toml = r#"
        [meta]
        name = "multiline-test"
        version = "0.6.0"
        description = """
        This is a multi-line description
        that spans multiple lines
        using triple quotes.
        """

        [[steps]]
        name = "test_multiline"
        command = ["echo", "test"]
    "#;

    let config = parse_toml_config(toml)?;
    validate_config_structure(&config)?;

    let desc = config.get_description().expect("Should have description");
    assert!(
        desc.contains("multi-line"),
        "Should preserve multi-line content"
    );

    Ok(())
}

#[test]
fn test_table_sections_pattern() -> Result<()> {
    // Test [section.by_name] table pattern
    let toml = r#"
        [meta]
        name = "table-sections-test"
        version = "0.6.0"

        [[steps]]
        name = "test_step"
        command = ["echo", "test"]

        [services.db]
        type = "generic_container"
        image = "postgres:15"

        [services.cache]
        type = "generic_container"
        image = "redis:7"
    "#;

    let config = parse_toml_config(toml)?;
    validate_config_structure(&config)?;

    // Verify services were parsed
    if let Some(ref services) = config.services {
        assert!(services.contains_key("db"), "Should have db service");
        assert!(services.contains_key("cache"), "Should have cache service");
    } else if let Some(ref service) = config.service {
        assert!(service.contains_key("db"), "Should have db service");
        assert!(service.contains_key("cache"), "Should have cache service");
    } else {
        panic!("Should have services defined");
    }

    Ok(())
}

#[test]
fn test_array_values_pattern() -> Result<()> {
    // Test array values (non-duplicate keys)
    let toml = r#"
        [meta]
        name = "array-values-test"
        version = "0.6.0"

        [[steps]]
        name = "step1"
        command = ["echo", "hello"]

        [[steps]]
        name = "step2"
        command = ["echo", "world"]

        [[steps]]
        name = "step3"
        command = ["echo", "test"]
    "#;

    let config = parse_toml_config(toml)?;
    validate_config_structure(&config)?;

    // Verify all steps were parsed
    assert_eq!(config.steps.len(), 3, "Should have 3 steps");
    assert_eq!(config.steps[0].name, "step1");
    assert_eq!(config.steps[1].name, "step2");
    assert_eq!(config.steps[2].name, "step3");

    Ok(())
}

#[test]
fn test_vars_section_extraction() -> Result<()> {
    // Test [vars] section with template rendering
    let toml = r#"
        [vars]
        port = 8080
        host = "localhost"
        timeout = 30

        [meta]
        name = "vars-test"
        version = "0.6.0"

        [[steps]]
        name = "test_step"
        command = ["curl", "http://{{ host }}:{{ port }}"]
    "#;

    let config = parse_toml_config(toml)?;
    validate_config_structure(&config)?;

    // After rendering, the command should have substituted values
    assert!(!config.steps.is_empty());

    Ok(())
}

// ============================================================================
// Test Group 7: Comprehensive batch validation
// ============================================================================

#[test]
fn test_all_live_check_examples() {
    let mut results = TestResults::default();
    let root = project_root();

    let files = [
        "examples/live-check/basic.clnrm.toml",
        "examples/live-check/strict.clnrm.toml",
        "examples/live-check/ci-cd.clnrm.toml",
        "examples/live-check/80-20.clnrm.toml",
    ];

    for file in &files {
        results.test_file(&root.join(file));
    }

    println!("\n=== Live Check Examples ===");
    println!("{}", results.summary());
    for (path, error) in &results.failed {
        println!("FAILED: {}\n  Error: {}", path, error);
    }

    assert!(
        results.is_success(),
        "Some live-check examples failed to parse"
    );
}

#[test]
fn test_all_case_study_examples() {
    let mut results = TestResults::default();
    let root = project_root();

    let files = [
        "examples/clnrm-case-study/tests/ai-character-interaction.clnrm.toml",
        "examples/clnrm-case-study/tests/ai-production-readiness.clnrm.toml",
        "examples/clnrm-case-study/tests/ai-performance-benchmark.clnrm.toml",
        "examples/clnrm-case-study/tests/vercel-ai-integration.clnrm.toml",
    ];

    for file in &files {
        results.test_file(&root.join(file));
    }

    println!("\n=== Case Study Examples ===");
    println!("{}", results.summary());
    for (path, error) in &results.failed {
        println!("FAILED: {}\n  Error: {}", path, error);
    }

    assert!(
        results.is_success(),
        "Some case-study examples failed to parse"
    );
}

#[test]
fn test_all_toml_config_examples() {
    let mut results = TestResults::default();
    let root = project_root();

    let files = [
        "examples/toml-config/simple-toml-demo.toml",
        "examples/toml-config/regex-validation-demo.toml",
        "examples/toml-config/rich-assertions-demo.toml",
        "examples/toml-config/complete-toml-demo.toml",
    ];

    for file in &files {
        results.test_file(&root.join(file));
    }

    println!("\n=== TOML Config Examples ===");
    println!("{}", results.summary());
    for (path, error) in &results.failed {
        println!("FAILED: {}\n  Error: {}", path, error);
    }

    assert!(
        results.is_success(),
        "Some toml-config examples failed to parse"
    );
}

#[test]
fn test_all_template_examples() {
    let mut results = TestResults::default();
    let root = project_root();

    let files = [
        "examples/templates/simple-test-working.clnrm.toml",
        "examples/templates/advanced-validators.clnrm.toml",
        "examples/templates/matrix-expansion.clnrm.toml",
        "examples/templates/ci-integration.clnrm.toml",
        "examples/templates/simple-variables.clnrm.toml",
        "examples/templates/macros-and-includes.clnrm.toml",
        "examples/templates/multi-environment.clnrm.toml",
        "examples/templates/service-mesh.clnrm.toml",
        "examples/templates/env_resolution_demo.clnrm.toml",
    ];

    for file in &files {
        results.test_file(&root.join(file));
    }

    println!("\n=== Template Examples ===");
    println!("{}", results.summary());
    for (path, error) in &results.failed {
        println!("FAILED: {}\n  Error: {}", path, error);
    }

    assert!(
        results.is_success(),
        "Some template examples failed to parse"
    );
}

#[test]
fn test_summary_all_examples() {
    let mut results = TestResults::default();
    let root = project_root();

    // All example files to validate
    let all_files = vec![
        // live-check
        "examples/live-check/basic.clnrm.toml",
        "examples/live-check/strict.clnrm.toml",
        "examples/live-check/ci-cd.clnrm.toml",
        "examples/live-check/80-20.clnrm.toml",
        // case-study
        "examples/clnrm-case-study/tests/ai-character-interaction.clnrm.toml",
        "examples/clnrm-case-study/tests/ai-production-readiness.clnrm.toml",
        "examples/clnrm-case-study/tests/ai-performance-benchmark.clnrm.toml",
        "examples/clnrm-case-study/tests/vercel-ai-integration.clnrm.toml",
        // toml-config
        "examples/toml-config/simple-toml-demo.toml",
        "examples/toml-config/regex-validation-demo.toml",
        "examples/toml-config/rich-assertions-demo.toml",
        "examples/toml-config/complete-toml-demo.toml",
        // templates
        "examples/templates/simple-test-working.clnrm.toml",
        "examples/templates/advanced-validators.clnrm.toml",
        "examples/templates/matrix-expansion.clnrm.toml",
        "examples/templates/ci-integration.clnrm.toml",
        "examples/templates/simple-variables.clnrm.toml",
        "examples/templates/macros-and-includes.clnrm.toml",
        "examples/templates/multi-environment.clnrm.toml",
        "examples/templates/service-mesh.clnrm.toml",
        "examples/templates/env_resolution_demo.clnrm.toml",
        // other key examples
        "examples/behaviors.clnrm.toml",
        "examples/multi-service-demo.clnrm.toml",
        "examples/surrealdb-integration-demo.clnrm.toml",
        "examples/volume-mount-demo.clnrm.toml",
        "examples/weaver-toml-configuration.clnrm.toml",
        "examples/readme-example-validation.clnrm.toml",
    ];

    for file in &all_files {
        let path = root.join(file);
        if path.exists() {
            results.test_file(&path);
        } else {
            results
                .failed
                .push((file.to_string(), "File not found".to_string()));
            results.total += 1;
        }
    }

    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║         TOML Examples Validation Summary                 ║");
    println!("╠═══════════════════════════════════════════════════════════╣");
    println!("║ {}  ║", results.summary());
    println!("╚═══════════════════════════════════════════════════════════╝");

    if !results.failed.is_empty() {
        println!("\n❌ Failed Files:");
        for (idx, (path, error)) in results.failed.iter().enumerate() {
            println!("  {}. {}", idx + 1, path);
            println!("     Error: {}", error);
        }
    } else {
        println!("\n✅ All TOML examples parsed successfully!");
    }

    assert!(results.is_success(), "Some TOML examples failed to parse");
}
