//! OTEL validation command - runs 7 validators on collected traces
//!
//! Provides the `clnrm analyze` command to validate OpenTelemetry traces
//! against TOML-defined expectations, catching fake-green tests.
//!
//! This module implements first-failing-rule reporting, showing the FIRST
//! validator that fails with detailed context and recommendations.

use crate::config::types::TestConfig;
use crate::error::{CleanroomError, Result};
use crate::validation::count_validator::{CountBound, CountExpectation};
use crate::validation::graph_validator::GraphExpectation;
use crate::validation::hermeticity_validator::HermeticityExpectation;
use crate::validation::order_validator::OrderExpectation;
use crate::validation::span_validator::{SpanData, SpanValidator};
use crate::validation::status_validator::{StatusCode, StatusExpectation};
use crate::validation::window_validator::WindowExpectation;
use sha2::{Digest, Sha256};
use std::path::Path;

/// Load spans from artifact directories for scenarios
///
/// Scans `.clnrm/artifacts/<scenario-name>/spans.json` for each scenario
/// defined in the test configuration.
///
/// # Arguments
/// * `test_config` - Test configuration with scenario definitions
///
/// # Returns
/// * `Result<Vec<SpanData>>` - All spans loaded from artifacts
///
/// # Errors
/// * No artifacts found for any scenario
fn load_spans_from_artifacts(test_config: &TestConfig) -> Result<Vec<SpanData>> {
    let mut all_spans = Vec::new();
    let mut found_any_artifacts = false;

    // Check for scenarios (v0.6.0+ format)
    if !test_config.scenario.is_empty() {
        for scenario in &test_config.scenario {
            let artifact_path = format!(".clnrm/artifacts/{}/spans.json", scenario.name);

            if Path::new(&artifact_path).exists() {
                tracing::info!(
                    scenario = %scenario.name,
                    path = %artifact_path,
                    "Loading spans from artifact"
                );

                let validator = SpanValidator::from_file(&artifact_path)?;
                let spans = validator.spans();
                all_spans.extend_from_slice(spans);
                found_any_artifacts = true;
            } else {
                tracing::debug!(
                    scenario = %scenario.name,
                    path = %artifact_path,
                    "No artifacts found for scenario"
                );
            }
        }
    }

    // If no scenarios, try to load from test name (v0.4.x compatibility)
    if !found_any_artifacts {
        let test_name = test_config.get_name()?;
        let artifact_path = format!(".clnrm/artifacts/{}/spans.json", test_name);

        if Path::new(&artifact_path).exists() {
            tracing::info!(
                test = %test_name,
                path = %artifact_path,
                "Loading spans from artifact (legacy format)"
            );

            let validator = SpanValidator::from_file(&artifact_path)?;
            let spans = validator.spans();
            all_spans.extend_from_slice(spans);
            found_any_artifacts = true;
        }
    }

    if !found_any_artifacts {
        return Err(CleanroomError::validation_error(
            "No artifact files found. Run tests with artifact collection enabled first, \
             or provide --traces flag explicitly.",
        ));
    }

    tracing::info!(span_count = all_spans.len(), "Loaded spans from artifacts");

    Ok(all_spans)
}

/// Run OTEL validation on collected traces
///
/// # Arguments
/// * `test_file` - Path to test TOML file with expectations
/// * `traces_file` - Optional path to JSON file containing OTEL traces.
///   If not provided, auto-loads from `.clnrm/artifacts/<scenario>/spans.json`
///
/// # Returns
/// * `Result<AnalysisReport>` - Validation report with pass/fail status
///
/// # Exit Code
/// * 0 = All validators passed
/// * 1 = Any validator failed
pub fn analyze_traces(test_file: &Path, traces_file: Option<&Path>) -> Result<AnalysisReport> {
    // Load test configuration to extract expectations
    let config_str = std::fs::read_to_string(test_file).map_err(|e| {
        CleanroomError::config_error(format!(
            "Failed to read test file {}: {}",
            test_file.display(),
            e
        ))
    })?;

    let config: TestConfig = toml::from_str(&config_str)
        .map_err(|e| CleanroomError::config_error(format!("Failed to parse test TOML: {}", e)))?;

    // Load OTEL traces from explicit file or artifacts
    let (validator, traces_source) = if let Some(traces_path) = traces_file {
        tracing::info!(
            path = %traces_path.display(),
            "Loading spans from explicit traces file"
        );

        // Check if trace file exists and provide helpful error if not
        if !traces_path.exists() {
            return Err(CleanroomError::validation_error(format!(
                "Trace file not found: {}\n\n\
                 📚 OTEL Collector Setup Required:\n\
                 \n\
                 To collect OTEL traces:\n\
                 1. Start OTEL collector: docker-compose up otel-collector\n\
                 2. Configure exporter in your tests\n\
                 3. Run tests to generate traces\n\
                 4. Traces will be in: /tmp/traces/ or collector output\n\
                 \n\
                 📖 Full documentation:\n\
                 - docs/OPENTELEMETRY_INTEGRATION_GUIDE.md\n\
                 - https://github.com/seanchatmangpt/clnrm#opentelemetry\n\
                 \n\
                 💡 Quick start:\n\
                 export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318\n\
                 clnrm run --otel-enabled tests/example.toml",
                traces_path.display()
            )));
        }

        let validator = SpanValidator::from_file(traces_path)?;
        (validator, traces_path.display().to_string())
    } else {
        tracing::info!("Auto-loading spans from artifacts");
        let spans = load_spans_from_artifacts(&config)?;
        let validator = SpanValidator { spans };
        (validator, ".clnrm/artifacts/**/spans.json".to_string())
    };

    let spans = validator.spans();

    // Extract test name for report
    let test_name = config
        .meta
        .as_ref()
        .map(|m| m.name.clone())
        .or_else(|| config.test.as_ref().map(|t| t.metadata.name.clone()))
        .unwrap_or_else(|| "unknown".to_string());

    // Compute digest of traces for reproducibility
    let digest = compute_trace_digest(spans)?;

    let mut report = AnalysisReport {
        test_name: test_name.clone(),
        traces_file: traces_source,
        span_count: spans.len(),
        event_count: count_events(spans),
        digest,
        validators: Vec::new(),
    };

    // Run validators based on expectations in config
    if let Some(ref expect) = config.expect {
        // 1. Span Expectations Validator
        if !expect.span.is_empty() {
            let result = validate_span_expectations(&expect.span, spans);
            report.validators.push(result);
        }

        // 2. Graph Structure Validator
        if let Some(ref graph_config) = expect.graph {
            let result = validate_graph_structure(graph_config, spans);
            report.validators.push(result);
        }

        // 3. Counts Validator
        if let Some(ref counts_config) = expect.counts {
            let result = validate_counts(counts_config, spans);
            report.validators.push(result);
        }

        // 4. Window Containment Validator
        if !expect.window.is_empty() {
            let result = validate_windows(&expect.window, spans);
            report.validators.push(result);
        }

        // 5. Ordering Validator
        if let Some(ref order_config) = expect.order {
            let result = validate_ordering(order_config, spans);
            report.validators.push(result);
        }

        // 6. Status Validator
        if let Some(ref status_config) = expect.status {
            let result = validate_status(status_config, spans);
            report.validators.push(result);
        }

        // 7. Hermeticity Validator
        if let Some(ref hermetic_config) = expect.hermeticity {
            let result = validate_hermeticity(hermetic_config, spans);
            report.validators.push(result);
        }
    }

    Ok(report)
}

/// Validate span expectations (name, kind, attributes, events, duration)
fn validate_span_expectations(
    span_configs: &[crate::config::otel::SpanExpectationConfig],
    spans: &[SpanData],
) -> ValidatorResult {
    let mut passed_count = 0;
    let total_count = span_configs.len();
    let mut errors = Vec::new();

    for config in span_configs {
        // Find matching span(s)
        let matching_spans: Vec<_> = spans.iter().filter(|s| s.name == config.name).collect();

        if matching_spans.is_empty() {
            errors.push(format!("Expected span '{}' not found", config.name));
            continue;
        }

        // Validate each matching span
        for span in matching_spans {
            let mut span_valid = true;

            // Validate attributes.all
            if let Some(ref attrs_config) = config.attrs {
                if let Some(ref all_attrs) = attrs_config.all {
                    for (key, expected_value) in all_attrs {
                        if let Some(actual_value) = span.attributes.get(key) {
                            let actual_str = actual_value.to_string();
                            if !actual_str.contains(expected_value) {
                                errors.push(format!(
                                    "Span '{}': attribute '{}' expected '{}', got '{}'",
                                    config.name, key, expected_value, actual_str
                                ));
                                span_valid = false;
                            }
                        } else {
                            errors.push(format!(
                                "Span '{}': missing expected attribute '{}'",
                                config.name, key
                            ));
                            span_valid = false;
                        }
                    }
                }
            }

            if span_valid {
                passed_count += 1;
            }
        }
    }

    ValidatorResult {
        name: "Span Expectations".to_string(),
        passed: errors.is_empty(),
        details: if passed_count > 0 {
            format!("{}/{} passed", passed_count, total_count)
        } else {
            format!("FAIL: {}", errors.join(", "))
        },
    }
}

/// Validate graph structure (parent-child relationships)
fn validate_graph_structure(
    graph_config: &crate::config::otel::GraphExpectationConfig,
    spans: &[SpanData],
) -> ValidatorResult {
    // Handle Option<Vec<Vec<String>>> from v1.0 schema
    let edges: Vec<_> = graph_config
        .must_include
        .as_ref()
        .map(|edges| {
            edges
                .iter()
                .filter_map(|edge| {
                    if edge.len() >= 2 {
                        Some((edge[0].clone(), edge[1].clone()))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    if edges.is_empty() {
        return ValidatorResult {
            name: "Graph Structure".to_string(),
            passed: true,
            details: "no edges to validate".to_string(),
        };
    }

    let graph = GraphExpectation::new(edges.clone());

    match graph.validate(spans) {
        Ok(_) => ValidatorResult {
            name: "Graph Structure".to_string(),
            passed: true,
            details: format!("all {} edges present", edges.len()),
        },
        Err(e) => ValidatorResult {
            name: "Graph Structure".to_string(),
            passed: false,
            details: format!("FAIL: {}", e),
        },
    }
}

/// Validate span counts
fn validate_counts(
    counts_config: &crate::config::otel::CountExpectationConfig,
    spans: &[SpanData],
) -> ValidatorResult {
    let mut expectation = CountExpectation::new();

    // Add total span count bounds
    if let Some(ref total) = counts_config.spans_total {
        let bound = if let Some(eq) = total.eq {
            CountBound::eq(eq)
        } else if let Some(gte) = total.gte {
            if let Some(lte) = total.lte {
                match CountBound::range(gte, lte) {
                    Ok(b) => b,
                    Err(_) => CountBound::gte(gte),
                }
            } else {
                CountBound::gte(gte)
            }
        } else if let Some(lte) = total.lte {
            CountBound::lte(lte)
        } else {
            // No constraints
            CountBound {
                gte: None,
                lte: None,
                eq: None,
            }
        };
        expectation = expectation.with_spans_total(bound);
    }

    // Add per-name count bounds
    if let Some(ref by_name) = counts_config.by_name {
        for (name, bounds) in by_name {
            let bound = if let Some(eq) = bounds.eq {
                CountBound::eq(eq)
            } else if let Some(gte) = bounds.gte {
                if let Some(lte) = bounds.lte {
                    match CountBound::range(gte, lte) {
                        Ok(b) => b,
                        Err(_) => CountBound::gte(gte),
                    }
                } else {
                    CountBound::gte(gte)
                }
            } else if let Some(lte) = bounds.lte {
                CountBound::lte(lte)
            } else {
                continue;
            };

            expectation = expectation.with_name_count(name.clone(), bound);
        }
    }

    match expectation.validate(spans) {
        Ok(_) => ValidatorResult {
            name: "Counts".to_string(),
            passed: true,
            details: format!("spans_total: {}", spans.len()),
        },
        Err(e) => ValidatorResult {
            name: "Counts".to_string(),
            passed: false,
            details: format!("FAIL: {}", e),
        },
    }
}

/// Validate temporal windows (containment)
fn validate_windows(
    window_configs: &[crate::config::otel::WindowExpectationConfig],
    spans: &[SpanData],
) -> ValidatorResult {
    let mut passed = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    for config in window_configs {
        let window = WindowExpectation {
            outer: config.outer.clone(),
            contains: config.contains.clone(),
        };

        match window.validate(spans) {
            Ok(_) => passed += 1,
            Err(e) => {
                failed += 1;
                errors.push(format!("window '{}': {}", config.outer, e));
            }
        }
    }

    ValidatorResult {
        name: "Window Containment".to_string(),
        passed: failed == 0,
        details: if failed == 0 {
            format!("all {} windows satisfied", passed)
        } else {
            format!("FAIL: {}", errors.join(", "))
        },
    }
}

/// Validate temporal ordering constraints
fn validate_ordering(
    order_config: &crate::config::otel::OrderExpectationConfig,
    spans: &[SpanData],
) -> ValidatorResult {
    let mut must_precede = Vec::new();
    let mut must_follow = Vec::new();

    // Add must_precede rules (v1.0 schema: Vec<Vec<String>>)
    if let Some(ref precede) = order_config.must_precede {
        for edge in precede {
            if edge.len() >= 2 {
                must_precede.push((edge[0].clone(), edge[1].clone()));
            }
        }
    }

    // Add must_follow rules (v1.0 schema: Vec<Vec<String>>)
    if let Some(ref follow) = order_config.must_follow {
        for edge in follow {
            if edge.len() >= 2 {
                must_follow.push((edge[0].clone(), edge[1].clone()));
            }
        }
    }

    if must_precede.is_empty() && must_follow.is_empty() {
        return ValidatorResult {
            name: "Ordering".to_string(),
            passed: true,
            details: "no ordering constraints".to_string(),
        };
    }

    let expectation = OrderExpectation::new()
        .with_must_precede(must_precede)
        .with_must_follow(must_follow);

    match expectation.validate(spans) {
        Ok(_) => ValidatorResult {
            name: "Ordering".to_string(),
            passed: true,
            details: "all constraints satisfied".to_string(),
        },
        Err(e) => ValidatorResult {
            name: "Ordering".to_string(),
            passed: false,
            details: format!("FAIL: {}", e),
        },
    }
}

/// Validate status codes
fn validate_status(
    status_config: &crate::config::otel::StatusExpectationConfig,
    spans: &[SpanData],
) -> ValidatorResult {
    let mut expectation = StatusExpectation::new();

    // Add global status rule
    if let Some(ref all_status) = status_config.all {
        if let Ok(status) = StatusCode::parse(all_status) {
            expectation = expectation.with_all(status);
        } else {
            return ValidatorResult {
                name: "Status".to_string(),
                passed: false,
                details: format!("FAIL: invalid status code '{}'", all_status),
            };
        }
    }

    // Add per-name status rules
    if let Some(ref by_name) = status_config.by_name {
        for (pattern, expected) in by_name {
            if let Ok(status) = StatusCode::parse(expected) {
                expectation.by_name.insert(pattern.clone(), status);
            } else {
                return ValidatorResult {
                    name: "Status".to_string(),
                    passed: false,
                    details: format!(
                        "FAIL: invalid status code '{}' for pattern '{}'",
                        expected, pattern
                    ),
                };
            }
        }
    }

    match expectation.validate(spans) {
        Ok(_) => ValidatorResult {
            name: "Status".to_string(),
            passed: true,
            details: "all spans OK".to_string(),
        },
        Err(e) => ValidatorResult {
            name: "Status".to_string(),
            passed: false,
            details: format!("FAIL: {}", e),
        },
    }
}

/// Validate hermeticity (isolation, no external services)
fn validate_hermeticity(
    hermetic_config: &crate::config::otel::HermeticityExpectationConfig,
    spans: &[SpanData],
) -> ValidatorResult {
    let mut expectation = HermeticityExpectation {
        no_external_services: hermetic_config.no_external_services,
        resource_attrs_must_match: None,
        sdk_resource_attrs_must_match: None,
        span_attrs_forbid_keys: None,
    };

    // Handle v1.0 nested schema: resource_attrs.must_match
    if let Some(ref resource_attrs) = hermetic_config.resource_attrs {
        if let Some(ref must_match) = resource_attrs.must_match {
            expectation.resource_attrs_must_match = Some(must_match.clone());
        }
    }

    // Handle v1.0 nested schema: span_attrs.forbid_keys
    if let Some(ref span_attrs) = hermetic_config.span_attrs {
        if let Some(ref forbid_keys) = span_attrs.forbid_keys {
            expectation.span_attrs_forbid_keys = Some(forbid_keys.clone());
        }
    }

    match expectation.validate(spans) {
        Ok(_) => ValidatorResult {
            name: "Hermeticity".to_string(),
            passed: true,
            details: "no external services detected".to_string(),
        },
        Err(e) => ValidatorResult {
            name: "Hermeticity".to_string(),
            passed: false,
            details: format!("FAIL: {}", e),
        },
    }
}

/// Count total events across all spans
fn count_events(spans: &[SpanData]) -> usize {
    spans
        .iter()
        .filter_map(|s| s.events.as_ref())
        .map(|events| events.len())
        .sum()
}

/// Compute SHA256 digest of traces for reproducibility
fn compute_trace_digest(spans: &[SpanData]) -> Result<String> {
    let json = serde_json::to_string(spans)
        .map_err(|e| CleanroomError::internal_error(format!("Failed to serialize spans: {}", e)))?;

    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    let result = hasher.finalize();

    Ok(format!("sha256:{:x}", result))
}

/// Analysis report containing all validation results
#[derive(Debug, Clone)]
pub struct AnalysisReport {
    /// Test name from TOML
    pub test_name: String,
    /// Path to traces file
    pub traces_file: String,
    /// Total span count
    pub span_count: usize,
    /// Total event count
    pub event_count: usize,
    /// SHA256 digest of traces
    pub digest: String,
    /// Individual validator results
    pub validators: Vec<ValidatorResult>,
}

impl AnalysisReport {
    /// Check if all validators passed
    pub fn is_success(&self) -> bool {
        self.validators.iter().all(|v| v.passed)
    }

    /// Count failed validators
    pub fn failure_count(&self) -> usize {
        self.validators.iter().filter(|v| !v.passed).count()
    }

    /// Count passed validators
    pub fn pass_count(&self) -> usize {
        self.validators.iter().filter(|v| v.passed).count()
    }

    /// Generate human-readable report
    pub fn format_report(&self) -> String {
        let mut output = String::new();

        output.push_str("📊 OTEL Validation Report\n");
        output.push_str("========================\n\n");

        output.push_str(&format!("Test: {}\n", self.test_name));
        output.push_str(&format!(
            "Traces: {} spans, {} events\n\n",
            self.span_count, self.event_count
        ));

        output.push_str("Validators:\n");
        for validator in &self.validators {
            let icon = if validator.passed { "✅" } else { "❌" };
            output.push_str(&format!(
                "  {} {} ({})\n",
                icon, validator.name, validator.details
            ));
        }

        output.push('\n');

        if self.is_success() {
            output.push_str(&format!(
                "Result: PASS ({}/{} validators passed)\n",
                self.pass_count(),
                self.validators.len()
            ));
        } else {
            output.push_str(&format!(
                "Result: FAIL ({}/{} validators failed)\n",
                self.failure_count(),
                self.validators.len()
            ));
        }

        output.push_str(&format!(
            "Digest: {} (recorded for reproduction)\n",
            self.digest
        ));

        output
    }
}

/// Individual validator result
#[derive(Debug, Clone)]
pub struct ValidatorResult {
    /// Validator name
    pub name: String,
    /// Whether validation passed
    pub passed: bool,
    /// Details or error message
    pub details: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{MetaConfig, ScenarioConfig};
    use crate::validation::span_validator::SpanData;
    use std::collections::HashMap;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_span(name: &str, span_id: &str, parent_id: Option<&str>) -> SpanData {
        let mut attrs = HashMap::new();
        attrs.insert(
            "result".to_string(),
            serde_json::Value::String("pass".to_string()),
        );

        SpanData {
            name: name.to_string(),
            trace_id: "test_trace".to_string(),
            span_id: span_id.to_string(),
            parent_span_id: parent_id.map(|s| s.to_string()),
            attributes: attrs,
            start_time_unix_nano: Some(1000000),
            end_time_unix_nano: Some(2000000),
            kind: None,
            events: None,
            resource_attributes: HashMap::new(),
        }
    }

    #[test]
    fn test_load_spans_from_artifacts_with_scenarios() -> Result<()> {
        // Arrange - Create temporary artifact directory structure
        let temp_dir = TempDir::new().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
        })?;
        let original_dir = std::env::current_dir().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to get current dir: {}", e))
        })?;

        std::env::set_current_dir(&temp_dir)
            .map_err(|e| CleanroomError::internal_error(format!("Failed to change dir: {}", e)))?;

        // Create artifact directories
        fs::create_dir_all(".clnrm/artifacts/scenario1")
            .map_err(|e| CleanroomError::internal_error(format!("Failed to create dir: {}", e)))?;
        fs::create_dir_all(".clnrm/artifacts/scenario2")
            .map_err(|e| CleanroomError::internal_error(format!("Failed to create dir: {}", e)))?;

        // Write span files
        let span1_json = r#"{"name":"span1","trace_id":"trace1","span_id":"s1","parent_span_id":null,"attributes":{}}"#;
        let span2_json = r#"{"name":"span2","trace_id":"trace1","span_id":"s2","parent_span_id":"s1","attributes":{}}"#;

        let mut file1 = fs::File::create(".clnrm/artifacts/scenario1/spans.json")
            .map_err(|e| CleanroomError::internal_error(format!("Failed to create file: {}", e)))?;
        file1
            .write_all(span1_json.as_bytes())
            .map_err(|e| CleanroomError::internal_error(format!("Failed to write file: {}", e)))?;

        let mut file2 = fs::File::create(".clnrm/artifacts/scenario2/spans.json")
            .map_err(|e| CleanroomError::internal_error(format!("Failed to create file: {}", e)))?;
        file2
            .write_all(span2_json.as_bytes())
            .map_err(|e| CleanroomError::internal_error(format!("Failed to write file: {}", e)))?;

        // Create test config
        let config = TestConfig {
            meta: Some(MetaConfig {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                description: None,
            }),
            test: None,
            services: None,
            service: None,
            steps: vec![],
            scenario: vec![
                ScenarioConfig {
                    name: "scenario1".to_string(),
                    steps: vec![],
                    service: None,
                    run: None,
                    concurrent: None,
                    timeout_ms: None,
                    policy: None,
                    artifacts: None,
                },
                ScenarioConfig {
                    name: "scenario2".to_string(),
                    steps: vec![],
                    service: None,
                    run: None,
                    concurrent: None,
                    timeout_ms: None,
                    policy: None,
                    artifacts: None,
                },
            ],
            assertions: None,
            otel_validation: None,
            otel: None,
            vars: None,
            matrix: None,
            expect: None,
            report: None,
            determinism: None,
            limits: None,
            otel_headers: None,
            otel_propagators: None,
        };

        // Act
        let spans = load_spans_from_artifacts(&config)?;

        // Assert
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].name, "span1");
        assert_eq!(spans[1].name, "span2");

        // Cleanup
        std::env::set_current_dir(&original_dir)
            .map_err(|e| CleanroomError::internal_error(format!("Failed to restore dir: {}", e)))?;

        Ok(())
    }

    #[test]
    fn test_load_spans_from_artifacts_no_artifacts() -> Result<()> {
        // Arrange - Config with no matching artifacts
        let config = TestConfig {
            meta: Some(MetaConfig {
                name: "nonexistent".to_string(),
                version: "1.0.0".to_string(),
                description: None,
            }),
            test: None,
            services: None,
            service: None,
            steps: vec![],
            scenario: vec![ScenarioConfig {
                name: "nonexistent_scenario".to_string(),
                steps: vec![],
                service: None,
                run: None,
                concurrent: None,
                timeout_ms: None,
                policy: None,
                artifacts: None,
            }],
            assertions: None,
            otel_validation: None,
            otel: None,
            vars: None,
            matrix: None,
            expect: None,
            report: None,
            determinism: None,
            limits: None,
            otel_headers: None,
            otel_propagators: None,
        };

        // Act
        let result = load_spans_from_artifacts(&config);

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("No artifact files found"));

        Ok(())
    }

    #[test]
    fn test_artifact_path_resolution_format() {
        // Arrange
        let scenario_name = "my_test_scenario";
        let expected_path = ".clnrm/artifacts/my_test_scenario/spans.json";

        // Act
        let actual_path = format!(".clnrm/artifacts/{}/spans.json", scenario_name);

        // Assert
        assert_eq!(actual_path, expected_path);
    }

    #[test]
    fn test_count_events() {
        // Arrange
        let mut span1 = create_test_span("root", "s1", None);
        span1.events = Some(vec!["event1".to_string(), "event2".to_string()]);

        let mut span2 = create_test_span("child", "s2", Some("s1"));
        span2.events = Some(vec!["event3".to_string()]);

        let spans = vec![span1, span2];

        // Act
        let count = count_events(&spans);

        // Assert
        assert_eq!(count, 3);
    }

    #[test]
    fn test_compute_trace_digest() -> Result<()> {
        // Arrange
        let spans = vec![create_test_span("root", "s1", None)];

        // Act
        let digest = compute_trace_digest(&spans)?;

        // Assert
        assert!(digest.starts_with("sha256:"));
        assert!(digest.len() > 10);

        Ok(())
    }

    #[test]
    fn test_analysis_report_success() {
        // Arrange
        let report = AnalysisReport {
            test_name: "test".to_string(),
            traces_file: "traces.json".to_string(),
            span_count: 10,
            event_count: 20,
            digest: "sha256:abc123".to_string(),
            validators: vec![
                ValidatorResult {
                    name: "Validator 1".to_string(),
                    passed: true,
                    details: "OK".to_string(),
                },
                ValidatorResult {
                    name: "Validator 2".to_string(),
                    passed: true,
                    details: "OK".to_string(),
                },
            ],
        };

        // Act & Assert
        assert!(report.is_success());
        assert_eq!(report.pass_count(), 2);
        assert_eq!(report.failure_count(), 0);
    }

    #[test]
    fn test_analysis_report_failure() {
        // Arrange
        let report = AnalysisReport {
            test_name: "test".to_string(),
            traces_file: "traces.json".to_string(),
            span_count: 10,
            event_count: 20,
            digest: "sha256:abc123".to_string(),
            validators: vec![
                ValidatorResult {
                    name: "Validator 1".to_string(),
                    passed: true,
                    details: "OK".to_string(),
                },
                ValidatorResult {
                    name: "Validator 2".to_string(),
                    passed: false,
                    details: "FAIL: missing edge".to_string(),
                },
            ],
        };

        // Act & Assert
        assert!(!report.is_success());
        assert_eq!(report.pass_count(), 1);
        assert_eq!(report.failure_count(), 1);
    }

    #[test]
    fn test_format_report_output() {
        // Arrange
        let report = AnalysisReport {
            test_name: "test".to_string(),
            traces_file: "traces.json".to_string(),
            span_count: 5,
            event_count: 10,
            digest: "sha256:abc123".to_string(),
            validators: vec![ValidatorResult {
                name: "Test Validator".to_string(),
                passed: true,
                details: "all good".to_string(),
            }],
        };

        // Act
        let output = report.format_report();

        // Assert
        assert!(output.contains("📊 OTEL Validation Report"));
        assert!(output.contains("Test: test"));
        assert!(output.contains("5 spans"));
        assert!(output.contains("10 events"));
        assert!(output.contains("✅ Test Validator"));
        assert!(output.contains("PASS"));
        assert!(output.contains("sha256:abc123"));
    }
}
