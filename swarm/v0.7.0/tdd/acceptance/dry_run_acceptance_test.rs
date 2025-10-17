//! Acceptance tests for `clnrm dry-run` command
//!
//! London School TDD approach:
//! - Start with user workflow (outside-in)
//! - Mock all collaborators (Renderer, Parser, Validator)
//! - CRITICAL: Verify TestExecutor is NEVER called

#[cfg(test)]
mod dry_run_acceptance {
    use std::path::{Path, PathBuf};
    use mockall::predicate::*;
    use mockall::mock;

    // Import types from clnrm-core
    use clnrm_core::error::CleanroomError;
    use clnrm_core::config::types::TestConfig;
    use clnrm_core::Result;

    // Simple validation error for testing
    #[derive(Debug, Clone)]
    struct ShapeValidationError {
        message: String,
        line: Option<usize>,
        is_error: bool,
    }

    mock! {
        pub TemplateRenderer {}
        impl TemplateRenderer {
            fn render_file(&self, path: &Path) -> Result<String, CleanroomError>;
        }
    }

    mock! {
        pub TomlParser {}
        impl TomlParser {
            fn parse(&self, content: &str) -> Result<TestConfig, CleanroomError>;
        }
    }

    mock! {
        pub SchemaValidator {}
        impl SchemaValidator {
            fn validate_structure(&self, config: &TestConfig) -> Result<Vec<ShapeValidationError>, CleanroomError>;
        }
    }

    mock! {
        pub TestExecutor {}
        impl TestExecutor {
            fn run_test(&self, path: &Path) -> Result<TestResult, CleanroomError>;
        }
    }

    /// Acceptance Test: User runs `clnrm dry-run tests/valid.clnrm.toml`
    /// Expected: Validate without executing (no Docker)
    #[test]
    fn acceptance_dry_run_validates_without_execution() {
        // Arrange: Valid test file
        let test_file = PathBuf::from("tests/valid.clnrm.toml");

        let mut mock_renderer = MockTemplateRenderer::new();
        let mut mock_parser = MockTomlParser::new();
        let mut mock_validator = MockSchemaValidator::new();
        let mut mock_executor = MockTestExecutor::new();

        // User expectation: File is rendered
        mock_renderer
            .expect_render_file()
            .with(eq(test_file.as_path()))
            .times(1)
            .returning(|_| {
                Ok(r#"[meta]
name = "example"
version = "0.1.0"

[[scenario]]
name = "test_scenario"
steps = []
"#.to_string())
            });

        // User expectation: Rendered content is parsed
        mock_parser
            .expect_parse()
            .times(1)
            .returning(|_| {
                Ok(TestConfig {
                    meta: Some(MetaConfig {
                        name: "example".to_string(),
                        version: "0.1.0".to_string(),
                    }),
                    scenario: vec![],
                })
            });

        // User expectation: Structure is validated
        mock_validator
            .expect_validate_structure()
            .times(1)
            .returning(|_| Ok(vec![])); // No validation errors

        // CRITICAL: TestExecutor should NEVER be called
        mock_executor
            .expect_run_test()
            .times(0);

        // Act: User runs dry-run
        let dry_run = DryRunCommand::new(
            Box::new(mock_renderer),
            Box::new(mock_parser),
            Box::new(mock_validator),
            Box::new(mock_executor),
        );

        let result = dry_run.execute(&test_file, false);

        // Assert: Validation succeeded, NO execution
        assert!(result.is_ok());
        let dry_run_result = result.unwrap();
        assert!(matches!(
            dry_run_result.validation_status,
            ValidationStatus::Valid
        ));
        assert_eq!(dry_run_result.issues.len(), 0);

        // Mock expectations verify TestExecutor.run_test() was NOT called
    }

    /// Acceptance Test: Detect missing [meta] section
    /// Expected: Error with helpful message
    #[test]
    fn acceptance_dry_run_detects_missing_meta_section() {
        // Arrange: Test file missing [meta]
        let test_file = PathBuf::from("tests/missing-meta.clnrm.toml");

        let mut mock_renderer = MockTemplateRenderer::new();
        let mut mock_parser = MockTomlParser::new();
        let mut mock_validator = MockSchemaValidator::new();
        let mock_executor = MockTestExecutor::new();

        mock_renderer
            .expect_render_file()
            .returning(|_| {
                Ok(r#"[[scenario]]
name = "test"
steps = []
"#.to_string())
            });

        mock_parser
            .expect_parse()
            .returning(|_| {
                Ok(TestConfig {
                    meta: None,
                    scenario: vec![],
                })
            });

        // User expectation: Validator detects missing [meta]
        mock_validator
            .expect_validate_structure()
            .times(1)
            .returning(|config| {
                if config.meta.is_none() {
                    Ok(vec![ValidationIssue {
                        severity: Severity::Error,
                        message: "Missing required [meta] section".to_string(),
                        location: None,
                        suggestion: Some("Add [meta] with name and version keys".to_string()),
                    }])
                } else {
                    Ok(vec![])
                }
            });

        // Act
        let dry_run = DryRunCommand::new(
            Box::new(mock_renderer),
            Box::new(mock_parser),
            Box::new(mock_validator),
            Box::new(mock_executor),
        );

        let result = dry_run.execute(&test_file, false);

        // Assert: User sees validation error
        assert!(result.is_ok());
        let dry_run_result = result.unwrap();
        assert!(matches!(
            dry_run_result.validation_status,
            ValidationStatus::Invalid { .. }
        ));
        assert_eq!(dry_run_result.issues.len(), 1);
        assert!(dry_run_result.issues[0]
            .message
            .contains("Missing required [meta]"));
    }

    /// Acceptance Test: Detect undefined service reference
    /// Expected: Error with service name in message
    #[test]
    fn acceptance_dry_run_detects_undefined_service() {
        // Arrange
        let test_file = PathBuf::from("tests/undefined-service.clnrm.toml");

        let mut mock_renderer = MockTemplateRenderer::new();
        let mut mock_parser = MockTomlParser::new();
        let mut mock_validator = MockSchemaValidator::new();
        let mock_executor = MockTestExecutor::new();

        mock_renderer
            .expect_render_file()
            .returning(|_| {
                Ok(r#"[meta]
name = "test"
version = "0.1.0"

[[scenario]]
name = "use_db"
service = "database"  # Not defined!
steps = []
"#.to_string())
            });

        mock_parser
            .expect_parse()
            .returning(|_| {
                Ok(TestConfig {
                    meta: Some(MetaConfig {
                        name: "test".to_string(),
                        version: "0.1.0".to_string(),
                    }),
                    scenario: vec![ScenarioConfig {
                        service: Some("database".to_string()),
                    }],
                })
            });

        // User expectation: Validator detects undefined service
        mock_validator
            .expect_validate_structure()
            .returning(|_| {
                Ok(vec![ValidationIssue {
                    severity: Severity::Error,
                    message: "Service 'database' referenced but not defined".to_string(),
                    location: None,
                    suggestion: Some("Add [service.database] section".to_string()),
                }])
            });

        // Act
        let dry_run = DryRunCommand::new(
            Box::new(mock_renderer),
            Box::new(mock_parser),
            Box::new(mock_validator),
            Box::new(mock_executor),
        );

        let result = dry_run.execute(&test_file, false);

        // Assert
        assert!(result.is_ok());
        let dry_run_result = result.unwrap();
        assert!(matches!(
            dry_run_result.validation_status,
            ValidationStatus::Invalid { .. }
        ));
        assert!(dry_run_result.issues[0].message.contains("database"));
    }

    /// Acceptance Test: Fast validation (<500ms)
    /// Expected: Completes quickly without Docker overhead
    #[test]
    fn acceptance_dry_run_completes_under_500ms() {
        // Arrange
        let test_file = PathBuf::from("tests/example.clnrm.toml");

        let mut mock_renderer = MockTemplateRenderer::new();
        let mut mock_parser = MockTomlParser::new();
        let mut mock_validator = MockSchemaValidator::new();
        let mock_executor = MockTestExecutor::new();

        mock_renderer
            .expect_render_file()
            .returning(|_| Ok("[meta]\nname=\"test\"\nversion=\"0.1.0\"".to_string()));

        mock_parser
            .expect_parse()
            .returning(|_| {
                Ok(TestConfig {
                    meta: Some(MetaConfig {
                        name: "test".to_string(),
                        version: "0.1.0".to_string(),
                    }),
                    scenario: vec![],
                })
            });

        mock_validator
            .expect_validate_structure()
            .returning(|_| Ok(vec![]));

        // Act: Measure validation time
        let dry_run = DryRunCommand::new(
            Box::new(mock_renderer),
            Box::new(mock_parser),
            Box::new(mock_validator),
            Box::new(mock_executor),
        );

        let start = std::time::Instant::now();
        let result = dry_run.execute(&test_file, false);
        let duration = start.elapsed();

        // Assert: Fast validation
        assert!(result.is_ok());
        assert!(
            duration < std::time::Duration::from_millis(500),
            "Dry-run should complete in <500ms, took {:?}",
            duration
        );
    }

    /// Acceptance Test: Verbose mode shows rendered content
    /// Expected: User sees rendered TOML in output
    #[test]
    fn acceptance_dry_run_verbose_shows_rendered_content() {
        // Arrange
        let test_file = PathBuf::from("tests/example.clnrm.toml");

        let mut mock_renderer = MockTemplateRenderer::new();
        let mut mock_parser = MockTomlParser::new();
        let mut mock_validator = MockSchemaValidator::new();
        let mock_executor = MockTestExecutor::new();

        let rendered_content = r#"[meta]
name = "example"
version = "0.1.0"
"#;

        mock_renderer
            .expect_render_file()
            .returning(move |_| Ok(rendered_content.to_string()));

        mock_parser
            .expect_parse()
            .returning(|_| {
                Ok(TestConfig {
                    meta: Some(MetaConfig {
                        name: "example".to_string(),
                        version: "0.1.0".to_string(),
                    }),
                    scenario: vec![],
                })
            });

        mock_validator
            .expect_validate_structure()
            .returning(|_| Ok(vec![]));

        // Act: Verbose mode
        let dry_run = DryRunCommand::new(
            Box::new(mock_renderer),
            Box::new(mock_parser),
            Box::new(mock_validator),
            Box::new(mock_executor),
        );

        let result = dry_run.execute(&test_file, true); // verbose = true

        // Assert: Rendered content included in result
        assert!(result.is_ok());
        let dry_run_result = result.unwrap();
        assert_eq!(dry_run_result.rendered_content, rendered_content);
    }

    // Supporting types
    struct CleanroomError {
        message: String,
    }

    impl CleanroomError {
        fn validation_error(msg: impl Into<String>) -> Self {
            Self { message: msg.into() }
        }
    }

    struct TestConfig {
        meta: Option<MetaConfig>,
        scenario: Vec<ScenarioConfig>,
    }

    struct MetaConfig {
        name: String,
        version: String,
    }

    struct ScenarioConfig {
        service: Option<String>,
    }

    struct ValidationIssue {
        severity: Severity,
        message: String,
        location: Option<Location>,
        suggestion: Option<String>,
    }

    enum Severity {
        Error,
        Warning,
    }

    struct Location {
        line: usize,
        column: usize,
    }

    struct DryRunResult {
        file_path: PathBuf,
        rendered_content: String,
        validation_status: ValidationStatus,
        issues: Vec<ValidationIssue>,
    }

    enum ValidationStatus {
        Valid,
        Invalid { error_count: usize },
    }

    struct TestResult {
        passed: bool,
    }

    // DryRunCommand (to be implemented)
    struct DryRunCommand {
        renderer: Box<dyn TemplateRenderer>,
        parser: Box<dyn TomlParser>,
        validator: Box<dyn SchemaValidator>,
        executor: Box<dyn TestExecutor>,
    }

    impl DryRunCommand {
        fn new(
            renderer: Box<dyn TemplateRenderer>,
            parser: Box<dyn TomlParser>,
            validator: Box<dyn SchemaValidator>,
            executor: Box<dyn TestExecutor>,
        ) -> Self {
            Self {
                renderer,
                parser,
                validator,
                executor,
            }
        }

        fn execute(&self, path: &Path, verbose: bool) -> Result<DryRunResult, CleanroomError> {
            // Step 1: Render template file
            let rendered_content = self.renderer.render_file(path)?;

            // Step 2: Parse rendered TOML content
            let test_config = self.parser.parse(&rendered_content)?;

            // Step 3: Validate configuration structure
            let validation_issues = self.validator.validate_structure(&test_config)?;

            // Step 4: Convert validation issues to our format
            let issues: Vec<ValidationIssue> = validation_issues
                .into_iter()
                .map(|issue| ValidationIssue {
                    severity: Severity::Error, // Shape validation errors are always errors
                    message: issue.message,
                    location: issue.line.map(|line| Location {
                        line,
                        column: 0, // Column not available in shape validation
                    }),
                    suggestion: None, // Suggestions not available in shape validation
                })
                .collect();

            // Step 5: Determine validation status
            let error_count = issues.iter().filter(|i| matches!(i.severity, Severity::Error)).count();
            let validation_status = if error_count == 0 {
                ValidationStatus::Valid
            } else {
                ValidationStatus::Invalid { error_count }
            };

            // Step 6: Print verbose output if requested
            if verbose {
                println!("🔍 Dry-run validation for: {}", path.display());
                println!("📄 Rendered content length: {} bytes", rendered_content.len());

                if !issues.is_empty() {
                    println!("⚠️  Found {} issues:", issues.len());
                    for issue in &issues {
                        let severity_icon = match issue.severity {
                            Severity::Error => "❌",
                            Severity::Warning => "⚠️ ",
                        };
                        println!("  {} {}", severity_icon, issue.message);

                        if let Some(suggestion) = &issue.suggestion {
                            println!("    💡 {}", suggestion);
                        }
                    }
                } else {
                    println!("✅ No validation issues found");
                }
            }

            // Step 7: Return result (CRITICAL: TestExecutor.run_test() should NEVER be called)
            Ok(DryRunResult {
                file_path: path.to_path_buf(),
                rendered_content,
                validation_status,
                issues,
            })
        }
    }

    trait TemplateRenderer {
        fn render_file(&self, path: &Path) -> Result<String, CleanroomError>;
    }

    trait TomlParser {
        fn parse(&self, content: &str) -> Result<TestConfig, CleanroomError>;
    }

    trait SchemaValidator {
        fn validate_structure(
            &self,
            config: &TestConfig,
        ) -> Result<Vec<ValidationIssue>, CleanroomError>;
    }

    trait TestExecutor {
        fn run_test(&self, path: &Path) -> Result<TestResult, CleanroomError>;
    }
}
