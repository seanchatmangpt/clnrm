//! Red/Green TDD workflow validation command
//!
//! Implements PRD v1.0 `clnrm redgreen` command for TDD validation.
//!
//! This module provides the public API for red/green TDD validation.
//! The actual implementation is in the redgreen_impl module.

use crate::cli::commands::v0_7_0::redgreen_impl::run_red_green_validation as run_red_green_validation_impl;
use crate::cli::types::TddState;
use crate::error::Result;
use std::path::PathBuf;

/// Run red/green TDD workflow validation
///
/// Validates test-driven development workflow by ensuring tests fail before
/// implementation and pass after.
///
/// # Arguments
///
/// * `paths` - Test files to validate
/// * `verify_red` - Verify all tests initially fail (red state) [Legacy]
/// * `verify_green` - Verify all tests pass after implementation (green state) [Legacy]
///
/// # Core Team Standards
///
/// - No unwrap() or expect()
/// - Returns Result<T, CleanroomError>
/// - Proper error handling
/// - Delegates to comprehensive implementation in redgreen_impl module
///
/// # Examples
///
/// ```rust,no_run
/// use std::path::PathBuf;
///
/// // Run red/green validation with legacy flags
/// let paths = vec![PathBuf::from("tests/test.toml")];
/// run_red_green_validation(&paths, true, false).await?;
/// ```
pub async fn run_red_green_validation(
    paths: &[PathBuf],
    verify_red: bool,
    verify_green: bool,
) -> Result<()> {
    // Convert legacy flags to new API
    let expect = if verify_red {
        Some(TddState::Red)
    } else if verify_green {
        Some(TddState::Green)
    } else {
        None
    };

    // Delegate to the comprehensive implementation
    run_red_green_validation_impl(paths, expect, verify_red, verify_green).await
}
