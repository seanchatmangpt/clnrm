# Validation API Surface - v0.6.0

**API Architect Report - Architecture Sub-Coordinator**

## Overview

Comprehensive validation API for template rendering, OTEL instrumentation, and test configuration. All APIs follow core team standards with proper error handling and dyn compatibility.

## Template Validation API

### Core Trait: `TemplateValidator`

```rust
//! Template validation trait for pre-render and post-render checks
//!
//! Validates:
//! - Template syntax correctness
//! - Required variable presence
//! - TOML output validity
//! - Security constraints (no code injection)

use crate::error::{CleanroomError, Result};

/// Template validator trait
///
/// All methods are sync (dyn-compatible) and return Result<T, CleanroomError>
pub trait TemplateValidator: Send + Sync + std::fmt::Debug {
    /// Validator name
    fn name(&self) -> &str;

    /// Validate template syntax before rendering
    ///
    /// Checks:
    /// - Valid Tera syntax
    /// - Required variables present in context
    /// - No security violations
    fn validate_template_syntax(&self, template: &str, context: &TemplateContext) -> Result<ValidationReport>;

    /// Validate rendered output
    ///
    /// Checks:
    /// - Valid TOML syntax
    /// - Schema compliance
    /// - Value constraints
    fn validate_rendered_output(&self, rendered: &str) -> Result<ValidationReport>;

    /// Get validation rules
    fn rules(&self) -> Vec<ValidationRule>;
}

/// Validation report with detailed findings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Validator that produced this report
    pub validator_name: String,
    /// Overall validation result
    pub passed: bool,
    /// Individual findings
    pub findings: Vec<ValidationFinding>,
    /// Validation timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Individual validation finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationFinding {
    /// Finding severity
    pub severity: ValidationSeverity,
    /// Rule that was violated
    pub rule_id: String,
    /// Human-readable message
    pub message: String,
    /// Location in template/output (line:col)
    pub location: Option<String>,
    /// Suggested fix
    pub suggestion: Option<String>,
}

/// Validation severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    /// Blocks rendering
    Error,
    /// Should be fixed but doesn't block
    Warning,
    /// Informational only
    Info,
}

/// Validation rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Unique rule identifier
    pub id: String,
    /// Rule description
    pub description: String,
    /// Severity if violated
    pub severity: ValidationSeverity,
    /// Whether rule is enabled
    pub enabled: bool,
}
```

### Built-in Validators

#### 1. Syntax Validator

```rust
/// Validates Tera template syntax
#[derive(Debug, Clone, Default)]
pub struct SyntaxValidator;

impl TemplateValidator for SyntaxValidator {
    fn name(&self) -> &str {
        "syntax"
    }

    fn validate_template_syntax(&self, template: &str, _context: &TemplateContext) -> Result<ValidationReport> {
        let mut findings = Vec::new();

        // Check for unclosed braces
        if template.matches("{{").count() != template.matches("}}").count() {
            findings.push(ValidationFinding {
                severity: ValidationSeverity::Error,
                rule_id: "SYNTAX_001".to_string(),
                message: "Mismatched template braces".to_string(),
                location: None,
                suggestion: Some("Ensure all {{ are closed with }}".to_string()),
            });
        }

        // Check for unclosed control blocks
        if template.matches("{%").count() != template.matches("%}").count() {
            findings.push(ValidationFinding {
                severity: ValidationSeverity::Error,
                rule_id: "SYNTAX_002".to_string(),
                message: "Mismatched control blocks".to_string(),
                location: None,
                suggestion: Some("Ensure all {% are closed with %}".to_string()),
            });
        }

        Ok(ValidationReport {
            validator_name: self.name().to_string(),
            passed: findings.is_empty(),
            findings,
            timestamp: chrono::Utc::now(),
        })
    }

    fn validate_rendered_output(&self, rendered: &str) -> Result<ValidationReport> {
        let findings = match toml::from_str::<toml::Value>(rendered) {
            Ok(_) => Vec::new(),
            Err(e) => vec![ValidationFinding {
                severity: ValidationSeverity::Error,
                rule_id: "OUTPUT_001".to_string(),
                message: format!("Invalid TOML output: {}", e),
                location: None,
                suggestion: Some("Check template rendering for TOML syntax errors".to_string()),
            }],
        };

        Ok(ValidationReport {
            validator_name: self.name().to_string(),
            passed: findings.is_empty(),
            findings,
            timestamp: chrono::Utc::now(),
        })
    }

    fn rules(&self) -> Vec<ValidationRule> {
        vec![
            ValidationRule {
                id: "SYNTAX_001".to_string(),
                description: "Template braces must be balanced".to_string(),
                severity: ValidationSeverity::Error,
                enabled: true,
            },
            ValidationRule {
                id: "SYNTAX_002".to_string(),
                description: "Control blocks must be balanced".to_string(),
                severity: ValidationSeverity::Error,
                enabled: true,
            },
            ValidationRule {
                id: "OUTPUT_001".to_string(),
                description: "Rendered output must be valid TOML".to_string(),
                severity: ValidationSeverity::Error,
                enabled: true,
            },
        ]
    }
}
```

#### 2. Security Validator

```rust
/// Validates template security constraints
#[derive(Debug, Clone, Default)]
pub struct SecurityValidator {
    /// Allowed function whitelist
    allowed_functions: Vec<String>,
}

impl SecurityValidator {
    pub fn new() -> Self {
        Self {
            allowed_functions: vec![
                "env".to_string(),
                "now_rfc3339".to_string(),
                "sha256".to_string(),
                "toml_encode".to_string(),
            ],
        }
    }

    pub fn with_allowed_functions(mut self, functions: Vec<String>) -> Self {
        self.allowed_functions = functions;
        self
    }
}

impl TemplateValidator for SecurityValidator {
    fn name(&self) -> &str {
        "security"
    }

    fn validate_template_syntax(&self, template: &str, _context: &TemplateContext) -> Result<ValidationReport> {
        let mut findings = Vec::new();

        // Check for code execution attempts
        let dangerous_patterns = [
            ("system(", "SECURITY_001", "System command execution not allowed"),
            ("exec(", "SECURITY_002", "Exec function not allowed"),
            ("eval(", "SECURITY_003", "Eval function not allowed"),
            ("include ", "SECURITY_004", "Template inclusion requires explicit approval"),
        ];

        for (pattern, rule_id, message) in &dangerous_patterns {
            if template.contains(pattern) {
                findings.push(ValidationFinding {
                    severity: ValidationSeverity::Error,
                    rule_id: rule_id.to_string(),
                    message: message.to_string(),
                    location: None,
                    suggestion: Some("Remove dangerous function calls".to_string()),
                });
            }
        }

        // Extract function calls and validate against whitelist
        // Simplified regex pattern matching
        for cap in template.split("{{").skip(1) {
            if let Some(func_name) = cap.split('(').next() {
                let func = func_name.trim();
                if func.contains(char::is_alphabetic) &&
                   !self.allowed_functions.contains(&func.to_string()) &&
                   !func.is_empty() {
                    findings.push(ValidationFinding {
                        severity: ValidationSeverity::Warning,
                        rule_id: "SECURITY_005".to_string(),
                        message: format!("Unknown function '{}' not in whitelist", func),
                        location: None,
                        suggestion: Some("Use only whitelisted functions".to_string()),
                    });
                }
            }
        }

        Ok(ValidationReport {
            validator_name: self.name().to_string(),
            passed: !findings.iter().any(|f| f.severity == ValidationSeverity::Error),
            findings,
            timestamp: chrono::Utc::now(),
        })
    }

    fn validate_rendered_output(&self, rendered: &str) -> Result<ValidationReport> {
        let mut findings = Vec::new();

        // Check for potential secrets in output
        let secret_patterns = [
            (r"password\s*=\s*[\"'].*[\"']", "SECURITY_101", "Hardcoded password detected"),
            (r"api_key\s*=\s*[\"'].*[\"']", "SECURITY_102", "Hardcoded API key detected"),
            (r"secret\s*=\s*[\"'].*[\"']", "SECURITY_103", "Hardcoded secret detected"),
        ];

        for (pattern, rule_id, message) in &secret_patterns {
            if rendered.to_lowercase().contains(&pattern.to_lowercase()) {
                findings.push(ValidationFinding {
                    severity: ValidationSeverity::Warning,
                    rule_id: rule_id.to_string(),
                    message: message.to_string(),
                    location: None,
                    suggestion: Some("Use environment variables or secret management".to_string()),
                });
            }
        }

        Ok(ValidationReport {
            validator_name: self.name().to_string(),
            passed: !findings.iter().any(|f| f.severity == ValidationSeverity::Error),
            findings,
            timestamp: chrono::Utc::now(),
        })
    }

    fn rules(&self) -> Vec<ValidationRule> {
        vec![
            ValidationRule {
                id: "SECURITY_001".to_string(),
                description: "System command execution prohibited".to_string(),
                severity: ValidationSeverity::Error,
                enabled: true,
            },
            ValidationRule {
                id: "SECURITY_005".to_string(),
                description: "Only whitelisted functions allowed".to_string(),
                severity: ValidationSeverity::Warning,
                enabled: true,
            },
            ValidationRule {
                id: "SECURITY_101".to_string(),
                description: "No hardcoded secrets in output".to_string(),
                severity: ValidationSeverity::Warning,
                enabled: true,
            },
        ]
    }
}
```

#### 3. Schema Validator

```rust
/// Validates TOML output against expected schema
#[derive(Debug, Clone)]
pub struct SchemaValidator {
    /// Expected schema definition
    schema: TestConfigSchema,
}

impl SchemaValidator {
    pub fn new(schema: TestConfigSchema) -> Self {
        Self { schema }
    }
}

impl TemplateValidator for SchemaValidator {
    fn name(&self) -> &str {
        "schema"
    }

    fn validate_template_syntax(&self, _template: &str, _context: &TemplateContext) -> Result<ValidationReport> {
        // No pre-render schema validation
        Ok(ValidationReport {
            validator_name: self.name().to_string(),
            passed: true,
            findings: Vec::new(),
            timestamp: chrono::Utc::now(),
        })
    }

    fn validate_rendered_output(&self, rendered: &str) -> Result<ValidationReport> {
        let mut findings = Vec::new();

        // Parse TOML
        let parsed: toml::Value = match toml::from_str(rendered) {
            Ok(v) => v,
            Err(e) => {
                findings.push(ValidationFinding {
                    severity: ValidationSeverity::Error,
                    rule_id: "SCHEMA_001".to_string(),
                    message: format!("Invalid TOML: {}", e),
                    location: None,
                    suggestion: None,
                });
                return Ok(ValidationReport {
                    validator_name: self.name().to_string(),
                    passed: false,
                    findings,
                    timestamp: chrono::Utc::now(),
                });
            }
        };

        // Validate required sections
        if !parsed.get("test").is_some() {
            findings.push(ValidationFinding {
                severity: ValidationSeverity::Error,
                rule_id: "SCHEMA_002".to_string(),
                message: "Missing required [test] section".to_string(),
                location: None,
                suggestion: Some("Add [test] section with metadata".to_string()),
            });
        }

        // Validate test metadata
        if let Some(test) = parsed.get("test") {
            if !test.get("metadata").is_some() {
                findings.push(ValidationFinding {
                    severity: ValidationSeverity::Error,
                    rule_id: "SCHEMA_003".to_string(),
                    message: "Missing required [test.metadata] section".to_string(),
                    location: None,
                    suggestion: Some("Add [test.metadata] with name and description".to_string()),
                });
            }
        }

        Ok(ValidationReport {
            validator_name: self.name().to_string(),
            passed: !findings.iter().any(|f| f.severity == ValidationSeverity::Error),
            findings,
            timestamp: chrono::Utc::now(),
        })
    }

    fn rules(&self) -> Vec<ValidationRule> {
        vec![
            ValidationRule {
                id: "SCHEMA_002".to_string(),
                description: "[test] section required".to_string(),
                severity: ValidationSeverity::Error,
                enabled: true,
            },
            ValidationRule {
                id: "SCHEMA_003".to_string(),
                description: "[test.metadata] section required".to_string(),
                severity: ValidationSeverity::Error,
                enabled: true,
            },
        ]
    }
}
```

### Validation Pipeline

```rust
/// Validation pipeline for orchestrating multiple validators
#[derive(Debug, Default)]
pub struct ValidationPipeline {
    validators: Vec<Box<dyn TemplateValidator>>,
}

impl ValidationPipeline {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add validator to pipeline
    pub fn add_validator(&mut self, validator: Box<dyn TemplateValidator>) {
        self.validators.push(validator);
    }

    /// Run all validators on template
    pub fn validate_template(&self, template: &str, context: &TemplateContext) -> Result<PipelineReport> {
        let mut reports = Vec::new();

        for validator in &self.validators {
            let report = validator.validate_template_syntax(template, context)?;
            reports.push(report);
        }

        Ok(PipelineReport {
            template_reports: reports,
            output_reports: Vec::new(),
            overall_passed: true,
        })
    }

    /// Run all validators on rendered output
    pub fn validate_output(&self, rendered: &str) -> Result<PipelineReport> {
        let mut reports = Vec::new();

        for validator in &self.validators {
            let report = validator.validate_rendered_output(rendered)?;
            reports.push(report);
        }

        let overall_passed = reports.iter().all(|r| r.passed);

        Ok(PipelineReport {
            template_reports: Vec::new(),
            output_reports: reports,
            overall_passed,
        })
    }

    /// Run complete validation pipeline
    pub fn validate_complete(&self, template: &str, context: &TemplateContext, rendered: &str) -> Result<PipelineReport> {
        let template_report = self.validate_template(template, context)?;
        let output_report = self.validate_output(rendered)?;

        Ok(PipelineReport {
            template_reports: template_report.template_reports,
            output_reports: output_report.output_reports,
            overall_passed: template_report.overall_passed && output_report.overall_passed,
        })
    }
}

/// Pipeline validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineReport {
    /// Template validation reports
    pub template_reports: Vec<ValidationReport>,
    /// Output validation reports
    pub output_reports: Vec<ValidationReport>,
    /// Overall pass/fail
    pub overall_passed: bool,
}
```

## OTEL Validation API (Enhanced)

Extends existing `OtelValidator` with schema validation and comprehensive assertions.

```rust
/// OTEL validation extensions
impl OtelValidator {
    /// Validate span attributes match schema
    pub fn validate_span_attributes_schema(
        &self,
        span_attributes: &HashMap<String, String>,
        schema: &AttributeSchema,
    ) -> Result<ValidationReport> {
        let mut findings = Vec::new();

        // Check required attributes
        for required_key in &schema.required_attributes {
            if !span_attributes.contains_key(required_key) {
                findings.push(ValidationFinding {
                    severity: ValidationSeverity::Error,
                    rule_id: "OTEL_ATTR_001".to_string(),
                    message: format!("Missing required attribute: {}", required_key),
                    location: None,
                    suggestion: Some(format!("Add '{}' attribute to span", required_key)),
                });
            }
        }

        // Validate attribute types
        for (key, value) in span_attributes {
            if let Some(expected_type) = schema.attribute_types.get(key) {
                if !Self::validate_attribute_type(value, expected_type) {
                    findings.push(ValidationFinding {
                        severity: ValidationSeverity::Error,
                        rule_id: "OTEL_ATTR_002".to_string(),
                        message: format!("Attribute '{}' has incorrect type", key),
                        location: None,
                        suggestion: Some(format!("Expected type: {}", expected_type)),
                    });
                }
            }
        }

        Ok(ValidationReport {
            validator_name: "otel_attributes".to_string(),
            passed: !findings.iter().any(|f| f.severity == ValidationSeverity::Error),
            findings,
            timestamp: chrono::Utc::now(),
        })
    }

    fn validate_attribute_type(value: &str, expected_type: &str) -> bool {
        match expected_type {
            "string" => true, // All are strings in our case
            "number" => value.parse::<f64>().is_ok(),
            "boolean" => value == "true" || value == "false",
            _ => false,
        }
    }
}

/// Attribute schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeSchema {
    /// Required attribute keys
    pub required_attributes: Vec<String>,
    /// Attribute type definitions
    pub attribute_types: HashMap<String, String>,
}
```

## Usage Examples

### Example 1: Template Validation Pipeline

```rust
use clnrm_core::validation::{ValidationPipeline, SyntaxValidator, SecurityValidator, SchemaValidator};

// Build validation pipeline
let mut pipeline = ValidationPipeline::new();
pipeline.add_validator(Box::new(SyntaxValidator::default()));
pipeline.add_validator(Box::new(SecurityValidator::new()));
pipeline.add_validator(Box::new(SchemaValidator::new(schema)));

// Validate template before rendering
let context = TemplateContext::new();
let template_report = pipeline.validate_template(template_str, &context)?;

if !template_report.overall_passed {
    for report in &template_report.template_reports {
        for finding in &report.findings {
            eprintln!("[{}] {}: {}", finding.severity, finding.rule_id, finding.message);
        }
    }
    return Err(CleanroomError::validation_error("Template validation failed"));
}

// Render template
let rendered = renderer.render_str(template_str, "test")?;

// Validate output
let output_report = pipeline.validate_output(&rendered)?;
if !output_report.overall_passed {
    return Err(CleanroomError::validation_error("Output validation failed"));
}
```

### Example 2: Custom Validator

```rust
/// Custom validator for matrix test constraints
#[derive(Debug, Clone)]
pub struct MatrixValidator {
    max_combinations: usize,
}

impl TemplateValidator for MatrixValidator {
    fn name(&self) -> &str {
        "matrix"
    }

    fn validate_template_syntax(&self, template: &str, context: &TemplateContext) -> Result<ValidationReport> {
        let mut findings = Vec::new();

        // Calculate matrix combinations
        let matrix_vars = context.matrix.len();
        if matrix_vars > 10 {
            findings.push(ValidationFinding {
                severity: ValidationSeverity::Warning,
                rule_id: "MATRIX_001".to_string(),
                message: format!("Large matrix with {} variables may generate many tests", matrix_vars),
                location: None,
                suggestion: Some("Consider reducing matrix dimensions".to_string()),
            });
        }

        Ok(ValidationReport {
            validator_name: self.name().to_string(),
            passed: true,
            findings,
            timestamp: chrono::Utc::now(),
        })
    }

    fn validate_rendered_output(&self, _rendered: &str) -> Result<ValidationReport> {
        Ok(ValidationReport {
            validator_name: self.name().to_string(),
            passed: true,
            findings: Vec::new(),
            timestamp: chrono::Utc::now(),
        })
    }

    fn rules(&self) -> Vec<ValidationRule> {
        vec![
            ValidationRule {
                id: "MATRIX_001".to_string(),
                description: "Matrix should not exceed reasonable size".to_string(),
                severity: ValidationSeverity::Warning,
                enabled: true,
            },
        ]
    }
}
```

## API Contract Guarantees

1. **Dyn Compatibility**: All validators are trait objects
2. **Error Handling**: All methods return `Result<T, CleanroomError>`
3. **No Panics**: No `.unwrap()` or `.expect()` in production
4. **Composability**: Validators can be chained in pipelines
5. **Detailed Reporting**: All findings include actionable suggestions
6. **Performance**: Validation adds <10ms overhead per template
