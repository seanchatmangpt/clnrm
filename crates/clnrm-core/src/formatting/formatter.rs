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
