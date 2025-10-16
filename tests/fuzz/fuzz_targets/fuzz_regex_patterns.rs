//! Fuzz target for regex pattern validation
//!
//! Tests regex compilation and matching for:
//! - ReDoS (Regular Expression Denial of Service)
//! - Invalid regex syntax
//! - Catastrophic backtracking
//! - Memory exhaustion in regex matching

#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::{Arbitrary, Unstructured};
use regex::Regex;

#[derive(Debug, Arbitrary)]
struct FuzzRegex {
    pattern: String,
    test_string: String,
}

fuzz_target!(|data: &[u8]| {
    let mut unstructured = Unstructured::new(data);

    if let Ok(fuzz_regex) = FuzzRegex::arbitrary(&mut unstructured) {
        // Limit pattern and test string length to prevent excessive memory usage
        let pattern = limit_string(&fuzz_regex.pattern, 1000);
        let test_string = limit_string(&fuzz_regex.test_string, 10000);

        // Test regex compilation (can panic on invalid regex)
        if let Ok(regex) = Regex::new(&pattern) {
            // Test matching (potential ReDoS attack vector)
            // Use a timeout mechanism in production fuzzing
            let _ = regex.is_match(&test_string);

            // Test find operations
            let _ = regex.find(&test_string);

            // Test captures
            let _ = regex.captures(&test_string);

            // Test find_iter (can be expensive)
            let matches: Vec<_> = regex.find_iter(&test_string).take(100).collect();
            let _ = matches.len();
        }

        // Test common regex patterns used in the codebase
        test_expected_output_regex(&pattern, &test_string);
        test_file_pattern_regex(&pattern, &test_string);
    }
});

fn limit_string(s: &str, max_len: usize) -> String {
    s.chars().take(max_len).collect()
}

fn test_expected_output_regex(pattern: &str, test_str: &str) {
    // This tests the expected_output_regex field from StepConfig
    if let Ok(regex) = Regex::new(pattern) {
        // Test if the regex matches expected output
        let _ = regex.is_match(test_str);

        // Test multiline matching
        let multiline_pattern = format!("(?m){}", pattern);
        if let Ok(ml_regex) = Regex::new(&multiline_pattern) {
            let _ = ml_regex.is_match(test_str);
        }
    }
}

fn test_file_pattern_regex(pattern: &str, test_str: &str) {
    // Test file pattern matching (glob-like patterns)
    let glob_pattern = pattern.replace("*", ".*");

    if let Ok(regex) = Regex::new(&glob_pattern) {
        let _ = regex.is_match(test_str);
    }

    // Test escaped special characters
    let escaped = regex::escape(pattern);
    let _ = Regex::new(&escaped);
}
