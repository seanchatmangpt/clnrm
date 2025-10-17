//! JUnit XML Formatter
//!
//! Generates JUnit-compatible XML output for CI/CD integration.
//! Follows the JUnit XML schema specification.

use crate::error::Result;
use crate::formatting::formatter::{Formatter, FormatterType};
use crate::formatting::test_result::{TestStatus, TestSuite};

/// JUnit XML formatter for test results
#[derive(Debug, Default)]
pub struct JunitFormatter;

impl JunitFormatter {
    /// Create a new JUnit formatter
    pub fn new() -> Self {
        Self
    }

    /// Escape XML special characters
    fn escape_xml(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }

    /// Generate XML header
    fn generate_header() -> String {
        r#"<?xml version="1.0" encoding="UTF-8"?>"#.to_string()
    }

    /// Generate testsuite opening tag
    fn generate_testsuite_open(suite: &TestSuite) -> String {
        let mut output = format!(
            r#"<testsuite name="{}" tests="{}" failures="{}" skipped="{}" errors="0""#,
            Self::escape_xml(&suite.name),
            suite.total_count(),
            suite.failed_count(),
            suite.skipped_count()
        );

        if let Some(duration) = suite.duration {
            output.push_str(&format!(" time=\"{:.3}\"", duration.as_secs_f64()));
        }

        output.push('>');
        output
    }

    /// Generate testcase element
    fn generate_testcase(result: &crate::formatting::test_result::TestResult) -> String {
        let mut output = format!(
            r#"  <testcase name="{}" classname="{}""#,
            Self::escape_xml(&result.name),
            Self::escape_xml(&result.name)
        );

        if let Some(duration) = result.duration {
            output.push_str(&format!(" time=\"{:.3}\"", duration.as_secs_f64()));
        }

        match result.status {
            TestStatus::Passed => {
                output.push_str(" />");
            }
            TestStatus::Failed => {
                output.push_str(">\n");
                if let Some(error) = &result.error {
                    output.push_str(&format!(
                        r#"    <failure message="{}" />"#,
                        Self::escape_xml(error)
                    ));
                } else {
                    output.push_str(r#"    <failure message="Test failed" />"#);
                }
                output.push_str("\n  </testcase>");
            }
            TestStatus::Skipped => {
                output.push_str(">\n");
                output.push_str("    <skipped />");
                output.push_str("\n  </testcase>");
            }
            TestStatus::Unknown => {
                output.push_str(" />");
            }
        }

        output
    }

    /// Generate system-out element if needed
    fn generate_system_out(suite: &TestSuite) -> Option<String> {
        let stdout_outputs: Vec<String> = suite
            .results
            .iter()
            .filter_map(|r| r.stdout.as_ref())
            .map(|s| Self::escape_xml(s))
            .collect();

        if stdout_outputs.is_empty() {
            None
        } else {
            Some(format!(
                "  <system-out>\n{}\n  </system-out>",
                stdout_outputs.join("\n")
            ))
        }
    }

    /// Generate system-err element if needed
    fn generate_system_err(suite: &TestSuite) -> Option<String> {
        let stderr_outputs: Vec<String> = suite
            .results
            .iter()
            .filter_map(|r| r.stderr.as_ref())
            .map(|s| Self::escape_xml(s))
            .collect();

        if stderr_outputs.is_empty() {
            None
        } else {
            Some(format!(
                "  <system-err>\n{}\n  </system-err>",
                stderr_outputs.join("\n")
            ))
        }
    }
}

impl Formatter for JunitFormatter {
    fn format(&self, suite: &TestSuite) -> Result<String> {
        let mut output = Vec::new();

        // XML header
        output.push(Self::generate_header());

        // Testsuite opening tag
        output.push(Self::generate_testsuite_open(suite));

        // Test cases
        for result in &suite.results {
            output.push(Self::generate_testcase(result));
        }

        // System output if present
        if let Some(system_out) = Self::generate_system_out(suite) {
            output.push(system_out);
        }

        // System error if present
        if let Some(system_err) = Self::generate_system_err(suite) {
            output.push(system_err);
        }

        // Testsuite closing tag
        output.push("</testsuite>".to_string());

        Ok(output.join("\n"))
    }

    fn name(&self) -> &'static str {
        "junit"
    }

    fn formatter_type(&self) -> FormatterType {
        FormatterType::Junit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatting::test_result::TestResult;
    use std::time::Duration;

    #[test]
    fn test_junit_formatter_empty_suite() -> Result<()> {
        // Arrange
        let formatter = JunitFormatter::new();
        let suite = TestSuite::new("empty_suite");

        // Act
        let output = formatter.format(&suite)?;

        // Assert
        assert!(output.contains(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
        assert!(
            output.contains(r#"<testsuite name="empty_suite" tests="0" failures="0" skipped="0""#)
        );
        assert!(output.contains("</testsuite>"));

        Ok(())
    }

    #[test]
    fn test_junit_formatter_all_passed() -> Result<()> {
        // Arrange
        let formatter = JunitFormatter::new();
        let suite = TestSuite::new("passing_suite")
            .add_result(TestResult::passed("test1"))
            .add_result(TestResult::passed("test2"));

        // Act
        let output = formatter.format(&suite)?;

        // Assert
        assert!(output.contains(r#"tests="2" failures="0""#));
        assert!(output.contains(r#"<testcase name="test1""#));
        assert!(output.contains(r#"<testcase name="test2""#));
        assert!(!output.contains("<failure"));

        Ok(())
    }

    #[test]
    fn test_junit_formatter_with_failures() -> Result<()> {
        // Arrange
        let formatter = JunitFormatter::new();
        let suite = TestSuite::new("failing_suite")
            .add_result(TestResult::passed("test1"))
            .add_result(TestResult::failed("test2", "assertion failed"));

        // Act
        let output = formatter.format(&suite)?;

        // Assert
        assert!(output.contains(r#"tests="2" failures="1""#));
        assert!(output.contains(r#"<testcase name="test2""#));
        assert!(output.contains(r#"<failure message="assertion failed" />"#));

        Ok(())
    }

    #[test]
    fn test_junit_formatter_with_skipped() -> Result<()> {
        // Arrange
        let formatter = JunitFormatter::new();
        let suite = TestSuite::new("suite_with_skipped")
            .add_result(TestResult::passed("test1"))
            .add_result(TestResult::skipped("test2"));

        // Act
        let output = formatter.format(&suite)?;

        // Assert
        assert!(output.contains(r#"skipped="1""#));
        assert!(output.contains("<skipped />"));

        Ok(())
    }

    #[test]
    fn test_junit_formatter_with_duration() -> Result<()> {
        // Arrange
        let formatter = JunitFormatter::new();
        let suite = TestSuite::new("suite_with_duration")
            .add_result(TestResult::passed("test1").with_duration(Duration::from_millis(150)))
            .with_duration(Duration::from_secs(1));

        // Act
        let output = formatter.format(&suite)?;

        // Assert
        assert!(output.contains(r#"time="1.000">"#)); // Suite duration
        assert!(output.contains(r#"time="0.150""#)); // Test duration

        Ok(())
    }

    #[test]
    fn test_junit_formatter_xml_escaping() -> Result<()> {
        // Arrange
        let formatter = JunitFormatter::new();
        let suite = TestSuite::new("test_<suite>").add_result(TestResult::failed(
            "test_name",
            r#"Error: "value" & 'expected' < 10 > 5"#,
        ));

        // Act
        let output = formatter.format(&suite)?;

        // Assert
        assert!(output.contains("&lt;"));
        assert!(output.contains("&gt;"));
        assert!(output.contains("&amp;"));
        assert!(output.contains("&quot;"));
        assert!(output.contains("&apos;"));

        Ok(())
    }

    #[test]
    fn test_junit_formatter_with_stdout() -> Result<()> {
        // Arrange
        let formatter = JunitFormatter::new();
        let suite = TestSuite::new("suite_with_stdout")
            .add_result(TestResult::passed("test1").with_stdout("stdout output"));

        // Act
        let output = formatter.format(&suite)?;

        // Assert
        assert!(output.contains("<system-out>"));
        assert!(output.contains("stdout output"));
        assert!(output.contains("</system-out>"));

        Ok(())
    }

    #[test]
    fn test_junit_formatter_with_stderr() -> Result<()> {
        // Arrange
        let formatter = JunitFormatter::new();
        let suite = TestSuite::new("suite_with_stderr")
            .add_result(TestResult::passed("test1").with_stderr("stderr output"));

        // Act
        let output = formatter.format(&suite)?;

        // Assert
        assert!(output.contains("<system-err>"));
        assert!(output.contains("stderr output"));
        assert!(output.contains("</system-err>"));

        Ok(())
    }

    #[test]
    fn test_junit_formatter_name() {
        // Arrange
        let formatter = JunitFormatter::new();

        // Act & Assert
        assert_eq!(formatter.name(), "junit");
    }

    #[test]
    fn test_junit_formatter_type() {
        // Arrange
        let formatter = JunitFormatter::new();

        // Act & Assert
        assert_eq!(formatter.formatter_type(), FormatterType::Junit);
    }

    #[test]
    fn test_escape_xml() {
        // Arrange
        let input = r#"<test>"value"&'data'"#;

        // Act
        let escaped = JunitFormatter::escape_xml(input);

        // Assert
        assert_eq!(
            escaped,
            "&lt;test&gt;&quot;value&quot;&amp;&apos;data&apos;"
        );
    }
}
