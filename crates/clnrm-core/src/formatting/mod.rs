//! Formatting Module for Cleanroom v0.7.0
//!
//! Provides multiple formatting capabilities:
//! 1. TOML formatting - Deterministic TOML file formatting
//! 2. Test output formatting - Multiple formats for test results

// TOML formatting submodule
pub mod toml_fmt;

// Test output formatting submodules
pub mod formatter;
pub mod test_result;
pub mod human;
pub mod json;
pub mod junit;
pub mod tap;

use crate::error::Result;

// Re-export TOML formatting functions for backward compatibility
pub use toml_fmt::{format_toml_content, format_toml_file, needs_formatting, verify_idempotency};

// Re-export test output formatting
pub use formatter::{Formatter, FormatterType};
pub use test_result::{TestResult, TestStatus, TestSuite};
pub use human::HumanFormatter;
pub use json::JsonFormatter;
pub use junit::JunitFormatter;
pub use tap::TapFormatter;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_test_results_human() -> Result<()> {
        // Arrange
        let suite = TestSuite::new("test_suite");

        // Act
        let result = format_test_results(FormatterType::Human, &suite)?;

        // Assert
        assert!(!result.is_empty());

        Ok(())
    }

    #[test]
    fn test_format_test_results_json() -> Result<()> {
        // Arrange
        let suite = TestSuite::new("test_suite");

        // Act
        let result = format_test_results(FormatterType::Json, &suite)?;

        // Assert
        assert!(!result.is_empty());

        Ok(())
    }

    #[test]
    fn test_format_test_results_junit() -> Result<()> {
        // Arrange
        let suite = TestSuite::new("test_suite");

        // Act
        let result = format_test_results(FormatterType::Junit, &suite)?;

        // Assert
        assert!(!result.is_empty());

        Ok(())
    }

    #[test]
    fn test_format_test_results_tap() -> Result<()> {
        // Arrange
        let suite = TestSuite::new("test_suite");

        // Act
        let result = format_test_results(FormatterType::Tap, &suite)?;

        // Assert
        assert!(!result.is_empty());

        Ok(())
    }
}
