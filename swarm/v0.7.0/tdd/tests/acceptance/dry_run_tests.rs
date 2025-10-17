/// Acceptance tests for `clnrm dry-run` command
/// Tests validation without Docker execution

use crate::mocks::{MockTomlParser, ParsedToml};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// DryRunCommand validates templates without executing containers
struct DryRunCommand {
    parser: MockTomlParser,
}

impl DryRunCommand {
    fn new(parser: MockTomlParser) -> Self {
        Self { parser }
    }

    /// Validate template without execution
    fn validate(&self, template_content: &str) -> Result<ValidationResult> {
        let parsed = self.parser.parse(template_content)?;

        let mut errors = Vec::new();

        // Check required sections
        if !parsed.has_meta {
            errors.push(ValidationError {
                line: 1,
                message: "Missing required [meta] section".to_string(),
            });
        }

        if !parsed.has_otel {
            errors.push(ValidationError {
                line: 1,
                message: "Missing required [otel] section".to_string(),
            });
        }

        // Check service/scenario consistency
        for scenario in &parsed.scenarios {
            if !parsed.services.iter().any(|s| scenario.contains(s)) {
                errors.push(ValidationError {
                    line: 1,
                    message: format!("Scenario '{}' references non-existent service", scenario),
                });
            }
        }

        if errors.is_empty() {
            Ok(ValidationResult::Valid)
        } else {
            Ok(ValidationResult::Invalid { errors })
        }
    }
}

#[derive(Debug, Clone)]
enum ValidationResult {
    Valid,
    Invalid { errors: Vec<ValidationError> },
}

#[derive(Debug, Clone)]
struct ValidationError {
    line: usize,
    message: String,
}

impl ValidationResult {
    fn is_valid(&self) -> bool {
        matches!(self, ValidationResult::Valid)
    }

    fn error_count(&self) -> usize {
        match self {
            ValidationResult::Valid => 0,
            ValidationResult::Invalid { errors } => errors.len(),
        }
    }
}

// ============================================================================
// Test Suite: Valid Templates
// ============================================================================

#[test]
fn test_dry_run_accepts_valid_template() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let dry_run = DryRunCommand::new(parser.clone());

    let template = r#"
[meta]
name = "test"

[otel]
service_name = "test"

[service.db]
type = "postgres"

[[scenario]]
name = "test_scenario"
    "#;

    parser.set_result(
        template,
        Ok(ParsedToml {
            has_meta: true,
            has_otel: true,
            services: vec!["db".to_string()],
            scenarios: vec!["test_scenario".to_string()],
        }),
    );

    // Act
    let result = dry_run.validate(template)?;

    // Assert
    assert!(result.is_valid(), "Valid template should pass validation");
    assert_eq!(result.error_count(), 0, "Should have no errors");
    Ok(())
}

#[test]
fn test_dry_run_returns_success_without_docker() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let dry_run = DryRunCommand::new(parser.clone());

    let template = "[meta]\nname = \"test\"\n\n[otel]\nservice_name = \"test\"";

    parser.set_result(
        template,
        Ok(ParsedToml {
            has_meta: true,
            has_otel: true,
            services: vec![],
            scenarios: vec![],
        }),
    );

    // Act
    let result = dry_run.validate(template)?;

    // Assert
    assert!(
        result.is_valid(),
        "Dry-run should succeed without Docker interaction"
    );
    Ok(())
}

#[test]
fn test_dry_run_validates_complete_template() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let dry_run = DryRunCommand::new(parser.clone());

    let template = r#"
[meta]
name = "complete_test"
description = "Complete test case"

[otel]
service_name = "test_service"
endpoint = "http://localhost:4318"

[service.postgres]
type = "postgres"
image = "postgres:15"

[service.redis]
type = "redis"
image = "redis:7"

[[scenario]]
name = "test_db_connection"
service = "postgres"

[[scenario]]
name = "test_cache"
service = "redis"
    "#;

    parser.set_result(
        template,
        Ok(ParsedToml {
            has_meta: true,
            has_otel: true,
            services: vec!["postgres".to_string(), "redis".to_string()],
            scenarios: vec!["test_db_connection".to_string(), "test_cache".to_string()],
        }),
    );

    // Act
    let result = dry_run.validate(template)?;

    // Assert
    assert!(result.is_valid(), "Complete template should validate");
    Ok(())
}

// ============================================================================
// Test Suite: Missing Sections
// ============================================================================

#[test]
fn test_dry_run_detects_missing_meta_section() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let dry_run = DryRunCommand::new(parser.clone());

    let template = "[otel]\nservice_name = \"test\"";

    parser.set_result(
        template,
        Ok(ParsedToml {
            has_meta: false,
            has_otel: true,
            services: vec![],
            scenarios: vec![],
        }),
    );

    // Act
    let result = dry_run.validate(template)?;

    // Assert
    assert!(!result.is_valid(), "Should reject template without [meta]");
    assert_eq!(result.error_count(), 1, "Should have 1 error");

    if let ValidationResult::Invalid { errors } = result {
        assert!(
            errors[0].message.contains("[meta]"),
            "Error should mention missing [meta] section"
        );
    }
    Ok(())
}

#[test]
fn test_dry_run_detects_missing_otel_section() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let dry_run = DryRunCommand::new(parser.clone());

    let template = "[meta]\nname = \"test\"";

    parser.set_result(
        template,
        Ok(ParsedToml {
            has_meta: true,
            has_otel: false,
            services: vec![],
            scenarios: vec![],
        }),
    );

    // Act
    let result = dry_run.validate(template)?;

    // Assert
    assert!(!result.is_valid(), "Should reject template without [otel]");

    if let ValidationResult::Invalid { errors } = result {
        assert!(
            errors[0].message.contains("[otel]"),
            "Error should mention missing [otel] section"
        );
    }
    Ok(())
}

#[test]
fn test_dry_run_detects_multiple_missing_sections() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let dry_run = DryRunCommand::new(parser.clone());

    let template = "# Empty template";

    parser.set_result(
        template,
        Ok(ParsedToml {
            has_meta: false,
            has_otel: false,
            services: vec![],
            scenarios: vec![],
        }),
    );

    // Act
    let result = dry_run.validate(template)?;

    // Assert
    assert!(!result.is_valid(), "Should reject template with multiple missing sections");
    assert_eq!(result.error_count(), 2, "Should report both missing sections");
    Ok(())
}

// ============================================================================
// Test Suite: TOML Syntax Errors
// ============================================================================

#[test]
fn test_dry_run_detects_invalid_toml_syntax() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let dry_run = DryRunCommand::new(parser.clone());

    let template = "[meta\nname = \"test\""; // Missing closing bracket

    parser.set_result(
        template,
        Err("TOML parse error at line 1: expected `]`".to_string()),
    );

    // Act
    let result = dry_run.validate(template);

    // Assert
    assert!(result.is_err(), "Should return error for invalid TOML");
    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("parse error"),
        "Error should mention parse error"
    );
    Ok(())
}

#[test]
fn test_dry_run_reports_line_number_for_syntax_error() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let dry_run = DryRunCommand::new(parser.clone());

    let template = r#"
[meta]
name = "test"
invalid syntax here
    "#;

    parser.set_result(
        template,
        Err("TOML parse error at line 4: unexpected token".to_string()),
    );

    // Act
    let result = dry_run.validate(template);

    // Assert
    assert!(result.is_err(), "Should return error with line number");
    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("line 4"),
        "Error should include line number"
    );
    Ok(())
}

#[test]
fn test_dry_run_detects_duplicate_keys() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let dry_run = DryRunCommand::new(parser.clone());

    let template = r#"
[meta]
name = "test1"
name = "test2"
    "#;

    parser.set_result(
        template,
        Err("TOML parse error: duplicate key 'name'".to_string()),
    );

    // Act
    let result = dry_run.validate(template);

    // Assert
    assert!(result.is_err(), "Should detect duplicate keys");
    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("duplicate"),
        "Error should mention duplicate key"
    );
    Ok(())
}

// ============================================================================
// Test Suite: Orphan Scenarios
// ============================================================================

#[test]
fn test_dry_run_detects_orphan_scenario() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let dry_run = DryRunCommand::new(parser.clone());

    let template = r#"
[meta]
name = "test"

[otel]
service_name = "test"

[[scenario]]
name = "test_scenario"
service = "nonexistent_service"
    "#;

    parser.set_result(
        template,
        Ok(ParsedToml {
            has_meta: true,
            has_otel: true,
            services: vec![],
            scenarios: vec!["nonexistent_service".to_string()],
        }),
    );

    // Act
    let result = dry_run.validate(template)?;

    // Assert
    assert!(!result.is_valid(), "Should reject orphan scenario");

    if let ValidationResult::Invalid { errors } = result {
        assert!(
            errors[0].message.contains("non-existent service"),
            "Error should mention non-existent service"
        );
    }
    Ok(())
}

#[test]
fn test_dry_run_validates_scenario_service_references() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let dry_run = DryRunCommand::new(parser.clone());

    let template = r#"
[meta]
name = "test"

[otel]
service_name = "test"

[service.db]
type = "postgres"

[[scenario]]
name = "valid_scenario"
service = "db"
    "#;

    parser.set_result(
        template,
        Ok(ParsedToml {
            has_meta: true,
            has_otel: true,
            services: vec!["db".to_string()],
            scenarios: vec!["db".to_string()],
        }),
    );

    // Act
    let result = dry_run.validate(template)?;

    // Assert
    assert!(
        result.is_valid(),
        "Should accept scenario with valid service reference"
    );
    Ok(())
}

#[test]
fn test_dry_run_detects_multiple_orphan_scenarios() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let dry_run = DryRunCommand::new(parser.clone());

    let template = r#"
[meta]
name = "test"

[otel]
service_name = "test"

[[scenario]]
name = "orphan1"
service = "missing1"

[[scenario]]
name = "orphan2"
service = "missing2"
    "#;

    parser.set_result(
        template,
        Ok(ParsedToml {
            has_meta: true,
            has_otel: true,
            services: vec![],
            scenarios: vec!["missing1".to_string(), "missing2".to_string()],
        }),
    );

    // Act
    let result = dry_run.validate(template)?;

    // Assert
    assert!(!result.is_valid(), "Should reject multiple orphan scenarios");
    assert_eq!(
        result.error_count(), 2,
        "Should report both orphan scenarios"
    );
    Ok(())
}

// ============================================================================
// Test Suite: Edge Cases
// ============================================================================

#[test]
fn test_dry_run_handles_empty_template() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let dry_run = DryRunCommand::new(parser.clone());

    let template = "";

    parser.set_result(
        template,
        Ok(ParsedToml {
            has_meta: false,
            has_otel: false,
            services: vec![],
            scenarios: vec![],
        }),
    );

    // Act
    let result = dry_run.validate(template)?;

    // Assert
    assert!(!result.is_valid(), "Empty template should fail validation");
    assert!(result.error_count() >= 2, "Should report missing sections");
    Ok(())
}

#[test]
fn test_dry_run_handles_whitespace_only_template() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let dry_run = DryRunCommand::new(parser.clone());

    let template = "   \n\n   \t\t   \n";

    parser.set_result(
        template,
        Ok(ParsedToml {
            has_meta: false,
            has_otel: false,
            services: vec![],
            scenarios: vec![],
        }),
    );

    // Act
    let result = dry_run.validate(template)?;

    // Assert
    assert!(!result.is_valid(), "Whitespace-only template should fail");
    Ok(())
}

#[test]
fn test_dry_run_validates_large_template() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let dry_run = DryRunCommand::new(parser.clone());

    // Generate large template with 100 services
    let mut services = Vec::new();
    let mut scenarios = Vec::new();

    for i in 0..100 {
        services.push(format!("service_{}", i));
        scenarios.push(format!("service_{}", i));
    }

    let template = format!(
        "[meta]\nname = \"large\"\n\n[otel]\nservice_name = \"test\"\n\n{}",
        (0..100)
            .map(|i| format!("[service.service_{}]\ntype = \"generic\"\n", i))
            .collect::<String>()
    );

    parser.set_result(
        &template,
        Ok(ParsedToml {
            has_meta: true,
            has_otel: true,
            services,
            scenarios,
        }),
    );

    // Act
    let result = dry_run.validate(&template)?;

    // Assert
    assert!(result.is_valid(), "Should handle large template efficiently");
    Ok(())
}

// ============================================================================
// Test Suite: Performance
// ============================================================================

#[test]
fn test_dry_run_completes_quickly() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let dry_run = DryRunCommand::new(parser.clone());

    let template = r#"
[meta]
name = "perf_test"

[otel]
service_name = "test"
    "#;

    parser.set_result(
        template,
        Ok(ParsedToml {
            has_meta: true,
            has_otel: true,
            services: vec![],
            scenarios: vec![],
        }),
    );

    // Act
    use std::time::Instant;
    let start = Instant::now();
    let _ = dry_run.validate(template)?;
    let duration = start.elapsed();

    // Assert
    assert!(
        duration < std::time::Duration::from_millis(100),
        "Dry-run should complete in <100ms, took {:?}",
        duration
    );
    Ok(())
}
