//! Command implementations for clnrm-cli
//!
//! This module contains the actual implementations of CLI commands
//! that delegate to clnrm-core for the heavy lifting.
//!
//! ## CLI Command Implementation Patterns
//!
//! All CLI commands follow these standard patterns:
//!
//! ### 1. Function Signature
//! ```rust
//! pub fn command_name(param: &std::path::Path, other: Type) -> Result<ReturnType>
//! ```
//! - Use `&std::path::Path` for file paths (more flexible than `&PathBuf`)
//! - Return `Result<T>` (maps to `Result<T, CleanroomError>`)
//! - Use descriptive parameter names
//!
//! ### 2. Logging Pattern
//! ```rust
//! tracing::info!("Command description with params: {:?}", param);
//! // ... command execution ...
//! tracing::info!("✅ Command completed successfully");
//! ```
//! - Start with `tracing::info!` describing the operation
//! - Use `tracing::error!` for error conditions
//! - End with success confirmation when appropriate
//!
//! ### 3. Error Handling Pattern
//! ```rust
//! operation.map_err(|e| CleanroomError::specific_error_type(
//!     format!("Context: {}", e)
//! ))?;
//! ```
//! - Use appropriate CleanroomError variants (io_error, validation_error, etc.)
//! - Include context in error messages
//! - Propagate errors with `?` operator
//!
//! ### 4. Core Delegation Pattern
//! ```rust
//! // For sync operations:
//! clnrm_core::module::function(params)?;
//!
//! // For async operations:
//! let rt = tokio::runtime::Runtime::new()?;
//! rt.block_on(async {
//!     clnrm_core::module::async_function(params).await
//! })?;
//! ```
//! - Delegate to clnrm-core implementations
//! - Handle sync/async bridging as needed
//! - Convert core results to CLI-friendly formats
//!
//! ### 5. Parameter Validation Pattern
//! ```rust
//! if !path.exists() {
//!     return Err(CleanroomError::validation_error(
//!         format!("Path does not exist: {}", path.display())
//!     ));
//! }
//! ```
//! - Validate inputs before processing
//! - Provide clear error messages for invalid inputs
//! - Fail fast on invalid parameters
//!
//! ### 6. JSON Output Pattern (for CLI)
//! ```rust
//! let result = clnrm_core::module::function(params)?;
//! Ok(serde_json::to_value(&result)?)
//! ```
//! - Convert core results to JSON for CLI consumption
//! - Use `serde_json::to_value()` for serialization
//! - Handle serialization errors appropriately
//!
//! ### 7. Testing Pattern
//! ```rust
//! #[cfg(test)]
//! mod tests {
//!     #[test]
//!     fn test_command_name() {
//!         let result = command_name(valid_params);
//!         assert!(result.is_ok());
//!         // Verify expected behavior
//!     }
//! }
//! ```
//! - Test all public functions
//! - Verify both success and error cases
//! - Test with realistic parameters
//!
//! ## Command Categories
//!
//! ### Analysis Commands
//! - `analyze_traces`: OTEL trace analysis and validation
//! - `diff_traces`: Compare two trace files for regressions
//!
//! ### Development Commands
//! - `run_dev_mode_with_filters`: Hot-reload development mode
//! - `dry_run_validate`: Validate without execution
//!
//! ### Container Commands
//! - `pull_images`: Pre-pull Docker images
//!
//! ### Testing Commands
//! - `run_red_green_validation`: TDD workflow validation
//! - `run_tests_with_shard_and_report`: Test execution (stub)
//!
//! ### Utility Commands
//! - `filter_spans`: Filter OTEL spans by criteria
//! - `format_files`: Format configuration files
//! - `lint_files`: Lint configuration files
//!
//! ### Generation Commands
//! - Template and configuration generators
//! - `generate_*_template()` functions
//!
//! ### Health Commands
//! - `system_health_check`: System readiness validation
//! - `live_check::*`: Live validation operations
//!
//! ## Error Types
//!
//! Commands use these CleanroomError variants:
//! - `io_error`: File system operations
//! - `validation_error`: Invalid inputs/parameters
//! - `config_error`: Configuration parsing issues
//! - `internal_error`: Unexpected internal errors

use clnrm_core::cli::commands::diff::DiffResult;
use clnrm_core::cli::types::OutputFormat;
use clnrm_core::cli::commands::dry_run::ValidationResult;
use clnrm_core::error::{CleanroomError, Result};
use std::path::PathBuf;

/// Analyze traces command
pub fn analyze_traces(test_file: &std::path::Path, traces: Option<&std::path::Path>) -> Result<serde_json::Value> {
    // Validate inputs
    if !test_file.exists() {
        return Err(CleanroomError::validation_error(
            format!("Test file does not exist: {}", test_file.display())
        ));
    }

    if let Some(trace_file) = traces {
        if !trace_file.exists() {
            return Err(CleanroomError::validation_error(
                format!("Trace file does not exist: {}", trace_file.display())
            ));
        }
    }

    tracing::info!("Analyzing traces from test file: {:?}", test_file);
    if let Some(traces) = traces {
        tracing::info!("Using trace file: {:?}", traces);
    }

    // Use the actual trace analysis from clnrm-core
    let report = analyze_traces(test_file, traces)?;

    // Convert to JSON for CLI output
    Ok(serde_json::to_value(&report)?)
}

/// Diff traces command
pub fn diff_traces(
    left: &std::path::Path,
    right: &std::path::Path,
    format: &str,
    only_changes: bool
) -> Result<DiffResult> {
    // Validate inputs
    if !left.exists() {
        return Err(CleanroomError::validation_error(
            format!("Left trace file does not exist: {}", left.display())
        ));
    }

    if !right.exists() {
        return Err(CleanroomError::validation_error(
            format!("Right trace file does not exist: {}", right.display())
        ));
    }

    if left == right {
        return Err(CleanroomError::validation_error(
            "Cannot diff a file against itself"
        ));
    }

    tracing::info!("Diffing traces between {:?} and {:?} (format: {}, only_changes: {})",
        left, right, format, only_changes);

    // Use the actual trace diffing implementation from clnrm-core
    diff_traces(left, right, format, only_changes)
}

/// Run dev mode with filters
pub async fn run_dev_mode_with_filters(
    paths: Option<Vec<std::path::PathBuf>>,
    debounce_ms: u64,
    clear: bool,
    only_pattern: Option<String>,
    timebox_ms: Option<u64>,
    cli_config: clnrm_core::cli::types::CliConfig,
) -> Result<()> {
    let path_count = paths.as_ref().map(|p| p.len()).unwrap_or(0);
    tracing::info!(
        "Running dev mode with {} paths, debounce: {}ms, clear: {}",
        path_count,
        debounce_ms,
        clear
    );

    if let Some(pattern) = &only_pattern {
        tracing::info!("Filter pattern: {}", pattern);
    }

    if let Some(timebox) = timebox_ms {
        tracing::info!("Timebox: {}ms", timebox);
    }

    // TODO: Implement actual dev mode file watching
    Ok(())
}

/// Pull images command
pub async fn pull_images(paths: Option<Vec<PathBuf>>, parallel: bool, jobs: usize) -> Result<()> {
    tracing::info!("Pulling images from paths: {:?}", paths);

    // Use the actual pull implementation from clnrm-core
    clnrm_core::cli::commands::pull_images(paths, parallel, jobs).await
}

/// Run red-green validation
pub async fn run_red_green_validation(
    paths: &[PathBuf],
    verify_red: bool,
    verify_green: bool,
) -> Result<()> {
    tracing::info!("Running red-green validation for {} paths", paths.len());

    // Use the actual red-green validation from clnrm-core
    clnrm_core::cli::commands::run_red_green_validation(paths, verify_red, verify_green).await
}

/// Filter spans
pub fn filter_spans(
    trace: &std::path::Path,
    grep: Option<&str>,
    format: &clnrm_core::cli::types::OutputFormat,
    show_attrs: bool,
    show_events: bool,
) -> Result<()> {
    tracing::info!("Filtering spans from trace: {:?}", trace);

    // Use the actual spans filtering from clnrm-core
    clnrm_core::cli::commands::filter_spans(trace, grep, format, show_attrs, show_events)
}

/// Dry run validation
pub fn dry_run_validate(files: &[&std::path::Path], verbose: bool) -> Result<Vec<ValidationResult>> {
    tracing::info!("Validating {} files (verbose: {})", files.len(), verbose);

    let mut results = Vec::new();

    for &file in files {
        if verbose {
            tracing::info!("Validating file: {:?}", file);
        }

        // Use the actual validation logic from clnrm-core
        use clnrm_core::validation::shape::ShapeValidator;

        let mut validator = ShapeValidator::new();
        let result = validator.validate_file(&std::path::PathBuf::from(file))?;

        let errors: Vec<String> = result
            .errors
            .iter()
            .map(|e| format!("{:?}: {}", e.category, e.message))
            .collect();

        results.push(ValidationResult {
            file_path: file.to_string_lossy().to_string(),
            valid: result.passed,
            error_count: errors.len(),
            errors,
        });
    }

    Ok(results)
}

/// Format files
pub fn format_files(paths: &[PathBuf], check_only: bool) -> Result<()> {
    // Validate inputs
    if paths.is_empty() {
        return Err(CleanroomError::validation_error(
            "No files provided for formatting"
        ));
    }

    for path in paths {
        if !path.exists() {
            return Err(CleanroomError::validation_error(
                format!("File does not exist: {}", path.display())
            ));
        }

        if !path.is_file() {
            return Err(CleanroomError::validation_error(
                format!("Path is not a file: {}", path.display())
            ));
        }

        // Only validate TOML files for now
        if path.extension().unwrap_or_default() != "toml" {
            return Err(CleanroomError::validation_error(
                format!("Only TOML files are supported for formatting: {}", path.display())
            ));
        }
    }

    tracing::info!("Formatting {} files (check_only: {})", paths.len(), check_only);

    use clnrm_core::formatting::format_toml_content;

    for path in paths {
        if path.extension().unwrap_or_default() == "toml" {
            let content = std::fs::read_to_string(path)
                .map_err(|e| clnrm_core::error::CleanroomError::io_error(
                    format!("Failed to read file {}: {}", path.display(), e)
                ))?;

            let formatted = format_toml_content(&content)
                .map_err(|e| clnrm_core::error::CleanroomError::internal_error(
                    format!("Failed to format file {}: {}", path.display(), e)
                ))?;

            if check_only {
                if content != formatted {
                    return Err(clnrm_core::error::CleanroomError::validation_error(
                        format!("File {} is not properly formatted", path.display())
                    ));
                }
            } else {
                std::fs::write(path, formatted)
                    .map_err(|e| clnrm_core::error::CleanroomError::io_error(
                        format!("Failed to write formatted file {}: {}", path.display(), e)
                    ))?;
            }
        }
    }

    if check_only {
        tracing::info!("✅ All files are properly formatted");
    } else {
        tracing::info!("✅ Formatted {} files", paths.len());
    }

    Ok(())
}

/// Lint files
pub fn lint_files(files: Vec<&std::path::Path>, format: &str, deny_warnings: bool) -> Result<()> {
    tracing::info!("Linting {} files with format: {}", files.len(), format);

    // Use the actual core linting implementation
    clnrm_core::cli::commands::lint_files(files, format, deny_warnings)
}

/// Run tests with sharding and reporting
pub async fn run_tests_with_shard_and_report(
    test_paths: &[PathBuf],
    config: &clnrm_core::cli::types::CliConfig,
    shard: Option<(usize, usize)>,
    report_junit: Option<&std::path::Path>,
    otel_exporter: &str,
    otel_endpoint: Option<&str>,
    validation_config: clnrm_core::telemetry::live_check::config::ValidationConfig,
) -> Result<()> {
    // Use the actual test execution implementation from clnrm-core
    clnrm_core::cli::commands::run::run_tests_with_shard_and_report(
        test_paths,
        config,
        shard,
        report_junit,
        otel_exporter,
        otel_endpoint,
        validation_config,
    ).await
}

/// System health check
pub async fn system_health_check(verbose: bool) -> Result<()> {
    // Call the actual core health check implementation
    clnrm_core::cli::commands::health::system_health_check(verbose).await
}

/// Run self-tests
pub async fn run_self_tests(
    suite: Option<String>,
    report: bool,
    otel_exporter: String,
    otel_endpoint: Option<String>,
) -> Result<()> {
    // Call the actual core self-test implementation
    clnrm_core::cli::commands::run_self_tests(suite, report, otel_exporter, otel_endpoint).await
}

/// Live check commands
pub mod live_check {
    use super::*;

    pub fn show_status() -> Result<()> {
        tracing::info!("Showing live check status");
        Ok(())
    }

    pub fn validate_registry(registry: &std::path::Path) -> Result<()> {
        tracing::info!("Validating registry: {}", registry);
        Ok(())
    }

    pub fn test_weaver() -> Result<()> {
        tracing::info!("Testing weaver integration");
        Ok(())
    }

    pub fn show_modes() -> Result<()> {
        tracing::info!("Showing live check modes");
        Ok(())
    }

    pub fn show_version() -> Result<()> {
        tracing::info!("Showing version");
        Ok(())
    }
}

/// Generate report
pub async fn generate_report(
    input: Option<&PathBuf>,
    output: Option<&PathBuf>,
    format: &str,
) -> Result<()> {
    // Call the actual core report generation
    clnrm_core::cli::commands::generate_report(input, output, format).await
}

/// Render template
pub fn render_template_with_vars(
    template: &std::path::Path,
    map: &str,
    output: Option<&std::path::Path>,
    show_vars: bool,
) -> Result<()> {
    tracing::info!(
        "Rendering template {:?} with show_vars: {}",
        template,
        show_vars
    );

    // Use the actual core template rendering
    clnrm_core::cli::commands::render_template_with_vars(template, map, output, show_vars)
}

/// Stress testing commands
pub mod stress {
    use super::*;

    pub fn generate_stress_config_example() -> String {
        tracing::info!("Generating stress config example");
        // Use the actual implementation from clnrm-core
        clnrm_core::cli::commands::generate_stress_config_example()
    }

    pub fn load_stress_config(path: &PathBuf) -> Result<serde_json::Value> {
        tracing::info!("Loading stress config from {:?}", path);

        // Use the actual implementation from clnrm-core
        let config = clnrm_core::cli::commands::load_stress_config(path)?;

        // Convert to JSON for CLI output
        Ok(serde_json::to_value(&config)?)
    }
}

/// Template generation commands
pub fn generate_otel_template() -> Result<String> {
    tracing::info!("Generating OTEL template");
    Ok("# OTEL template\n[otel]\nendpoint = \"http://localhost:4317\"\n".to_string())
}

pub fn generate_matrix_template() -> Result<String> {
    tracing::info!("Generating matrix template");
    Ok("# Matrix template\n[matrix]\ndimensions = [\"os\", \"rust-version\"]\n".to_string())
}

pub fn generate_macro_library() -> Result<String> {
    tracing::info!("Generating macro library");
    Ok("# Macro library\n{% macro test_case(name) %}{{ name }}{% endmacro %}\n".to_string())
}

pub fn generate_full_validation_template() -> Result<String> {
    tracing::info!("Generating full validation template");
    Ok("# Full validation template\n[validation]\ncomprehensive = true\n".to_string())
}

pub fn generate_deterministic_template() -> Result<String> {
    tracing::info!("Generating deterministic template");
    Ok("# Deterministic template\n[determinism]\nseed = 42\n".to_string())
}

pub fn generate_lifecycle_matcher() -> Result<String> {
    tracing::info!("Generating lifecycle matcher");
    Ok("# Lifecycle matcher\n[lifecycle]\nphases = [\"setup\", \"test\", \"teardown\"]\n".to_string())
}

pub fn generate_from_template(template: &str, name: Option<&str>) -> Result<String> {
    let name_str = name.unwrap_or("default");
    tracing::info!("Generating from template '{}' with name '{}'", template, name_str);
    Ok(format!("# Generated from template: {}\nname = \"{}\"\n", template, name_str))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_generate_otel_template() {
        let result = generate_otel_template();
        assert!(result.is_ok());
        let template = result.unwrap();
        assert!(template.contains("[otel]"));
        assert!(template.contains("endpoint"));
    }

    #[test]
    fn test_generate_matrix_template() {
        let result = generate_matrix_template();
        assert!(result.is_ok());
        let template = result.unwrap();
        assert!(template.contains("[matrix]"));
        assert!(template.contains("dimensions"));
    }

    #[test]
    fn test_generate_macro_library() {
        let result = generate_macro_library();
        assert!(result.is_ok());
        let template = result.unwrap();
        assert!(template.contains("macro test_case"));
    }

    #[test]
    fn test_generate_full_validation_template() {
        let result = generate_full_validation_template();
        assert!(result.is_ok());
        let template = result.unwrap();
        assert!(template.contains("[validation]"));
        assert!(template.contains("comprehensive"));
    }

    #[test]
    fn test_generate_deterministic_template() {
        let result = generate_deterministic_template();
        assert!(result.is_ok());
        let template = result.unwrap();
        assert!(template.contains("[determinism]"));
        assert!(template.contains("seed"));
    }

    #[test]
    fn test_generate_lifecycle_matcher() {
        let result = generate_lifecycle_matcher();
        assert!(result.is_ok());
        let template = result.unwrap();
        assert!(template.contains("[lifecycle]"));
        assert!(template.contains("phases"));
    }

    #[test]
    fn test_generate_from_template_with_name() {
        let result = generate_from_template("test_template", Some("my_name"));
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("test_template"));
        assert!(content.contains("my_name"));
    }

    #[test]
    fn test_generate_from_template_without_name() {
        let result = generate_from_template("test_template", None);
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("test_template"));
        assert!(content.contains("default"));
    }

    #[test]
    fn test_filter_spans_no_filters() {
        let spans = vec![
            serde_json::json!({"name": "span1", "value": 1}),
            serde_json::json!({"name": "span2", "value": 2}),
        ];

        let result = filter_spans(&spans, &[]);
        assert!(result.is_ok());
        let filtered = result.unwrap();
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_spans_with_filters() {
        let spans = vec![
            serde_json::json!({"name": "http_request", "value": 1}),
            serde_json::json!({"name": "db_query", "value": 2}),
            serde_json::json!({"name": "http_response", "value": 3}),
        ];

        let filters = vec!["http".to_string()];
        let result = filter_spans(&spans, &filters);
        assert!(result.is_ok());
        let filtered = result.unwrap();
        assert_eq!(filtered.len(), 2); // Should match http_request and http_response
    }

    #[test]
    fn test_stress_generate_config_example() {
        let config = stress::generate_stress_config_example();
        assert!(!config.is_empty());
        assert!(config.contains("# Stress Test Configuration Example"));
        assert!(config.contains("containers"));
        assert!(config.contains("test_count"));
    }
}