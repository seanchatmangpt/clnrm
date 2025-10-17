//! Integration tests for artifacts collection
//!
//! Tests the artifact collection functionality for scenarios, including:
//! - Parsing artifact specifications
//! - Collecting OTEL spans from stdout
//! - Saving artifacts to the artifact directory
//! - Validating artifact paths and metadata

use clnrm_core::config::ScenarioConfig;
use clnrm_core::error::Result;
use clnrm_core::scenario::artifacts::{ArtifactCollector, ArtifactType};
use std::fs;

#[test]
fn test_artifacts_config_parsing_from_toml() -> Result<()> {
    // Arrange
    let toml_content = r#"
[[scenario]]
name = "test_scenario"
service = "test_service"
run = "echo test"

[scenario.artifacts]
collect = ["spans:default", "logs:stderr"]
"#;

    // Act
    let config: toml::Value = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(format!("Parse error: {}", e)))?;

    let scenarios = config
        .get("scenario")
        .and_then(|s| s.as_array())
        .ok_or_else(|| clnrm_core::error::CleanroomError::validation_error("No scenarios found"))?;

    let scenario: ScenarioConfig = toml::from_str(&toml::to_string(&scenarios[0]).map_err(|e| {
        clnrm_core::error::CleanroomError::internal_error(format!("Serialization error: {}", e))
    })?)
    .map_err(|e| clnrm_core::error::CleanroomError::config_error(format!("Parse error: {}", e)))?;

    // Assert
    assert_eq!(scenario.name, "test_scenario");
    assert!(scenario.artifacts.is_some());
    let artifacts = scenario.artifacts.as_ref().unwrap();
    assert_eq!(artifacts.collect.len(), 2);
    assert_eq!(artifacts.collect[0], "spans:default");
    assert_eq!(artifacts.collect[1], "logs:stderr");

    Ok(())
}

#[test]
fn test_artifacts_config_optional_in_scenario() -> Result<()> {
    // Arrange
    let toml_content = r#"
[[scenario]]
name = "test_scenario_no_artifacts"
service = "test_service"
run = "echo test"
"#;

    // Act
    let config: toml::Value = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(format!("Parse error: {}", e)))?;

    let scenarios = config
        .get("scenario")
        .and_then(|s| s.as_array())
        .ok_or_else(|| clnrm_core::error::CleanroomError::validation_error("No scenarios found"))?;

    let scenario: ScenarioConfig = toml::from_str(&toml::to_string(&scenarios[0]).map_err(|e| {
        clnrm_core::error::CleanroomError::internal_error(format!("Serialization error: {}", e))
    })?)
    .map_err(|e| clnrm_core::error::CleanroomError::config_error(format!("Parse error: {}", e)))?;

    // Assert
    assert_eq!(scenario.name, "test_scenario_no_artifacts");
    assert!(scenario.artifacts.is_none());

    Ok(())
}

#[tokio::test]
async fn test_artifact_collector_collects_spans_from_stdout() -> Result<()> {
    // Arrange
    let temp_dir = tempfile::tempdir().map_err(|e| {
        clnrm_core::error::CleanroomError::internal_error("Failed to create temp dir")
            .with_source(e.to_string())
    })?;

    let artifact_dir = temp_dir.path().join("artifacts").join("test_scenario");
    fs::create_dir_all(&artifact_dir).map_err(|e| {
        clnrm_core::error::CleanroomError::io_error("Failed to create artifact dir")
            .with_source(e.to_string())
    })?;

    // Create a custom collector with test directory
    let test_collector = ArtifactCollector::with_artifact_dir("test_scenario", artifact_dir);

    let stdout = r#"{"traceId":"trace123","spanId":"span456","name":"test_span"}
{"traceId":"trace789","spanId":"spanabc","name":"another_span"}
Not a JSON line
{"traceId":"tracedef","spanId":"spanghi","name":"final_span"}"#;

    // Act
    let artifact_specs = vec!["spans:default".to_string()];
    let artifacts = test_collector.collect(&artifact_specs, stdout, "").await?;

    // Assert
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].artifact_type, "spans:default");
    assert!(artifacts[0].path.exists());
    assert_eq!(artifacts[0].item_count, Some(3)); // Should have 3 valid spans

    // Verify content
    let content = fs::read_to_string(&artifacts[0].path).map_err(|e| {
        clnrm_core::error::CleanroomError::io_error("Failed to read artifact").with_source(e.to_string())
    })?;
    let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(lines.len(), 3);

    Ok(())
}

#[tokio::test]
async fn test_artifact_collector_collects_logs() -> Result<()> {
    // Arrange
    let temp_dir = tempfile::tempdir().map_err(|e| {
        clnrm_core::error::CleanroomError::internal_error("Failed to create temp dir")
            .with_source(e.to_string())
    })?;

    let artifact_dir = temp_dir.path().join("artifacts").join("test_scenario");
    fs::create_dir_all(&artifact_dir).map_err(|e| {
        clnrm_core::error::CleanroomError::io_error("Failed to create artifact dir")
            .with_source(e.to_string())
    })?;

    let test_collector = ArtifactCollector::with_artifact_dir("test_scenario", artifact_dir);

    let stderr = "Error message line 1\nError message line 2\n";

    // Act
    let artifact_specs = vec!["logs:stderr".to_string()];
    let artifacts = test_collector.collect(&artifact_specs, "", stderr).await?;

    // Assert
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].artifact_type, "logs:stderr");
    assert!(artifacts[0].path.exists());

    // Verify content
    let content = fs::read_to_string(&artifacts[0].path).map_err(|e| {
        clnrm_core::error::CleanroomError::io_error("Failed to read artifact").with_source(e.to_string())
    })?;
    assert_eq!(content, stderr);

    Ok(())
}

#[tokio::test]
async fn test_artifact_collector_multiple_artifact_types() -> Result<()> {
    // Arrange
    let temp_dir = tempfile::tempdir().map_err(|e| {
        clnrm_core::error::CleanroomError::internal_error("Failed to create temp dir")
            .with_source(e.to_string())
    })?;

    let artifact_dir = temp_dir.path().join("artifacts").join("multi_test");
    fs::create_dir_all(&artifact_dir).map_err(|e| {
        clnrm_core::error::CleanroomError::io_error("Failed to create artifact dir")
            .with_source(e.to_string())
    })?;

    let test_collector = ArtifactCollector::with_artifact_dir("multi_test", artifact_dir);

    let stdout = r#"{"traceId":"123","spanId":"456","name":"span1"}
{"traceId":"789","spanId":"abc","name":"span2"}"#;
    let stderr = "Error output";

    // Act
    let artifact_specs = vec![
        "spans:default".to_string(),
        "logs:stderr".to_string(),
        "logs:stdout".to_string(),
    ];
    let artifacts = test_collector
        .collect(&artifact_specs, stdout, stderr)
        .await?;

    // Assert
    assert_eq!(artifacts.len(), 3);

    // Verify each artifact type
    let span_artifact = artifacts.iter().find(|a| a.artifact_type == "spans:default");
    assert!(span_artifact.is_some());
    assert_eq!(span_artifact.unwrap().item_count, Some(2));

    let stderr_artifact = artifacts.iter().find(|a| a.artifact_type == "logs:stderr");
    assert!(stderr_artifact.is_some());

    let stdout_artifact = artifacts.iter().find(|a| a.artifact_type == "logs:stdout");
    assert!(stdout_artifact.is_some());

    Ok(())
}

#[test]
fn test_artifact_type_parse_spans() -> Result<()> {
    // Arrange & Act
    let artifact_type = ArtifactType::parse("spans:default")?;

    // Assert
    match artifact_type {
        ArtifactType::Spans { exporter } => {
            assert_eq!(exporter, "default");
        }
        _ => panic!("Expected Spans artifact type"),
    }

    Ok(())
}

#[test]
fn test_artifact_type_parse_logs() -> Result<()> {
    // Arrange & Act
    let artifact_type = ArtifactType::parse("logs:stdout")?;

    // Assert
    match artifact_type {
        ArtifactType::Logs { stream } => {
            assert_eq!(stream, "stdout");
        }
        _ => panic!("Expected Logs artifact type"),
    }

    Ok(())
}

#[test]
fn test_artifact_type_parse_files() -> Result<()> {
    // Arrange & Act
    let artifact_type = ArtifactType::parse("files:/tmp/test.txt")?;

    // Assert
    match artifact_type {
        ArtifactType::Files { path } => {
            assert_eq!(path, "/tmp/test.txt");
        }
        _ => panic!("Expected Files artifact type"),
    }

    Ok(())
}

#[test]
fn test_artifact_type_parse_invalid_format() {
    // Arrange & Act
    let result = ArtifactType::parse("invalid");

    // Assert
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("expected format 'type:param'"));
}

#[test]
fn test_artifact_type_parse_unknown_type() {
    // Arrange & Act
    let result = ArtifactType::parse("unknown:value");

    // Assert
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Unknown artifact type"));
}

#[tokio::test]
async fn test_artifact_path_validation() -> Result<()> {
    // Arrange
    let collector = ArtifactCollector::new("path_test");

    // Act
    let artifact_dir = collector.artifact_dir();

    // Assert
    assert_eq!(
        artifact_dir,
        std::path::Path::new(".clnrm/artifacts/path_test")
    );

    Ok(())
}
