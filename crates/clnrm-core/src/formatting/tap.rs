//! TAP (Test Anything Protocol) Formatter
//!
//! Generates TAP version 13 compatible output.
//! Widely used in Perl and other testing ecosystems.

use crate::error::Result;
use crate::formatting::formatter::{Formatter, FormatterType};
use crate::formatting::test_result::{TestStatus, TestSuite};

/// TAP formatter for test results
#[derive(Debug, Default)]
pub struct TapFormatter;

impl TapFormatter {
    /// Create a new TAP formatter
    pub fn new() -> Self {
        Self
    }

    /// Generate TAP version header
    fn generate_header() -> String {
        "TAP version 13".to_string()
    }

    /// Generate TAP plan line
    fn generate_plan(total: usize) -> String {
        format!("1..{}", total)
    }

    /// Generate TAP test line
    fn generate_test_line(
        index: usize,
        result: &crate::formatting::test_result::TestResult,
    ) -> Vec<String> {
        let mut output = Vec::new();

        let status = match result.status {
            TestStatus::Passed => "ok",
            TestStatus::Failed => "not ok",
            TestStatus::Skipped => "ok",
            TestStatus::Unknown => "not ok",
        };

        let mut line = format!("{} {} - {}", status, index, result.name);

        // Add skip directive for skipped tests
        if result.status == TestStatus::Skipped {
            line.push_str(" # SKIP");
        }

        output.push(line);

        // Add diagnostic lines for failures
        if result.status == TestStatus::Failed {
            if let Some(error) = &result.error {
                output.push("  ---".to_string());
                output.push(format!("  message: {}", Self::escape_yaml_string(error)));
                output.push("  ...".to_string());
            }
        }

        // Add duration if present
        if let Some(duration) = result.duration {
            output.push(format!("  # Duration: {:.3}s", duration.as_secs_f64()));
        }

        // Add stdout/stderr as diagnostics
        if let Some(stdout) = &result.stdout {
            output.push("  # stdout:".to_string());
            for line in stdout.lines() {
                output.push(format!("  # {}", line));
            }
        }

        if let Some(stderr) = &result.stderr {
            output.push("  # stderr:".to_string());
            for line in stderr.lines() {
                output.push(format!("  # {}", line));
            }
        }

        output
    }

    /// Escape YAML string for TAP diagnostics
    fn escape_yaml_string(s: &str) -> String {
        // Simple escaping for YAML values in TAP
        if s.contains('\n') || s.contains('#') || s.contains(':') {
            format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
        } else {
            s.to_string()
        }
    }
}

impl Formatter for TapFormatter {
    fn format(&self, suite: &TestSuite) -> Result<String> {
        let mut output = Vec::new();

        // TAP version header
        output.push(Self::generate_header());

        // TAP plan
        output.push(Self::generate_plan(suite.total_count()));

        // Test lines
        for (index, result) in suite.results.iter().enumerate() {
            let test_lines = Self::generate_test_line(index + 1, result);
            output.extend(test_lines);
        }

        // Summary comment
        output.push(format!(
            "# tests {}, passed {}, failed {}, skipped {}",
            suite.total_count(),
            suite.passed_count(),
            suite.failed_count(),
            suite.skipped_count()
        ));

        if let Some(duration) = suite.duration {
            output.push(format!("# duration: {:.3}s", duration.as_secs_f64()));
        }

        Ok(output.join("\n"))
    }

    fn name(&self) -> &'static str {
        "tap"
    }

    fn formatter_type(&self) -> FormatterType {
        FormatterType::Tap
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatting::test_result::TestResult;
    use std::time::Duration;

    #[test]
    fn test_tap_formatter_empty_suite() -> Result<()> {
        // Arrange
        let formatter = TapFormatter::new();
        let suite = TestSuite::new("empty_suite");

        // Act
        let output = formatter.format(&suite)?;

        // Assert
        assert!(output.contains("TAP version 13"));
        assert!(output.contains("1..0"));
        assert!(output.contains("# tests 0"));

        Ok(())
    }

    #[test]
    fn test_tap_formatter_all_passed() -> Result<()> {
        // Arrange
        let formatter = TapFormatter::new();
        let suite = TestSuite::new("passing_suite")
            .add_result(TestResult::passed("test1"))
            .add_result(TestResult::passed("test2"));

        // Act
        let output = formatter.format(&suite)?;

        // Assert
        assert!(output.contains("TAP version 13"));
        assert!(output.contains("1..2"));
        assert!(output.contains("ok 1 - test1"));
        assert!(output.contains("ok 2 - test2"));
        assert!(output.contains("# tests 2, passed 2, failed 0"));

        Ok(())
    }

    #[test]
    fn test_tap_formatter_with_failures() -> Result<()> {
        // Arrange
        let formatter = TapFormatter::new();
        let suite = TestSuite::new("failing_suite")
            .add_result(TestResult::passed("test1"))
            .add_result(TestResult::failed(
                "test2",
                "assertion failed: expected 2, got 1",
            ));

        // Act
        let output = formatter.format(&suite)?;

        // Assert - TAP format uses YAML diagnostics
        assert!(output.contains("ok 1 - test1"));
        assert!(output.contains("not ok 2 - test2"));
        assert!(output.contains("  ---"));
        // The message might be quoted due to the colon
        assert!(output.contains("message:") && output.contains("assertion failed"));
        assert!(output.contains("  ..."));
        assert!(output.contains("# tests 2, passed 1, failed 1"));

        Ok(())
    }

    #[test]
    fn test_tap_formatter_with_skipped() -> Result<()> {
        // Arrange
        let formatter = TapFormatter::new();
        let suite = TestSuite::new("suite_with_skipped")
            .add_result(TestResult::passed("test1"))
            .add_result(TestResult::skipped("test2"));

        // Act
        let output = formatter.format(&suite)?;

        // Assert
        assert!(output.contains("ok 1 - test1"));
        assert!(output.contains("ok 2 - test2 # SKIP"));
        assert!(output.contains("# tests 2, passed 1, failed 0, skipped 1"));

        Ok(())
    }

    #[test]
    fn test_tap_formatter_with_duration() -> Result<()> {
        // Arrange
        let formatter = TapFormatter::new();
        let suite = TestSuite::new("suite_with_duration")
            .add_result(TestResult::passed("test1").with_duration(Duration::from_millis(150)))
            .with_duration(Duration::from_secs(1));

        // Act
        let output = formatter.format(&suite)?;

        // Assert
        assert!(output.contains("# Duration: 0.150s"));
        assert!(output.contains("# duration: 1.000s"));

        Ok(())
    }

    #[test]
    fn test_tap_formatter_with_stdout_stderr() -> Result<()> {
        // Arrange
        let formatter = TapFormatter::new();
        let suite = TestSuite::new("suite_with_output").add_result(
            TestResult::passed("test1")
                .with_stdout("stdout line 1\nstdout line 2")
                .with_stderr("stderr line 1"),
        );

        // Act
        let output = formatter.format(&suite)?;

        // Assert
        assert!(output.contains("# stdout:"));
        assert!(output.contains("  # stdout line 1"));
        assert!(output.contains("  # stdout line 2"));
        assert!(output.contains("# stderr:"));
        assert!(output.contains("  # stderr line 1"));

        Ok(())
    }

    #[test]
    fn test_tap_formatter_name() {
        // Arrange
        let formatter = TapFormatter::new();

        // Act & Assert
        assert_eq!(formatter.name(), "tap");
    }

    #[test]
    fn test_tap_formatter_type() {
        // Arrange
        let formatter = TapFormatter::new();

        // Act & Assert
        assert_eq!(formatter.formatter_type(), FormatterType::Tap);
    }

    #[test]
    fn test_escape_yaml_string_simple() {
        // Arrange
        let input = "simple string";

        // Act
        let escaped = TapFormatter::escape_yaml_string(input);

        // Assert
        assert_eq!(escaped, "simple string");
    }

    #[test]
    fn test_escape_yaml_string_with_special_chars() {
        // Arrange
        let input = "string: with # special chars";

        // Act
        let escaped = TapFormatter::escape_yaml_string(input);

        // Assert
        assert!(escaped.starts_with('"'));
        assert!(escaped.ends_with('"'));
    }

    #[test]
    fn test_escape_yaml_string_with_newlines() {
        // Arrange
        let input = "line 1\nline 2";

        // Act
        let escaped = TapFormatter::escape_yaml_string(input);

        // Assert
        assert!(escaped.starts_with('"'));
        assert!(escaped.ends_with('"'));
    }
}
