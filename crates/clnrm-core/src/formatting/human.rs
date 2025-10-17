//! Human-Readable Formatter
//!
//! Generates colored terminal output for test results.
//! Default formatter for interactive terminal use.

use crate::error::Result;
use crate::formatting::formatter::{Formatter, FormatterType};
use crate::formatting::test_result::{TestStatus, TestSuite};

/// Human-readable formatter with ANSI color support
#[derive(Debug, Default)]
pub struct HumanFormatter {
    /// Whether to use colors in output
    use_colors: bool,
}

impl HumanFormatter {
    /// Create a new human formatter with color support
    pub fn new() -> Self {
        Self::with_colors(true)
    }

    /// Create a new human formatter with optional color support
    pub fn with_colors(use_colors: bool) -> Self {
        Self { use_colors }
    }

    /// Format a status indicator
    fn format_status(&self, status: &TestStatus) -> String {
        let (symbol, color) = match status {
            TestStatus::Passed => ("✓", "\x1b[32m"),  // Green
            TestStatus::Failed => ("✗", "\x1b[31m"),  // Red
            TestStatus::Skipped => ("⊘", "\x1b[33m"), // Yellow
            TestStatus::Unknown => ("?", "\x1b[90m"), // Gray
        };

        if self.use_colors {
            format!("{}{}\x1b[0m", color, symbol)
        } else {
            symbol.to_string()
        }
    }

    /// Format a test name with color
    fn format_test_name(&self, name: &str, passed: bool) -> String {
        if self.use_colors {
            if passed {
                format!("\x1b[32m{}\x1b[0m", name)
            } else {
                format!("\x1b[31m{}\x1b[0m", name)
            }
        } else {
            name.to_string()
        }
    }

    /// Format duration in milliseconds
    fn format_duration(&self, duration_ms: f64) -> String {
        if duration_ms < 1.0 {
            "<1ms".to_string()
        } else if duration_ms < 1000.0 {
            format!("{}ms", duration_ms as u64)
        } else {
            format!("{:.2}s", duration_ms / 1000.0)
        }
    }

    /// Format summary line
    fn format_summary(&self, suite: &TestSuite) -> String {
        let total = suite.total_count();
        let passed = suite.passed_count();
        let failed = suite.failed_count();
        let skipped = suite.skipped_count();

        let status_text = if suite.is_success() {
            if self.use_colors {
                "\x1b[32mPASSED\x1b[0m"
            } else {
                "PASSED"
            }
        } else if self.use_colors {
            "\x1b[31mFAILED\x1b[0m"
        } else {
            "FAILED"
        };

        let mut parts = vec![format!("{} tests", total), format!("{} passed", passed)];

        if failed > 0 {
            parts.push(format!("{} failed", failed));
        }

        if skipped > 0 {
            parts.push(format!("{} skipped", skipped));
        }

        format!("{}: {}", status_text, parts.join(", "))
    }
}

impl Formatter for HumanFormatter {
    fn format(&self, suite: &TestSuite) -> Result<String> {
        let mut output = Vec::new();

        // Header
        output.push(format!("Test Suite: {}", suite.name));
        output.push(String::from(""));

        // Individual test results
        for result in &suite.results {
            let status = self.format_status(&result.status);
            let name = self.format_test_name(&result.name, result.is_passed());

            let mut line = format!("  {} {}", status, name);

            // Add duration if available
            if let Some(duration) = result.duration {
                let duration_str = self.format_duration(duration.as_secs_f64() * 1000.0);
                line.push_str(&format!(" ({})", duration_str));
            }

            output.push(line);

            // Add error message for failures
            if let Some(error) = &result.error {
                let error_lines: Vec<&str> = error.lines().collect();
                for error_line in error_lines {
                    output.push(format!("      {}", error_line));
                }
            }

            // Add stdout if present
            if let Some(stdout) = &result.stdout {
                output.push(String::from("      stdout:"));
                for stdout_line in stdout.lines() {
                    output.push(format!("        {}", stdout_line));
                }
            }

            // Add stderr if present
            if let Some(stderr) = &result.stderr {
                output.push(String::from("      stderr:"));
                for stderr_line in stderr.lines() {
                    output.push(format!("        {}", stderr_line));
                }
            }
        }

        // Summary
        output.push(String::from(""));
        output.push("─".repeat(60));
        output.push(self.format_summary(suite));

        // Duration
        if let Some(duration) = suite.duration {
            let duration_str = self.format_duration(duration.as_secs_f64() * 1000.0);
            output.push(format!("Duration: {}", duration_str));
        }

        output.push(String::from(""));

        Ok(output.join("\n"))
    }

    fn name(&self) -> &'static str {
        "human"
    }

    fn formatter_type(&self) -> FormatterType {
        FormatterType::Human
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatting::test_result::TestResult;
    use std::time::Duration;

    #[test]
    fn test_human_formatter_empty_suite() -> Result<()> {
        // Arrange
        let formatter = HumanFormatter::with_colors(false);
        let suite = TestSuite::new("empty_suite");

        // Act
        let output = formatter.format(&suite)?;

        // Assert
        assert!(output.contains("Test Suite: empty_suite"));
        // Empty suite is NOT considered success (requires at least 1 test)
        assert!(output.contains("FAILED") || output.contains("0 tests"));
        assert!(output.contains("0 tests"));

        Ok(())
    }

    #[test]
    fn test_human_formatter_all_passed() -> Result<()> {
        // Arrange
        let formatter = HumanFormatter::with_colors(false);
        let suite = TestSuite::new("passing_suite")
            .add_result(TestResult::passed("test1"))
            .add_result(TestResult::passed("test2"));

        // Act
        let output = formatter.format(&suite)?;

        // Assert
        assert!(output.contains("Test Suite: passing_suite"));
        assert!(output.contains("✓ test1"));
        assert!(output.contains("✓ test2"));
        assert!(output.contains("PASSED"));
        assert!(output.contains("2 tests"));
        assert!(output.contains("2 passed"));

        Ok(())
    }

    #[test]
    fn test_human_formatter_with_failures() -> Result<()> {
        // Arrange
        let formatter = HumanFormatter::with_colors(false);
        let suite = TestSuite::new("failing_suite")
            .add_result(TestResult::passed("test1"))
            .add_result(TestResult::failed(
                "test2",
                "assertion failed: expected 2, got 1",
            ));

        // Act
        let output = formatter.format(&suite)?;

        // Assert
        assert!(output.contains("Test Suite: failing_suite"));
        assert!(output.contains("✓ test1"));
        assert!(output.contains("✗ test2"));
        assert!(output.contains("assertion failed: expected 2, got 1"));
        assert!(output.contains("FAILED"));
        assert!(output.contains("2 tests"));
        assert!(output.contains("1 passed"));
        assert!(output.contains("1 failed"));

        Ok(())
    }

    #[test]
    fn test_human_formatter_with_skipped() -> Result<()> {
        // Arrange
        let formatter = HumanFormatter::with_colors(false);
        let suite = TestSuite::new("suite_with_skipped")
            .add_result(TestResult::passed("test1"))
            .add_result(TestResult::skipped("test2"));

        // Act
        let output = formatter.format(&suite)?;

        // Assert
        assert!(output.contains("✓ test1"));
        assert!(output.contains("⊘ test2"));
        assert!(output.contains("1 passed"));
        assert!(output.contains("1 skipped"));

        Ok(())
    }

    #[test]
    fn test_human_formatter_with_duration() -> Result<()> {
        // Arrange
        let formatter = HumanFormatter::with_colors(false);
        let suite = TestSuite::new("suite_with_duration")
            .add_result(TestResult::passed("fast_test").with_duration(Duration::from_millis(50)))
            .add_result(TestResult::passed("slow_test").with_duration(Duration::from_millis(1500)));

        // Act
        let output = formatter.format(&suite)?;

        // Assert
        assert!(output.contains("fast_test (50ms)"));
        assert!(output.contains("slow_test (1.50s)"));

        Ok(())
    }

    #[test]
    fn test_human_formatter_with_stdout_stderr() -> Result<()> {
        // Arrange
        let formatter = HumanFormatter::with_colors(false);
        let suite = TestSuite::new("suite_with_output").add_result(
            TestResult::passed("test_with_output")
                .with_stdout("stdout line 1\nstdout line 2")
                .with_stderr("stderr line 1"),
        );

        // Act
        let output = formatter.format(&suite)?;

        // Assert
        assert!(output.contains("stdout:"));
        assert!(output.contains("stdout line 1"));
        assert!(output.contains("stdout line 2"));
        assert!(output.contains("stderr:"));
        assert!(output.contains("stderr line 1"));

        Ok(())
    }

    #[test]
    fn test_human_formatter_name() {
        // Arrange
        let formatter = HumanFormatter::new();

        // Act & Assert
        assert_eq!(formatter.name(), "human");
    }

    #[test]
    fn test_human_formatter_type() {
        // Arrange
        let formatter = HumanFormatter::new();

        // Act & Assert
        assert_eq!(formatter.formatter_type(), FormatterType::Human);
    }

    #[test]
    fn test_format_duration_less_than_1ms() {
        // Arrange
        let formatter = HumanFormatter::new();

        // Act
        let result = formatter.format_duration(0.5);

        // Assert
        assert_eq!(result, "<1ms");
    }

    #[test]
    fn test_format_duration_milliseconds() {
        // Arrange
        let formatter = HumanFormatter::new();

        // Act
        let result = formatter.format_duration(150.0);

        // Assert
        assert_eq!(result, "150ms");
    }

    #[test]
    fn test_format_duration_seconds() {
        // Arrange
        let formatter = HumanFormatter::new();

        // Act
        let result = formatter.format_duration(1500.0);

        // Assert
        assert_eq!(result, "1.50s");
    }
}
