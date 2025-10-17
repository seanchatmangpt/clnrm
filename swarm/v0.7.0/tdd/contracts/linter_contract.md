# Linter Contract (London TDD)

## Interface Design (Outside-In)

The Linter is a collaborator that validates TOML structure and required keys. From the user's perspective:

```
User runs: clnrm lint tests/example.clnrm.toml
Expected: Detect missing [meta], [service], [[scenario]], [otel] sections
```

## Mock Contract

```rust
pub trait Linter: Send + Sync {
    /// Lint a test configuration file
    fn lint(&self, path: &Path) -> Result<LintResult>;

    /// Get linter configuration (rules, severity levels)
    fn config(&self) -> &LintConfig;
}

pub struct LintConfig {
    pub deny_warnings: bool,
    pub rules: Vec<LintRule>,
}

pub enum LintRule {
    RequireMetaSection,
    RequireServiceSections,
    RequireScenarioSections,
    RequireOtelSection,
    NoUnusedServices,
    NoUnreferencedVariables,
    ValidateVersionFormat,
}

pub struct LintResult {
    pub file_path: PathBuf,
    pub diagnostics: Vec<Diagnostic>,
    pub error_count: usize,
    pub warning_count: usize,
}

pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub rule: String,
    pub message: String,
    pub location: Option<Location>,
    pub suggestion: Option<String>,
}

pub enum DiagnosticSeverity {
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

### Scenario: User runs `clnrm lint tests/missing-meta.clnrm.toml`

```rust
#[test]
fn test_lint_detects_missing_meta_section() {
    // Arrange: Set up mock collaborators
    let mock_reader = MockFileReader::new();
    let mock_parser = MockTomlParser::new();
    let mock_rule_engine = MockRuleEngine::new();

    // Configure mock expectations
    mock_reader.expect_read()
        .with(eq(Path::new("tests/missing-meta.clnrm.toml")))
        .times(1)
        .returning(|_| Ok(r#"[[scenario]]
name = "test"
steps = []
"#.to_string()));

    mock_parser.expect_parse()
        .times(1)
        .returning(|content| {
            toml::from_str::<TestConfig>(content)
                .map_err(|e| CleanroomError::config_error(e.to_string()))
        });

    mock_rule_engine.expect_check_rule()
        .with(eq(LintRule::RequireMetaSection), any())
        .times(1)
        .returning(|_, config| {
            if config.meta.is_none() && config.test.is_none() {
                vec![Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    rule: "require-meta-section".to_string(),
                    message: "Missing required [meta] or [test.metadata] section".to_string(),
                    location: None,
                    suggestion: Some("Add [meta] section with name and version".to_string()),
                }]
            } else {
                vec![]
            }
        });

    // Act: Run lint command
    let lint_command = LintCommand::new(
        mock_reader,
        mock_parser,
        mock_rule_engine,
    );

    let result = lint_command.run(&[PathBuf::from("tests/missing-meta.clnrm.toml")]);

    // Assert: Verify error detected
    assert!(result.is_ok());
    let lint_results = result.unwrap();
    assert_eq!(lint_results.len(), 1);
    assert_eq!(lint_results[0].error_count, 1);
    assert!(lint_results[0].diagnostics.iter().any(|d| d.rule == "require-meta-section"));
}
```

### Scenario: Detect missing [service] when referenced in scenario

```rust
#[test]
fn test_lint_detects_undefined_service_reference() {
    // Arrange
    let mock_reader = MockFileReader::new();
    let mock_parser = MockTomlParser::new();
    let mock_rule_engine = MockRuleEngine::new();

    mock_reader.expect_read()
        .returning(|_| Ok(r#"[meta]
name = "test"
version = "0.1.0"

[[scenario]]
name = "use_db"
service = "database"  # Referenced but not defined!
steps = []
"#.to_string()));

    mock_parser.expect_parse()
        .returning(|content| toml::from_str(content).map_err(|e| CleanroomError::config_error(e.to_string())));

    mock_rule_engine.expect_check_rule()
        .with(eq(LintRule::RequireServiceSections), any())
        .returning(|_, config| {
            let mut diagnostics = vec![];
            for scenario in &config.scenario {
                if let Some(ref service_name) = scenario.service {
                    let service_exists = config.service.as_ref()
                        .map(|services| services.contains_key(service_name))
                        .unwrap_or(false);

                    if !service_exists {
                        diagnostics.push(Diagnostic {
                            severity: DiagnosticSeverity::Error,
                            rule: "undefined-service".to_string(),
                            message: format!("Service '{}' referenced but not defined", service_name),
                            location: None,
                            suggestion: Some(format!("Add [service.{}] section", service_name)),
                        });
                    }
                }
            }
            diagnostics
        });

    // Act
    let lint_command = LintCommand::new(mock_reader, mock_parser, mock_rule_engine);
    let result = lint_command.run(&[PathBuf::from("test.toml")]);

    // Assert
    assert!(result.is_ok());
    let lint_results = result.unwrap();
    assert!(lint_results[0].error_count > 0);
    assert!(lint_results[0].diagnostics.iter().any(|d| d.message.contains("database")));
}
```

### Scenario: Deny warnings mode

```rust
#[test]
fn test_lint_deny_warnings_treats_warnings_as_errors() {
    // Arrange
    let mock_reader = MockFileReader::new();
    let mock_parser = MockTomlParser::new();
    let mock_rule_engine = MockRuleEngine::new();

    mock_reader.expect_read()
        .returning(|_| Ok(r#"[meta]
name = "test"
version = "0.1.0"

[service.unused]
type = "generic"
plugin = "generic"
image = "alpine:latest"

[[scenario]]
name = "test"
steps = []
"#.to_string()));

    mock_rule_engine.expect_check_rule()
        .with(eq(LintRule::NoUnusedServices), any())
        .returning(|_, _| {
            vec![Diagnostic {
                severity: DiagnosticSeverity::Warning,
                rule: "unused-service".to_string(),
                message: "Service 'unused' is defined but never used".to_string(),
                location: None,
                suggestion: Some("Remove unused service or reference it in a scenario".to_string()),
            }]
        });

    // Act
    let lint_config = LintConfig {
        deny_warnings: true,
        rules: vec![LintRule::NoUnusedServices],
    };
    let lint_command = LintCommand::new_with_config(
        mock_reader,
        mock_parser,
        mock_rule_engine,
        lint_config,
    );

    let result = lint_command.run(&[PathBuf::from("test.toml")]);

    // Assert: Warning should cause failure with deny_warnings=true
    assert!(result.is_err() || result.unwrap()[0].error_count > 0);
}
```

## Critical Interaction Sequence

1. User → LintCommand: run(files)
2. LintCommand → FileReader: read(file_path)
3. LintCommand → TomlParser: parse(content)
4. LintCommand → RuleEngine: check_rule(RequireMetaSection, config)
5. LintCommand → RuleEngine: check_rule(RequireServiceSections, config)
6. LintCommand → RuleEngine: check_rule(RequireScenarioSections, config)
7. LintCommand → User: Display LintResult with diagnostics

## Linting Rules (80/20 Priority)

### MUST detect (Errors):
1. Missing [meta] section
2. Undefined service references
3. Missing [[scenario]] sections
4. Invalid version format
5. Missing required keys (name, version)

### SHOULD detect (Warnings):
1. Unused services
2. Unreferenced variables
3. Deprecated syntax
4. Missing [otel] section

### COULD detect (Info):
1. Inconsistent naming conventions
2. Missing descriptions

## Performance Contract

- Lint typical test file: &lt;100ms
- No Docker operations
- No network calls
- Deterministic results (same input = same output)

## Output Formats

### Human-readable
```
error: Missing required [meta] section
  --> tests/example.clnrm.toml
  |
  | Add [meta] section with name and version

warning: Service 'unused' is defined but never used
  --> tests/example.clnrm.toml:10:1
  |
10 | [service.unused]
  | ^^^^^^^^^^^^^^^^ unused service
  |
  = help: Remove unused service or reference it in a scenario
```

### JSON (for IDE integration)
```json
{
  "diagnostics": [
    {
      "severity": "error",
      "rule": "require-meta-section",
      "message": "Missing required [meta] section",
      "location": null,
      "suggestion": "Add [meta] section with name and version"
    }
  ],
  "error_count": 1,
  "warning_count": 0
}
```

### GitHub Actions annotations
```
::error file=tests/example.clnrm.toml::Missing required [meta] section
::warning file=tests/example.clnrm.toml,line=10::Service 'unused' is defined but never used
```

## Error Scenarios

### Invalid TOML syntax
```rust
mock_parser.expect_parse()
    .returning(|_| Err(CleanroomError::config_error("Expected '=', found ']'")));
// Linter should report parse error
```

### File read fails
```rust
mock_reader.expect_read()
    .returning(|_| Err(CleanroomError::io_error("File not found")));
```

## Implementation Notes

- Use `toml` crate for parsing (not `toml-edit` - simpler for linting)
- RuleEngine should be extensible (easy to add new rules)
- Location tracking requires TOML parser with span information
- All trait methods MUST be sync
- Rules should be composable and independent
