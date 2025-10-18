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
