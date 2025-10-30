//! Validation test suite for false positive prevention
//!
//! This test suite ensures that the clnrm framework does not produce false positives.
//! It validates that:
//! - Error cases actually fail (not fake success with Ok(()))
//! - Assertions properly check container state
//! - Hermetic isolation is maintained between tests
//! - Async operations are properly synchronized
//!
//! Based on false positive analysis from:
//! - docs/research/FALSE_POSITIVE_ANALYSIS_REPORT.md
//! - docs/FALSE_POSITIVES_DETECTED.md
//! - docs/README_FALSE_POSITIVES.md

mod assertion_validation;
mod async_synchronization;
mod error_cases;
mod hermetic_isolation;
