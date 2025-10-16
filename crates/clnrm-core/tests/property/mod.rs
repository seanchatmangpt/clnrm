//! Property-based test suite for CLNRM
//!
//! This module contains comprehensive property-based tests that validate
//! critical system invariants across randomly generated inputs.
//!
//! ## Running Property Tests
//!
//! ```bash
//! # Run all property tests with default configuration (256 cases)
//! cargo test --test property_tests
//!
//! # Run with increased test cases for thorough checking
//! PROPTEST_CASES=10000 cargo test --test property_tests
//!
//! # Run specific property test module
//! cargo test --test property_tests policy_properties
//!
//! # Run with specific seed for reproducibility
//! PROPTEST_SEED=1234567890 cargo test --test property_tests
//! ```

mod policy_properties;
mod utils_properties;

// Re-export for convenience
pub use policy_properties::*;
pub use utils_properties::*;
