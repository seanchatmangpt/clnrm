# MockDryRunValidator Contract

## Purpose
Define interaction contract for dry-run validation that checks rendered TOML without executing tests.

## Mock Trait Definition

```rust
use std::path::Path;
use crate::core::{Result, ValidationReport};

/// Mock implementation of dry-run validation behavior
pub trait MockDryRunValidator: Send + Sync {
    /// Validate rendered TOML without execution
    ///
    /// Interactions to verify:
    /// - Called after template rendering
    /// - Called before actual test execution
    /// - Returns detailed validation report
    fn validate_toml(&self, content: &str) -> Result<ValidationReport>;

    /// Check for required keys in TOML
    ///
    /// Interactions to verify:
    /// - Called as part of validation
    /// - Reports missing required sections
    fn check_required_keys(&self, content: &str) -> Result<Vec<String>>;

    /// Validate service configurations
    ///
    /// Interactions to verify:
    /// - Called for each service in config
    /// - Checks image availability, ports, etc.
    fn validate_services(&self, content: &str) -> Result<Vec<ServiceValidation>>;

    /// Validate step definitions
    ///
    /// Interactions to verify:
    /// - Called for each step in test
    /// - Checks command syntax, assertions
    fn validate_steps(&self, content: &str) -> Result<Vec<StepValidation>>;

    /// Validate assertion syntax
    fn validate_assertions(&self, content: &str) -> Result<Vec<AssertionValidation>>;

    /// Get validation warnings (non-fatal issues)
    fn get_warnings(&self, content: &str) -> Vec<ValidationWarning>;
}

/// Validation report structure
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationReport {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub summary: ValidationSummary,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    pub kind: ErrorKind,
    pub message: String,
    pub location: Option<Location>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    MissingKey,
    InvalidSyntax,
    InvalidService,
    InvalidStep,
    InvalidAssertion,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidationWarning {
    pub kind: WarningKind,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WarningKind {
    DeprecatedFeature,
    SuboptimalConfiguration,
    MissingOptionalField,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidationSummary {
    pub total_errors: usize,
    pub total_warnings: usize,
    pub services_validated: usize,
    pub steps_validated: usize,
}
```

## Mock Implementation for Tests

```rust
use std::sync::{Arc, Mutex};

/// Test mock with interaction tracking and configurable validation results
pub struct TestDryRunValidator {
    /// Tracks validate_toml() calls
    validation_calls: Arc<Mutex<Vec<ValidationCall>>>,

    /// Tracks check_required_keys() calls
    key_check_calls: Arc<Mutex<Vec<String>>>,

    /// Tracks validate_services() calls
    service_validation_calls: Arc<Mutex<Vec<String>>>,

    /// Configured validation outcomes
    validation_config: Arc<Mutex<HashMap<String, ValidationOutcome>>>,

    /// Default validation behavior
    default_outcome: Arc<Mutex<ValidationOutcome>>,
}

#[derive(Debug, Clone)]
struct ValidationCall {
    content_hash: String,
    result: ValidationReport,
    timestamp: std::time::Instant,
}

#[derive(Debug, Clone)]
enum ValidationOutcome {
    Pass,
    FailWithErrors(Vec<ValidationError>),
    PassWithWarnings(Vec<ValidationWarning>),
}

impl TestDryRunValidator {
    pub fn new() -> Self {
        Self {
            validation_calls: Arc::new(Mutex::new(Vec::new())),
            key_check_calls: Arc::new(Mutex::new(Vec::new())),
            service_validation_calls: Arc::new(Mutex::new(Vec::new())),
            validation_config: Arc::new(Mutex::new(HashMap::new())),
            default_outcome: Arc::new(Mutex::new(ValidationOutcome::Pass)),
        }
    }

    /// Configure mock to return validation error
    pub fn configure_validation_error(&self, pattern: &str, error: ValidationError) {
        let mut config = self.validation_config.lock().unwrap();
        let outcome = config.entry(pattern.to_string())
            .or_insert_with(|| ValidationOutcome::FailWithErrors(Vec::new()));

        if let ValidationOutcome::FailWithErrors(errors) = outcome {
            errors.push(error);
        }
    }

    /// Configure mock to return validation warning
    pub fn configure_validation_warning(&self, pattern: &str, warning: ValidationWarning) {
        let mut config = self.validation_config.lock().unwrap();
        config.insert(
            pattern.to_string(),
            ValidationOutcome::PassWithWarnings(vec![warning]),
        );
    }

    /// Configure mock to pass validation
    pub fn configure_validation_pass(&self, pattern: &str) {
        self.validation_config.lock().unwrap()
            .insert(pattern.to_string(), ValidationOutcome::Pass);
    }

    /// Verify validate_toml() was called
    pub fn verify_validation_called(&self) -> bool {
        !self.validation_calls.lock().unwrap().is_empty()
    }

    /// Get validation call count
    pub fn validation_call_count(&self) -> usize {
        self.validation_calls.lock().unwrap().len()
    }

    /// Verify validation passed for content
    pub fn verify_validation_passed(&self) -> bool {
        self.validation_calls.lock().unwrap()
            .iter()
            .last()
            .map(|call| call.result.valid)
            .unwrap_or(false)
    }

    /// Get last validation report
    pub fn last_validation_report(&self) -> Option<ValidationReport> {
        self.validation_calls.lock().unwrap()
            .last()
            .map(|call| call.result.clone())
    }

    /// Verify required keys check was called
    pub fn verify_keys_checked(&self) -> bool {
        !self.key_check_calls.lock().unwrap().is_empty()
    }
}

impl MockDryRunValidator for TestDryRunValidator {
    fn validate_toml(&self, content: &str) -> Result<ValidationReport> {
        // Compute content hash for tracking
        let content_hash = format!("{:x}", md5::compute(content));

        // Determine outcome based on configuration
        let outcome = self.find_matching_outcome(content);

        let report = match outcome {
            ValidationOutcome::Pass => ValidationReport {
                valid: true,
                errors: vec![],
                warnings: vec![],
                summary: ValidationSummary {
                    total_errors: 0,
                    total_warnings: 0,
                    services_validated: 1,
                    steps_validated: 1,
                },
            },
            ValidationOutcome::FailWithErrors(errors) => ValidationReport {
                valid: false,
                errors,
                warnings: vec![],
                summary: ValidationSummary {
                    total_errors: errors.len(),
                    total_warnings: 0,
                    services_validated: 0,
                    steps_validated: 0,
                },
            },
            ValidationOutcome::PassWithWarnings(warnings) => ValidationReport {
                valid: true,
                errors: vec![],
                warnings,
                summary: ValidationSummary {
                    total_errors: 0,
                    total_warnings: warnings.len(),
                    services_validated: 1,
                    steps_validated: 1,
                },
            },
        };

        // Track call
        self.validation_calls.lock().unwrap().push(ValidationCall {
            content_hash,
            result: report.clone(),
            timestamp: std::time::Instant::now(),
        });

        Ok(report)
    }

    fn check_required_keys(&self, content: &str) -> Result<Vec<String>> {
        self.key_check_calls.lock().unwrap().push(content.to_string());

        // Check for required sections
        let mut missing = Vec::new();

        if !content.contains("[test.metadata]") {
            missing.push("test.metadata".to_string());
        }
        if !content.contains("[[steps]]") {
            missing.push("steps".to_string());
        }

        Ok(missing)
    }

    fn validate_services(&self, content: &str) -> Result<Vec<ServiceValidation>> {
        self.service_validation_calls.lock().unwrap()
            .push(content.to_string());

        // Default: valid service configuration
        Ok(vec![ServiceValidation {
            service_name: "test_service".to_string(),
            valid: true,
            issues: vec![],
        }])
    }

    fn validate_steps(&self, content: &str) -> Result<Vec<StepValidation>> {
        // Default: valid steps
        Ok(vec![StepValidation {
            step_number: 1,
            valid: true,
            issues: vec![],
        }])
    }

    fn validate_assertions(&self, content: &str) -> Result<Vec<AssertionValidation>> {
        Ok(vec![])
    }

    fn get_warnings(&self, content: &str) -> Vec<ValidationWarning> {
        vec![]
    }
}

impl TestDryRunValidator {
    fn find_matching_outcome(&self, content: &str) -> ValidationOutcome {
        let config = self.validation_config.lock().unwrap();

        // Check for pattern matches
        for (pattern, outcome) in config.iter() {
            if content.contains(pattern) {
                return outcome.clone();
            }
        }

        // Return default
        self.default_outcome.lock().unwrap().clone()
    }
}
```

## Test Examples

### Example 1: Verify Validation Called After Rendering

```rust
#[tokio::test]
async fn test_live_reload_validates_after_rendering() -> Result<()> {
    // Arrange
    let mock_renderer = Arc::new(TestTemplateRenderer::new());
    let mock_validator = Arc::new(TestDryRunValidator::new());

    mock_renderer.configure_response(
        "test.toml.tera",
        "[test.metadata]\nname = \"test\"\n[[steps]]\nname = \"step1\"\n".to_string(),
    );

    let orchestrator = LiveReloadOrchestrator::new(
        mock_watcher.clone(),
        mock_renderer.clone(),
        mock_validator.clone(),
    );

    // Act
    orchestrator.render_and_validate("test.toml.tera").await?;

    // Assert - Verify sequence: render then validate
    assert!(mock_renderer.verify_rendered("test.toml.tera"));
    assert!(mock_validator.verify_validation_called());

    Ok(())
}
```

### Example 2: Verify Validation Failure Prevents Execution

```rust
#[tokio::test]
async fn test_live_reload_prevents_execution_on_validation_failure() -> Result<()> {
    // Arrange
    let mock_validator = Arc::new(TestDryRunValidator::new());
    let mock_executor = Arc::new(TestExecutor::new());

    // Configure validation to fail
    mock_validator.configure_validation_error(
        "test",
        ValidationError {
            kind: ErrorKind::MissingKey,
            message: "Missing required key: test.metadata.name".to_string(),
            location: None,
        },
    );

    let orchestrator = LiveReloadOrchestrator::new(
        mock_watcher.clone(),
        mock_renderer.clone(),
        mock_validator.clone(),
    );

    // Act
    let result = orchestrator.validate_and_execute("test.toml.tera").await;

    // Assert - Verify validation failed and execution skipped
    assert!(result.is_err());
    assert!(mock_validator.verify_validation_called());
    assert!(!mock_validator.verify_validation_passed());
    assert!(
        !mock_executor.verify_execution_attempted(),
        "Execution should be skipped on validation failure"
    );

    Ok(())
}
```

### Example 3: Verify Missing Key Detection

```rust
#[tokio::test]
async fn test_validator_detects_missing_required_keys() -> Result<()> {
    // Arrange
    let mock_validator = Arc::new(TestDryRunValidator::new());

    // Incomplete TOML content
    let incomplete_toml = r#"
        [services.test]
        type = "generic_container"
        # Missing test.metadata section
    "#;

    // Act
    let report = mock_validator.validate_toml(incomplete_toml)?;

    // Assert - Verify missing keys detected
    assert!(!report.valid, "Validation should fail for incomplete TOML");
    assert!(
        report.errors.iter().any(|e| {
            e.kind == ErrorKind::MissingKey && e.message.contains("test.metadata")
        }),
        "Should detect missing test.metadata"
    );

    Ok(())
}
```

### Example 4: Verify Warnings Don't Block Execution

```rust
#[tokio::test]
async fn test_live_reload_allows_execution_with_warnings() -> Result<()> {
    // Arrange
    let mock_validator = Arc::new(TestDryRunValidator::new());
    let mock_executor = Arc::new(TestExecutor::new());

    // Configure validation to pass with warnings
    mock_validator.configure_validation_warning(
        "test",
        ValidationWarning {
            kind: WarningKind::SuboptimalConfiguration,
            message: "Consider using a more recent base image".to_string(),
        },
    );

    let orchestrator = LiveReloadOrchestrator::new(
        mock_watcher.clone(),
        mock_renderer.clone(),
        mock_validator.clone(),
    );

    // Act
    let result = orchestrator.validate_and_execute("test.toml.tera").await;

    // Assert - Verify validation passed with warnings
    assert!(result.is_ok());
    assert!(mock_validator.verify_validation_passed());

    let report = mock_validator.last_validation_report().unwrap();
    assert!(report.valid);
    assert!(report.errors.is_empty());
    assert!(!report.warnings.is_empty());

    // Execution should proceed despite warnings
    assert!(mock_executor.verify_execution_attempted());

    Ok(())
}
```

### Example 5: Verify Service Configuration Validation

```rust
#[tokio::test]
async fn test_validator_checks_service_configurations() -> Result<()> {
    // Arrange
    let mock_validator = Arc::new(TestDryRunValidator::new());

    let toml_with_invalid_service = r#"
        [test.metadata]
        name = "test"

        [services.invalid]
        type = "unknown_type"
        image = "nonexistent:latest"

        [[steps]]
        name = "step1"
    "#;

    // Configure validation to detect invalid service
    mock_validator.configure_validation_error(
        "unknown_type",
        ValidationError {
            kind: ErrorKind::InvalidService,
            message: "Unknown service type: unknown_type".to_string(),
            location: None,
        },
    );

    // Act
    let report = mock_validator.validate_toml(toml_with_invalid_service)?;

    // Assert - Verify service validation ran
    assert!(mock_validator.verify_keys_checked());
    assert!(!report.valid);
    assert!(
        report.errors.iter().any(|e| e.kind == ErrorKind::InvalidService),
        "Should detect invalid service configuration"
    );

    Ok(())
}
```

### Example 6: Verify Validation Report Contains Details

```rust
#[tokio::test]
async fn test_validation_report_provides_detailed_feedback() -> Result<()> {
    // Arrange
    let mock_validator = Arc::new(TestDryRunValidator::new());

    // Configure multiple validation issues
    mock_validator.configure_validation_error(
        "test",
        ValidationError {
            kind: ErrorKind::MissingKey,
            message: "Missing: test.metadata.name".to_string(),
            location: Some(Location { line: 1, column: 0 }),
        },
    );

    let incomplete_toml = "[test.metadata]\n# Missing name field";

    // Act
    let report = mock_validator.validate_toml(incomplete_toml)?;

    // Assert - Verify detailed report
    assert!(!report.valid);
    assert_eq!(report.summary.total_errors, 1);

    let error = &report.errors[0];
    assert_eq!(error.kind, ErrorKind::MissingKey);
    assert!(error.message.contains("test.metadata.name"));
    assert!(error.location.is_some());

    Ok(())
}
```

## Interaction Patterns to Verify

### Pattern 1: Complete Validation Sequence
```
1. validate_toml(content)
   a. check_required_keys(content)
   b. validate_services(content)
   c. validate_steps(content)
   d. validate_assertions(content)
   e. get_warnings(content)
2. Return ValidationReport
```

### Pattern 2: Early Exit on Critical Error
```
1. check_required_keys(content)
2. If missing critical keys:
   a. Return error immediately
   b. Skip remaining validation
```

### Pattern 3: Validation in Live Reload Cycle
```
1. File change detected
2. Template rendered
3. validate_toml(rendered_content)
4. If valid: proceed to execution
5. If invalid: display errors, wait for fix
```

## Contract Guarantees

### Pre-conditions
- Content must be valid UTF-8
- Content should be TOML-parseable (or return syntax error)

### Post-conditions
- ValidationReport indicates pass/fail
- All errors include descriptive messages
- Warnings are informational only

### Invariants
- Valid content always passes validation
- Invalid content never executes
- Warnings don't affect validity

## Mock Configuration Helpers

```rust
impl TestDryRunValidator {
    /// Configure common validation scenarios
    pub fn with_scenario(self, scenario: ValidationScenario) -> Self {
        match scenario {
            ValidationScenario::AllPass => {
                *self.default_outcome.lock().unwrap() = ValidationOutcome::Pass;
            }
            ValidationScenario::MissingMetadata => {
                self.configure_validation_error(
                    "",
                    ValidationError {
                        kind: ErrorKind::MissingKey,
                        message: "Missing test.metadata section".to_string(),
                        location: None,
                    },
                );
            }
            ValidationScenario::InvalidService => {
                self.configure_validation_error(
                    "",
                    ValidationError {
                        kind: ErrorKind::InvalidService,
                        message: "Invalid service configuration".to_string(),
                        location: None,
                    },
                );
            }
        }
        self
    }

    /// Reset all tracking state
    pub fn reset(&self) {
        self.validation_calls.lock().unwrap().clear();
        self.key_check_calls.lock().unwrap().clear();
        self.service_validation_calls.lock().unwrap().clear();
    }
}

pub enum ValidationScenario {
    AllPass,
    MissingMetadata,
    InvalidService,
}
```

## Design Notes

1. **Pre-Execution Validation**: Catches errors before container startup
2. **Detailed Reporting**: Provides actionable feedback for fixes
3. **Interaction Recording**: Tracks all validation attempts
4. **Configuration Flexibility**: Easy setup of pass/fail scenarios
5. **Warning vs Error**: Distinguishes blocking vs non-blocking issues
