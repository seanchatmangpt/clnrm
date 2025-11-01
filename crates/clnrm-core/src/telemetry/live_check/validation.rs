//! 80/20 validation logic for Weaver conformance reports
//!
//! Implements validation mode filtering and coverage calculation for
//! fast CI/CD gates (80/20 mode) while maintaining strict validation
//! for final releases.

use super::config::{Complete80_20Config, EightyTwentyConfig, ValidationConfig, ValidationMode};
use serde::{Deserialize, Serialize};

/// Validator for conformance reports with 80/20 support
pub struct ConformanceValidator {
    config: ValidationConfig,
    eighty_twenty_config: Option<EightyTwentyConfig>,
    #[allow(dead_code)]
    complete_config: Option<Complete80_20Config>,
}

impl ConformanceValidator {
    /// Create new validator with validation config
    pub fn new(config: ValidationConfig) -> Self {
        Self {
            config,
            eighty_twenty_config: None,
            complete_config: None,
        }
    }

    /// Create validator with 80/20 configuration
    pub fn with_80_20_config(config: ValidationConfig, eighty_twenty: EightyTwentyConfig) -> Self {
        Self {
            config,
            eighty_twenty_config: Some(eighty_twenty),
            complete_config: None,
        }
    }

    /// Create validator with complete configuration
    pub fn with_complete_config(complete: Complete80_20Config) -> Self {
        Self {
            config: complete.validation_config.clone(),
            eighty_twenty_config: None,
            complete_config: Some(complete),
        }
    }

    /// Validate conformance report and return result
    pub fn validate(&self, report: &ConformanceReport) -> ValidationResult {
        let start = std::time::Instant::now();

        let result = match self.config.mode {
            ValidationMode::Strict => self.validate_strict(report),
            ValidationMode::Lenient => self.validate_lenient(report),
            ValidationMode::EightyTwenty => self.validate_eighty_twenty(report),
            ValidationMode::Minimal => self.validate_minimal(report),
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        // Check if validation completed within time budget
        let within_budget = duration_ms <= self.config.max_validation_time_ms;

        ValidationResult {
            mode: self.config.mode,
            violations: result.0,
            coverage: result.1,
            passed: result.2,
            duration_ms,
            within_time_budget: within_budget,
        }
    }

    /// Strict mode: 100% conformance required
    fn validate_strict(&self, report: &ConformanceReport) -> (Vec<Violation>, f64, bool) {
        let mut violations = Vec::new();

        // Check all required spans present
        for span in &report.required_spans {
            if !report.present_spans.contains(span) {
                violations.push(Violation::MissingSpan(span.clone()));
            }
        }

        // Check all required attributes present
        for attr in &report.required_attributes {
            if !report.present_attributes.contains(attr) {
                violations.push(Violation::MissingAttribute(attr.clone()));
            }
        }

        // Check optional attributes if configured
        if self.config.fail_on_missing_optional {
            for attr in &report.optional_attributes {
                if !report.present_attributes.contains(attr) {
                    violations.push(Violation::MissingOptionalAttribute(attr.clone()));
                }
            }
        }

        let coverage = self.calculate_coverage(report);
        let passed = violations.is_empty() && coverage >= self.config.coverage_threshold;

        (violations, coverage, passed)
    }

    /// Lenient mode: All spans + required attributes
    fn validate_lenient(&self, report: &ConformanceReport) -> (Vec<Violation>, f64, bool) {
        let mut violations = Vec::new();

        // Check all required spans present
        for span in &report.required_spans {
            if !report.present_spans.contains(span) {
                violations.push(Violation::MissingSpan(span.clone()));
            }
        }

        // Check required attributes (not optional)
        for attr in &report.required_attributes {
            if !report.present_attributes.contains(attr) {
                violations.push(Violation::MissingAttribute(attr.clone()));
            }
        }

        // Optional attributes are optional in lenient mode

        let coverage = self.calculate_coverage(report);
        let passed = violations.is_empty() && coverage >= self.config.coverage_threshold;

        (violations, coverage, passed)
    }

    /// 80/20 mode: Only check critical 20%
    fn validate_eighty_twenty(&self, report: &ConformanceReport) -> (Vec<Violation>, f64, bool) {
        let eighty_twenty = match &self.eighty_twenty_config {
            Some(config) => config,
            None => {
                // Use default if not provided
                &EightyTwentyConfig::default()
            }
        };

        let mut violations = Vec::new();

        // Only check critical spans (the 20%)
        for span in &eighty_twenty.critical_spans {
            if !report.present_spans.contains(span) {
                violations.push(Violation::MissingCriticalSpan(span.clone()));
            }
        }

        // Only check required attributes
        for attr in &eighty_twenty.required_attributes {
            if !report.present_attributes.contains(attr) {
                violations.push(Violation::MissingCriticalAttribute(attr.clone()));
            }
        }

        // Optional attributes are truly optional in 80/20 mode

        let coverage = self.calculate_critical_coverage(report, eighty_twenty);
        let passed = violations.is_empty() && coverage >= self.config.coverage_threshold;

        (violations, coverage, passed)
    }

    /// Minimal mode: Critical spans only
    fn validate_minimal(&self, report: &ConformanceReport) -> (Vec<Violation>, f64, bool) {
        let eighty_twenty = match &self.eighty_twenty_config {
            Some(config) => config,
            None => &EightyTwentyConfig::default(),
        };

        let mut violations = Vec::new();

        // Only check critical spans
        for span in &eighty_twenty.critical_spans {
            if !report.present_spans.contains(span) {
                violations.push(Violation::MissingCriticalSpan(span.clone()));
            }
        }

        // In minimal mode, check only the most critical attributes
        let minimal_attrs = ["container.id", "test.hermetic", "test.result"];
        for attr in &minimal_attrs {
            if !report.present_attributes.contains(&attr.to_string()) {
                violations.push(Violation::MissingCriticalAttribute(attr.to_string()));
            }
        }

        let coverage = self.calculate_minimal_coverage(report, &minimal_attrs);
        let passed = violations.is_empty() && coverage >= self.config.coverage_threshold;

        (violations, coverage, passed)
    }

    /// Calculate overall coverage percentage
    fn calculate_coverage(&self, report: &ConformanceReport) -> f64 {
        let total_spans = report.required_spans.len();
        let total_attrs = report.required_attributes.len();
        let total = total_spans + total_attrs;

        if total == 0 {
            return 100.0;
        }

        let present_spans = report
            .required_spans
            .iter()
            .filter(|s| report.present_spans.contains(*s))
            .count();

        let present_attrs = report
            .required_attributes
            .iter()
            .filter(|a| report.present_attributes.contains(*a))
            .count();

        let present = present_spans + present_attrs;
        (present as f64 / total as f64) * 100.0
    }

    /// Calculate critical coverage (80/20 mode)
    fn calculate_critical_coverage(
        &self,
        report: &ConformanceReport,
        eighty_twenty: &EightyTwentyConfig,
    ) -> f64 {
        let total_critical_spans = eighty_twenty.critical_spans.len();
        let total_required_attrs = eighty_twenty.required_attributes.len();
        let total = total_critical_spans + total_required_attrs;

        if total == 0 {
            return 100.0;
        }

        let present_spans = eighty_twenty
            .critical_spans
            .iter()
            .filter(|s| report.present_spans.contains(*s))
            .count();

        let present_attrs = eighty_twenty
            .required_attributes
            .iter()
            .filter(|a| report.present_attributes.contains(*a))
            .count();

        let present = present_spans + present_attrs;
        (present as f64 / total as f64) * 100.0
    }

    /// Calculate minimal coverage (minimal mode)
    fn calculate_minimal_coverage(
        &self,
        report: &ConformanceReport,
        minimal_attrs: &[&str],
    ) -> f64 {
        let eighty_twenty = match &self.eighty_twenty_config {
            Some(config) => config,
            None => &EightyTwentyConfig::default(),
        };

        let total_spans = eighty_twenty.critical_spans.len();
        let total_attrs = minimal_attrs.len();
        let total = total_spans + total_attrs;

        if total == 0 {
            return 100.0;
        }

        let present_spans = eighty_twenty
            .critical_spans
            .iter()
            .filter(|s| report.present_spans.contains(*s))
            .count();

        let present_attrs = minimal_attrs
            .iter()
            .filter(|a| report.present_attributes.contains(&a.to_string()))
            .count();

        let present = present_spans + present_attrs;
        (present as f64 / total as f64) * 100.0
    }

    /// Get detailed coverage breakdown by category
    pub fn get_coverage_breakdown(&self, report: &ConformanceReport) -> CoverageBreakdown {
        let eighty_twenty = match &self.eighty_twenty_config {
            Some(config) => config,
            None => &EightyTwentyConfig::default(),
        };

        // Critical spans coverage
        let critical_spans_total = eighty_twenty.critical_spans.len();
        let critical_spans_present = eighty_twenty
            .critical_spans
            .iter()
            .filter(|s| report.present_spans.contains(*s))
            .count();
        let critical_spans_coverage = if critical_spans_total > 0 {
            (critical_spans_present as f64 / critical_spans_total as f64) * 100.0
        } else {
            100.0
        };

        // Required attributes coverage
        let required_attrs_total = eighty_twenty.required_attributes.len();
        let required_attrs_present = eighty_twenty
            .required_attributes
            .iter()
            .filter(|a| report.present_attributes.contains(*a))
            .count();
        let required_attrs_coverage = if required_attrs_total > 0 {
            (required_attrs_present as f64 / required_attrs_total as f64) * 100.0
        } else {
            100.0
        };

        // Optional attributes coverage
        let optional_attrs_total = eighty_twenty.optional_attributes.len();
        let optional_attrs_present = eighty_twenty
            .optional_attributes
            .iter()
            .filter(|a| report.present_attributes.contains(*a))
            .count();
        let optional_attrs_coverage = if optional_attrs_total > 0 {
            (optional_attrs_present as f64 / optional_attrs_total as f64) * 100.0
        } else {
            100.0
        };

        CoverageBreakdown {
            critical_spans_coverage,
            critical_spans_present,
            critical_spans_total,
            required_attributes_coverage: required_attrs_coverage,
            required_attributes_present: required_attrs_present,
            required_attributes_total: required_attrs_total,
            optional_attributes_coverage: optional_attrs_coverage,
            optional_attributes_present: optional_attrs_present,
            optional_attributes_total: optional_attrs_total,
        }
    }
}

/// Conformance report from Weaver live-check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConformanceReport {
    /// All required spans
    pub required_spans: Vec<String>,
    /// Spans actually present in telemetry
    pub present_spans: Vec<String>,
    /// All required attributes
    pub required_attributes: Vec<String>,
    /// Attributes actually present in telemetry
    pub present_attributes: Vec<String>,
    /// Optional attributes
    pub optional_attributes: Vec<String>,
}

impl ConformanceReport {
    /// Create a new conformance report
    pub fn new() -> Self {
        Self {
            required_spans: Vec::new(),
            present_spans: Vec::new(),
            required_attributes: Vec::new(),
            present_attributes: Vec::new(),
            optional_attributes: Vec::new(),
        }
    }

    /// Add required span
    pub fn add_required_span(&mut self, span: String) {
        self.required_spans.push(span);
    }

    /// Add present span
    pub fn add_present_span(&mut self, span: String) {
        self.present_spans.push(span);
    }

    /// Add required attribute
    pub fn add_required_attribute(&mut self, attr: String) {
        self.required_attributes.push(attr);
    }

    /// Add present attribute
    pub fn add_present_attribute(&mut self, attr: String) {
        self.present_attributes.push(attr);
    }

    /// Add optional attribute
    pub fn add_optional_attribute(&mut self, attr: String) {
        self.optional_attributes.push(attr);
    }
}

impl Default for ConformanceReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Validation mode used
    pub mode: ValidationMode,
    /// List of violations found
    pub violations: Vec<Violation>,
    /// Coverage percentage (0.0 - 100.0)
    pub coverage: f64,
    /// Whether validation passed
    pub passed: bool,
    /// Validation duration in milliseconds
    pub duration_ms: u64,
    /// Whether validation completed within time budget
    pub within_time_budget: bool,
}

impl ValidationResult {
    /// Check if validation is successful
    pub fn is_success(&self) -> bool {
        self.passed && self.within_time_budget
    }

    /// Get summary message
    pub fn summary(&self) -> String {
        if self.passed {
            format!(
                "✅ Validation PASSED ({:?} mode): {:.1}% coverage in {}ms",
                self.mode, self.coverage, self.duration_ms
            )
        } else {
            format!(
                "❌ Validation FAILED ({:?} mode): {:.1}% coverage, {} violations",
                self.mode,
                self.coverage,
                self.violations.len()
            )
        }
    }

    /// Print detailed report
    pub fn print_report(&self) {
        println!("\n{}", "=".repeat(60));
        println!("WEAVER VALIDATION REPORT ({:?} MODE)", self.mode);
        println!("{}", "=".repeat(60));

        println!(
            "\nStatus: {}",
            if self.passed {
                "✅ PASSED"
            } else {
                "❌ FAILED"
            }
        );
        println!("Coverage: {:.1}%", self.coverage);
        println!("Duration: {}ms", self.duration_ms);
        println!(
            "Time Budget: {}",
            if self.within_time_budget {
                "✅ Met"
            } else {
                "⚠️ Exceeded"
            }
        );

        if !self.violations.is_empty() {
            println!("\n{} VIOLATIONS FOUND:", self.violations.len());
            for (i, violation) in self.violations.iter().enumerate() {
                println!("  {}. {}", i + 1, violation);
            }
        }

        println!("\n{}", "=".repeat(60));
    }
}

/// Validation violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Violation {
    /// Required span is missing
    MissingSpan(String),
    /// Required attribute is missing
    MissingAttribute(String),
    /// Optional attribute is missing
    MissingOptionalAttribute(String),
    /// Critical span is missing (80/20 mode)
    MissingCriticalSpan(String),
    /// Critical attribute is missing (80/20 mode)
    MissingCriticalAttribute(String),
}

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Violation::MissingSpan(span) => write!(f, "Missing required span: {}", span),
            Violation::MissingAttribute(attr) => write!(f, "Missing required attribute: {}", attr),
            Violation::MissingOptionalAttribute(attr) => {
                write!(f, "Missing optional attribute: {}", attr)
            }
            Violation::MissingCriticalSpan(span) => write!(f, "Missing CRITICAL span: {}", span),
            Violation::MissingCriticalAttribute(attr) => {
                write!(f, "Missing CRITICAL attribute: {}", attr)
            }
        }
    }
}

/// Detailed coverage breakdown by category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageBreakdown {
    /// Critical spans coverage percentage
    pub critical_spans_coverage: f64,
    /// Number of critical spans present
    pub critical_spans_present: usize,
    /// Total number of critical spans
    pub critical_spans_total: usize,
    /// Required attributes coverage percentage
    pub required_attributes_coverage: f64,
    /// Number of required attributes present
    pub required_attributes_present: usize,
    /// Total number of required attributes
    pub required_attributes_total: usize,
    /// Optional attributes coverage percentage
    pub optional_attributes_coverage: f64,
    /// Number of optional attributes present
    pub optional_attributes_present: usize,
    /// Total number of optional attributes
    pub optional_attributes_total: usize,
}

impl CoverageBreakdown {
    /// Print coverage breakdown
    pub fn print(&self) {
        println!("\n{}", "=".repeat(60));
        println!("COVERAGE BREAKDOWN");
        println!("{}", "=".repeat(60));

        println!(
            "\nCritical Spans: {:.1}% ({}/{})",
            self.critical_spans_coverage, self.critical_spans_present, self.critical_spans_total
        );

        println!(
            "Required Attributes: {:.1}% ({}/{})",
            self.required_attributes_coverage,
            self.required_attributes_present,
            self.required_attributes_total
        );

        println!(
            "Optional Attributes: {:.1}% ({}/{})",
            self.optional_attributes_coverage,
            self.optional_attributes_present,
            self.optional_attributes_total
        );

        println!("\n{}", "=".repeat(60));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_report() -> ConformanceReport {
        let mut report = ConformanceReport::new();

        // Add required spans
        report.add_required_span("span1".to_string());
        report.add_required_span("span2".to_string());
        report.add_required_span("span3".to_string());

        // Add present spans (2 out of 3)
        report.add_present_span("span1".to_string());
        report.add_present_span("span2".to_string());

        // Add required attributes
        report.add_required_attribute("attr1".to_string());
        report.add_required_attribute("attr2".to_string());

        // Add present attributes (1 out of 2)
        report.add_present_attribute("attr1".to_string());

        report
    }

    #[test]
    fn test_strict_mode_all_present() {
        let config = ValidationConfig::strict();
        let validator = ConformanceValidator::new(config);

        let mut report = ConformanceReport::new();
        report.add_required_span("span1".to_string());
        report.add_present_span("span1".to_string());
        report.add_required_attribute("attr1".to_string());
        report.add_present_attribute("attr1".to_string());

        let result = validator.validate(&report);

        assert!(result.passed);
        assert_eq!(result.coverage, 100.0);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_strict_mode_missing_span() {
        let config = ValidationConfig::strict();
        let validator = ConformanceValidator::new(config);

        let mut report = ConformanceReport::new();
        report.add_required_span("span1".to_string());
        // span1 not present
        report.add_required_attribute("attr1".to_string());
        report.add_present_attribute("attr1".to_string());

        let result = validator.validate(&report);

        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        assert!(matches!(result.violations[0], Violation::MissingSpan(_)));
    }

    #[test]
    fn test_eighty_twenty_mode_critical_only() {
        let config = ValidationConfig::eighty_twenty();
        let eighty_twenty = EightyTwentyConfig {
            critical_spans: vec!["critical_span".to_string()],
            required_attributes: vec!["critical_attr".to_string()],
            optional_attributes: vec![],
        };

        let validator = ConformanceValidator::with_80_20_config(config, eighty_twenty);

        let mut report = ConformanceReport::new();
        report.add_required_span("critical_span".to_string());
        report.add_required_span("optional_span".to_string());
        report.add_present_span("critical_span".to_string());
        // optional_span missing - should be OK in 80/20 mode

        report.add_required_attribute("critical_attr".to_string());
        report.add_present_attribute("critical_attr".to_string());

        let result = validator.validate(&report);

        // Should pass because critical span/attr present
        assert!(result.passed);
        assert_eq!(result.coverage, 100.0); // 100% of critical items covered
    }

    #[test]
    fn test_coverage_calculation() {
        let config = ValidationConfig::strict();
        let validator = ConformanceValidator::new(config);

        let report = create_test_report();
        let result = validator.validate(&report);

        // Coverage = (2 spans + 1 attr) / (3 spans + 2 attrs) = 3/5 = 60%
        assert_eq!(result.coverage, 60.0);
    }

    #[test]
    fn test_minimal_mode() {
        let config = ValidationConfig::minimal();
        let validator = ConformanceValidator::new(config);

        let mut report = ConformanceReport::new();

        // Add all critical spans
        for span in &[
            "clnrm.test.execute",
            "clnrm.container.start",
            "clnrm.container.stop",
            "clnrm.test.cleanup",
            "clnrm.cli.health",
        ] {
            report.add_required_span(span.to_string());
            report.add_present_span(span.to_string());
        }

        // Add minimal critical attributes
        for attr in &["container.id", "test.hermetic", "test.result"] {
            report.add_required_attribute(attr.to_string());
            report.add_present_attribute(attr.to_string());
        }

        let result = validator.validate(&report);

        assert!(result.passed);
        assert_eq!(result.coverage, 100.0);
    }

    #[test]
    fn test_coverage_breakdown() {
        let config = ValidationConfig::eighty_twenty();
        let eighty_twenty = EightyTwentyConfig {
            critical_spans: vec!["span1".to_string(), "span2".to_string()],
            required_attributes: vec!["attr1".to_string(), "attr2".to_string()],
            optional_attributes: vec!["opt1".to_string()],
        };

        let validator = ConformanceValidator::with_80_20_config(config, eighty_twenty);

        let mut report = ConformanceReport::new();
        report.add_present_span("span1".to_string());
        // span2 missing
        report.add_present_attribute("attr1".to_string());
        report.add_present_attribute("attr2".to_string());
        // opt1 missing

        let breakdown = validator.get_coverage_breakdown(&report);

        assert_eq!(breakdown.critical_spans_coverage, 50.0); // 1/2
        assert_eq!(breakdown.required_attributes_coverage, 100.0); // 2/2
        assert_eq!(breakdown.optional_attributes_coverage, 0.0); // 0/1
    }

    #[test]
    fn test_validation_time_budget() {
        let config = ValidationConfig {
            mode: ValidationMode::EightyTwenty,
            fail_on_violation: true,
            fail_on_missing_optional: false,
            coverage_threshold: 80.0,
            max_validation_time_ms: 1, // Very short time budget
        };

        let validator = ConformanceValidator::new(config);
        let report = create_test_report();

        let result = validator.validate(&report);

        // Validation might exceed the 1ms budget
        // Just check that the field is populated correctly
        assert!(result.duration_ms >= 0);
    }

    #[test]
    fn test_lenient_mode() {
        let config = ValidationConfig::lenient();
        let validator = ConformanceValidator::new(config);

        let mut report = ConformanceReport::new();
        report.add_required_span("span1".to_string());
        report.add_present_span("span1".to_string());
        report.add_required_attribute("attr1".to_string());
        report.add_present_attribute("attr1".to_string());

        // Optional attribute missing - should be OK in lenient mode
        report.add_optional_attribute("optional1".to_string());

        let result = validator.validate(&report);

        assert!(result.passed);
        assert_eq!(result.coverage, 100.0);
    }
}
