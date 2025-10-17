//! Artifact collection for scenario execution
//!
//! Provides functionality to collect and store artifacts from test scenarios,
//! including OTEL spans, logs, and file outputs.
//!
//! ## Features
//!
//! - **Span Collection**: Parse and store OTEL JSON spans from stdout
//! - **Log Collection**: Capture stderr/stdout logs
//! - **File Collection**: Copy specified files to artifact directory
//! - **Path Management**: Automatic artifact directory creation and cleanup
//!
//! ## Usage Examples
//!
//! ```no_run
//! use clnrm_core::scenario::artifacts::{ArtifactCollector, ArtifactType};
//!
//! # async fn example() -> clnrm_core::error::Result<()> {
//! let collector = ArtifactCollector::new("test_scenario");
//!
//! // Parse artifact spec
//! let artifact_type = ArtifactType::parse("spans:default")?;
//!
//! // Collect spans from stdout
//! let stdout = "{\"traceId\":\"123\",\"spanId\":\"456\",\"name\":\"test\"}";
//! let path = collector.collect_spans(stdout, "default").await?;
//! println!("Spans saved to: {}", path.display());
//! # Ok(())
//! # }
//! ```

use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info};

/// Artifact collector for scenario execution
#[derive(Debug)]
pub struct ArtifactCollector {
    /// Scenario name
    scenario_name: String,
    /// Base artifact directory
    artifact_dir: PathBuf,
}

/// Type of artifact to collect
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactType {
    /// OTEL spans with optional exporter name
    Spans { exporter: String },
    /// Logs with optional stream (stdout, stderr)
    Logs { stream: String },
    /// Files with path
    Files { path: String },
}

/// Collected artifact information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactInfo {
    /// Type of artifact
    pub artifact_type: String,
    /// Path where artifact was saved
    pub path: PathBuf,
    /// Size in bytes
    pub size_bytes: u64,
    /// Number of items (e.g., span count)
    pub item_count: Option<usize>,
}

/// OTEL span structure for parsing (minimal fields)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OtelSpan {
    /// Trace ID
    pub trace_id: String,
    /// Span ID
    pub span_id: String,
    /// Span name
    pub name: String,
    /// Parent span ID (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<String>,
    /// Start time (nanoseconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time_unix_nano: Option<String>,
    /// End time (nanoseconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time_unix_nano: Option<String>,
    /// Additional fields captured as-is
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl ArtifactType {
    /// Parse artifact type from string specification
    ///
    /// Format: "type:param" where type is spans/logs/files and param is type-specific
    ///
    /// # Examples
    ///
    /// ```
    /// use clnrm_core::scenario::artifacts::ArtifactType;
    ///
    /// # fn example() -> clnrm_core::error::Result<()> {
    /// let spans = ArtifactType::parse("spans:default")?;
    /// let logs = ArtifactType::parse("logs:stderr")?;
    /// let files = ArtifactType::parse("files:/tmp/output.txt")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse(spec: &str) -> Result<Self> {
        let parts: Vec<&str> = spec.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(CleanroomError::validation_error(format!(
                "Invalid artifact spec '{}': expected format 'type:param'",
                spec
            )));
        }

        let artifact_type = parts[0];
        let param = parts[1];

        match artifact_type {
            "spans" => Ok(ArtifactType::Spans {
                exporter: param.to_string(),
            }),
            "logs" => Ok(ArtifactType::Logs {
                stream: param.to_string(),
            }),
            "files" => Ok(ArtifactType::Files {
                path: param.to_string(),
            }),
            _ => Err(CleanroomError::validation_error(format!(
                "Unknown artifact type '{}': expected 'spans', 'logs', or 'files'",
                artifact_type
            ))),
        }
    }
}

impl ArtifactCollector {
    /// Create a new artifact collector for a scenario
    pub fn new(scenario_name: impl Into<String>) -> Self {
        let scenario_name = scenario_name.into();
        let artifact_dir = PathBuf::from(".clnrm")
            .join("artifacts")
            .join(&scenario_name);

        Self {
            scenario_name,
            artifact_dir,
        }
    }

    /// Create a collector with a custom artifact directory (for testing)
    #[doc(hidden)]
    pub fn with_artifact_dir(scenario_name: impl Into<String>, artifact_dir: PathBuf) -> Self {
        Self {
            scenario_name: scenario_name.into(),
            artifact_dir,
        }
    }

    /// Get the artifact directory path
    pub fn artifact_dir(&self) -> &Path {
        &self.artifact_dir
    }

    /// Ensure artifact directory exists
    pub async fn ensure_artifact_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.artifact_dir).await.map_err(|e| {
            CleanroomError::io_error(format!(
                "Failed to create artifact directory: {}",
                self.artifact_dir.display()
            ))
            .with_source(e.to_string())
        })?;

        info!(
            "Created artifact directory: {}",
            self.artifact_dir.display()
        );
        Ok(())
    }

    /// Collect artifacts based on configuration
    pub async fn collect(
        &self,
        artifact_specs: &[String],
        stdout: &str,
        stderr: &str,
    ) -> Result<Vec<ArtifactInfo>> {
        self.ensure_artifact_dir().await?;

        let mut artifacts = Vec::new();

        for spec in artifact_specs {
            debug!("Processing artifact spec: {}", spec);
            let artifact_type = ArtifactType::parse(spec)?;

            match artifact_type {
                ArtifactType::Spans { ref exporter } => {
                    let path = self.collect_spans(stdout, exporter).await?;
                    let metadata = fs::metadata(&path).await.map_err(|e| {
                        CleanroomError::io_error("Failed to read artifact metadata")
                            .with_source(e.to_string())
                    })?;

                    // Count lines to estimate span count
                    let content = fs::read_to_string(&path).await.map_err(|e| {
                        CleanroomError::io_error("Failed to read artifact content")
                            .with_source(e.to_string())
                    })?;
                    let item_count = content.lines().filter(|l| !l.trim().is_empty()).count();

                    artifacts.push(ArtifactInfo {
                        artifact_type: format!("spans:{}", exporter),
                        path,
                        size_bytes: metadata.len(),
                        item_count: Some(item_count),
                    });
                }
                ArtifactType::Logs { ref stream } => {
                    let log_content = match stream.as_str() {
                        "stdout" => stdout,
                        "stderr" => stderr,
                        _ => {
                            return Err(CleanroomError::validation_error(format!(
                                "Unknown log stream '{}': expected 'stdout' or 'stderr'",
                                stream
                            )))
                        }
                    };

                    let path = self.collect_logs(log_content, stream).await?;
                    let metadata = fs::metadata(&path).await.map_err(|e| {
                        CleanroomError::io_error("Failed to read artifact metadata")
                            .with_source(e.to_string())
                    })?;

                    artifacts.push(ArtifactInfo {
                        artifact_type: format!("logs:{}", stream),
                        path,
                        size_bytes: metadata.len(),
                        item_count: None,
                    });
                }
                ArtifactType::Files { ref path } => {
                    let dest_path = self.collect_file(path).await?;
                    let metadata = fs::metadata(&dest_path).await.map_err(|e| {
                        CleanroomError::io_error("Failed to read artifact metadata")
                            .with_source(e.to_string())
                    })?;

                    artifacts.push(ArtifactInfo {
                        artifact_type: format!("files:{}", path),
                        path: dest_path,
                        size_bytes: metadata.len(),
                        item_count: None,
                    });
                }
            }
        }

        info!(
            "Collected {} artifact(s) for scenario '{}'",
            artifacts.len(),
            self.scenario_name
        );
        Ok(artifacts)
    }

    /// Collect OTEL spans from stdout (NDJSON format)
    ///
    /// Parses stdout for JSON lines containing OTEL spans and saves them to
    /// `<artifact_dir>/spans.json`
    pub async fn collect_spans(&self, stdout: &str, exporter: &str) -> Result<PathBuf> {
        let mut spans = Vec::new();

        // Parse NDJSON format - each line is a JSON object
        for (line_num, line) in stdout.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Try to parse as JSON
            match serde_json::from_str::<serde_json::Value>(trimmed) {
                Ok(json) => {
                    // Check if it looks like an OTEL span (has traceId, spanId, name)
                    if json.get("traceId").is_some()
                        && json.get("spanId").is_some()
                        && json.get("name").is_some()
                    {
                        debug!("Found OTEL span on line {}: {}", line_num + 1, trimmed);
                        spans.push(json);
                    }
                }
                Err(_) => {
                    // Not JSON, skip
                    continue;
                }
            }
        }

        if spans.is_empty() {
            debug!("No OTEL spans found in stdout for exporter '{}'", exporter);
        }

        // Save spans to file
        let filename = if exporter == "default" {
            "spans.json"
        } else {
            &format!("spans-{}.json", exporter)
        };
        let path = self.artifact_dir.join(filename);

        let mut file = fs::File::create(&path).await.map_err(|e| {
            CleanroomError::io_error(format!("Failed to create spans file: {}", path.display()))
                .with_source(e.to_string())
        })?;

        // Write spans as NDJSON
        for span in &spans {
            let json_line = serde_json::to_string(span).map_err(|e| {
                CleanroomError::internal_error("Failed to serialize span to JSON")
                    .with_source(e.to_string())
            })?;
            file.write_all(json_line.as_bytes()).await.map_err(|e| {
                CleanroomError::io_error("Failed to write span to file").with_source(e.to_string())
            })?;
            file.write_all(b"\n").await.map_err(|e| {
                CleanroomError::io_error("Failed to write newline to file")
                    .with_source(e.to_string())
            })?;
        }

        info!("Collected {} span(s) to: {}", spans.len(), path.display());
        Ok(path)
    }

    /// Collect logs from stdout or stderr
    async fn collect_logs(&self, content: &str, stream: &str) -> Result<PathBuf> {
        let filename = format!("{}.log", stream);
        let path = self.artifact_dir.join(filename);

        fs::write(&path, content).await.map_err(|e| {
            CleanroomError::io_error(format!("Failed to write log file: {}", path.display()))
                .with_source(e.to_string())
        })?;

        info!("Collected {} log to: {}", stream, path.display());
        Ok(path)
    }

    /// Collect a file from the filesystem
    async fn collect_file(&self, source_path: &str) -> Result<PathBuf> {
        let source = Path::new(source_path);
        let filename = source
            .file_name()
            .ok_or_else(|| {
                CleanroomError::validation_error(format!(
                    "Invalid file path '{}': no filename",
                    source_path
                ))
            })?
            .to_string_lossy();

        let dest_path = self.artifact_dir.join(filename.as_ref());

        fs::copy(source, &dest_path).await.map_err(|e| {
            CleanroomError::io_error(format!(
                "Failed to copy file from '{}' to '{}'",
                source_path,
                dest_path.display()
            ))
            .with_source(e.to_string())
        })?;

        info!(
            "Collected file from '{}' to: {}",
            source_path,
            dest_path.display()
        );
        Ok(dest_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_artifact_type_spans() -> Result<()> {
        // Arrange & Act
        let artifact_type = ArtifactType::parse("spans:default")?;

        // Assert
        assert_eq!(
            artifact_type,
            ArtifactType::Spans {
                exporter: "default".to_string()
            }
        );
        Ok(())
    }

    #[test]
    fn test_parse_artifact_type_logs() -> Result<()> {
        // Arrange & Act
        let artifact_type = ArtifactType::parse("logs:stderr")?;

        // Assert
        assert_eq!(
            artifact_type,
            ArtifactType::Logs {
                stream: "stderr".to_string()
            }
        );
        Ok(())
    }

    #[test]
    fn test_parse_artifact_type_files() -> Result<()> {
        // Arrange & Act
        let artifact_type = ArtifactType::parse("files:/tmp/output.txt")?;

        // Assert
        assert_eq!(
            artifact_type,
            ArtifactType::Files {
                path: "/tmp/output.txt".to_string()
            }
        );
        Ok(())
    }

    #[test]
    fn test_parse_artifact_type_invalid_format() {
        // Arrange & Act
        let result = ArtifactType::parse("spans");

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("expected format 'type:param'"));
    }

    #[test]
    fn test_parse_artifact_type_unknown_type() {
        // Arrange & Act
        let result = ArtifactType::parse("unknown:value");

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Unknown artifact type"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_artifact_collector_new() {
        // Arrange & Act
        let collector = ArtifactCollector::new("test_scenario");

        // Assert
        assert_eq!(collector.scenario_name, "test_scenario");
        assert_eq!(
            collector.artifact_dir,
            PathBuf::from(".clnrm/artifacts/test_scenario")
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_collect_spans_parses_ndjson() -> Result<()> {
        // Arrange
        let temp_dir = tempfile::tempdir().map_err(|e| {
            CleanroomError::internal_error("Failed to create temp dir").with_source(e.to_string())
        })?;
        let collector = ArtifactCollector {
            scenario_name: "test".to_string(),
            artifact_dir: temp_dir.path().to_path_buf(),
        };

        let stdout = r#"{"traceId":"123","spanId":"456","name":"span1"}
{"traceId":"789","spanId":"abc","name":"span2"}
not json line
{"traceId":"def","spanId":"ghi","name":"span3"}"#;

        // Act
        let path = collector.collect_spans(stdout, "default").await?;

        // Assert
        assert!(path.exists());
        let content = fs::read_to_string(&path).await.map_err(|e| {
            CleanroomError::io_error("Failed to read spans file").with_source(e.to_string())
        })?;

        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 3); // Should have 3 valid spans

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_collect_spans_empty_stdout() -> Result<()> {
        // Arrange
        let temp_dir = tempfile::tempdir().map_err(|e| {
            CleanroomError::internal_error("Failed to create temp dir").with_source(e.to_string())
        })?;
        let collector = ArtifactCollector {
            scenario_name: "test".to_string(),
            artifact_dir: temp_dir.path().to_path_buf(),
        };

        // Act
        let path = collector.collect_spans("", "default").await?;

        // Assert
        assert!(path.exists());
        let content = fs::read_to_string(&path).await.map_err(|e| {
            CleanroomError::io_error("Failed to read spans file").with_source(e.to_string())
        })?;
        assert!(content.is_empty());

        Ok(())
    }
}
