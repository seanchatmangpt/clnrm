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
