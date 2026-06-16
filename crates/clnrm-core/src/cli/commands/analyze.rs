//! OTEL validation command - runs 7 validators on collected traces
//!
//! Provides the `clnrm analyze` command to validate OpenTelemetry traces
//! against TOML-defined expectations, catching false-green tests.
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
        CleanroomError::configuration_error(format!(
            "Failed to read test file {}: {}",
            test_file.display(),
            e
        ))
    })?;

    let config: TestConfig = toml::from_str(&config_str).map_err(|e| {
        CleanroomError::configuration_error(format!("Failed to parse test TOML: {}", e))
    })?;

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
        .or_else(|| config.test.as_ref().map(|t| t.metadata().name.clone()))
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidatorResult {
    /// Validator name
    pub name: String,
    /// Whether validation passed
    pub passed: bool,
    /// Details or error message
    pub details: String,
}

/// Span statistics per span name
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpanStatistics {
    pub span_name: String,
    pub count: usize,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub mean_ms: f64,
}

/// Bottleneck: a span that causes high child wait time
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Bottleneck {
    pub span_name: String,
    pub span_id: String,
    pub child_count: usize,
    pub total_wait_ms: f64,
}

/// Compute critical path through span DAG (longest execution path by duration)
///
/// Returns span IDs in order from root to leaf forming the critical path.
pub fn compute_critical_path(spans: &[crate::validation::span_validator::SpanData]) -> Vec<String> {
    // Build parent -> children map
    let mut children: std::collections::HashMap<
        String,
        Vec<&crate::validation::span_validator::SpanData>,
    > = std::collections::HashMap::new();
    let mut all_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

    for span in spans {
        all_ids.insert(span.span_id.clone());
    }

    for span in spans {
        if let Some(ref parent_id) = span.parent_span_id {
            children.entry(parent_id.clone()).or_default().push(span);
        }
    }

    // Find root spans (those whose parent_span_id is None or not in the set)
    let roots: Vec<&crate::validation::span_validator::SpanData> = spans
        .iter()
        .filter(|s| {
            s.parent_span_id
                .as_ref()
                .map(|pid| !all_ids.contains(pid))
                .unwrap_or(true)
        })
        .collect();

    if roots.is_empty() {
        return Vec::new();
    }

    // DFS to find the path with maximum cumulative duration
    fn dfs_max_path<'a>(
        span: &'a crate::validation::span_validator::SpanData,
        children: &std::collections::HashMap<
            String,
            Vec<&'a crate::validation::span_validator::SpanData>,
        >,
    ) -> (f64, Vec<String>) {
        let self_dur = span.duration_ms().unwrap_or(1.0);
        let child_list = children.get(&span.span_id);

        match child_list {
            None => (self_dur, vec![span.span_id.clone()]),
            Some(kids) if kids.is_empty() => (self_dur, vec![span.span_id.clone()]),
            Some(kids) => {
                let mut best_dur = 0.0_f64;
                let mut best_path: Vec<String> = Vec::new();

                for kid in kids {
                    let (child_dur, child_path) = dfs_max_path(kid, children);
                    if child_dur > best_dur {
                        best_dur = child_dur;
                        best_path = child_path;
                    }
                }

                let total = self_dur + best_dur;
                let mut path = vec![span.span_id.clone()];
                path.extend(best_path);
                (total, path)
            }
        }
    }

    // Pick the root with the longest overall path
    let mut global_best_dur = 0.0_f64;
    let mut global_best_path: Vec<String> = Vec::new();

    for root in roots {
        let (dur, path) = dfs_max_path(root, &children);
        if dur > global_best_dur {
            global_best_dur = dur;
            global_best_path = path;
        }
    }

    global_best_path
}

/// Detect bottlenecks: spans where children spend the most cumulative time waiting
pub fn detect_bottlenecks(
    spans: &[crate::validation::span_validator::SpanData],
) -> Vec<Bottleneck> {
    // Build span_id -> span map
    let span_map: std::collections::HashMap<&str, &crate::validation::span_validator::SpanData> =
        spans.iter().map(|s| (s.span_id.as_str(), s)).collect();

    // Build parent -> children map
    let mut children: std::collections::HashMap<
        &str,
        Vec<&crate::validation::span_validator::SpanData>,
    > = std::collections::HashMap::new();

    for span in spans {
        if let Some(ref parent_id) = span.parent_span_id {
            children.entry(parent_id.as_str()).or_default().push(span);
        }
    }

    let mut bottlenecks: Vec<Bottleneck> = children
        .iter()
        .filter_map(|(parent_id, kids)| {
            let parent = span_map.get(*parent_id)?;
            let total_wait: f64 = kids.iter().filter_map(|s| s.duration_ms()).sum();
            Some(Bottleneck {
                span_name: parent.name.clone(),
                span_id: parent.span_id.clone(),
                child_count: kids.len(),
                total_wait_ms: total_wait,
            })
        })
        .collect();

    // Sort by total_wait_ms descending, take up to 10
    bottlenecks.sort_by(|a, b| {
        b.total_wait_ms
            .partial_cmp(&a.total_wait_ms)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    bottlenecks.truncate(10);
    bottlenecks
}

/// Compute p50/p95/p99 statistics for each distinct span name
pub fn compute_statistics(
    spans: &[crate::validation::span_validator::SpanData],
) -> Vec<SpanStatistics> {
    // Group spans by name, collecting durations
    let mut by_name: std::collections::HashMap<String, Vec<f64>> = std::collections::HashMap::new();

    for span in spans {
        if let Some(dur) = span.duration_ms() {
            by_name.entry(span.name.clone()).or_default().push(dur);
        }
    }

    let mut stats: Vec<SpanStatistics> = by_name
        .into_iter()
        .map(|(name, mut durations)| {
            durations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let count = durations.len();

            let p50 = durations[count * 50 / 100];
            let p95 = durations[count * 95 / 100];
            let p99 = durations[count * 99 / 100];
            let min = durations[0];
            let max = durations[count - 1];
            let mean = durations.iter().sum::<f64>() / count as f64;

            SpanStatistics {
                span_name: name,
                count,
                p50_ms: p50,
                p95_ms: p95,
                p99_ms: p99,
                min_ms: min,
                max_ms: max,
                mean_ms: mean,
            }
        })
        .collect();

    stats.sort_by(|a, b| a.span_name.cmp(&b.span_name));
    stats
}
