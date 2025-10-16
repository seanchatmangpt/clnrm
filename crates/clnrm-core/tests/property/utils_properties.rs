//! Property-based tests for utility functions
//!
//! This module tests that utility functions maintain critical invariants
//! across a wide range of randomly generated inputs.

use clnrm_core::utils::*;
use proptest::prelude::*;

// Import custom generators
use clnrm_core::testing::property_generators::*;

// =============================================================================
// Property 1: Regex Validation Consistency
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Property: If regex validation succeeds, the pattern can be used for matching
    ///
    /// Rationale: A pattern that passes validation should be usable in match operations
    /// without causing errors.
    ///
    /// Invariant: validate_regex(p).is_ok() ⟹ execute_regex_match(text, p).is_ok()
    #[test]
    fn prop_regex_validation_consistency(
        pattern in arb_safe_regex(),
        text in "[a-zA-Z0-9 @./#-]{0,100}",
    ) {
        // If validation succeeds
        if validate_regex(&pattern).is_ok() {
            // Then matching should not fail (even if no match found)
            let match_result = execute_regex_match(&text, &pattern);
            prop_assert!(
                match_result.is_ok(),
                "Pattern '{}' passed validation but failed during matching: {:?}",
                pattern,
                match_result.err()
            );
        }
    }
}

// =============================================================================
// Property 2: Regex Match Determinism
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Property: Regex matching is deterministic
    ///
    /// Rationale: The same pattern and text should always produce the same result
    ///
    /// Invariant: execute_regex_match(text, pattern) result is consistent
    #[test]
    fn prop_regex_match_deterministic(
        pattern in arb_safe_regex(),
        text in "[a-zA-Z0-9 ]{0,100}",
    ) {
        if validate_regex(&pattern).is_ok() {
            // Execute match twice
            let result1 = execute_regex_match(&text, &pattern);
            let result2 = execute_regex_match(&text, &pattern);

            // Results must be identical
            prop_assert_eq!(
                result1.is_ok(),
                result2.is_ok(),
                "Regex match consistency check failed"
            );

            if let (Ok(matches1), Ok(matches2)) = (result1, result2) {
                prop_assert_eq!(
                    matches1,
                    matches2,
                    "Regex match results must be deterministic"
                );
            }
        }
    }
}

// =============================================================================
// Property 3: TOML Parsing Validity
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Property: Valid TOML strings parse successfully
    ///
    /// Rationale: Known-valid TOML should always parse without errors
    ///
    /// Invariant: parse_toml_config(valid_toml).is_ok()
    #[test]
    fn prop_toml_parsing_validity(toml_str in arb_toml_config()) {
        let result = parse_toml_config(&toml_str);

        prop_assert!(
            result.is_ok(),
            "Valid TOML failed to parse: {:?}\nTOML content:\n{}",
            result.err(),
            toml_str
        );

        // Parsed result should be a JSON object or valid value
        if let Ok(json_value) = result {
            prop_assert!(
                json_value.is_object() || json_value.is_array() || json_value.is_string(),
                "Parsed TOML must produce valid JSON structure"
            );
        }
    }
}

// =============================================================================
// Property 4: Session ID Uniqueness
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Property: Generated session IDs are unique
    ///
    /// Rationale: Session IDs must have high collision resistance for
    /// concurrent test execution.
    ///
    /// Invariant: Probability of collision is negligible (< 1/1000)
    #[test]
    fn prop_session_id_uniqueness(_seed in any::<u64>()) {
        // Generate multiple session IDs
        let mut ids = std::collections::HashSet::new();
        let count = 100;

        for _ in 0..count {
            let id = generate_session_id();

            // Check format
            prop_assert!(
                id.starts_with("session_"),
                "Session ID must start with 'session_'"
            );

            // Check uniqueness
            prop_assert!(
                !ids.contains(&id),
                "Session ID collision detected: {}",
                id
            );

            ids.insert(id);
        }

        // All IDs should be unique
        prop_assert_eq!(
            ids.len(),
            count,
            "Not all generated session IDs were unique"
        );
    }
}

// =============================================================================
// Property 5: Duration Formatting Consistency
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Property: Duration formatting is consistent and readable
    ///
    /// Rationale: Formatted durations should be human-readable and contain
    /// appropriate units.
    ///
    /// Invariant: format_duration produces non-empty string with unit suffix
    #[test]
    fn prop_duration_formatting_consistency(duration in arb_duration()) {
        let formatted = format_duration(duration);

        // Must not be empty
        prop_assert!(
            !formatted.is_empty(),
            "Formatted duration must not be empty"
        );

        // Must contain a unit suffix
        let has_unit = formatted.ends_with('s')
            || formatted.ends_with("ms")
            || formatted.ends_with("μs")
            || formatted.ends_with("ns");

        prop_assert!(
            has_unit,
            "Formatted duration '{}' must contain time unit",
            formatted
        );

        // Must contain numeric portion
        let has_number = formatted.chars().any(|c| c.is_numeric());
        prop_assert!(
            has_number,
            "Formatted duration '{}' must contain numeric value",
            formatted
        );
    }
}

// =============================================================================
// Property 6: Duration Formatting Order of Magnitude
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Property: Duration formatting uses appropriate units for magnitude
    ///
    /// Rationale: Large durations should use seconds, small ones microseconds
    ///
    /// Invariant: Unit selection matches duration magnitude
    #[test]
    fn prop_duration_formatting_magnitude(duration in arb_duration()) {
        let formatted = format_duration(duration);

        if duration.as_secs() > 0 {
            // Should use seconds for durations >= 1s
            prop_assert!(
                formatted.ends_with('s') && !formatted.ends_with("ms") && !formatted.ends_with("μs"),
                "Duration >= 1s should use 's' unit, got: {}",
                formatted
            );
        } else if duration.as_millis() > 0 {
            // Should use milliseconds for durations >= 1ms
            prop_assert!(
                formatted.ends_with("ms"),
                "Duration >= 1ms should use 'ms' unit, got: {}",
                formatted
            );
        } else {
            // Should use microseconds for durations < 1ms
            prop_assert!(
                formatted.ends_with("μs"),
                "Duration < 1ms should use 'μs' unit, got: {}",
                formatted
            );
        }
    }
}

// =============================================================================
// Property 7: Path Validation Idempotence
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Path validation result doesn't change on repeated calls
    ///
    /// Rationale: Validation should be pure and not modify filesystem state
    ///
    /// Invariant: validate_file_path(p).is_ok() consistency across calls
    #[test]
    fn prop_path_validation_idempotent(path in prop_oneof![
        Just("."),
        Just(".."),
        Just("Cargo.toml"),
        Just("src"),
        Just("nonexistent_file_xyz"),
    ]) {
        let result1 = validate_file_path(&path);
        let result2 = validate_file_path(&path);

        // Results must be consistent
        prop_assert_eq!(
            result1.is_ok(),
            result2.is_ok(),
            "Path validation must be idempotent for path: {}",
            path
        );
    }
}

// =============================================================================
// Property 8: Regex Empty Pattern Handling
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Property: Empty regex patterns are handled consistently
    ///
    /// Rationale: Edge case of empty pattern should be well-defined
    ///
    /// Invariant: Empty pattern validation and matching behave consistently
    #[test]
    fn prop_regex_empty_pattern_handling(text in "[a-zA-Z0-9]{0,50}") {
        let empty_pattern = "";

        // Validation might succeed or fail, but should be consistent
        let validation = validate_regex(empty_pattern);

        if validation.is_ok() {
            // If validation passes, matching should work
            let match_result = execute_regex_match(&text, empty_pattern);
            prop_assert!(
                match_result.is_ok(),
                "Empty pattern validation passed but matching failed"
            );
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_property_tests_can_run() {
        // Smoke test to ensure property tests are properly configured
        let result = validate_regex(r"[a-z]+");
        assert!(result.is_ok());
    }
}
