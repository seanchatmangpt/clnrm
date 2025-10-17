# Dry-Run Validator Architecture - v0.7.0

## Overview

The dry-run validator provides fast TOML structure validation without starting containers, enabling rapid feedback during development. Validates rendered Tera templates against the v0.6.0 schema.

## Architecture Components

### 1. DryRunValidator

```rust
// crates/clnrm-core/src/validation/dry_run.rs
pub struct DryRunValidator {
    schema: SchemaValidator,
    tera_cache: Arc<Mutex<Tera>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Overall validation status
    pub passed: bool,

    /// Template path being validated
    pub template_path: PathBuf,

    /// Rendered TOML content
    pub rendered_content: String,

    /// Validation errors (with line numbers)
    pub errors: Vec<ValidationError>,

    /// Validation warnings
    pub warnings: Vec<ValidationWarning>,

    /// Schema version validated against
    pub schema_version: String,

    /// Validation duration in milliseconds
    pub duration_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error message
    pub message: String,

    /// Line number in rendered TOML (if applicable)
    pub line: Option<usize>,

    /// Column number
    pub column: Option<usize>,

    /// Error category
    pub category: ErrorCategory,

    /// Suggested fix (if available)
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Tera template syntax error
    TemplateSyntax,

    /// TOML parsing error
    TomlSyntax,

    /// Missing required field
    MissingField,

    /// Invalid field value
    InvalidValue,

    /// Schema violation
    SchemaViolation,

    /// Type mismatch
    TypeMismatch,
}
```

### 2. Schema Validator

Validates against v0.6.0 TOML schema with required sections.

```rust
// crates/clnrm-core/src/validation/schema.rs
pub struct SchemaValidator {
    required_sections: Vec<String>,
    optional_sections: Vec<String>,
    field_validators: HashMap<String, Box<dyn FieldValidator>>,
}

impl SchemaValidator {
    /// Create validator for v0.6.0 schema
    pub fn v0_6_0() -> Self {
        Self {
            required_sections: vec![
                "meta".to_string(),
                "otel".to_string(),
                "service".to_string(),
                "scenario".to_string(),
            ],
            optional_sections: vec![
                "vars".to_string(),
                "matrix".to_string(),
                "expect".to_string(),
                "report".to_string(),
                "determinism".to_string(),
                "limits".to_string(),
            ],
            field_validators: Self::build_validators(),
        }
    }

    fn build_validators() -> HashMap<String, Box<dyn FieldValidator>> {
        let mut validators: HashMap<String, Box<dyn FieldValidator>> = HashMap::new();

        // [meta] section validators
        validators.insert(
            "meta.name".to_string(),
            Box::new(NonEmptyStringValidator::new("Test name cannot be empty"))
        );
        validators.insert(
            "meta.version".to_string(),
            Box::new(VersionValidator::new())
        );

        // [otel] section validators
        validators.insert(
            "otel.exporter".to_string(),
            Box::new(EnumValidator::new(vec!["otlp-http", "otlp-grpc", "stdout"]))
        );
        validators.insert(
            "otel.sample_ratio".to_string(),
            Box::new(RangeValidator::new(0.0, 1.0))
        );

        // [service.*] validators
        validators.insert(
            "service.*.type".to_string(),
            Box::new(NonEmptyStringValidator::new("Service type required"))
        );
        validators.insert(
            "service.*.plugin".to_string(),
            Box::new(PluginValidator::new())
        );

        // [[scenario]] validators
        validators.insert(
            "scenario.name".to_string(),
            Box::new(NonEmptyStringValidator::new("Scenario name required"))
        );

        validators
    }

    /// Validate parsed TOML config
    pub fn validate(&self, config: &TestConfig) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Check required sections
        errors.extend(self.validate_required_sections(config)?);

        // Validate individual fields
        errors.extend(self.validate_fields(config)?);

        // Validate relationships (e.g., service references)
        errors.extend(self.validate_relationships(config)?);

        Ok(errors)
    }

    fn validate_required_sections(&self, config: &TestConfig) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Check [meta] section
        if config.meta.is_none() {
            errors.push(ValidationError {
                message: "Missing required [meta] section".to_string(),
                line: None,
                column: None,
                category: ErrorCategory::MissingField,
                suggestion: Some("Add [meta] section with name and version".to_string()),
            });
        }

        // Check [otel] section
        if config.otel.is_none() {
            errors.push(ValidationError {
                message: "Missing required [otel] section".to_string(),
                line: None,
                column: None,
                category: ErrorCategory::MissingField,
                suggestion: Some("Add [otel] section with exporter configuration".to_string()),
            });
        }

        // Check [service.*] sections
        if config.service.is_none() || config.service.as_ref().unwrap().is_empty() {
            errors.push(ValidationError {
                message: "At least one [service.*] section required".to_string(),
                line: None,
                column: None,
                category: ErrorCategory::MissingField,
                suggestion: Some("Add service definition like [service.myservice]".to_string()),
            });
        }

        // Check [[scenario]] sections
        if config.scenario.is_empty() {
            errors.push(ValidationError {
                message: "At least one [[scenario]] section required".to_string(),
                line: None,
                column: None,
                category: ErrorCategory::MissingField,
                suggestion: Some("Add scenario with [[scenario]]".to_string()),
            });
        }

        Ok(errors)
    }

    fn validate_fields(&self, config: &TestConfig) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate [meta] fields
        if let Some(ref meta) = config.meta {
            if meta.name.trim().is_empty() {
                errors.push(ValidationError {
                    message: "meta.name cannot be empty".to_string(),
                    line: None,
                    column: None,
                    category: ErrorCategory::InvalidValue,
                    suggestion: Some("Provide a meaningful test name".to_string()),
                });
            }

            // Validate version format (semver)
            if !Self::is_valid_semver(&meta.version) {
                errors.push(ValidationError {
                    message: format!("Invalid version format: {}", meta.version),
                    line: None,
                    column: None,
                    category: ErrorCategory::InvalidValue,
                    suggestion: Some("Use semantic versioning: X.Y.Z".to_string()),
                });
            }
        }

        // Validate [otel] fields
        if let Some(ref otel) = config.otel {
            if !["otlp-http", "otlp-grpc", "stdout"].contains(&otel.exporter.as_str()) {
                errors.push(ValidationError {
                    message: format!("Invalid OTEL exporter: {}", otel.exporter),
                    line: None,
                    column: None,
                    category: ErrorCategory::InvalidValue,
                    suggestion: Some("Use 'otlp-http', 'otlp-grpc', or 'stdout'".to_string()),
                });
            }

            if let Some(ratio) = otel.sample_ratio {
                if !(0.0..=1.0).contains(&ratio) {
                    errors.push(ValidationError {
                        message: format!("sample_ratio {} outside range [0.0, 1.0]", ratio),
                        line: None,
                        column: None,
                        category: ErrorCategory::InvalidValue,
                        suggestion: Some("Set sample_ratio between 0.0 and 1.0".to_string()),
                    });
                }
            }
        }

        Ok(errors)
    }

    fn validate_relationships(&self, config: &TestConfig) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Build service name set
        let service_names: HashSet<String> = config.service
            .as_ref()
            .map(|s| s.keys().cloned().collect())
            .unwrap_or_default();

        // Validate scenario step service references
        for scenario in &config.scenario {
            for (step_idx, step) in scenario.steps.iter().enumerate() {
                if let Some(ref service) = step.service {
                    if !service_names.contains(service) {
                        errors.push(ValidationError {
                            message: format!(
                                "Scenario '{}' step {} references undefined service '{}'",
                                scenario.name, step_idx, service
                            ),
                            line: None,
                            column: None,
                            category: ErrorCategory::SchemaViolation,
                            suggestion: Some(format!(
                                "Define [service.{}] or remove service reference",
                                service
                            )),
                        });
                    }
                }
            }
        }

        Ok(errors)
    }

    fn is_valid_semver(version: &str) -> bool {
        // Simple semver validation (X.Y.Z format)
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return false;
        }
        parts.iter().all(|p| p.parse::<u32>().is_ok())
    }
}
```

### 3. Field Validators

Trait-based validators for extensibility.

```rust
// crates/clnrm-core/src/validation/field_validators.rs
pub trait FieldValidator: Send + Sync {
    fn validate(&self, value: &serde_json::Value) -> Result<(), String>;
    fn name(&self) -> &str;
}

pub struct NonEmptyStringValidator {
    error_message: String,
}

impl FieldValidator for NonEmptyStringValidator {
    fn validate(&self, value: &serde_json::Value) -> Result<(), String> {
        match value.as_str() {
            Some(s) if !s.trim().is_empty() => Ok(()),
            _ => Err(self.error_message.clone()),
        }
    }

    fn name(&self) -> &str {
        "non_empty_string"
    }
}

pub struct RangeValidator {
    min: f64,
    max: f64,
}

impl FieldValidator for RangeValidator {
    fn validate(&self, value: &serde_json::Value) -> Result<(), String> {
        match value.as_f64() {
            Some(v) if v >= self.min && v <= self.max => Ok(()),
            Some(v) => Err(format!("Value {} outside range [{}, {}]", v, self.min, self.max)),
            None => Err("Expected numeric value".to_string()),
        }
    }

    fn name(&self) -> &str {
        "range"
    }
}

pub struct EnumValidator {
    allowed_values: Vec<String>,
}

impl FieldValidator for EnumValidator {
    fn validate(&self, value: &serde_json::Value) -> Result<(), String> {
        match value.as_str() {
            Some(s) if self.allowed_values.contains(&s.to_string()) => Ok(()),
            Some(s) => Err(format!(
                "Invalid value '{}'. Allowed: {:?}",
                s, self.allowed_values
            )),
            None => Err("Expected string value".to_string()),
        }
    }

    fn name(&self) -> &str {
        "enum"
    }
}
```

### 4. Tera Template Validation

```rust
impl DryRunValidator {
    /// Validate Tera template rendering
    fn validate_template_rendering(
        &self,
        template_path: &Path,
        content: &str,
    ) -> Result<String> {
        let mut tera = self.tera_cache.lock()
            .map_err(|e| CleanroomError::internal_error(
                format!("Failed to lock Tera cache: {}", e)
            ))?;

        // Try to render template
        match tera.render_str(content, &tera::Context::new()) {
            Ok(rendered) => Ok(rendered),
            Err(e) => {
                // Extract line/column info from Tera error
                let (line, column) = Self::parse_tera_error_location(&e);

                Err(CleanroomError::template_error(
                    format!("Template rendering failed at {}:{}: {}",
                        line.unwrap_or(0),
                        column.unwrap_or(0),
                        e
                    )
                ))
            }
        }
    }

    fn parse_tera_error_location(error: &tera::Error) -> (Option<usize>, Option<usize>) {
        // Parse Tera error message for line/column info
        // Tera errors typically include location like "at line 5, column 12"
        let error_str = error.to_string();

        let line = error_str
            .split("line ")
            .nth(1)
            .and_then(|s| s.split(',').next())
            .and_then(|s| s.trim().parse().ok());

        let column = error_str
            .split("column ")
            .nth(1)
            .and_then(|s| s.split_whitespace().next())
            .and_then(|s| s.trim().parse().ok());

        (line, column)
    }
}
```

## Data Flow

```
Template File (.clnrm.toml.tera)
    ↓
Read File Content
    ↓
Tera Template Rendering
    ↓
Rendered TOML String
    ↓
TOML Parser
    ↓
Parsed TestConfig
    ↓
Schema Validation
    ↓
┌──────────────────────────────────────┐
│ 1. Check required sections          │
│ 2. Validate field types/values      │
│ 3. Validate relationships           │
│ 4. Check service references         │
│ 5. Validate OTEL expectations       │
└──────────────────────────────────────┘
    ↓
ValidationResult { errors, warnings }
```

## Validation Phases

### Phase 1: Template Syntax
- Tera template syntax validation
- Variable reference checking
- Function call validation

### Phase 2: TOML Parsing
- TOML syntax validation
- Section structure validation
- Data type validation

### Phase 3: Schema Validation
- Required section presence
- Field type checking
- Value range validation
- Enum validation

### Phase 4: Semantic Validation
- Service reference resolution
- OTEL span expectation validation
- Determinism configuration validation

## CLI Integration

```bash
# Validate without execution
clnrm validate tests/test_auth.clnrm.toml.tera

# Validate all templates
clnrm validate tests/

# Validate and show detailed errors
clnrm validate --verbose tests/

# Validate with specific context
clnrm validate --var version=2.0 tests/test.clnrm.toml.tera

# JSON output for CI
clnrm validate --format json tests/ > validation-results.json
```

## Output Formats

### Human-Readable

```
Validating: tests/test_auth.clnrm.toml.tera
  ✗ Error at line 15, column 3: Missing required field 'meta.version'
    Suggestion: Add version field to [meta] section

  ⚠ Warning: Service 'database' not used in any scenario

Validation failed: 1 error, 1 warning
```

### JSON Format

```json
{
  "template_path": "tests/test_auth.clnrm.toml.tera",
  "passed": false,
  "errors": [
    {
      "message": "Missing required field 'meta.version'",
      "line": 15,
      "column": 3,
      "category": "MissingField",
      "suggestion": "Add version field to [meta] section"
    }
  ],
  "warnings": [
    {
      "message": "Service 'database' not used in any scenario",
      "line": null,
      "severity": "low"
    }
  ],
  "schema_version": "0.6.0",
  "duration_ms": 12.5
}
```

## Performance Optimizations

### 1. Tera Template Caching

```rust
impl DryRunValidator {
    /// Cache compiled Tera templates
    pub fn with_template_cache(cache_size: usize) -> Self {
        let tera = Tera::default();
        // Set cache size (Tera doesn't expose this directly, so we use Arc<Mutex<>>)

        Self {
            schema: SchemaValidator::v0_6_0(),
            tera_cache: Arc::new(Mutex::new(tera)),
        }
    }
}
```

### 2. Parallel Validation

```rust
pub async fn validate_directory(
    &self,
    dir: &Path,
    max_parallel: usize,
) -> Result<Vec<ValidationResult>> {
    let templates = self.find_templates(dir)?;

    let semaphore = Arc::new(Semaphore::new(max_parallel));
    let mut tasks = Vec::new();

    for template_path in templates {
        let sem = semaphore.clone();
        let validator = self.clone();

        tasks.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            validator.validate_file(&template_path).await
        }));
    }

    let results = futures::future::try_join_all(tasks).await?;
    Ok(results)
}
```

### 3. Incremental Validation

Only validate changed files using change detection.

```rust
pub async fn validate_incremental(
    &self,
    dir: &Path,
    change_detector: &mut ChangeDetector,
) -> Result<Vec<ValidationResult>> {
    let mut results = Vec::new();

    for template_path in self.find_templates(dir)? {
        // Render template
        let rendered = self.render_template(&template_path).await?;

        // Check if changed
        let change = change_detector.detect_changes(
            &template_path,
            &rendered,
            &TemplateContext::new()
        )?;

        if change.should_execute {
            // Only validate if changed
            let result = self.validate_rendered(&template_path, &rendered)?;
            results.push(result);
        } else {
            // Use cached validation result
            tracing::debug!("Skipping unchanged template: {:?}", template_path);
        }
    }

    Ok(results)
}
```

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validates_missing_required_section() -> Result<()> {
        // Arrange
        let validator = DryRunValidator::new();
        let toml = r#"
            [service.db]
            type = "surrealdb"
            plugin = "surrealdb"
        "#;

        // Act
        let result = validator.validate_toml_str(toml)?;

        // Assert
        assert!(!result.passed);
        assert!(result.errors.iter().any(|e|
            e.message.contains("Missing required [meta]")
        ));

        Ok(())
    }

    #[tokio::test]
    async fn test_validates_invalid_otel_exporter() -> Result<()> {
        // Arrange
        let validator = DryRunValidator::new();
        let toml = r#"
            [meta]
            name = "test"
            version = "1.0.0"

            [otel]
            exporter = "invalid-exporter"

            [service.db]
            type = "surrealdb"
            plugin = "surrealdb"

            [[scenario]]
            name = "test"
            steps = []
        "#;

        // Act
        let result = validator.validate_toml_str(toml)?;

        // Assert
        assert!(!result.passed);
        assert!(result.errors.iter().any(|e|
            e.message.contains("Invalid OTEL exporter")
        ));

        Ok(())
    }
}
```

## Dependencies

```toml
[dependencies]
toml = "0.8"
tera = "1.19"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Future Enhancements

1. **LSP Server**: Real-time validation in code editors
2. **Auto-Fix**: Suggest and apply automated fixes
3. **Custom Validators**: User-defined validation rules
4. **Schema Versioning**: Support multiple schema versions
