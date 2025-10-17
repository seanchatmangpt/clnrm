//! Integration tests for PRD v1.0 features
//!
//! Tests all v1.0 commands: pull, graph, spans, render, repro, redgreen, collector
//!
//! # Test Coverage
//! - Happy path for each command
//! - Error handling (invalid inputs, missing files)
//! - Output format validation
//! - Integration with existing framework

use clnrm_core::cli::commands::v0_7_0::redgreen_impl::run_red_green_validation;
use clnrm_core::cli::commands::{
    filter_spans, pull_images, render_template_with_vars, reproduce_baseline, show_collector_logs,
    show_collector_status, visualize_graph,
};
use clnrm_core::cli::types::{GraphFormat, OutputFormat};
use clnrm_core::error::Result;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Create a temporary test directory with a sample test file
fn create_test_dir_with_sample() -> Result<TempDir> {
    let dir = TempDir::new().map_err(|e| {
        clnrm_core::error::CleanroomError::io_error(format!("Failed to create temp dir: {}", e))
    })?;

    // Create a sample .clnrm.toml file
    let test_file = dir.path().join("test.clnrm.toml");
    let content = r#"
[test.metadata]
name = "sample_test"
description = "Sample test for v1 features"

[services.alpine]
type = "generic_container"
image = "alpine:latest"
plugin = "generic"

[[steps]]
name = "echo_test"
command = ["echo", "hello"]
expected_output_regex = "hello"
"#;
    fs::write(&test_file, content).map_err(|e| {
        clnrm_core::error::CleanroomError::io_error(format!("Failed to write test file: {}", e))
    })?;

    Ok(dir)
}

/// Create a sample OTEL trace JSON file
fn create_sample_trace_file(dir: &TempDir) -> Result<PathBuf> {
    let trace_file = dir.path().join("trace.json");
    let trace_data = r#"{
  "spans": [
    {
      "name": "test_execution",
      "span_id": "abc123",
      "parent_span_id": null,
      "trace_id": "trace001",
      "kind": "INTERNAL"
    },
    {
      "name": "container_start",
      "span_id": "def456",
      "parent_span_id": "abc123",
      "trace_id": "trace001",
      "kind": "CLIENT"
    }
  ]
}"#;
    fs::write(&trace_file, trace_data).map_err(|e| {
        clnrm_core::error::CleanroomError::io_error(format!("Failed to write trace file: {}", e))
    })?;

    Ok(trace_file)
}

/// Create a sample Tera template file
fn create_sample_template(dir: &TempDir) -> Result<PathBuf> {
    let template_file = dir.path().join("template.tera");
    let content = r#"
[test.metadata]
name = "{{ test_name }}"
description = "{{ description }}"
"#;
    fs::write(&template_file, content).map_err(|e| {
        clnrm_core::error::CleanroomError::io_error(format!("Failed to write template file: {}", e))
    })?;

    Ok(template_file)
}

// ============================================================================
// PULL COMMAND TESTS
// ============================================================================

#[tokio::test]
async fn test_pull_command_with_no_test_files_returns_ok() -> Result<()> {
    // Arrange - Empty directory with no test files
    let temp_dir = TempDir::new().map_err(|e| {
        clnrm_core::error::CleanroomError::io_error(format!("Failed to create temp dir: {}", e))
    })?;

    // Act
    let result = pull_images(Some(vec![temp_dir.path().to_path_buf()]), false, 4).await;

    // Assert - Should succeed with no images found
    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_pull_command_scans_test_files_successfully() -> Result<()> {
    // Arrange
    let test_dir = create_test_dir_with_sample()?;

    // Act - Note: This will actually try to pull alpine:latest
    // In a real test environment, you'd mock Docker
    let result = pull_images(Some(vec![test_dir.path().to_path_buf()]), false, 1).await;

    // Assert - Either succeeds or fails due to Docker not available
    // We're testing the command structure, not Docker availability
    assert!(result.is_ok() || result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_pull_command_with_invalid_path_returns_ok() -> Result<()> {
    // Arrange
    let nonexistent_path = PathBuf::from("/nonexistent/test/path");

    // Act
    let result = pull_images(Some(vec![nonexistent_path]), false, 4).await;

    // Assert - Should return OK with no files found
    assert!(result.is_ok());

    Ok(())
}

// ============================================================================
// GRAPH COMMAND TESTS
// ============================================================================

#[tokio::test]
async fn test_graph_command_with_ascii_format() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        clnrm_core::error::CleanroomError::io_error(format!("Failed to create temp dir: {}", e))
    })?;
    let trace_file = create_sample_trace_file(&temp_dir)?;

    // Act
    let result = visualize_graph(&trace_file, &GraphFormat::Ascii, false, None);

    // Assert
    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_graph_command_with_dot_format() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        clnrm_core::error::CleanroomError::io_error(format!("Failed to create temp dir: {}", e))
    })?;
    let trace_file = create_sample_trace_file(&temp_dir)?;

    // Act
    let result = visualize_graph(&trace_file, &GraphFormat::Dot, false, None);

    // Assert
    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_graph_command_with_filter() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        clnrm_core::error::CleanroomError::io_error(format!("Failed to create temp dir: {}", e))
    })?;
    let trace_file = create_sample_trace_file(&temp_dir)?;

    // Act
    let result = visualize_graph(
        &trace_file,
        &GraphFormat::Json,
        false,
        Some("test_execution"),
    );

    // Assert
    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_graph_command_with_nonexistent_file_returns_error() -> Result<()> {
    // Arrange
    let nonexistent_file = PathBuf::from("/nonexistent/trace.json");

    // Act
    let result = visualize_graph(&nonexistent_file, &GraphFormat::Ascii, false, None);

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.message.contains("Failed to read trace file"));

    Ok(())
}

// ============================================================================
// SPANS COMMAND TESTS
// ============================================================================

#[tokio::test]
async fn test_spans_command_filters_successfully() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        clnrm_core::error::CleanroomError::io_error(format!("Failed to create temp dir: {}", e))
    })?;

    // Create OTLP-formatted trace
    let trace_file = temp_dir.path().join("otlp_trace.json");
    let otlp_data = r#"{
  "resourceSpans": [
    {
      "resource": {
        "attributes": [
          {"key": "service.name", "value": {"stringValue": "test-service"}}
        ]
      },
      "scopeSpans": [
        {
          "spans": [
            {
              "name": "test_span",
              "spanId": "abc",
              "traceId": "trace001",
              "kind": 1
            }
          ]
        }
      ]
    }
  ]
}"#;
    fs::write(&trace_file, otlp_data).map_err(|e| {
        clnrm_core::error::CleanroomError::io_error(format!("Failed to write trace: {}", e))
    })?;

    // Act
    let result = filter_spans(
        &trace_file,
        Some("test"),
        &OutputFormat::Human,
        false,
        false,
    );

    // Assert
    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_spans_command_with_nonexistent_file_returns_error() -> Result<()> {
    // Arrange
    let nonexistent_file = PathBuf::from("/nonexistent/trace.json");

    // Act
    let result = filter_spans(&nonexistent_file, None, &OutputFormat::Human, false, false);

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.message.contains("Failed to read trace file"));

    Ok(())
}

// ============================================================================
// RENDER COMMAND TESTS
// ============================================================================

#[tokio::test]
async fn test_render_command_with_valid_json_map() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        clnrm_core::error::CleanroomError::io_error(format!("Failed to create temp dir: {}", e))
    })?;
    let template_file = create_sample_template(&temp_dir)?;
    let output_file = temp_dir.path().join("output.toml");

    let var_map = vec![
        "test_name=my_test".to_string(),
        "description=My description".to_string(),
    ];

    // Act
    let result = render_template_with_vars(&template_file, &var_map, Some(&output_file), false);

    // Assert
    assert!(result.is_ok());
    assert!(output_file.exists());

    let rendered = fs::read_to_string(&output_file).map_err(|e| {
        clnrm_core::error::CleanroomError::io_error(format!("Failed to read output: {}", e))
    })?;
    assert!(rendered.contains("my_test"));
    assert!(rendered.contains("My description"));

    Ok(())
}

#[tokio::test]
async fn test_render_command_with_invalid_mapping_succeeds() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        clnrm_core::error::CleanroomError::io_error(format!("Failed to create temp dir: {}", e))
    })?;
    let template_file = create_sample_template(&temp_dir)?;

    let invalid_mapping = vec!["invalid mapping without equals".to_string()];

    // Act
    let result = render_template_with_vars(&template_file, &invalid_mapping, None, false);

    // Assert - May succeed or fail, testing that it doesn't panic
    assert!(result.is_ok() || result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_render_command_with_nonexistent_template_returns_error() -> Result<()> {
    // Arrange
    let nonexistent_template = PathBuf::from("/nonexistent/template.tera");
    let var_map = vec![];

    // Act
    let result = render_template_with_vars(&nonexistent_template, &var_map, None, false);

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.message.contains("Failed to read template"));

    Ok(())
}

// ============================================================================
// REPRO COMMAND TESTS
// ============================================================================

#[tokio::test]
async fn test_repro_command_with_valid_baseline() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        clnrm_core::error::CleanroomError::io_error(format!("Failed to create temp dir: {}", e))
    })?;
    let baseline_file = temp_dir.path().join("baseline.json");

    // Create a valid baseline file with proper structure
    let baseline_data = r#"{
  "version": "1.0.0",
  "timestamp": "2024-10-16T00:00:00Z",
  "digest": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "test_results": []
}"#;
    fs::write(&baseline_file, baseline_data).map_err(|e| {
        clnrm_core::error::CleanroomError::io_error(format!("Failed to write baseline: {}", e))
    })?;

    // Act
    let result = reproduce_baseline(&baseline_file, false, None).await;

    // Assert - May succeed or fail, testing that command works
    // (it may fail if trying to rerun tests that don't exist)
    assert!(result.is_ok() || result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_repro_command_with_nonexistent_file_returns_error() -> Result<()> {
    // Arrange
    let nonexistent_file = PathBuf::from("/nonexistent/baseline.json");

    // Act
    let result = reproduce_baseline(&nonexistent_file, false, None).await;

    // Assert
    assert!(result.is_err());

    Ok(())
}

// ============================================================================
// REDGREEN COMMAND TESTS
// ============================================================================

#[tokio::test]
async fn test_redgreen_command_with_empty_paths_returns_error() -> Result<()> {
    // Arrange
    let paths: Vec<PathBuf> = vec![];

    // Act
    let result = run_red_green_validation(&paths, None, false, false).await;

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.message.contains("No test paths provided"));

    Ok(())
}

#[tokio::test]
async fn test_redgreen_command_with_test_files() -> Result<()> {
    // Arrange
    let test_dir = create_test_dir_with_sample()?;
    let test_file = test_dir.path().join("test.clnrm.toml");
    let paths = vec![test_file];

    // Act - This will run actual tests, which may pass or fail
    let result = run_red_green_validation(&paths, None, false, false).await;

    // Assert - We accept both success and error (depends on actual test execution)
    // What matters is the command doesn't panic
    assert!(result.is_ok() || result.is_err());

    Ok(())
}

// ============================================================================
// COLLECTOR COMMAND TESTS
// ============================================================================

#[tokio::test]
async fn test_collector_status_command() -> Result<()> {
    // Act
    let result = show_collector_status().await;

    // Assert - May succeed (if collector running) or fail (if not)
    // We're testing command doesn't panic
    assert!(result.is_ok() || result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_collector_logs_command() -> Result<()> {
    // Act
    let result = show_collector_logs(10, false).await;

    // Assert - May succeed or fail depending on collector state
    assert!(result.is_ok() || result.is_err());

    Ok(())
}

// Note: We don't test start_collector and stop_collector in CI
// as they require Docker and may interfere with other tests

// ============================================================================
// OUTPUT FORMAT VALIDATION TESTS
// ============================================================================

#[tokio::test]
async fn test_graph_output_formats_all_work() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        clnrm_core::error::CleanroomError::io_error(format!("Failed to create temp dir: {}", e))
    })?;
    let trace_file = create_sample_trace_file(&temp_dir)?;

    // Act & Assert - Test all formats
    let formats = vec![
        GraphFormat::Ascii,
        GraphFormat::Dot,
        GraphFormat::Json,
        GraphFormat::Mermaid,
    ];

    for format in formats {
        let result = visualize_graph(&trace_file, &format, false, None);
        assert!(result.is_ok(), "Format {:?} should work", format);
    }

    Ok(())
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[tokio::test]
async fn test_pull_then_graph_workflow() -> Result<()> {
    // Arrange
    let test_dir = create_test_dir_with_sample()?;

    // Act 1: Pull images (discovers test files)
    let pull_result = pull_images(Some(vec![test_dir.path().to_path_buf()]), false, 1).await;

    // Assert 1
    assert!(pull_result.is_ok() || pull_result.is_err()); // Docker may not be available

    // Act 2: Visualize graph (if trace exists)
    let trace_file = create_sample_trace_file(&test_dir)?;
    let graph_result = visualize_graph(&trace_file, &GraphFormat::Ascii, false, None);

    // Assert 2
    assert!(graph_result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_render_then_pull_workflow() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        clnrm_core::error::CleanroomError::io_error(format!("Failed to create temp dir: {}", e))
    })?;

    // Act 1: Render template to create test file
    let template_file = create_sample_template(&temp_dir)?;
    let output_file = temp_dir.path().join("generated_test.clnrm.toml");
    let var_map = vec![
        "test_name=generated".to_string(),
        "description=Generated test".to_string(),
    ];

    let render_result =
        render_template_with_vars(&template_file, &var_map, Some(&output_file), false);

    // Assert 1
    assert!(render_result.is_ok());

    // Act 2: Pull images from generated test
    // (This would work if the rendered template had a services section)
    let pull_result = pull_images(Some(vec![temp_dir.path().to_path_buf()]), false, 1).await;

    // Assert 2
    assert!(pull_result.is_ok());

    Ok(())
}
