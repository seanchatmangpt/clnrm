# Dry Run Contract (London TDD)

## Interface Design (Outside-In)

The DryRun validator is a collaborator that validates without executing. From the user's perspective:

```
User runs: clnrm dry-run tests/example.clnrm.toml
Expected: Render template, validate TOML structure, check required keys, NO Docker execution
```

## Mock Contract

```rust
pub trait DryRunValidator: Send + Sync {
    /// Validate a test file without executing
    /// Returns validation results with no side effects
    fn validate_dry_run(&self, path: &Path) -> Result<DryRunResult>;
}

pub struct DryRunResult {
    pub file_path: PathBuf,
    pub rendered_content: String,
    pub validation_status: ValidationStatus,
    pub issues: Vec<ValidationIssue>,
}

pub enum ValidationStatus {
    Valid,
    Invalid { error_count: usize },
}

pub struct ValidationIssue {
    pub severity: Severity,
    pub message: String,
    pub location: Option<Location>,
}

pub enum Severity {
    Error,
    Warning,
    Info,
}

pub struct Location {
    pub line: usize,
    pub column: usize,
}
```

## Interaction Expectations (Behavior Verification)

### Scenario: User runs `clnrm dry-run tests/valid.clnrm.toml`

```rust
#[tokio::test]
async fn test_dry_run_validates_without_execution() {
    // Arrange: Set up mock collaborators
    let mock_renderer = MockTemplateRenderer::new();
    let mock_toml_parser = MockTomlParser::new();
    let mock_schema_validator = MockSchemaValidator::new();
    let mock_executor = MockTestExecutor::new();

    // Configure mock expectations
    mock_renderer.expect_render_file()
        .with(eq(Path::new("tests/valid.clnrm.toml")))
        .times(1)
        .returning(|_| Ok(r#"[meta]
name = "example"
version = "0.1.0"

[[scenario]]
name = "test_scenario"
steps = []
"#.to_string()));

    mock_toml_parser.expect_parse()
        .times(1)
        .returning(|content| {
            toml::from_str::<TestConfig>(content)
                .map_err(|e| CleanroomError::config_error(e.to_string()))
        });

    mock_schema_validator.expect_validate_structure()
        .times(1)
        .returning(|_| Ok(vec![])); // No issues

    // CRITICAL: TestExecutor should NOT be called in dry-run
    mock_executor.expect_run_test()
        .times(0); // Verify NO execution

    // Act: Run dry-run command
    let dry_run = DryRunCommand::new(
        mock_renderer,
        mock_toml_parser,
        mock_schema_validator,
        mock_executor,
    );

    let result = dry_run.execute("tests/valid.clnrm.toml").await;

    // Assert: Verify validation succeeded and NO execution occurred
    assert!(result.is_ok());
    let dry_run_result = result.unwrap();
    assert!(matches!(dry_run_result.validation_status, ValidationStatus::Valid));
    assert_eq!(dry_run_result.issues.len(), 0);
}
```

### Scenario: Missing required section [meta]

```rust
#[test]
fn test_dry_run_detects_missing_meta_section() {
    // Arrange
    let mock_renderer = MockTemplateRenderer::new();
    let mock_schema_validator = MockSchemaValidator::new();

    mock_renderer.expect_render_file()
        .returning(|_| Ok(r#"[[scenario]]
name = "test"
"#.to_string()));

    mock_schema_validator.expect_validate_structure()
        .times(1)
        .returning(|config| {
            if config.meta.is_none() && config.test.is_none() {
                Ok(vec![ValidationIssue {
                    severity: Severity::Error,
                    message: "Missing required [meta] or [test.metadata] section".to_string(),
                    location: None,
                }])
            } else {
                Ok(vec![])
            }
        });

    // Act
    let dry_run = DryRunCommand::new(mock_renderer, mock_schema_validator);
    let result = dry_run.execute("tests/invalid.clnrm.toml").await;

    // Assert
    assert!(result.is_ok());
    let dry_run_result = result.unwrap();
    assert!(matches!(dry_run_result.validation_status, ValidationStatus::Invalid { error_count: 1 }));
}
```

## Critical Interaction Sequence

1. User → DryRunCommand: execute("tests/example.clnrm.toml")
2. DryRunCommand → TemplateRenderer: render_file("tests/example.clnrm.toml")
3. DryRunCommand → TomlParser: parse(rendered_content)
4. DryRunCommand → SchemaValidator: validate_structure(config)
5. DryRunCommand → User: Display DryRunResult
6. **NEVER** → TestExecutor: run_test() (must be verified as NOT called)

## Validation Checks (80/20 Priority)

### MUST detect:
1. Missing [meta] section
2. Missing [service.name] sections when referenced
3. Missing [[scenario]] sections
4. Invalid TOML syntax
5. Missing required keys (name, version)

### SHOULD detect:
1. Unused services
2. Unreferenced variables
3. Deprecated syntax

## Performance Contract

- Dry-run validation: &lt;500ms for typical test file
- No Docker operations (0 container starts)
- No network calls
- No file system modifications

## Error Scenarios

### Template rendering fails
```rust
mock_renderer.expect_render_file()
    .returning(|_| Err(CleanroomError::template_error("Undefined variable 'foo'")));
```

### Invalid TOML syntax
```rust
mock_toml_parser.expect_parse()
    .returning(|_| Err(CleanroomError::config_error("Expected '=', found ']'")));
```

## Implementation Notes

- DryRunValidator is purely functional (no side effects)
- All validations must be deterministic
- Mock should verify TestExecutor is NEVER called
- Results should include line/column for errors when possible
