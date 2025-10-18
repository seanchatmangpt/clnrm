//! Formatting Module for Cleanroom v0.7.0
//!
//! Provides multiple formatting capabilities:
//! 1. TOML formatting - Deterministic TOML file formatting
//! 2. Test output formatting - Multiple formats for test results

// TOML formatting submodule
pub mod toml_fmt;

// Test output formatting submodules
pub mod formatter;
pub mod human;
pub mod json;
pub mod junit;
pub mod tap;
pub mod test_result;

use crate::error::Result;

// Re-export TOML formatting functions for backward compatibility
pub use toml_fmt::{format_toml_content, format_toml_file, needs_formatting, verify_idempotency};

// Re-export test output formatting
pub use formatter::{Formatter, FormatterType};
pub use human::HumanFormatter;
pub use json::JsonFormatter;
pub use junit::JunitFormatter;
pub use tap::TapFormatter;
pub use test_result::{TestResult, TestStatus, TestSuite};

/// Format test results using the specified formatter
///
/// # Arguments
/// * `formatter_type` - Type of formatter to use
/// * `suite` - Test suite containing test results
///
/// # Returns
/// * `Result<String>` - Formatted output string
///
/// # Errors
/// Returns error if formatting fails
pub fn format_test_results(formatter_type: FormatterType, suite: &TestSuite) -> Result<String> {
    let formatter: Box<dyn Formatter> = match formatter_type {
        FormatterType::Human => Box::new(HumanFormatter::new()),
        FormatterType::Json => Box::new(JsonFormatter::new()),
        FormatterType::Junit => Box::new(JunitFormatter::new()),
        FormatterType::Tap => Box::new(TapFormatter::new()),
    };

    formatter.format(suite)
}
