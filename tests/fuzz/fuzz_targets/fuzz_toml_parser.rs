//! Fuzz target for TOML configuration parser
//!
//! This fuzzer tests the robustness of the TOML parser against malformed,
//! malicious, and edge-case inputs. It aims to discover:
//! - Parser crashes or panics
//! - Infinite loops or excessive resource consumption
//! - Memory safety issues
//! - Denial of service vectors
//! - Validation bypass opportunities

#![no_main]

use libfuzzer_sys::fuzz_target;
use clnrm_core::config::{parse_toml_config, TestConfig};

fuzz_target!(|data: &[u8]| {
    // Convert fuzzer input to UTF-8 string (lossy conversion for invalid UTF-8)
    let input = String::from_utf8_lossy(data);

    // Attempt to parse the input as TOML configuration
    // We don't care if it fails, we just care that it doesn't crash
    let _ = parse_toml_config(&input);

    // If parsing succeeds, attempt validation to test the validation logic
    if let Ok(config) = parse_toml_config(&input) {
        let _ = config.validate();

        // Test serialization round-trip
        if let Ok(serialized) = toml::to_string(&config) {
            let _ = parse_toml_config(&serialized);
        }

        // Test JSON serialization (another potential attack vector)
        let _ = serde_json::to_string(&config);
    }
});
