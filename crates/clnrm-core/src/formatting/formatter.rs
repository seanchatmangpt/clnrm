//! Formatter Trait
//!
//! Core trait defining the contract for all test output formatters.
//! Follows London School TDD principles with clear collaboration contracts.

use crate::error::Result;
use crate::formatting::test_result::TestSuite;

/// Type of formatter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatterType {
    /// Human-readable terminal output (default)
    Human,
    /// Structured JSON output
    Json,
    /// JUnit XML format for CI integration
    Junit,
    /// Test Anything Protocol (TAP) format
    Tap,
}

impl FormatterType {
    /// Parse formatter type from string
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "human" | "h" => Some(Self::Human),
            "json" | "j" => Some(Self::Json),
            "junit" | "xml" => Some(Self::Junit),
            "tap" | "t" => Some(Self::Tap),
            _ => None,
        }
    }

    /// Get the default file extension for this formatter
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Human => "txt",
            Self::Json => "json",
            Self::Junit => "xml",
            Self::Tap => "tap",
        }
    }

    /// Get formatter type name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::Json => "json",
            Self::Junit => "junit",
            Self::Tap => "tap",
        }
    }
}

/// Formatter trait for test output
///
/// All formatters must implement this trait to provide consistent output generation.
/// This trait defines the collaboration contract between the test runner and formatters.
///
/// # London School TDD Note
/// This trait is designed for mock-based testing. Implementations should be
/// independently testable using mock test suites.
pub trait Formatter: Send + Sync {
    /// Format a test suite into a string
    ///
    /// # Arguments
    /// * `suite` - Test suite containing test results
    ///
    /// # Returns
    /// * `Result<String>` - Formatted output string
    ///
    /// # Errors
    /// Returns error if formatting fails (e.g., serialization errors)
    fn format(&self, suite: &TestSuite) -> Result<String>;

    /// Get the formatter name
    fn name(&self) -> &'static str;

    /// Get the formatter type
    fn formatter_type(&self) -> FormatterType;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formatter_type_from_str_human() {
        // Arrange & Act
        let result = FormatterType::from_string("human");

        // Assert
        assert_eq!(result, Some(FormatterType::Human));
    }

    #[test]
    fn test_formatter_type_from_str_json() {
        // Arrange & Act
        let result = FormatterType::from_string("json");

        // Assert
        assert_eq!(result, Some(FormatterType::Json));
    }

    #[test]
    fn test_formatter_type_from_str_junit() {
        // Arrange & Act
        let result = FormatterType::from_string("junit");

        // Assert
        assert_eq!(result, Some(FormatterType::Junit));
    }

    #[test]
    fn test_formatter_type_from_str_tap() {
        // Arrange & Act
        let result = FormatterType::from_string("tap");

        // Assert
        assert_eq!(result, Some(FormatterType::Tap));
    }

    #[test]
    fn test_formatter_type_from_str_case_insensitive() {
        // Arrange & Act
        let result = FormatterType::from_string("HUMAN");

        // Assert
        assert_eq!(result, Some(FormatterType::Human));
    }

    #[test]
    fn test_formatter_type_from_str_aliases() {
        // Arrange & Act
        let human_alias = FormatterType::from_string("h");
        let json_alias = FormatterType::from_string("j");
        let junit_alias = FormatterType::from_string("xml");

        // Assert
        assert_eq!(human_alias, Some(FormatterType::Human));
        assert_eq!(json_alias, Some(FormatterType::Json));
        assert_eq!(junit_alias, Some(FormatterType::Junit));
    }

    #[test]
    fn test_formatter_type_from_str_invalid() {
        // Arrange & Act
        let result = FormatterType::from_string("invalid");

        // Assert
        assert_eq!(result, None);
    }

    #[test]
    fn test_formatter_type_extension() {
        // Arrange & Act & Assert
        assert_eq!(FormatterType::Human.extension(), "txt");
        assert_eq!(FormatterType::Json.extension(), "json");
        assert_eq!(FormatterType::Junit.extension(), "xml");
        assert_eq!(FormatterType::Tap.extension(), "tap");
    }

    #[test]
    fn test_formatter_type_name() {
        // Arrange & Act & Assert
        assert_eq!(FormatterType::Human.name(), "human");
        assert_eq!(FormatterType::Json.name(), "json");
        assert_eq!(FormatterType::Junit.name(), "junit");
        assert_eq!(FormatterType::Tap.name(), "tap");
    }
}
