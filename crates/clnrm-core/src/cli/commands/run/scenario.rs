//! Scenario execution with OTEL validation
//!
//! Handles execution of test scenarios including command execution,
//! OTEL span parsing, determinism application, and validation.

use crate::cleanroom::CleanroomEnvironment;
use crate::config::types::parse_shell_command;
use crate::determinism::DeterminismEngine;
use crate::error::{CleanroomError, Result};
use crate::otel::stdout_parser::StdoutSpanParser;
use crate::reporting::{generate_reports, ReportConfig};
use crate::validation::orchestrator::PrdExpectations;
use crate::validation::{
    CountExpectation, GraphExpectation, HermeticityExpectation, WindowExpectation,
};
use std::collections::HashMap;
use tracing::{debug, error, info};

/// Execute a single scenario with OTEL validation
pub async fn execute_scenario(
    scenario: &crate::config::ScenarioConfig,
    env: &CleanroomEnvironment,
    service_handles: &HashMap<String, crate::cleanroom::ServiceHandle>,
    test_config: &crate::config::TestConfig,
) -> Result<()> {
    info!("🚀 Executing scenario: {}", scenario.name);

    // Validate scenario has required fields
    if scenario.service.is_none() && scenario.run.is_none() {
        return Err(CleanroomError::validation_error(format!(
            "Scenario '{}' must have 'service' and/or 'run' fields",
            scenario.name
        )));
    }

    // Get service handle
    let service_name = scenario.service.as_ref().ok_or_else(|| {
        CleanroomError::validation_error(format!(
            "Scenario '{}' missing 'service' field",
            scenario.name
        ))
    })?;

    let handle = service_handles.get(service_name).ok_or_else(|| {
        CleanroomError::validation_error(format!(
            "Scenario '{}' references unknown service '{}'",
            scenario.name, service_name
        ))
    })?;

    // Parse shell command
    let run_command = scenario.run.as_ref().ok_or_else(|| {
        CleanroomError::validation_error(format!(
            "Scenario '{}' missing 'run' field",
            scenario.name
        ))
    })?;

    let command_args = parse_shell_command(run_command)?;
    info!("🔧 Executing command in container: {}", run_command);

    // Execute command in container and capture stdout/stderr
    let output = env
        .execute_command_with_output(handle, &command_args)
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !stderr.is_empty() {
        info!("⚠️  Stderr: {}", stderr.trim());
    }

    if !output.status.success() {
        return Err(CleanroomError::validation_error(format!(
            "Scenario '{}' command failed with exit code: {}",
            scenario.name,
            output.status.code().unwrap_or(-1)
        )));
    }

    debug!("📤 Command stdout length: {} bytes", stdout.len());

    // Parse OTEL spans from stdout if artifacts.collect includes "spans:default"
    if let Some(ref artifacts) = scenario.artifacts {
        if artifacts.collect.iter().any(|a| a.starts_with("spans:")) {
            info!("🔍 Parsing OTEL spans from stdout...");
            let mut spans = StdoutSpanParser::parse(&stdout)?;
            info!("✅ Collected {} span(s) from stdout", spans.len());

            // Apply determinism if configured
            if let Some(ref det_config) = test_config.determinism {
                if det_config.is_deterministic() {
                    info!(
                        "🔒 Applying determinism: seed={:?}, freeze_clock={:?}",
                        det_config.seed, det_config.freeze_clock
                    );

                    let engine = DeterminismEngine::new(det_config.clone())?;

                    // Apply frozen timestamp to spans if configured
                    if engine.has_frozen_clock() {
                        let frozen_timestamp = engine.get_timestamp();
                        let frozen_nanos =
                            frozen_timestamp.timestamp_nanos_opt().unwrap_or(0) as u64;

                        for span in &mut spans {
                            if span.start_time_unix_nano.is_none() {
                                span.start_time_unix_nano = Some(frozen_nanos);
                            }
                            if span.end_time_unix_nano.is_none() {
                                span.end_time_unix_nano = Some(frozen_nanos + 1_000_000);
                                // +1ms
                            }
                        }
                    }
                }
            }

            // Build expectations from test_config.expect
            let expectations = build_prd_expectations(test_config)?;

            // Run all validations
            info!("🔬 Running validation layers...");
            let validation_report = expectations.validate_all(&spans)?;

            // Log validation results
            if validation_report.is_success() {
                info!(
                    "✅ All {} validation(s) passed",
                    validation_report.pass_count()
                );
                info!("✅ Validation: {}", validation_report.summary());
            } else {
                error!(
                    "❌ {} validation(s) failed",
                    validation_report.failure_count()
                );
                error!("❌ Validation: {}", validation_report.summary());
            }

            // Generate reports if configured
            if let Some(ref report_config) = test_config.report {
                info!("📊 Generating reports...");
                let report_cfg = ReportConfig::new()
                    .with_json(
                        report_config
                            .json
                            .as_ref()
                            .unwrap_or(&"report.json".to_string())
                            .clone(),
                    )
                    .with_junit(
                        report_config
                            .junit
                            .as_ref()
                            .unwrap_or(&"junit.xml".to_string())
                            .clone(),
                    )
                    .with_digest(
                        report_config
                            .digest
                            .as_ref()
                            .unwrap_or(&"digest.txt".to_string())
                            .clone(),
                    );

                let spans_json = serde_json::to_string_pretty(&spans).map_err(|e| {
                    CleanroomError::internal_error(format!(
                        "Failed to serialize spans to JSON: {}",
                        e
                    ))
                })?;

                generate_reports(&report_cfg, &validation_report, &spans_json)?;
                info!("✅ Reports generated successfully");
            }

            // Fail if validation failed
            if !validation_report.is_success() {
                return Err(CleanroomError::validation_error(format!(
                    "Scenario '{}' validation failed: {}",
                    scenario.name,
                    validation_report.first_error().unwrap_or("unknown error")
                )));
            }
        }
    }

    info!("✅ Scenario '{}' completed successfully", scenario.name);
    Ok(())
}

/// Build PrdExpectations from TestConfig.expect
fn build_prd_expectations(test_config: &crate::config::TestConfig) -> Result<PrdExpectations> {
    let mut expectations = PrdExpectations::new();

    if let Some(ref expect) = test_config.expect {
        // Build graph expectations
        if let Some(ref graph_config) = expect.graph {
            let mut edges = Vec::new();

            if let Some(ref must_include) = graph_config.must_include {
                for edge in must_include {
                    if edge.len() == 2 {
                        edges.push((edge[0].clone(), edge[1].clone()));
                    }
                }
            }

            if !edges.is_empty() {
                expectations = expectations.with_graph(GraphExpectation::new(edges));
            }
        }

        // Build count expectations
        if let Some(ref counts_config) = expect.counts {
            let mut count_exp = CountExpectation::new();

            // Total span count
            if let Some(ref total) = counts_config.spans_total {
                if let Some(eq) = total.eq {
                    count_exp = count_exp.with_spans_total(
                        crate::validation::count_validator::CountBound::eq(eq),
                    );
                } else if let Some(gte) = total.gte {
                    if let Some(lte) = total.lte {
                        count_exp = count_exp.with_spans_total(
                            crate::validation::count_validator::CountBound::range(gte, lte)?,
                        );
                    } else {
                        count_exp = count_exp.with_spans_total(
                            crate::validation::count_validator::CountBound::gte(gte),
                        );
                    }
                } else if let Some(lte) = total.lte {
                    count_exp = count_exp.with_spans_total(
                        crate::validation::count_validator::CountBound::lte(lte),
                    );
                }
            }

            // Per-name counts
            if let Some(ref by_name) = counts_config.by_name {
                for (name, bound_config) in by_name {
                    if let Some(eq) = bound_config.eq {
                        count_exp = count_exp.with_name_count(
                            name.clone(),
                            crate::validation::count_validator::CountBound::eq(eq),
                        );
                    } else if let Some(gte) = bound_config.gte {
                        if let Some(lte) = bound_config.lte {
                            count_exp = count_exp.with_name_count(
                                name.clone(),
                                crate::validation::count_validator::CountBound::range(
                                    gte, lte,
                                )?,
                            );
                        } else {
                            count_exp = count_exp.with_name_count(
                                name.clone(),
                                crate::validation::count_validator::CountBound::gte(gte),
                            );
                        }
                    }
                }
            }

            expectations = expectations.with_counts(count_exp);
        }

        // Build window expectations
        for window_config in &expect.window {
            let window =
                WindowExpectation::new(&window_config.outer, window_config.contains.clone());
            expectations = expectations.add_window(window);
        }

        // Build hermeticity expectations
        if let Some(ref hermetic_config) = expect.hermeticity {
            let hermetic = HermeticityExpectation {
                no_external_services: hermetic_config.no_external_services,
                resource_attrs_must_match: hermetic_config
                    .resource_attrs
                    .as_ref()
                    .and_then(|ra| ra.must_match.clone()),
                sdk_resource_attrs_must_match: None,
                span_attrs_forbid_keys: hermetic_config
                    .span_attrs
                    .as_ref()
                    .and_then(|sa| sa.forbid_keys.clone()),
            };
            expectations = expectations.with_hermeticity(hermetic);
        }
    }

    Ok(expectations)
}
