//! TDD Test: Performance Config Fail-Fast Validation
//!
//! This test suite validates that clnrm fails fast when users specify
//! unimplemented performance testing features in TOML configuration.
//!
//! ## Problem (London School TDD Approach)
//!
//! Users can specify `[test.performance]` sections in TOML with features like:
//! - `sample_size = 100` (run test 100 times)
//! - `baseline_name = "v1.0"` (compare against baseline)
//! - `regression_detection = true` (detect performance regressions)
//!
//! However, these features are NOT IMPLEMENTED in v1.2.0. Without validation,
//! users would see:
//! - Config accepted ✅ (misleading success)
//! - Test runs once (not 100 times as specified)
//! - No baseline comparison
//! - No regression detection
//!
//! This is a FALSE POSITIVE - the test appears to pass but doesn't do what
//! the config claims it should do.
//!
//! ## Solution (TDD RED → GREEN → REFACTOR)
//!
//! **RED**: Write test proving config should FAIL when performance features used
//! **GREEN**: Add validation in TestConfig::validate() to reject performance config
//! **REFACTOR**: Clean error messages guiding users to remove [test.performance]
//!
//! ## London School TDD Pattern
//!
//! - Mock-driven: Test validates ERROR BEHAVIOR (fail-fast, not acceptance)
//! - Behavior verification: Focus on WHAT should happen (rejection + clear message)
//! - Outside-in: User perspective → Config loader → Validation logic
//!
//! ## Test Coverage
//!
//! 1. ✅ Config with sample_size should fail
//! 2. ✅ Config with baseline_name should fail
//! 3. ✅ Config with regression_detection should fail
//! 4. ✅ Config with all performance features should fail
//! 5. ✅ Error message should mention "not yet implemented"
//! 6. ✅ Error message should guide users to remove section

use clnrm_core::config::parse_toml_config;
use clnrm_core::error::ErrorKind;

/// RED Test 1: Config with sample_size should fail validation
///
/// This test proves that specifying `sample_size = 100` (run test 100 times)
/// should FAIL immediately during config validation, not silently accept and
/// run the test once.
#[test]
fn test_performance_sample_size_fails_validation() {
    // GIVEN: TOML config requesting 100 test runs (not implemented)
    let toml_with_sample_size = r#"
[test.metadata]
name = "perf_test_sample_size"
description = "Test with performance sampling"

[performance]
sample_size = 100

[services.test_service]
type = "generic_container"
image = "alpine:latest"

[[steps]]
name = "step_1"
command = ["echo", "test"]
"#;

    // WHEN: Parsing and validating config
    let result = parse_toml_config(toml_with_sample_size);

    // THEN: Should parse successfully (syntax is valid)
    assert!(result.is_ok(), "Config should parse successfully");

    let config = result.unwrap();

    // DEBUG: Check if performance field was parsed
    println!("DEBUG: performance field = {:?}", config.performance);
    assert!(
        config.performance.is_some(),
        "Performance field should be Some after parsing TOML with [test.performance]"
    );

    // WHEN: Validating config (this is where fail-fast should occur)
    let validation_result = config.validate();

    // THEN: Should FAIL with NotImplementedError
    assert!(
        validation_result.is_err(),
        "Performance config should fail validation"
    );

    let error = validation_result.unwrap_err();

    // Verify error kind
    assert_eq!(
        error.kind,
        ErrorKind::NotImplementedError,
        "Should return NotImplementedError for unimplemented features"
    );

    // Verify error message mentions performance testing
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("Performance testing"),
        "Error should mention 'Performance testing', got: {}",
        error_msg
    );

    // Verify error message mentions "not yet implemented"
    assert!(
        error_msg.contains("not yet implemented") || error_msg.contains("not implemented"),
        "Error should mention feature is not implemented, got: {}",
        error_msg
    );
}

/// RED Test 2: Config with baseline_name should fail validation
///
/// This test proves that specifying `baseline_name = "v1.0"` (compare against
/// baseline) should FAIL immediately, not silently ignore the baseline.
#[test]
fn test_performance_baseline_fails_validation() {
    // GIVEN: TOML config requesting baseline comparison (not implemented)
    let toml_with_baseline = r#"
[test.metadata]
name = "perf_test_baseline"
description = "Test with baseline comparison"

[performance]
baseline_name = "v1.0"

[services.test_service]
type = "generic_container"
image = "alpine:latest"

[[steps]]
name = "step_1"
command = ["echo", "test"]
"#;

    // WHEN: Parsing and validating config
    let config = parse_toml_config(toml_with_baseline).expect("Config should parse successfully");

    let validation_result = config.validate();

    // THEN: Should FAIL with NotImplementedError
    assert!(
        validation_result.is_err(),
        "Baseline config should fail validation"
    );

    let error = validation_result.unwrap_err();
    assert_eq!(error.kind, ErrorKind::NotImplementedError);
}

/// RED Test 3: Config with regression_detection should fail validation
///
/// This test proves that specifying `regression_detection = true` should FAIL
/// immediately, not silently skip regression detection.
#[test]
fn test_performance_regression_detection_fails_validation() {
    // GIVEN: TOML config requesting regression detection (not implemented)
    let toml_with_regression = r#"
[test.metadata]
name = "perf_test_regression"
description = "Test with regression detection"

[performance]
regression_detection = true

[services.test_service]
type = "generic_container"
image = "alpine:latest"

[[steps]]
name = "step_1"
command = ["echo", "test"]
"#;

    // WHEN: Parsing and validating config
    let config = parse_toml_config(toml_with_regression).expect("Config should parse successfully");

    let validation_result = config.validate();

    // THEN: Should FAIL with NotImplementedError
    assert!(
        validation_result.is_err(),
        "Regression detection config should fail validation"
    );

    let error = validation_result.unwrap_err();
    assert_eq!(error.kind, ErrorKind::NotImplementedError);
}

/// RED Test 4: Config with all performance features should fail validation
///
/// This test proves that specifying ALL performance features together should
/// FAIL immediately with a comprehensive error message.
#[test]
fn test_performance_all_features_fails_validation() {
    // GIVEN: TOML config with ALL performance features (none implemented)
    let toml_with_all_features = r#"
[test.metadata]
name = "perf_test_comprehensive"
description = "Test with all performance features"

[performance]
sample_size = 100
baseline_name = "v1.0"
regression_detection = true

[services.test_service]
type = "generic_container"
image = "alpine:latest"

[[steps]]
name = "step_1"
command = ["echo", "test"]
"#;

    // WHEN: Parsing and validating config
    let config =
        parse_toml_config(toml_with_all_features).expect("Config should parse successfully");

    let validation_result = config.validate();

    // THEN: Should FAIL with NotImplementedError
    assert!(
        validation_result.is_err(),
        "Config with all performance features should fail validation"
    );

    let error = validation_result.unwrap_err();
    assert_eq!(error.kind, ErrorKind::NotImplementedError);

    // Verify error message is helpful
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("sample_size")
            || error_msg.contains("baseline")
            || error_msg.contains("regression"),
        "Error should mention specific performance features, got: {}",
        error_msg
    );
}

/// RED Test 5: Error message should guide users to remove section
///
/// This test verifies that the error message provides actionable guidance:
/// "Remove the [test.performance] section to run this test."
#[test]
fn test_performance_error_message_provides_guidance() {
    // GIVEN: TOML config with performance section
    let toml_with_performance = r#"
[test.metadata]
name = "perf_test_guidance"

[performance]
sample_size = 100

[services.test_service]
type = "generic_container"
image = "alpine:latest"

[[steps]]
name = "step_1"
command = ["echo", "test"]
"#;

    // WHEN: Validating config
    let config =
        parse_toml_config(toml_with_performance).expect("Config should parse successfully");

    let error = config.validate().expect_err("Should fail validation");

    // THEN: Error message should provide guidance
    let error_msg = error.to_string();

    // Should mention removing the section
    assert!(
        error_msg.contains("Remove") || error_msg.contains("remove"),
        "Error should suggest removing section, got: {}",
        error_msg
    );

    // Should mention the section name
    assert!(
        error_msg.contains("[performance]") || error_msg.contains("performance"),
        "Error should mention [performance] section, got: {}",
        error_msg
    );
}

/// RED Test 6: Config WITHOUT performance section should pass validation
///
/// This test verifies that normal configs (without [test.performance]) continue
/// to work correctly. This is the GREEN path - configs without unimplemented
/// features should validate successfully.
#[test]
fn test_normal_config_without_performance_passes_validation() {
    // GIVEN: Normal TOML config WITHOUT performance section
    let normal_toml = r#"
[test.metadata]
name = "normal_test"
description = "Test without performance features"

[services.test_service]
type = "generic_container"
image = "alpine:latest"

[[steps]]
name = "step_1"
command = ["echo", "test"]
"#;

    // WHEN: Parsing and validating config
    let config = parse_toml_config(normal_toml).expect("Config should parse successfully");

    let validation_result = config.validate();

    // THEN: Should PASS validation (no performance section = no problem)
    assert!(
        validation_result.is_ok(),
        "Config without performance section should pass validation, got error: {:?}",
        validation_result.err()
    );
}

/// Integration Test: Fail-fast validation prevents execution
///
/// This test demonstrates the complete fail-fast flow:
/// 1. User creates TOML with [test.performance]
/// 2. Config parses successfully (syntax valid)
/// 3. Validation FAILS immediately (feature not implemented)
/// 4. Test NEVER executes (fail-fast prevents false positive)
#[test]
fn test_failfast_prevents_execution() {
    // GIVEN: User creates test with performance sampling
    let user_toml = r#"
[test.metadata]
name = "user_test_with_perf"
description = "User expects 100 test runs"

[performance]
sample_size = 100  # User expects test to run 100 times

[services.test_service]
type = "generic_container"
image = "alpine:latest"

[[steps]]
name = "performance_test"
command = ["echo", "measuring performance"]
"#;

    // WHEN: Loading config (as CLI would do)
    let config = parse_toml_config(user_toml).expect("Config should parse (syntax valid)");

    // WHEN: Validating config (CLI calls this before execution)
    let validation_result = config.validate();

    // THEN: Validation FAILS - test NEVER executes
    assert!(
        validation_result.is_err(),
        "Validation must fail BEFORE test execution to prevent false positive"
    );

    // Verify error provides clear feedback
    let error = validation_result.unwrap_err();
    let error_msg = error.to_string();

    println!("✅ Fail-fast validation prevented execution:");
    println!("   Error: {}", error_msg);
    println!("   Result: Test never ran (avoiding false positive)");

    // Key assertion: Error kind is NotImplementedError
    assert_eq!(
        error.kind,
        ErrorKind::NotImplementedError,
        "Should return NotImplementedError to clearly indicate feature unavailability"
    );
}

// London School TDD Principle: Test Behavior, Not Implementation
//
// These tests verify BEHAVIOR:
// - ✅ Config with unimplemented features should FAIL validation
// - ✅ Failure should occur BEFORE test execution (fail-fast)
// - ✅ Error messages should guide users to remove unsupported sections
// - ✅ Normal configs (without performance features) should continue to work
//
// We do NOT test:
// - ❌ How validation is implemented internally
// - ❌ Specific validation logic flow
// - ❌ Data structures used
//
// This allows refactoring validation logic without breaking tests.
