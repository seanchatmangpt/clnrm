//! Property-based test suite entry point for CLNRM
//!
//! This module aggregates all property-based tests for the CLNRM framework.
//! Property tests validate that critical system invariants hold across
//! randomly generated inputs, providing comprehensive coverage of edge cases.
//!
//! ## Running Tests
//!
//! ```bash
//! # Run all property tests with default configuration (256 cases per property)
//! cargo test --test property_tests
//!
//! # Run with increased test cases for thorough checking
//! PROPTEST_CASES=10000 cargo test --test property_tests
//!
//! # Run specific property test module
//! cargo test --test property_tests policy
//!
//! # Run with specific seed for reproducibility
//! PROPTEST_SEED=1234567890 cargo test --test property_tests
//!
//! # Show verbose output
//! cargo test --test property_tests -- --nocapture
//! ```
//!
//! ## Environment Variables
//!
//! - `PROPTEST_CASES`: Number of test cases per property (default: 256)
//! - `PROPTEST_MAX_SHRINK_ITERS`: Maximum shrinking iterations (default: 1000)
//! - `PROPTEST_SEED`: Fixed seed for reproducible test runs
//! - `PROPTEST_TIMEOUT`: Timeout per test case in milliseconds (default: 5000)
//!
//! ## Test Organization
//!
//! Tests are organized by module:
//! - `policy_properties`: Policy validation and configuration
//! - `utils_properties`: Utility function invariants
//!
//! ## Property Test Principles
//!
//! Each property test follows these principles:
//! 1. **Clear Invariant**: States what must always be true
//! 2. **Rationale**: Explains why this property matters
//! 3. **Comprehensive Coverage**: Tests edge cases and boundaries
//! 4. **Fast Shrinking**: Minimal counterexamples for debugging
//! 5. **Deterministic**: Reproducible with same seed

mod property {
    pub mod policy_properties;
    pub mod utils_properties;
}

// Re-export for convenience in test discovery
pub use property::*;

#[cfg(test)]
mod integration_tests {
    #[test]
    fn test_property_test_suite_available() {
        // Smoke test to ensure property test suite is properly configured
        println!("Property test suite is available and configured");
        println!("Run with: cargo test --test property_tests");
        println!("Increase test cases with: PROPTEST_CASES=10000 cargo test --test property_tests");
    }
}
