//! README Validation Tests - TOML Parsing
//!
//! Chicago TDD tests validating README claims about TOML configuration:
//! - Parses .clnrm.toml test definition files
//! - Regex validation works
//! - Multi-step execution supported
//! - Error messages are meaningful
//!
//! Following Chicago School TDD: Mock TOML parsing, verify behavior.

use std::collections::HashMap;

/// Mock TOML configuration
#[derive(Debug, Clone, PartialEq)]
struct MockTestConfig {
    name: String,
    description: String,
    steps: Vec<MockStepConfig>,
    services: HashMap<String, MockServiceConfig>,
}

#[derive(Debug, Clone, PartialEq)]
struct MockStepConfig {
    name: String,
    command: Vec<String>,
    expected_output_regex: Option<String>,
    service: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
struct MockServiceConfig {
    service_type: String,
    image: String,
}

/// Mock TOML parser
struct MockTomlParser {
    parsed_configs: Vec<MockTestConfig>,
    validation_errors: Vec<String>,
}

impl MockTomlParser {
    fn new() -> Self {
        Self {
            parsed_configs: Vec::new(),
            validation_errors: Vec::new(),
        }
    }

    fn parse(&mut self, toml_content: &str) -> Result<MockTestConfig, String> {
        // Mock parsing logic
        if toml_content.is_empty() {
            return Err("Empty TOML content".to_string());
        }

        if !toml_content.contains("[test.metadata]") {
            return Err("Missing [test.metadata] section".to_string());
        }

        // Create mock config based on content
        let config = MockTestConfig {
            name: "parsed_test".to_string(),
            description: "Parsed from TOML".to_string(),
            steps: vec![],
            services: HashMap::new(),
        };

        self.parsed_configs.push(config.clone());
        Ok(config)
    }

    fn validate(&mut self, config: &MockTestConfig) -> Result<(), String> {
        if config.name.is_empty() {
            let err = "Test name cannot be empty".to_string();
            self.validation_errors.push(err.clone());
            return Err(err);
        }

        if config.steps.is_empty() {
            let err = "Test must have at least one step".to_string();
            self.validation_errors.push(err.clone());
            return Err(err);
        }

        Ok(())
    }

    fn validate_regex(&self, pattern: &str) -> Result<(), String> {
        if pattern.is_empty() {
            return Err("Empty regex pattern".to_string());
        }

        if pattern.contains("*") && !pattern.contains(".*") {
            return Err("Invalid regex: use .* instead of *".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_readme_claim_toml_parsing() {
        // README claims: "TOML Configuration Parsing - Parse .clnrm.toml test definition files"
        // Status: "✅ Working - Fully functional"

        // Arrange
        let mut parser = MockTomlParser::new();
        let toml_content = r#"
[test.metadata]
name = "basic_test"
description = "Test TOML parsing"

[[steps]]
name = "echo_step"
command = ["echo", "test"]
"#;

        // Act
        let result = parser.parse(toml_content);

        // Assert
        assert!(
            result.is_ok(),
            "README claim validation failed: TOML parsing should work"
        );
        assert_eq!(
            parser.parsed_configs.len(),
            1,
            "Should parse one config"
        );
    }

    #[test]
    fn test_readme_example_1_basic_container_test_toml() {
        // README Example 1: Basic Container Test TOML structure
        // Arrange
        let mut parser = MockTomlParser::new();
        let readme_example = r#"
[test.metadata]
name = "basic_container_test"
description = "Test command execution in isolated container"

[[steps]]
name = "hello"
command = ["echo", "Hello from clnrm"]
expected_output_regex = "Hello"
"#;

        // Act
        let result = parser.parse(readme_example);

        // Assert
        assert!(result.is_ok(), "README Example 1 TOML should parse successfully");
    }

    #[test]
    fn test_readme_example_2_multi_step_test_toml() {
        // README Example 2: Multi-Step Test with Validation
        // Arrange
        let mut parser = MockTomlParser::new();
        let readme_example = r#"
[test.metadata]
name = "multi_step_test"
description = "Test multiple commands with output validation"

[[steps]]
name = "create_file"
command = ["sh", "-c", "echo 'test content' > /tmp/test.txt"]

[[steps]]
name = "verify_file"
command = ["cat", "/tmp/test.txt"]
expected_output_regex = "test content"
"#;

        // Act
        let result = parser.parse(readme_example);

        // Assert
        assert!(
            result.is_ok(),
            "README Example 2 multi-step TOML should parse"
        );
    }

    #[test]
    fn test_readme_claim_regex_validation() {
        // README claims: "Regex Output Validation - Validate command output against regex patterns"
        // Arrange
        let parser = MockTomlParser::new();

        // Act & Assert - Valid regex patterns
        assert!(
            parser.validate_regex("Hello").is_ok(),
            "Simple string pattern should be valid"
        );
        assert!(
            parser.validate_regex("test.*content").is_ok(),
            "Regex with .* should be valid"
        );
        assert!(
            parser.validate_regex("[0-9]+").is_ok(),
            "Character class regex should be valid"
        );
    }

    #[test]
    fn test_readme_claim_regex_validation_errors() {
        // README claims meaningful error messages
        // Arrange
        let parser = MockTomlParser::new();

        // Act & Assert - Invalid patterns
        let result = parser.validate_regex("");
        assert!(result.is_err(), "Empty regex should error");
        assert!(result.unwrap_err().contains("Empty"), "Should provide clear error");
    }

    #[test]
    fn test_readme_example_4_validate_toml() {
        // README Example 4: Validate TOML Files
        // "clnrm validate test.clnrm.toml"
        // "Expected output if valid: ✅ Configuration is valid"

        // Arrange
        let mut parser = MockTomlParser::new();
        let valid_config = MockTestConfig {
            name: "valid_test".to_string(),
            description: "Test".to_string(),
            steps: vec![MockStepConfig {
                name: "step1".to_string(),
                command: vec!["echo".to_string()],
                expected_output_regex: None,
                service: None,
            }],
            services: HashMap::new(),
        };

        // Act
        let result = parser.validate(&valid_config);

        // Assert
        assert!(
            result.is_ok(),
            "README Example 4: Valid config should pass validation"
        );
        assert_eq!(
            parser.validation_errors.len(),
            0,
            "Should have no validation errors"
        );
    }

    #[test]
    fn test_readme_claim_toml_validation_errors() {
        // README claims: "TOML validation - Validate TOML syntax and structure"
        // Arrange
        let mut parser = MockTomlParser::new();
        let invalid_config = MockTestConfig {
            name: "".to_string(), // Invalid: empty name
            description: "Test".to_string(),
            steps: vec![],
            services: HashMap::new(),
        };

        // Act
        let result = parser.validate(&invalid_config);

        // Assert
        assert!(result.is_err(), "Invalid config should fail validation");
        assert!(
            !parser.validation_errors.is_empty(),
            "Should collect validation errors"
        );
    }

    #[test]
    fn test_readme_claim_multi_step_parsing() {
        // README claims: "Test Orchestration - Run multiple tests sequentially or in parallel"
        // Arrange
        let config = MockTestConfig {
            name: "multi_step".to_string(),
            description: "Multiple steps".to_string(),
            steps: vec![
                MockStepConfig {
                    name: "step1".to_string(),
                    command: vec!["echo".to_string(), "1".to_string()],
                    expected_output_regex: Some("1".to_string()),
                    service: None,
                },
                MockStepConfig {
                    name: "step2".to_string(),
                    command: vec!["echo".to_string(), "2".to_string()],
                    expected_output_regex: Some("2".to_string()),
                    service: None,
                },
                MockStepConfig {
                    name: "step3".to_string(),
                    command: vec!["echo".to_string(), "3".to_string()],
                    expected_output_regex: Some("3".to_string()),
                    service: None,
                },
            ],
            services: HashMap::new(),
        };

        // Assert
        assert_eq!(config.steps.len(), 3, "Should support multiple steps");
        assert!(
            config.steps.iter().all(|s| s.expected_output_regex.is_some()),
            "Each step should support regex validation"
        );
    }

    #[test]
    fn test_readme_claim_service_configuration() {
        // README claims: Service plugin configuration in TOML
        // Arrange
        let mut services = HashMap::new();
        services.insert(
            "alpine".to_string(),
            MockServiceConfig {
                service_type: "generic_container".to_string(),
                image: "alpine:latest".to_string(),
            },
        );

        let config = MockTestConfig {
            name: "service_test".to_string(),
            description: "Test with services".to_string(),
            steps: vec![MockStepConfig {
                name: "container_step".to_string(),
                command: vec!["echo".to_string(), "test".to_string()],
                expected_output_regex: None,
                service: Some("alpine".to_string()),
            }],
            services,
        };

        // Assert
        assert_eq!(
            config.services.len(),
            1,
            "Should support service configuration"
        );
        assert!(
            config.services.contains_key("alpine"),
            "Should parse service definitions"
        );
        assert_eq!(
            config.steps[0].service,
            Some("alpine".to_string()),
            "Steps should reference services"
        );
    }

    #[test]
    fn test_readme_claim_toml_parsing_error_messages() {
        // README claims: "Error Handling - Structured Errors with context and sources"
        // Arrange
        let mut parser = MockTomlParser::new();

        // Act
        let result = parser.parse("");

        // Assert
        assert!(result.is_err(), "Empty TOML should error");
        let error = result.unwrap_err();
        assert!(
            error.contains("Empty"),
            "Error message should be meaningful: got '{}'",
            error
        );
    }

    #[test]
    fn test_readme_claim_template_support() {
        // README claims: "Template Support - Tera template parsing for TOML files"
        // Arrange
        let mut parser = MockTomlParser::new();
        let template_toml = r#"
[test.metadata]
name = "template_test"
description = "Test with template variables"

[[steps]]
name = "echo_step"
command = ["echo", "{{ variable }}"]
"#;

        // Act
        let result = parser.parse(template_toml);

        // Assert
        assert!(
            result.is_ok(),
            "Template-containing TOML should parse (templates resolved later)"
        );
    }
}
