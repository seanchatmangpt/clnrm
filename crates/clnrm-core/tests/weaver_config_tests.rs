//! Comprehensive tests for Weaver live-check configuration parsing and validation
//!
//! Tests cover:
//! - TOML parsing of all configuration variants
//! - Default value handling
//! - Validation logic for all constraints
//! - Backward compatibility with v1.2.1 TOML files
//! - Error handling and user-friendly error messages

use clnrm_core::config::{TestConfig, WeaverConfig};
use clnrm_core::error::Result;

// ============================================================================
// Basic Parsing Tests
// ============================================================================

#[test]
fn test_parse_minimal_weaver_config() -> Result<()> {
    let toml = r#"
        [test.metadata]
        name = "minimal_test"
        description = "Test with minimal Weaver config"

        [weaver]
        enabled = true

        [[steps]]
        name = "test_step"
        command = ["echo", "hello"]
    "#;

    let config: TestConfig = toml::from_str(toml)?;
    assert!(config.weaver.is_some());

    let weaver = config.weaver.unwrap();
    assert!(weaver.enabled);
    assert_eq!(weaver.registry_path, "registry");
    assert_eq!(weaver.otlp_port, 0); // Auto-discover
    assert_eq!(weaver.admin_port, 0); // Auto-discover

    Ok(())
}

#[test]
fn test_parse_complete_weaver_config() -> Result<()> {
    let toml = r#"
        [test.metadata]
        name = "complete_test"
        description = "Test with complete Weaver configuration"

        [weaver]
        enabled = true
        registry_path = "custom/registry/"
        otlp_port = 4317
        admin_port = 4318
        output_dir = "./custom_validation_output"
        stream = true
        fail_fast = true

        [weaver.validation]
        mode = "strict"
        fail_on_violation = true
        fail_on_missing_optional = false
        coverage_threshold = 95.0
        inactivity_timeout = 10
        diagnostic_format = "json"

        [weaver.collector]
        use_existing = false
        auto_start = true
        image = "otel/opentelemetry-collector:0.91.0"
        health_check_timeout = 60
        startup_grace_period = 5

        [weaver.reports]
        json_report = true
        json_report_file = "custom_validation.json"
        html_report = true
        html_report_file = "custom_validation.html"
        junit_report = true
        junit_report_file = "custom_validation.xml"
        include_samples = true
        max_samples_per_violation = 5

        [weaver.performance]
        buffer_size = 2097152
        max_workers = 8
        batching = true
        batch_size = 200
        batch_timeout_ms = 2000

        [[steps]]
        name = "test_step"
        command = ["echo", "hello"]
    "#;

    let config: TestConfig = toml::from_str(toml)?;
    let weaver = config.weaver.expect("Weaver config should be present");

    // Core config
    assert!(weaver.enabled);
    assert_eq!(weaver.registry_path, "custom/registry/");
    assert_eq!(weaver.otlp_port, 4317);
    assert_eq!(weaver.admin_port, 4318);
    assert_eq!(weaver.output_dir, "./custom_validation_output");
    assert!(weaver.stream);
    assert!(weaver.fail_fast);

    // Validation config
    let validation = weaver
        .validation
        .expect("Validation config should be present");
    assert_eq!(
        validation.mode,
        clnrm_core::config::weaver::ValidationMode::Strict
    );
    assert!(validation.fail_on_violation);
    assert!(!validation.fail_on_missing_optional);
    assert_eq!(validation.coverage_threshold, 95.0);
    assert_eq!(validation.inactivity_timeout, 10);
    assert_eq!(
        validation.diagnostic_format,
        clnrm_core::config::weaver::DiagnosticFormat::Json
    );

    // Collector config
    let collector = weaver
        .collector
        .expect("Collector config should be present");
    assert!(!collector.use_existing);
    assert!(collector.auto_start);
    assert_eq!(collector.image, "otel/opentelemetry-collector:0.91.0");
    assert_eq!(collector.health_check_timeout, 60);
    assert_eq!(collector.startup_grace_period, 5);

    // Reports config
    let reports = weaver.reports.expect("Reports config should be present");
    assert!(reports.json_report);
    assert_eq!(reports.json_report_file, "custom_validation.json");
    assert!(reports.html_report);
    assert_eq!(reports.html_report_file, "custom_validation.html");
    assert!(reports.junit_report);
    assert_eq!(reports.junit_report_file, "custom_validation.xml");
    assert!(reports.include_samples);
    assert_eq!(reports.max_samples_per_violation, 5);

    // Performance config
    let perf = weaver
        .performance
        .expect("Performance config should be present");
    assert_eq!(perf.buffer_size, 2097152);
    assert_eq!(perf.max_workers, 8);
    assert!(perf.batching);
    assert_eq!(perf.batch_size, 200);
    assert_eq!(perf.batch_timeout_ms, 2000);

    Ok(())
}

#[test]
fn test_parse_80_20_validation_mode() -> Result<()> {
    let toml = r#"
        [test.metadata]
        name = "eighty_twenty_test"
        description = "Test with 80/20 validation mode"

        [weaver]
        enabled = true
        registry_path = "registry/"

        [weaver.validation]
        mode = "80_20"
        coverage_threshold = 80.0

        [weaver.80_20]
        enabled = true
        critical_spans = [
            "clnrm.test.execute",
            "clnrm.container.start",
            "clnrm.container.cleanup"
        ]
        required_attributes = [
            "clnrm.version",
            "test.hermetic",
            "test.isolated",
            "container.id"
        ]
        optional_attributes = [
            "service.name",
            "service.type",
            "container.image"
        ]
        critical_span_coverage = 100.0
        required_attribute_coverage = 100.0
        optional_attribute_coverage = 50.0

        [[steps]]
        name = "test_step"
        command = ["echo", "hello"]
    "#;

    let config: TestConfig = toml::from_str(toml)?;
    let weaver = config.weaver.expect("Weaver config should be present");

    let validation = weaver
        .validation
        .expect("Validation config should be present");
    assert_eq!(
        validation.mode,
        clnrm_core::config::weaver::ValidationMode::EightyTwenty
    );
    assert_eq!(validation.coverage_threshold, 80.0);

    let eighty_twenty = weaver
        .eighty_twenty
        .expect("80/20 config should be present");
    assert!(eighty_twenty.enabled);
    assert_eq!(eighty_twenty.critical_spans.len(), 3);
    assert_eq!(eighty_twenty.required_attributes.len(), 4);
    assert_eq!(eighty_twenty.optional_attributes.len(), 3);
    assert_eq!(eighty_twenty.critical_span_coverage, 100.0);
    assert_eq!(eighty_twenty.required_attribute_coverage, 100.0);
    assert_eq!(eighty_twenty.optional_attribute_coverage, 50.0);

    Ok(())
}

#[test]
fn test_parse_lenient_validation_mode() -> Result<()> {
    let toml = r#"
        [test.metadata]
        name = "lenient_test"
        description = "Test with lenient validation mode"

        [weaver]
        enabled = true

        [weaver.validation]
        mode = "lenient"
        fail_on_violation = false
        coverage_threshold = 50.0

        [[steps]]
        name = "test_step"
        command = ["echo", "hello"]
    "#;

    let config: TestConfig = toml::from_str(toml)?;
    let weaver = config.weaver.expect("Weaver config should be present");
    let validation = weaver
        .validation
        .expect("Validation config should be present");

    assert_eq!(
        validation.mode,
        clnrm_core::config::weaver::ValidationMode::Lenient
    );
    assert!(!validation.fail_on_violation);
    assert_eq!(validation.coverage_threshold, 50.0);

    Ok(())
}

#[test]
fn test_parse_existing_collector_config() -> Result<()> {
    let toml = r#"
        [test.metadata]
        name = "existing_collector_test"
        description = "Test using existing collector"

        [weaver]
        enabled = true

        [weaver.collector]
        use_existing = true
        endpoint = "http://localhost:4317"
        auto_start = false

        [[steps]]
        name = "test_step"
        command = ["echo", "hello"]
    "#;

    let config: TestConfig = toml::from_str(toml)?;
    let weaver = config.weaver.expect("Weaver config should be present");
    let collector = weaver
        .collector
        .expect("Collector config should be present");

    assert!(collector.use_existing);
    assert_eq!(
        collector.endpoint,
        Some("http://localhost:4317".to_string())
    );
    assert!(!collector.auto_start);

    Ok(())
}

// ============================================================================
// Default Value Tests
// ============================================================================

#[test]
fn test_weaver_config_defaults() {
    let config = WeaverConfig::default();

    assert!(config.enabled);
    assert_eq!(config.registry_path, "registry");
    assert_eq!(config.otlp_port, 0);
    assert_eq!(config.admin_port, 0);
    assert_eq!(config.output_dir, "./validation_output");
    assert!(!config.stream);
    assert!(!config.fail_fast);

    // Check nested defaults
    assert!(config.validation.is_some());
    assert!(config.collector.is_some());
    assert!(config.reports.is_some());
    assert!(config.performance.is_some());
}

#[test]
fn test_parse_weaver_with_partial_config() -> Result<()> {
    let toml = r#"
        [test.metadata]
        name = "partial_config_test"

        [weaver]
        enabled = true
        otlp_port = 5000

        [[steps]]
        name = "test_step"
        command = ["echo", "hello"]
    "#;

    let config: TestConfig = toml::from_str(toml)?;
    let weaver = config.weaver.expect("Weaver config should be present");

    // Explicitly set value
    assert_eq!(weaver.otlp_port, 5000);

    // Should use defaults for unspecified fields
    assert_eq!(weaver.registry_path, "registry");
    assert_eq!(weaver.admin_port, 0);
    assert!(!weaver.stream);

    Ok(())
}

// ============================================================================
// Validation Tests
// ============================================================================

#[test]
fn test_validation_fails_for_invalid_otlp_port() {
    let mut config = WeaverConfig::default();
    config.otlp_port = 80; // Invalid (< 1024)

    let result = config.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("OTLP port must be >= 1024"));
}

#[test]
fn test_validation_fails_for_invalid_admin_port() {
    let mut config = WeaverConfig::default();
    config.admin_port = 1023; // Invalid (< 1024)

    let result = config.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Admin port must be >= 1024"));
}

#[test]
fn test_validation_fails_for_port_conflict() {
    let mut config = WeaverConfig::default();
    config.otlp_port = 5000;
    config.admin_port = 5000; // Conflict

    let result = config.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("must be different"));
}

#[test]
fn test_validation_succeeds_for_valid_ports() {
    let mut config = WeaverConfig::default();
    config.otlp_port = 4317;
    config.admin_port = 4318;

    let result = config.validate();
    assert!(result.is_ok());
}

#[test]
fn test_validation_succeeds_for_auto_discover_ports() {
    let config = WeaverConfig::default(); // otlp_port = 0, admin_port = 0

    let result = config.validate();
    assert!(result.is_ok());
}

#[test]
fn test_validation_fails_for_invalid_coverage_threshold() {
    use clnrm_core::config::weaver::{DiagnosticFormat, ValidationConfig, ValidationMode};

    let validation_config = ValidationConfig {
        mode: ValidationMode::Strict,
        fail_on_violation: true,
        fail_on_missing_optional: false,
        coverage_threshold: 150.0, // Invalid (> 100)
        inactivity_timeout: 5,
        diagnostic_format: DiagnosticFormat::Ansi,
    };

    let result = validation_config.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("between 0.0 and 100.0"));
}

#[test]
fn test_validation_fails_for_zero_inactivity_timeout() {
    use clnrm_core::config::weaver::{DiagnosticFormat, ValidationConfig, ValidationMode};

    let validation_config = ValidationConfig {
        mode: ValidationMode::Strict,
        fail_on_violation: true,
        fail_on_missing_optional: false,
        coverage_threshold: 80.0,
        inactivity_timeout: 0, // Invalid
        diagnostic_format: DiagnosticFormat::Ansi,
    };

    let result = validation_config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("greater than 0"));
}

#[test]
fn test_validation_fails_for_80_20_mode_without_config() {
    let toml = r#"
        [test.metadata]
        name = "invalid_80_20_test"

        [weaver]
        enabled = true

        [weaver.validation]
        mode = "80_20"

        [[steps]]
        name = "test_step"
        command = ["echo", "hello"]
    "#;

    let config: TestConfig = toml::from_str(toml).expect("Should parse TOML");
    let result = config.validate();

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("80/20 validation mode requires"));
}

#[test]
fn test_validation_fails_for_80_20_without_critical_spans() {
    use clnrm_core::config::weaver::EightyTwentyConfig;

    let config = EightyTwentyConfig {
        enabled: true,
        critical_spans: Vec::new(), // Empty
        required_attributes: vec!["clnrm.version".to_string()],
        optional_attributes: Vec::new(),
        critical_span_coverage: 100.0,
        required_attribute_coverage: 100.0,
        optional_attribute_coverage: 50.0,
    };

    let result = config.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("at least one critical span"));
}

#[test]
fn test_validation_fails_for_80_20_without_required_attributes() {
    use clnrm_core::config::weaver::EightyTwentyConfig;

    let config = EightyTwentyConfig {
        enabled: true,
        critical_spans: vec!["clnrm.test.execute".to_string()],
        required_attributes: Vec::new(), // Empty
        optional_attributes: Vec::new(),
        critical_span_coverage: 100.0,
        required_attribute_coverage: 100.0,
        optional_attribute_coverage: 50.0,
    };

    let result = config.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("at least one required attribute"));
}

#[test]
fn test_validation_fails_for_use_existing_without_endpoint() {
    use clnrm_core::config::weaver::CollectorConfig;

    let config = CollectorConfig {
        use_existing: true,
        endpoint: None, // Missing endpoint
        auto_start: false,
        image: "otel/opentelemetry-collector:latest".to_string(),
        health_check_timeout: 30,
        startup_grace_period: 2,
    };

    let result = config.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("endpoint must be provided"));
}

#[test]
fn test_validation_fails_for_invalid_endpoint_format() {
    use clnrm_core::config::weaver::CollectorConfig;

    let config = CollectorConfig {
        use_existing: true,
        endpoint: Some("localhost:4317".to_string()), // Missing http://
        auto_start: false,
        image: "otel/opentelemetry-collector:latest".to_string(),
        health_check_timeout: 30,
        startup_grace_period: 2,
    };

    let result = config.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("must start with http://"));
}

#[test]
fn test_validation_fails_for_zero_health_check_timeout() {
    use clnrm_core::config::weaver::CollectorConfig;

    let config = CollectorConfig {
        use_existing: false,
        endpoint: None,
        auto_start: true,
        image: "otel/opentelemetry-collector:latest".to_string(),
        health_check_timeout: 0, // Invalid
        startup_grace_period: 2,
    };

    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("greater than 0"));
}

// ============================================================================
// Backward Compatibility Tests
// ============================================================================

#[test]
fn test_v1_2_1_toml_without_weaver_still_works() -> Result<()> {
    // v1.2.1 TOML without [weaver] section should parse successfully
    let toml = r#"
        [test.metadata]
        name = "legacy_test"
        description = "v1.2.1 test without Weaver"

        [services.alpine]
        type = "generic_container"
        image = "alpine:latest"

        [[steps]]
        name = "hello"
        command = ["echo", "hello"]
        expected_output_regex = "hello"
    "#;

    let config: TestConfig = toml::from_str(toml)?;

    // Weaver config should be None (not enabled)
    assert!(config.weaver.is_none());

    // Should validate successfully
    config.validate()?;

    Ok(())
}

#[test]
fn test_v1_2_1_format_compatibility() -> Result<()> {
    // Ensure v1.2.1 format with services and steps still works
    let toml = r#"
        [test.metadata]
        name = "v1_2_1_test"
        description = "Test using v1.2.1 format"

        [services.postgres]
        type = "generic_container"
        image = "postgres:14"

        [services.postgres.env]
        POSTGRES_PASSWORD = "secret"

        [[steps]]
        name = "wait_for_db"
        command = ["pg_isready"]
        service = "postgres"
    "#;

    let config: TestConfig = toml::from_str(toml)?;
    assert!(config.weaver.is_none());
    config.validate()?;

    Ok(())
}

// ============================================================================
// Enum Parsing Tests
// ============================================================================

#[test]
fn test_validation_mode_enum_parsing() -> Result<()> {
    let test_cases = vec![
        ("strict", clnrm_core::config::weaver::ValidationMode::Strict),
        (
            "lenient",
            clnrm_core::config::weaver::ValidationMode::Lenient,
        ),
        (
            "80_20",
            clnrm_core::config::weaver::ValidationMode::EightyTwenty,
        ),
        (
            "minimal",
            clnrm_core::config::weaver::ValidationMode::Minimal,
        ),
    ];

    for (mode_str, expected_mode) in test_cases {
        let toml = format!(
            r#"
            [test.metadata]
            name = "mode_test"

            [weaver]
            enabled = true

            [weaver.validation]
            mode = "{}"

            [[steps]]
            name = "test"
            command = ["echo", "hello"]
        "#,
            mode_str
        );

        let config: TestConfig = toml::from_str(&toml)?;
        let weaver = config.weaver.expect("Weaver config should be present");
        let validation = weaver
            .validation
            .expect("Validation config should be present");

        assert_eq!(validation.mode, expected_mode);
    }

    Ok(())
}

#[test]
fn test_diagnostic_format_enum_parsing() -> Result<()> {
    let test_cases = vec![
        ("ansi", clnrm_core::config::weaver::DiagnosticFormat::Ansi),
        ("json", clnrm_core::config::weaver::DiagnosticFormat::Json),
        (
            "gh_workflow_command",
            clnrm_core::config::weaver::DiagnosticFormat::GhWorkflowCommand,
        ),
        ("auto", clnrm_core::config::weaver::DiagnosticFormat::Auto),
    ];

    for (format_str, expected_format) in test_cases {
        let toml = format!(
            r#"
            [test.metadata]
            name = "format_test"

            [weaver]
            enabled = true

            [weaver.validation]
            diagnostic_format = "{}"

            [[steps]]
            name = "test"
            command = ["echo", "hello"]
        "#,
            format_str
        );

        let config: TestConfig = toml::from_str(&toml)?;
        let weaver = config.weaver.expect("Weaver config should be present");
        let validation = weaver
            .validation
            .expect("Validation config should be present");

        assert_eq!(validation.diagnostic_format, expected_format);
    }

    Ok(())
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_weaver_disabled_explicitly() -> Result<()> {
    let toml = r#"
        [test.metadata]
        name = "disabled_test"

        [weaver]
        enabled = false

        [[steps]]
        name = "test"
        command = ["echo", "hello"]
    "#;

    let config: TestConfig = toml::from_str(toml)?;
    let weaver = config.weaver.expect("Weaver config should be present");

    assert!(!weaver.enabled);

    Ok(())
}

#[test]
fn test_empty_optional_attributes_allowed() -> Result<()> {
    let toml = r#"
        [test.metadata]
        name = "empty_optional_test"

        [weaver]
        enabled = true

        [weaver.validation]
        mode = "80_20"

        [weaver.80_20]
        enabled = true
        critical_spans = ["clnrm.test.execute"]
        required_attributes = ["clnrm.version"]
        optional_attributes = []

        [[steps]]
        name = "test"
        command = ["echo", "hello"]
    "#;

    let config: TestConfig = toml::from_str(toml)?;
    let weaver = config.weaver.expect("Weaver config should be present");
    let eighty_twenty = weaver
        .eighty_twenty
        .expect("80/20 config should be present");

    assert!(eighty_twenty.optional_attributes.is_empty());
    config.validate()?;

    Ok(())
}

#[test]
fn test_home_directory_path_resolution() -> Result<()> {
    let toml = r#"
        [test.metadata]
        name = "home_path_test"

        [weaver]
        enabled = true
        registry_path = "~/clnrm/registry"

        [[steps]]
        name = "test"
        command = ["echo", "hello"]
    "#;

    let config: TestConfig = toml::from_str(toml)?;
    let weaver = config.weaver.expect("Weaver config should be present");

    assert_eq!(weaver.registry_path, "~/clnrm/registry");

    Ok(())
}

#[test]
fn test_absolute_path_registry() -> Result<()> {
    let toml = r#"
        [test.metadata]
        name = "absolute_path_test"

        [weaver]
        enabled = true
        registry_path = "/usr/local/share/clnrm/registry"

        [[steps]]
        name = "test"
        command = ["echo", "hello"]
    "#;

    let config: TestConfig = toml::from_str(toml)?;
    let weaver = config.weaver.expect("Weaver config should be present");

    assert_eq!(weaver.registry_path, "/usr/local/share/clnrm/registry");

    Ok(())
}

// ============================================================================
// CI/CD Integration Tests
// ============================================================================

#[test]
fn test_ci_cd_pipeline_config() -> Result<()> {
    let toml = r#"
        [test.metadata]
        name = "ci_pipeline_test"
        description = "CI/CD with GitHub Actions integration"

        [weaver]
        enabled = true
        registry_path = "registry/"

        [weaver.validation]
        mode = "strict"
        fail_on_violation = true
        coverage_threshold = 90.0
        diagnostic_format = "gh_workflow_command"

        [weaver.reports]
        json_report = true
        junit_report = true
        junit_report_file = "weaver_results.xml"

        [weaver.collector]
        auto_start = true
        health_check_timeout = 60

        [[steps]]
        name = "ci_test"
        command = ["make", "test"]
    "#;

    let config: TestConfig = toml::from_str(toml)?;
    let weaver = config.weaver.expect("Weaver config should be present");

    let validation = weaver
        .validation
        .expect("Validation config should be present");
    assert_eq!(
        validation.diagnostic_format,
        clnrm_core::config::weaver::DiagnosticFormat::GhWorkflowCommand
    );
    assert_eq!(validation.coverage_threshold, 90.0);

    let reports = weaver.reports.expect("Reports config should be present");
    assert!(reports.junit_report);
    assert_eq!(reports.junit_report_file, "weaver_results.xml");

    Ok(())
}
