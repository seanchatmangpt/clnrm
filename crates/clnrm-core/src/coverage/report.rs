//! Coverage report generation in various formats

use crate::coverage::BehaviorCoverageReport;
use crate::error::{CleanroomError, Result};
use std::path::Path;

/// Report format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    /// Plain text output
    Text,
    /// JSON format
    Json,
    /// HTML format
    Html,
    /// Markdown format
    Markdown,
}

impl ReportFormat {
    /// Parse format from string (not implementing FromStr trait to keep simple API)
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "text" | "txt" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            "html" => Ok(Self::Html),
            "markdown" | "md" => Ok(Self::Markdown),
            _ => Err(CleanroomError::validation_error(format!(
                "Unknown report format: {}. Valid formats: text, json, html, markdown",
                s
            ))),
        }
    }
}

/// Coverage report generator
pub struct ReportGenerator;

impl ReportGenerator {
    /// Generate report in specified format
    pub fn generate(report: &BehaviorCoverageReport, format: ReportFormat) -> Result<String> {
        match format {
            ReportFormat::Text => Ok(report.format_text()),
            ReportFormat::Json => Self::generate_json(report),
            ReportFormat::Html => Self::generate_html(report),
            ReportFormat::Markdown => Self::generate_markdown(report),
        }
    }

    /// Generate JSON report
    fn generate_json(report: &BehaviorCoverageReport) -> Result<String> {
        serde_json::to_string_pretty(report).map_err(|e| {
            CleanroomError::validation_error(format!("Failed to serialize report to JSON: {}", e))
        })
    }

    /// Generate HTML report
    fn generate_html(report: &BehaviorCoverageReport) -> Result<String> {
        let mut html = String::new();

        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("  <meta charset=\"UTF-8\">\n");
        html.push_str(
            "  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
        );
        html.push_str("  <title>Behavior Coverage Report</title>\n");
        html.push_str("  <style>\n");
        html.push_str("    body { font-family: Arial, sans-serif; margin: 20px; }\n");
        html.push_str("    h1 { color: #333; }\n");
        html.push_str("    .summary { background: #f5f5f5; padding: 20px; border-radius: 8px; margin: 20px 0; }\n");
        html.push_str("    .grade { font-size: 48px; font-weight: bold; }\n");
        html.push_str("    .grade-a { color: #4caf50; }\n");
        html.push_str("    .grade-b { color: #8bc34a; }\n");
        html.push_str("    .grade-c { color: #ffc107; }\n");
        html.push_str("    .grade-d { color: #ff9800; }\n");
        html.push_str("    .grade-f { color: #f44336; }\n");
        html.push_str("    table { width: 100%; border-collapse: collapse; margin: 20px 0; }\n");
        html.push_str(
            "    th, td { padding: 12px; text-align: left; border-bottom: 1px solid #ddd; }\n",
        );
        html.push_str("    th { background-color: #4caf50; color: white; }\n");
        html.push_str("    .progress-bar { width: 100%; height: 20px; background: #f0f0f0; border-radius: 10px; overflow: hidden; }\n");
        html.push_str(
            "    .progress-fill { height: 100%; background: #4caf50; transition: width 0.3s; }\n",
        );
        html.push_str("    .uncovered { background: #fff3cd; padding: 10px; margin: 10px 0; border-left: 4px solid #ffc107; }\n");
        html.push_str("  </style>\n");
        html.push_str("</head>\n<body>\n");

        // Title and summary
        html.push_str("  <h1>Behavior Coverage Report</h1>\n");
        html.push_str("  <div class=\"summary\">\n");
        html.push_str(&format!(
            "    <div class=\"grade grade-{}\">{} {}</div>\n",
            report.grade().to_lowercase(),
            report.emoji(),
            report.grade()
        ));
        html.push_str(&format!(
            "    <p><strong>Overall Coverage:</strong> {:.1}%</p>\n",
            report.total_coverage
        ));
        html.push_str(&format!(
            "    <p><strong>Behaviors Covered:</strong> {} / {} ({:.1}%)</p>\n",
            report.covered_behaviors,
            report.total_behaviors,
            (report.covered_behaviors as f64 / report.total_behaviors.max(1) as f64) * 100.0
        ));
        html.push_str("    <div class=\"progress-bar\">\n");
        html.push_str(&format!(
            "      <div class=\"progress-fill\" style=\"width: {:.1}%\"></div>\n",
            report.total_coverage
        ));
        html.push_str("    </div>\n");
        html.push_str("  </div>\n");

        // Dimensions table
        html.push_str("  <h2>Dimension Breakdown</h2>\n");
        html.push_str("  <table>\n");
        html.push_str("    <tr><th>Dimension</th><th>Coverage</th><th>Weight</th><th>Score</th><th>Covered/Total</th></tr>\n");
        for dim in &report.dimensions {
            html.push_str(&format!(
                "    <tr><td>{}</td><td>{:.1}%</td><td>{:.0}%</td><td>{:.2}%</td><td>{}/{}</td></tr>\n",
                dim.name,
                dim.coverage * 100.0,
                dim.weight * 100.0,
                dim.weighted_score * 100.0,
                dim.covered,
                dim.total
            ));
        }
        html.push_str("  </table>\n");

        // Uncovered behaviors
        if !report.uncovered_behaviors.is_empty() {
            html.push_str("  <h2>Uncovered Behaviors</h2>\n");
            for behavior in report.uncovered_behaviors.top_priority(10) {
                html.push_str(&format!(
                    "  <div class=\"uncovered\"><strong>{}:</strong> {}</div>\n",
                    behavior.dimension, behavior.name
                ));
            }
        }

        html.push_str("</body>\n</html>");

        Ok(html)
    }

    /// Generate Markdown report
    fn generate_markdown(report: &BehaviorCoverageReport) -> Result<String> {
        let mut md = String::new();

        md.push_str("# Behavior Coverage Report\n\n");
        md.push_str(&format!(
            "**Overall Coverage:** {:.1}% {} (Grade: {})\n\n",
            report.total_coverage,
            report.emoji(),
            report.grade()
        ));

        md.push_str(&format!(
            "**Behaviors Covered:** {} / {}\n\n",
            report.covered_behaviors, report.total_behaviors
        ));

        // Dimensions table
        md.push_str("## Dimension Breakdown\n\n");
        md.push_str("| Dimension | Coverage | Weight | Score | Covered/Total |\n");
        md.push_str("|-----------|----------|--------|-------|---------------|\n");
        for dim in &report.dimensions {
            md.push_str(&format!(
                "| {} | {:.1}% | {:.0}% | {:.2}% | {}/{} |\n",
                dim.name,
                dim.coverage * 100.0,
                dim.weight * 100.0,
                dim.weighted_score * 100.0,
                dim.covered,
                dim.total
            ));
        }
        md.push('\n');

        // Uncovered behaviors
        if !report.uncovered_behaviors.is_empty() {
            md.push_str("## Uncovered Behaviors\n\n");
            for (i, behavior) in report
                .uncovered_behaviors
                .top_priority(10)
                .iter()
                .enumerate()
            {
                md.push_str(&format!(
                    "{}. **{}**: {}\n",
                    i + 1,
                    behavior.dimension,
                    behavior.name
                ));
            }
        }

        Ok(md)
    }

    /// Save report to file
    pub fn save(
        report: &BehaviorCoverageReport,
        path: impl AsRef<Path>,
        format: ReportFormat,
    ) -> Result<()> {
        let content = Self::generate(report, format)?;
        std::fs::write(path.as_ref(), content).map_err(|e| {
            CleanroomError::io_error(format!(
                "Failed to write report to {}: {}",
                path.as_ref().display(),
                e
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coverage::{DimensionCoverage, UncoveredBehaviors};

    fn sample_report() -> BehaviorCoverageReport {
        BehaviorCoverageReport {
            total_coverage: 75.5,
            dimensions: vec![
                DimensionCoverage::new("API Surface", 5, 10, 0.20),
                DimensionCoverage::new("State Transitions", 3, 5, 0.20),
            ],
            uncovered_behaviors: UncoveredBehaviors::new(),
            total_behaviors: 15,
            covered_behaviors: 8,
        }
    }

    #[test]
    fn test_format_from_str() -> Result<()> {
        assert_eq!(ReportFormat::from_str("text")?, ReportFormat::Text);
        assert_eq!(ReportFormat::from_str("json")?, ReportFormat::Json);
        assert_eq!(ReportFormat::from_str("html")?, ReportFormat::Html);
        assert_eq!(ReportFormat::from_str("markdown")?, ReportFormat::Markdown);
        Ok(())
    }

    #[test]
    fn test_format_from_str_invalid() {
        assert!(ReportFormat::from_str("invalid").is_err());
    }

    #[test]
    fn test_generate_text() -> Result<()> {
        let report = sample_report();
        let text = ReportGenerator::generate(&report, ReportFormat::Text)?;
        assert!(text.contains("Behavior Coverage Report"));
        assert!(text.contains("75.1%"));
        Ok(())
    }

    #[test]
    fn test_generate_json() -> Result<()> {
        let report = sample_report();
        let json = ReportGenerator::generate(&report, ReportFormat::Json)?;
        assert!(json.contains("total_coverage"));
        assert!(json.contains("75.5"));
        Ok(())
    }

    #[test]
    fn test_generate_html() -> Result<()> {
        let report = sample_report();
        let html = ReportGenerator::generate(&report, ReportFormat::Html)?;
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Behavior Coverage Report"));
        Ok(())
    }

    #[test]
    fn test_generate_markdown() -> Result<()> {
        let report = sample_report();
        let md = ReportGenerator::generate(&report, ReportFormat::Markdown)?;
        assert!(md.contains("# Behavior Coverage Report"));
        assert!(md.contains("| Dimension |"));
        Ok(())
    }
}
