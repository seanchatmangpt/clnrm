//! JUnit XML report format
//!
//! Generates JUnit-compatible XML reports for CI/CD integration.

use crate::error::{CleanroomError, Result};
use crate::validation::ValidationReport;
use std::path::Path;

/// JUnit XML report generator
pub struct JunitReporter;

impl JunitReporter {
    /// Write JUnit XML report to file
    ///
    /// # Arguments
    /// * `path` - File path for XML output
    /// * `report` - Validation report to convert
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Errors
    /// Returns error if file write fails
    pub fn write(path: &Path, report: &ValidationReport) -> Result<()> {
        let xml = Self::generate_xml(report);
        Self::write_file(path, &xml)
    }

    /// Generate complete JUnit XML document
    fn generate_xml(report: &ValidationReport) -> String {
        let mut xml = String::new();

        Self::append_xml_header(&mut xml);
        Self::append_testsuite_open(&mut xml, report);
        Self::append_passed_tests(&mut xml, report);
        Self::append_failed_tests(&mut xml, report);
        Self::append_testsuite_close(&mut xml);

        xml
    }

    /// Append XML header
    fn append_xml_header(xml: &mut String) {
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
    }

    /// Append testsuite opening tag
    fn append_testsuite_open(xml: &mut String, report: &ValidationReport) {
        let total = report.passes().len() + report.failures().len();
        xml.push_str(&format!(
            r#"<testsuite name="clnrm" tests="{}" failures="{}" errors="0">"#,
            total,
            report.failures().len()
        ));
        xml.push('\n');
    }

    /// Append passed test cases
    fn append_passed_tests(xml: &mut String, report: &ValidationReport) {
        for pass_name in report.passes() {
            xml.push_str(&format!(
                r#"  <testcase name="{}" />"#,
                Self::escape_xml(pass_name)
            ));
            xml.push('\n');
        }
    }

    /// Append failed test cases
    fn append_failed_tests(xml: &mut String, report: &ValidationReport) {
        for (fail_name, error) in report.failures() {
            xml.push_str(&format!(
                r#"  <testcase name="{}">"#,
                Self::escape_xml(fail_name)
            ));
            xml.push('\n');
            xml.push_str(&format!(
                r#"    <failure message="{}" />"#,
                Self::escape_xml(error)
            ));
            xml.push('\n');
            xml.push_str(r#"  </testcase>"#);
            xml.push('\n');
        }
    }

    /// Append testsuite closing tag
    fn append_testsuite_close(xml: &mut String) {
        xml.push_str("</testsuite>\n");
    }

    /// Escape XML special characters
    fn escape_xml(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }

    /// Write XML string to file
    fn write_file(path: &Path, content: &str) -> Result<()> {
        std::fs::write(path, content)
            .map_err(|e| CleanroomError::report_error(format!("Failed to write JUnit XML: {}", e)))
    }
}
