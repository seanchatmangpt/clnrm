//! Configuration types for Weaver live-check validation modes
//!
//! Provides comprehensive configuration for 80/20 validation, allowing
//! fast CI/CD gates (80% coverage in 5s) while maintaining strict mode
//! for final releases (100% coverage in 30s).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Validation mode for Weaver live-check
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ValidationMode {
    /// Local development - critical spans only (60% coverage, <2s)
    Minimal,
    /// CI/CD gate - critical spans + required attributes (80% coverage, <5s)
    #[serde(rename = "80_20")]
    #[default]
    EightyTwenty,
    /// QA/staging - all spans + required attributes (90% coverage, <15s)
    Lenient,
    /// Final release - all spans + all attributes (100% coverage, <30s)
    Strict,
}

/// Validation configuration for live-check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Validation mode
    pub mode: ValidationMode,

    /// Whether to fail on violations
    pub fail_on_violation: bool,

    /// Whether to fail on missing optional attributes
    pub fail_on_missing_optional: bool,

    /// Target coverage percentage (0.0 - 100.0)
    pub coverage_threshold: f64,

    /// Maximum validation time in milliseconds
    pub max_validation_time_ms: u64,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self::for_mode(ValidationMode::EightyTwenty)
    }
}

impl ValidationConfig {
    /// Create config for specific validation mode
    pub fn for_mode(mode: ValidationMode) -> Self {
        match mode {
            ValidationMode::Minimal => Self::minimal(),
            ValidationMode::EightyTwenty => Self::eighty_twenty(),
            ValidationMode::Lenient => Self::lenient(),
            ValidationMode::Strict => Self::strict(),
        }
    }

    /// Minimal mode: Critical spans only (local dev)
    pub fn minimal() -> Self {
        Self {
            mode: ValidationMode::Minimal,
            fail_on_violation: true,
            fail_on_missing_optional: false,
            coverage_threshold: 60.0,
            max_validation_time_ms: 2000,
        }
    }

    /// 80/20 mode: Critical spans + required attributes (CI/CD)
    pub fn eighty_twenty() -> Self {
        Self {
            mode: ValidationMode::EightyTwenty,
            fail_on_violation: true,
            fail_on_missing_optional: false,
            coverage_threshold: 80.0,
            max_validation_time_ms: 5000,
        }
    }

    /// Lenient mode: All spans + required attributes (QA)
    pub fn lenient() -> Self {
        Self {
            mode: ValidationMode::Lenient,
            fail_on_violation: true,
            fail_on_missing_optional: false,
            coverage_threshold: 90.0,
            max_validation_time_ms: 15000,
        }
    }

    /// Strict mode: All spans + all attributes (final release)
    pub fn strict() -> Self {
        Self {
            mode: ValidationMode::Strict,
            fail_on_violation: true,
            fail_on_missing_optional: true,
            coverage_threshold: 100.0,
            max_validation_time_ms: 30000,
        }
    }
}

/// 80/20 validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EightyTwentyConfig {
    /// Critical spans that MUST be validated (the 20%)
    pub critical_spans: Vec<String>,

    /// Required attributes that MUST be present
    pub required_attributes: Vec<String>,

    /// Optional attributes (validated only in strict mode)
    pub optional_attributes: Vec<String>,
}

impl Default for EightyTwentyConfig {
    fn default() -> Self {
        Self {
            // The critical 20% of spans that cover 80% of bugs
            critical_spans: vec![
                "clnrm.test.execute".to_string(),
                "clnrm.container.start".to_string(),
                "clnrm.container.stop".to_string(),
                "clnrm.test.cleanup".to_string(),
                "clnrm.cli.health".to_string(),
            ],
            // Critical attributes that cannot be faked
            required_attributes: vec![
                "clnrm.version".to_string(),
                "test.hermetic".to_string(),
                "test.name".to_string(),
                "container.id".to_string(),
                "service.name".to_string(),
                "test.result".to_string(),
                "container.destroyed_at".to_string(),
            ],
            // Optional attributes (nice-to-have)
            optional_attributes: vec![
                "test.flaky".to_string(),
                "test.slow".to_string(),
                "container.network.mode".to_string(),
                "plugin.version".to_string(),
            ],
        }
    }
}

/// Coverage thresholds for different attribute types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageThresholds {
    /// Critical spans coverage (MUST be 100% in all modes)
    pub critical_spans: f64,
    /// Critical attributes coverage (MUST be 100% in all modes)
    pub critical_attributes: f64,
    /// Required attributes coverage (varies by mode)
    pub required_attributes: f64,
    /// Optional attributes coverage (0% in 80/20, 80%+ in strict)
    pub optional_attributes: f64,
}

impl Default for CoverageThresholds {
    fn default() -> Self {
        // 80/20 mode defaults
        Self {
            critical_spans: 100.0,
            critical_attributes: 100.0,
            required_attributes: 80.0,
            optional_attributes: 0.0,
        }
    }
}

impl CoverageThresholds {
    /// Get thresholds for specific validation mode
    pub fn for_mode(mode: ValidationMode) -> Self {
        match mode {
            ValidationMode::Minimal => Self {
                critical_spans: 100.0,
                critical_attributes: 100.0,
                required_attributes: 0.0,
                optional_attributes: 0.0,
            },
            ValidationMode::EightyTwenty => Self::default(),
            ValidationMode::Lenient => Self {
                critical_spans: 100.0,
                critical_attributes: 100.0,
                required_attributes: 90.0,
                optional_attributes: 0.0,
            },
            ValidationMode::Strict => Self {
                critical_spans: 100.0,
                critical_attributes: 100.0,
                required_attributes: 100.0,
                optional_attributes: 80.0,
            },
        }
    }
}

/// Attribute criticality level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttributeCriticality {
    /// Critical attribute (cannot be faked, 20% of attributes, 60% of value)
    Critical,
    /// Required attribute (OTel semantic convention, 30% of attributes, 20% of value)
    Required,
    /// Optional attribute (nice-to-have, 50% of attributes, 20% of value)
    Optional,
}

/// Attribute metadata with criticality information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeMetadata {
    /// Attribute name
    pub name: String,
    /// Criticality level
    pub criticality: AttributeCriticality,
    /// Reason why this attribute is critical/required
    pub reason: String,
    /// Validation rule (optional)
    pub validation_rule: Option<String>,
}

/// Critical span definition
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CriticalSpan {
    /// Span ID (e.g., "span.clnrm.test_execution")
    pub id: String,
    /// Why this span is critical
    pub reason: String,
    /// Percentage of production bugs this span covers (0-100)
    pub risk_coverage: u8,
    /// Required attributes for this span
    pub required_attributes: Vec<String>,
}

/// Complete 80/20 validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Complete80_20Config {
    /// Validation mode
    pub mode: ValidationMode,

    /// Critical spans (the 20%)
    pub critical_spans: Vec<CriticalSpan>,

    /// Attribute metadata with criticality
    pub attribute_metadata: HashMap<String, AttributeMetadata>,

    /// Coverage thresholds
    pub coverage_thresholds: CoverageThresholds,

    /// Validation configuration
    pub validation_config: ValidationConfig,
}

impl Default for Complete80_20Config {
    fn default() -> Self {
        Self::for_mode(ValidationMode::EightyTwenty)
    }
}

impl Complete80_20Config {
    /// Create configuration for specific mode
    pub fn for_mode(mode: ValidationMode) -> Self {
        Self {
            mode,
            critical_spans: Self::default_critical_spans(),
            attribute_metadata: Self::default_attribute_metadata(),
            coverage_thresholds: CoverageThresholds::for_mode(mode),
            validation_config: ValidationConfig::for_mode(mode),
        }
    }

    /// Default critical spans (5 spans = 80% value)
    fn default_critical_spans() -> Vec<CriticalSpan> {
        vec![
            CriticalSpan {
                id: "clnrm.test.execute".to_string(),
                reason: "Primary proof of test execution".to_string(),
                risk_coverage: 40,
                required_attributes: vec![
                    "container.id".to_string(),
                    "test.hermetic".to_string(),
                    "test.result".to_string(),
                    "test.name".to_string(),
                ],
            },
            CriticalSpan {
                id: "clnrm.container.lifecycle".to_string(),
                reason: "Proves resource cleanup (no leaks)".to_string(),
                risk_coverage: 30,
                required_attributes: vec![
                    "container.id".to_string(),
                    "container.destroyed_at".to_string(),
                ],
            },
            CriticalSpan {
                id: "clnrm.cli.health".to_string(),
                reason: "Validates system prerequisites".to_string(),
                risk_coverage: 15,
                required_attributes: vec![
                    "health.overall".to_string(),
                    "docker.available".to_string(),
                ],
            },
            CriticalSpan {
                id: "clnrm.plugin.lifecycle".to_string(),
                reason: "Core plugin system validation".to_string(),
                risk_coverage: 10,
                required_attributes: vec!["plugin.name".to_string(), "plugin.loaded".to_string()],
            },
            CriticalSpan {
                id: "clnrm.backend.operation".to_string(),
                reason: "Backend integration proof".to_string(),
                risk_coverage: 5,
                required_attributes: vec![
                    "backend.type".to_string(),
                    "operation.success".to_string(),
                ],
            },
        ]
    }

    /// Default attribute metadata
    fn default_attribute_metadata() -> HashMap<String, AttributeMetadata> {
        let mut map = HashMap::new();

        // Critical attributes
        map.insert(
            "container.id".to_string(),
            AttributeMetadata {
                name: "container.id".to_string(),
                criticality: AttributeCriticality::Critical,
                reason: "Cannot exist without real container".to_string(),
                validation_rule: Some("must_be_valid_uuid".to_string()),
            },
        );

        map.insert(
            "test.hermetic".to_string(),
            AttributeMetadata {
                name: "test.hermetic".to_string(),
                criticality: AttributeCriticality::Critical,
                reason: "Proves hermetic isolation".to_string(),
                validation_rule: Some("must_be_true".to_string()),
            },
        );

        map.insert(
            "test.result".to_string(),
            AttributeMetadata {
                name: "test.result".to_string(),
                criticality: AttributeCriticality::Critical,
                reason: "Proves test execution completed".to_string(),
                validation_rule: None,
            },
        );

        map.insert(
            "container.destroyed_at".to_string(),
            AttributeMetadata {
                name: "container.destroyed_at".to_string(),
                criticality: AttributeCriticality::Critical,
                reason: "Proves cleanup (prevents resource leaks)".to_string(),
                validation_rule: Some("must_exist_for_every_container".to_string()),
            },
        );

        // Required attributes
        map.insert(
            "test.name".to_string(),
            AttributeMetadata {
                name: "test.name".to_string(),
                criticality: AttributeCriticality::Required,
                reason: "Required for test identification".to_string(),
                validation_rule: None,
            },
        );

        map.insert(
            "service.name".to_string(),
            AttributeMetadata {
                name: "service.name".to_string(),
                criticality: AttributeCriticality::Required,
                reason: "Required by OTel semantic conventions".to_string(),
                validation_rule: None,
            },
        );

        // Optional attributes
        map.insert(
            "test.flaky".to_string(),
            AttributeMetadata {
                name: "test.flaky".to_string(),
                criticality: AttributeCriticality::Optional,
                reason: "Nice-to-have for debugging".to_string(),
                validation_rule: None,
            },
        );

        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_mode_default() {
        assert_eq!(ValidationMode::default(), ValidationMode::EightyTwenty);
    }

    #[test]
    fn test_validation_config_for_mode() {
        let minimal = ValidationConfig::for_mode(ValidationMode::Minimal);
        assert_eq!(minimal.coverage_threshold, 60.0);
        assert_eq!(minimal.max_validation_time_ms, 2000);

        let eighty_twenty = ValidationConfig::for_mode(ValidationMode::EightyTwenty);
        assert_eq!(eighty_twenty.coverage_threshold, 80.0);
        assert_eq!(eighty_twenty.max_validation_time_ms, 5000);

        let strict = ValidationConfig::for_mode(ValidationMode::Strict);
        assert_eq!(strict.coverage_threshold, 100.0);
        assert!(strict.fail_on_missing_optional);
    }

    #[test]
    fn test_eighty_twenty_config_default() {
        let config = EightyTwentyConfig::default();
        assert_eq!(config.critical_spans.len(), 5);
        assert!(config
            .critical_spans
            .contains(&"clnrm.test.execute".to_string()));
        assert!(config
            .required_attributes
            .contains(&"container.id".to_string()));
    }

    #[test]
    fn test_coverage_thresholds_for_mode() {
        let minimal = CoverageThresholds::for_mode(ValidationMode::Minimal);
        assert_eq!(minimal.critical_spans, 100.0);
        assert_eq!(minimal.required_attributes, 0.0);

        let eighty_twenty = CoverageThresholds::for_mode(ValidationMode::EightyTwenty);
        assert_eq!(eighty_twenty.critical_spans, 100.0);
        assert_eq!(eighty_twenty.required_attributes, 80.0);

        let strict = CoverageThresholds::for_mode(ValidationMode::Strict);
        assert_eq!(strict.critical_spans, 100.0);
        assert_eq!(strict.optional_attributes, 80.0);
    }

    #[test]
    fn test_complete_80_20_config() {
        let config = Complete80_20Config::default();
        assert_eq!(config.mode, ValidationMode::EightyTwenty);
        assert_eq!(config.critical_spans.len(), 5);

        // Check risk coverage adds up to ~100%
        let total_risk: u8 = config.critical_spans.iter().map(|s| s.risk_coverage).sum();
        assert_eq!(total_risk, 100);
    }
}
