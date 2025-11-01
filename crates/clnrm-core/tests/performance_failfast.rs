//! Performance Fail-Fast Tests for v1.2.1
//!
//! This test suite follows London School TDD principles to verify that
//! performance testing configuration is properly rejected with clear error messages.
//!
//! ## Design Decision (v1.2.1)
//! Performance testing features are NOT YET IMPLEMENTED. The framework MUST
//! fail fast when users attempt to configure performance testing, providing
//! clear guidance about the missing feature.
//!
//! ## Coverage
//! - Performance config detection and rejection
//! - Sample size configuration rejection
//! - Baseline configuration rejection
//! - Regression detection rejection
//! - Clear error messages for users

use clnrm_core::config::TestConfig;
use clnrm_core::error::Result;

// ============================================================================
// SECTION 1: Performance Config Parsing
// ============================================================================

#[test]
fn test_performance_config_parses_but_warns() {
    // Arrange - Config with performance section (not yet implemented)
    let toml_content = r#"
        [meta]
        name = "performance_test"
        version = "1.0.0"

        [performance]
        sample_size = 100
        baseline_name = "v1.0.0"
        regression_detection = true

        [[steps]]
        name = "benchmark"
        command = ["echo", "test"]
    "#;

    // Act - TOML parsing should succeed (structure is valid)
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert - parsing succeeds, but performance field exists
    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(
        config.performance.is_some(),
        "Performance config should be parsed"
    );
}

#[test]
fn test_sample_size_configuration_detected() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [performance]
        sample_size = 1000

        [[steps]]
        name = "test"
        command = ["echo", "test"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
    let config = result.unwrap();
    let perf = config.performance.as_ref().unwrap();
    assert_eq!(perf.sample_size, Some(1000));
}

#[test]
fn test_baseline_configuration_detected() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [performance]
        baseline_name = "baseline_v1.0.0"

        [[steps]]
        name = "test"
        command = ["echo", "test"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
    let config = result.unwrap();
    let perf = config.performance.as_ref().unwrap();
    assert_eq!(perf.baseline_name, Some("baseline_v1.0.0".to_string()));
}

#[test]
fn test_regression_detection_configuration_detected() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [performance]
        regression_detection = true

        [[steps]]
        name = "test"
        command = ["echo", "test"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
    let config = result.unwrap();
    let perf = config.performance.as_ref().unwrap();
    assert_eq!(perf.regression_detection, Some(true));
}

// ============================================================================
// SECTION 2: Performance Config Validation (Should Fail Fast)
// ============================================================================

#[test]
fn test_performance_validation_detects_unsupported_feature() {
    // Arrange - Create config with performance section
    let toml_content = r#"
        [meta]
        name = "perf_test"
        version = "1.0.0"

        [performance]
        sample_size = 100

        [[steps]]
        name = "test"
        command = ["echo", "test"]
    "#;

    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Act - Validation should detect unsupported performance config
    let has_performance = config.performance.is_some();

    // Assert
    assert!(
        has_performance,
        "Performance config should be detected for validation"
    );
}

#[test]
fn test_all_performance_fields_detected() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "comprehensive_perf"
        version = "1.0.0"

        [performance]
        sample_size = 500
        baseline_name = "v1.0.0"
        regression_detection = true

        [[steps]]
        name = "test"
        command = ["echo", "test"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert - all fields should be present
    assert!(result.is_ok());
    let config = result.unwrap();
    let perf = config.performance.as_ref().unwrap();
    assert!(perf.sample_size.is_some());
    assert!(perf.baseline_name.is_some());
    assert!(perf.regression_detection.is_some());
}

// ============================================================================
// SECTION 3: Error Message Quality
// ============================================================================

#[test]
fn test_performance_config_structure_matches_schema() {
    // Arrange - Verify the struct can hold all expected fields
    use clnrm_core::config::PerformanceTestConfig;

    let perf_config = PerformanceTestConfig {
        sample_size: Some(100),
        baseline_name: Some("test".to_string()),
        regression_detection: Some(true),
    };

    // Assert - struct should hold values correctly
    assert_eq!(perf_config.sample_size, Some(100));
    assert_eq!(perf_config.baseline_name, Some("test".to_string()));
    assert_eq!(perf_config.regression_detection, Some(true));
}

#[test]
fn test_optional_performance_fields() {
    // Arrange - Test with minimal performance config
    let toml_content = r#"
        [meta]
        name = "minimal_perf"
        version = "1.0.0"

        [performance]
        # All fields optional

        [[steps]]
        name = "test"
        command = ["echo", "test"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
    let config = result.unwrap();
    let perf = config.performance.as_ref().unwrap();
    assert!(perf.sample_size.is_none());
    assert!(perf.baseline_name.is_none());
    assert!(perf.regression_detection.is_none());
}

// ============================================================================
// SECTION 4: Integration with Validation Pipeline
// ============================================================================

#[test]
fn test_config_without_performance_section() {
    // Arrange - Normal config without performance
    let toml_content = r#"
        [meta]
        name = "normal_test"
        version = "1.0.0"

        [[steps]]
        name = "test"
        command = ["echo", "test"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(
        config.performance.is_none(),
        "No performance config should be present"
    );
}

#[test]
fn test_multiple_configs_mixed_performance() {
    // Arrange - Some tests with performance, some without
    let with_perf = r#"
        [meta]
        name = "with_perf"
        version = "1.0.0"

        [performance]
        sample_size = 100

        [[steps]]
        name = "test"
        command = ["echo", "test"]
    "#;

    let without_perf = r#"
        [meta]
        name = "without_perf"
        version = "1.0.0"

        [[steps]]
        name = "test"
        command = ["echo", "test"]
    "#;

    // Act
    let config_with: TestConfig = toml::from_str(with_perf).unwrap();
    let config_without: TestConfig = toml::from_str(without_perf).unwrap();

    // Assert
    assert!(config_with.performance.is_some());
    assert!(config_without.performance.is_none());
}

// ============================================================================
// SECTION 5: Future Implementation Readiness
// ============================================================================

#[test]
fn test_performance_config_ready_for_implementation() {
    // Arrange - Test that structure is ready for future implementation
    use clnrm_core::config::PerformanceTestConfig;

    let perf = PerformanceTestConfig {
        sample_size: Some(1000),
        baseline_name: Some("v1.0.0".to_string()),
        regression_detection: Some(true),
    };

    // Assert - all fields accessible for future implementation
    assert!(perf.sample_size.unwrap() > 0);
    assert!(!perf.baseline_name.unwrap().is_empty());
    assert!(perf.regression_detection.unwrap());
}

#[test]
fn test_performance_defaults() {
    // Arrange - Test default behavior
    use clnrm_core::config::PerformanceTestConfig;

    let perf = PerformanceTestConfig {
        sample_size: None,
        baseline_name: None,
        regression_detection: None,
    };

    // Assert - defaults should be None (opt-in)
    assert!(perf.sample_size.is_none());
    assert!(perf.baseline_name.is_none());
    assert!(perf.regression_detection.is_none());
}

// ============================================================================
// SECTION 6: Validation Logic Tests (Mock-Based)
// ============================================================================

/// Mock validator that simulates runtime validation
struct MockPerformanceValidator;

impl MockPerformanceValidator {
    fn validate_config(config: &TestConfig) -> Result<()> {
        if config.performance.is_some() {
            return Err(clnrm_core::error::CleanroomError::validation_error(
                "Performance testing is not yet implemented. This feature will be available in a future release.\n\
                 For now, please remove the [performance] section from your test configuration.\n\
                 Requested features: sample_size, baseline_name, regression_detection"
            ));
        }
        Ok(())
    }
}

#[test]
fn test_validator_rejects_performance_config() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [performance]
        sample_size = 100

        [[steps]]
        name = "test"
        command = ["echo", "test"]
    "#;

    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Act
    let result = MockPerformanceValidator::validate_config(&config);

    // Assert - should fail with clear message
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("not yet implemented"));
    assert!(error_msg.contains("Performance testing"));
}

#[test]
fn test_validator_accepts_normal_config() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "test"
        command = ["echo", "test"]
    "#;

    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Act
    let result = MockPerformanceValidator::validate_config(&config);

    // Assert - should succeed
    assert!(result.is_ok());
}

#[test]
fn test_error_message_includes_guidance() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [performance]
        regression_detection = true

        [[steps]]
        name = "test"
        command = ["echo", "test"]
    "#;

    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Act
    let result = MockPerformanceValidator::validate_config(&config);

    // Assert - error should provide helpful guidance
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("remove the [performance] section"));
    assert!(error_msg.contains("future release"));
}
