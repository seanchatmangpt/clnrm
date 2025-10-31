//! Single test execution functionality
//!
//! Handles execution of individual test files with proper error handling,
//! template rendering, and service management.

use crate::cleanroom::CleanroomEnvironment;
use crate::cli::types::CliConfig;
use crate::error::{CleanroomError, Result};
use crate::telemetry::spans;
use crate::telemetry::span_storage;
use crate::validation::span_validator::SpanValidator;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info, warn};

use super::{scenario, services};

/// Run a single test file
///
/// Returns: Ok(Some(container_id)) on success with container ID for telemetry,
///          Ok(None) if no container was used,
///          Err on failure
#[tracing::instrument(name = "clnrm.test", skip(_config), fields(test.hermetic = true))]
pub async fn run_single_test(path: &PathBuf, _config: &CliConfig) -> Result<Option<String>> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        CleanroomError::config_error(format!("Failed to read config file: {}", e))
    })?;

    let test_config: crate::config::TestConfig = toml::from_str(&content)
        .map_err(|e| CleanroomError::config_error(format!("TOML parse error: {}", e)))?;

    // FAIL FAST: Validate config immediately after parsing (catches unsupported features)
    test_config.validate()?;

    let test_name = test_config.get_name()?;

    tracing::Span::current().record("test.name", &test_name);

    // Clear span storage for this test run (critical for validation isolation)
    span_storage::clear_collected_spans();

    info!("🚀 Executing test: {}", test_name);
    info!("🚀 Executing test: {}", test_name);

    if let Some(description) = test_config.get_description() {
        info!("📝 Description: {}", description);
        debug!("Test description: {}", description);
    }

    // Create template renderer with vars from test config
    let mut template_renderer = crate::TemplateRenderer::new()?;
    if let Some(vars) = &test_config.vars {
        template_renderer.merge_user_vars(vars.clone());
    }

    // Load cleanroom configuration for default container settings
    let cleanroom_config = match crate::config::load_cleanroom_config() {
        Ok(config) => {
            info!(
                "Successfully loaded cleanroom config with default_image: {}",
                config.containers.default_image
            );
            Some(config)
        }
        Err(e) => {
            info!("Failed to load cleanroom config: {}, using defaults", e);
            None
        }
    };

    let environment = CleanroomEnvironment::with_config(cleanroom_config)
        .await
        .map_err(|e| {
            CleanroomError::internal_error("Failed to create test environment")
                .with_context("Test execution requires cleanroom environment")
                .with_source(e.to_string())
        })?;

    // Track first container ID for telemetry (CRITICAL proof attribute)
    let mut first_container_id: Option<String> = None;

    // Load services from config (support both v0.4.x [services] and v1.0 [service] formats)
    let service_handles = if let Some(services) = &test_config.services {
        services::load_services_from_config(&environment, services).await?
    } else if let Some(services) = &test_config.service {
        // v1.0 format: [service.name]
        services::load_services_from_config(&environment, services).await?
    } else {
        HashMap::new()
    };

    // Execute test steps
    for (i, step) in test_config.steps.iter().enumerate() {
        info!("📋 Step {}: {}", i + 1, step.name);

        if step.command.is_empty() {
            return Err(CleanroomError::validation_error(format!(
                "Step '{}' has empty command",
                step.name
            )));
        }

        // Render command templates with vars context
          let rendered_command: Vec<String> = step
            .command
              .iter()
              .map(|arg| template_renderer.render_str(arg, &format!("step_{}_arg", step.name)).map_err(|e| e.into()))
              .collect::<std::result::Result<Vec<String>, CleanroomError>>()?;

        info!("🔧 Executing: {}", rendered_command.join(" "));
        info!("🔧 Executing: {}", rendered_command.join(" "));

        let command_span = spans::command_execute_span(&rendered_command.join(" "));

        let _command_guard = command_span.enter();

        let stdout = {
            // Execute command in a fresh container for proper isolation
            // Core Team Compliance: Use async for I/O, proper error handling, no unwrap/expect
            let container_name = format!("test-{}-step-{}", test_name, step.name);
            let execution_result = environment
                .execute_in_container(
                    &container_name,
                    &rendered_command,
                    step.workdir.as_deref(),
                    step.env.as_ref(),
                )
                .await
                .map_err(|e| {
                    CleanroomError::container_error(format!(
                        "Failed to execute command '{}' in container '{}': {}",
                        rendered_command.join(" "),
                        container_name,
                        e
                    ))
                })?;

            let stdout = &execution_result.stdout;
            let stderr = &execution_result.stderr;

            // Capture container ID for telemetry (first one wins)
            if first_container_id.is_none() {
                first_container_id = execution_result.container_id.clone();
            }

            if !stderr.is_empty() {
                warn!("⚠️  Stderr: {}", stderr.trim());
                info!("⚠️  Stderr: {}", stderr.trim());
            }

            // Validate exit code (default to 0 if not specified)
            let expected_exit_code = step.expected_exit_code.unwrap_or(0);
            if execution_result.exit_code != expected_exit_code {
                return Err(CleanroomError::validation_error(format!(
                    "Step '{}' exited with code {} but expected {}",
                    step.name, execution_result.exit_code, expected_exit_code
                )));
            }

            stdout.to_string()
        };

        info!("📤 Output: {}", stdout.trim());
        info!("📤 Output: {}", stdout.trim());

        if let Some(regex) = &step.expected_output_regex {
            debug!("Expected output regex: {}", regex);
            let re = regex::Regex::new(regex).map_err(|e| {
                CleanroomError::validation_error(format!(
                    "Invalid regex '{}' in step '{}': {}",
                    regex, step.name, e
                ))
            })?;

            // Trim output before regex match to handle trailing newlines from echo
            let trimmed_output = stdout.trim();
            if !re.is_match(trimmed_output) {
                return Err(CleanroomError::validation_error(format!(
                    "Step '{}' output did not match expected regex '{}'. Output: {}",
                    step.name, regex, trimmed_output
                )));
            }
            info!("✅ Output matches expected regex");
        }

        info!("✅ Step '{}' completed successfully", step.name);
    }

    // Execute scenario blocks (v1.0 format)
    if !test_config.scenario.is_empty() {
        info!("📋 Executing {} scenario(s)", test_config.scenario.len());

        for scenario in &test_config.scenario {
            scenario::execute_scenario(scenario, &environment, &service_handles, &test_config)
                .await?;
        }
    }

    // Cleanup services
    let service_handles_vec: Vec<_> = service_handles.iter().collect();
    for (service_name, handle) in service_handles_vec.iter().rev() {
        match environment.stop_service(&handle.id).await {
            Ok(()) => {
                info!("🛑 Service '{}' stopped successfully", service_name);
            }
            Err(e) => {
                warn!("⚠️  Failed to stop service '{}': {}", service_name, e);
            }
        }
    }

    // ========================================
    // CRITICAL: Validate span expectations if configured
    // ========================================
    if let Some(ref expect) = test_config.expect {
        if !expect.span.is_empty() {
            info!("🔍 Validating {} span expectation(s)...", expect.span.len());

            // Get collected spans from telemetry
            let collected_spans = span_storage::get_collected_spans();

            if collected_spans.is_empty() {
                warn!("⚠️  No spans collected - span validation skipped");
                warn!("    Enable OTEL exporter to validate spans");
            } else {
                // Convert SpanData to validation format
                let validator = SpanValidator::from_span_data(&collected_spans)?;

                info!("📊 Collected {} span(s) for validation", collected_spans.len());

                // Validate expectations
                let validation_result = validator.validate_expectations(&expect.span)?;

                if !validation_result.passed {
                    // Format error with all failures
                    let failure_messages: Vec<String> = validation_result
                        .failures
                        .iter()
                        .map(|f| format!("  - [{}] {}", f.rule, f.message))
                        .collect();

                    return Err(CleanroomError::validation_error(format!(
                        "Span validation failed ({} failures):\n{}",
                        validation_result.failures.len(),
                        failure_messages.join("\n")
                    )));
                }

                info!(
                    "✅ All span expectations validated ({} checks passed)",
                    validation_result.validations_count
                );
            }
        }
    }

    info!("🎉 Test '{}' completed successfully!", test_name);
    info!("🎉 Test '{}' completed successfully!", test_name);

    // Return container ID for telemetry emission
    Ok(first_container_id)
}
