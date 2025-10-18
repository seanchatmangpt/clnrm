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
