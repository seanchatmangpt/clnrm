/// Acceptance tests for `clnrm lint` command
/// Tests validation rules and error reporting

use crate::mocks::{MockTomlParser, ParsedToml};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// LintCommand validates template structure and requirements
struct LintCommand {
    parser: MockTomlParser,
}

impl LintCommand {
    fn new(parser: MockTomlParser) -> Self {
        Self { parser }
    }

    /// Lint template and return violations
    fn lint(&self, template_content: &str) -> Result<LintResult> {
        let parsed = self.parser.parse(template_content)?;

        let mut violations = Vec::new();

        // Rule: [otel] section is required
        if !parsed.has_otel {
            violations.push(LintViolation {
                rule: "required-otel-section".to_string(),
                severity: Severity::Error,
                message: "Missing required [otel] section".to_string(),
                line: 1,
            });
        }

        // Rule: [service.*] section is required
        if parsed.services.is_empty() {
            violations.push(LintViolation {
                rule: "required-service-section".to_string(),
                severity: Severity::Error,
                message: "At least one [service.*] section is required".to_string(),
                line: 1,
            });
        }

        // Rule: Scenarios must reference valid services
        for scenario in &parsed.scenarios {
            if !parsed.services.iter().any(|s| scenario.contains(s)) {
                violations.push(LintViolation {
                    rule: "valid-service-reference".to_string(),
                    severity: Severity::Error,
                    message: format!("Scenario references non-existent service: {}", scenario),
                    line: 1,
                });
            }
        }

        if violations.is_empty() {
            Ok(LintResult::Clean)
        } else {
            Ok(LintResult::HasViolations { violations })
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum LintResult {
    Clean,
    HasViolations { violations: Vec<LintViolation> },
}

#[derive(Debug, Clone, PartialEq)]
struct LintViolation {
    rule: String,
    severity: Severity,
    message: String,
    line: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum Severity {
    Error,
    Warning,
}

impl LintResult {
    fn is_clean(&self) -> bool {
        matches!(self, LintResult::Clean)
    }

    fn violation_count(&self) -> usize {
        match self {
            LintResult::Clean => 0,
            LintResult::HasViolations { violations } => violations.len(),
        }
    }

    fn error_count(&self) -> usize {
        match self {
            LintResult::Clean => 0,
            LintResult::HasViolations { violations } => {
                violations.iter().filter(|v| matches!(v.severity, Severity::Error)).count()
            }
        }
    }

    fn exit_code(&self) -> i32 {
        if self.error_count() > 0 { 1 } else { 0 }
    }
}

// ============================================================================
// Test Suite: Required Sections
// ============================================================================

#[test]
fn test_lint_detects_missing_otel_section() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let lint_cmd = LintCommand::new(parser.clone());

    let template = r#"
[meta]
name = "test"

[service.db]
type = "postgres"
    "#;

    parser.set_result(
        template,
        Ok(ParsedToml {
            has_meta: true,
            has_otel: false,
            services: vec!["db".to_string()],
            scenarios: vec![],
        }),
    );

    // Act
    let result = lint_cmd.lint(template)?;

    // Assert
    assert!(!result.is_clean(), "Should detect missing [otel] section");
    assert_eq!(result.error_count(), 1);

    if let LintResult::HasViolations { violations } = result {
        assert_eq!(violations[0].rule, "required-otel-section");
        assert!(violations[0].message.contains("[otel]"));
    }
    Ok(())
}

#[test]
fn test_lint_detects_missing_service_section() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let lint_cmd = LintCommand::new(parser.clone());

    let template = r#"
[meta]
name = "test"

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
    let result = lint_cmd.lint(template)?;

    // Assert
    assert!(!result.is_clean(), "Should detect missing [service.*] section");
    assert_eq!(result.error_count(), 1);

    if let LintResult::HasViolations { violations } = result {
        assert_eq!(violations[0].rule, "required-service-section");
        assert!(violations[0].message.contains("[service.*]"));
    }
    Ok(())
}

#[test]
fn test_lint_accepts_all_required_sections() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let lint_cmd = LintCommand::new(parser.clone());

    let template = r#"
[meta]
name = "test"

[otel]
service_name = "test"

[service.db]
type = "postgres"
    "#;

    parser.set_result(
        template,
        Ok(ParsedToml {
            has_meta: true,
            has_otel: true,
            services: vec!["db".to_string()],
            scenarios: vec![],
        }),
    );

    // Act
    let result = lint_cmd.lint(template)?;

    // Assert
    assert!(result.is_clean(), "Should accept template with all required sections");
    assert_eq!(result.exit_code(), 0);
    Ok(())
}

// ============================================================================
// Test Suite: Service References
// ============================================================================

#[test]
fn test_lint_detects_invalid_service_reference() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let lint_cmd = LintCommand::new(parser.clone());

    let template = r#"
[meta]
name = "test"

[otel]
service_name = "test"

[service.db]
type = "postgres"

[[scenario]]
name = "test"
service = "nonexistent"
    "#;

    parser.set_result(
        template,
        Ok(ParsedToml {
            has_meta: true,
            has_otel: true,
            services: vec!["db".to_string()],
            scenarios: vec!["nonexistent".to_string()],
        }),
    );

    // Act
    let result = lint_cmd.lint(template)?;

    // Assert
    assert!(!result.is_clean(), "Should detect invalid service reference");
    assert_eq!(result.error_count(), 1);

    if let LintResult::HasViolations { violations } = result {
        assert_eq!(violations[0].rule, "valid-service-reference");
        assert!(violations[0].message.contains("non-existent service"));
    }
    Ok(())
}

#[test]
fn test_lint_accepts_valid_service_reference() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let lint_cmd = LintCommand::new(parser.clone());

    let template = r#"
[meta]
name = "test"

[otel]
service_name = "test"

[service.db]
type = "postgres"

[[scenario]]
name = "test"
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
    let result = lint_cmd.lint(template)?;

    // Assert
    assert!(result.is_clean(), "Should accept valid service reference");
    Ok(())
}

#[test]
fn test_lint_detects_multiple_invalid_references() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let lint_cmd = LintCommand::new(parser.clone());

    let template = r#"
[meta]
name = "test"

[otel]
service_name = "test"

[service.db]
type = "postgres"

[[scenario]]
name = "test1"
service = "missing1"

[[scenario]]
name = "test2"
service = "missing2"
    "#;

    parser.set_result(
        template,
        Ok(ParsedToml {
            has_meta: true,
            has_otel: true,
            services: vec!["db".to_string()],
            scenarios: vec!["missing1".to_string(), "missing2".to_string()],
        }),
    );

    // Act
    let result = lint_cmd.lint(template)?;

    // Assert
    assert_eq!(result.error_count(), 2, "Should detect both invalid references");
    Ok(())
}

// ============================================================================
// Test Suite: Multiple Violations
// ============================================================================

#[test]
fn test_lint_detects_multiple_violations() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let lint_cmd = LintCommand::new(parser.clone());

    let template = r#"
[meta]
name = "test"
    "#;

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
    let result = lint_cmd.lint(template)?;

    // Assert
    assert!(!result.is_clean(), "Should detect multiple violations");
    assert_eq!(result.violation_count(), 2, "Should report both violations");
    assert_eq!(result.error_count(), 2, "Both should be errors");
    Ok(())
}

#[test]
fn test_lint_reports_all_violations_not_just_first() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let lint_cmd = LintCommand::new(parser.clone());

    let template = "# Incomplete template";

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
    let result = lint_cmd.lint(template)?;

    // Assert
    assert_eq!(
        result.violation_count(), 2,
        "Should report all violations, not fail-fast"
    );
    Ok(())
}

// ============================================================================
// Test Suite: Exit Codes
// ============================================================================

#[test]
fn test_lint_returns_exit_code_one_on_error() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let lint_cmd = LintCommand::new(parser.clone());

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
    let result = lint_cmd.lint(template)?;

    // Assert
    assert_eq!(result.exit_code(), 1, "Should return exit code 1 on errors");
    Ok(())
}

#[test]
fn test_lint_returns_exit_code_zero_on_success() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let lint_cmd = LintCommand::new(parser.clone());

    let template = r#"
[meta]
name = "test"

[otel]
service_name = "test"

[service.db]
type = "postgres"
    "#;

    parser.set_result(
        template,
        Ok(ParsedToml {
            has_meta: true,
            has_otel: true,
            services: vec!["db".to_string()],
            scenarios: vec![],
        }),
    );

    // Act
    let result = lint_cmd.lint(template)?;

    // Assert
    assert_eq!(result.exit_code(), 0, "Should return exit code 0 on success");
    Ok(())
}

// ============================================================================
// Test Suite: Error Messages
// ============================================================================

#[test]
fn test_lint_provides_helpful_error_messages() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let lint_cmd = LintCommand::new(parser.clone());

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
    let result = lint_cmd.lint(template)?;

    // Assert
    if let LintResult::HasViolations { violations } = result {
        for violation in violations {
            assert!(!violation.message.is_empty(), "Error message should not be empty");
            assert!(
                violation.message.len() > 10,
                "Error message should be descriptive"
            );
        }
    }
    Ok(())
}

#[test]
fn test_lint_includes_rule_names_in_violations() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let lint_cmd = LintCommand::new(parser.clone());

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
    let result = lint_cmd.lint(template)?;

    // Assert
    if let LintResult::HasViolations { violations } = result {
        for violation in violations {
            assert!(
                !violation.rule.is_empty(),
                "Each violation should have a rule name"
            );
        }
    }
    Ok(())
}

#[test]
fn test_lint_includes_line_numbers() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let lint_cmd = LintCommand::new(parser.clone());

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
    let result = lint_cmd.lint(template)?;

    // Assert
    if let LintResult::HasViolations { violations } = result {
        for violation in violations {
            assert!(violation.line > 0, "Line numbers should be positive");
        }
    }
    Ok(())
}

// ============================================================================
// Test Suite: Performance
// ============================================================================

#[test]
fn test_lint_completes_quickly() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let lint_cmd = LintCommand::new(parser.clone());

    let template = r#"
[meta]
name = "perf_test"

[otel]
service_name = "test"

[service.db]
type = "postgres"
    "#;

    parser.set_result(
        template,
        Ok(ParsedToml {
            has_meta: true,
            has_otel: true,
            services: vec!["db".to_string()],
            scenarios: vec![],
        }),
    );

    // Act
    use std::time::Instant;
    let start = Instant::now();
    let _ = lint_cmd.lint(template)?;
    let duration = start.elapsed();

    // Assert
    assert!(
        duration < std::time::Duration::from_millis(100),
        "Linting should complete in <100ms, took {:?}",
        duration
    );
    Ok(())
}

#[test]
fn test_lint_scales_to_large_templates() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let lint_cmd = LintCommand::new(parser.clone());

    // Generate large template
    let mut services = Vec::new();
    for i in 0..100 {
        services.push(format!("service_{}", i));
    }

    let template = format!(
        "[meta]\nname = \"large\"\n\n[otel]\nservice_name = \"test\"\n\n{}",
        (0..100)
            .map(|i| format!("[service.service_{}]\ntype = \"test\"\n", i))
            .collect::<String>()
    );

    parser.set_result(
        &template,
        Ok(ParsedToml {
            has_meta: true,
            has_otel: true,
            services,
            scenarios: vec![],
        }),
    );

    // Act
    use std::time::Instant;
    let start = Instant::now();
    let result = lint_cmd.lint(&template)?;
    let duration = start.elapsed();

    // Assert
    assert!(result.is_clean(), "Large template should lint successfully");
    assert!(
        duration < std::time::Duration::from_millis(500),
        "Should handle large template in <500ms"
    );
    Ok(())
}

// ============================================================================
// Test Suite: Edge Cases
// ============================================================================

#[test]
fn test_lint_handles_empty_template() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let lint_cmd = LintCommand::new(parser.clone());

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
    let result = lint_cmd.lint(template)?;

    // Assert
    assert!(!result.is_clean(), "Empty template should fail linting");
    assert!(result.violation_count() >= 2, "Should report multiple violations");
    Ok(())
}

#[test]
fn test_lint_handles_comments_only_template() -> Result<()> {
    // Arrange
    let parser = MockTomlParser::new();
    let lint_cmd = LintCommand::new(parser.clone());

    let template = "# Just comments\n# Nothing else";

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
    let result = lint_cmd.lint(template)?;

    // Assert
    assert!(!result.is_clean(), "Comments-only template should fail");
    Ok(())
}
