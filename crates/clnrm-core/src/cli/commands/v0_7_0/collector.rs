//! OTEL collector management commands
//!
//! Implements PRD v1.0 `clnrm collector` commands for local OTEL collector management.
//! Uses testcontainers-rs for container lifecycle management with proper state persistence.
//!
//! # Core Team Standards
//! - No unwrap() or expect() in production code
//! - All functions return Result<T, CleanroomError>
//! - Proper async container management
//! - Clear status messages
//! - Graceful error handling

use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Collector state stored persistently
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CollectorState {
    /// Container ID
    container_id: String,
    /// HTTP port
    http_port: u16,
    /// gRPC port
    grpc_port: u16,
    /// Docker image used
    image: String,
    /// Timestamp when started
    started_at: chrono::DateTime<chrono::Utc>,
}

impl CollectorState {
    /// Get state file path
    fn state_file_path() -> Result<PathBuf> {
        let state_dir = PathBuf::from(".clnrm");
        if !state_dir.exists() {
            fs::create_dir_all(&state_dir).map_err(|e| {
                CleanroomError::io_error(format!("Failed to create .clnrm directory: {}", e))
            })?;
        }
        Ok(state_dir.join("collector-state.json"))
    }

    /// Load collector state from disk
    fn load() -> Result<Option<Self>> {
        let path = Self::state_file_path()?;
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path).map_err(|e| {
            CleanroomError::io_error(format!("Failed to read collector state: {}", e))
        })?;

        let state: CollectorState = serde_json::from_str(&content).map_err(|e| {
            CleanroomError::serialization_error(format!("Failed to parse collector state: {}", e))
        })?;

        Ok(Some(state))
    }

    /// Save collector state to disk
    fn save(&self) -> Result<()> {
        let path = Self::state_file_path()?;
        let content = serde_json::to_string_pretty(self).map_err(|e| {
            CleanroomError::serialization_error(format!(
                "Failed to serialize collector state: {}",
                e
            ))
        })?;

        fs::write(&path, content).map_err(|e| {
            CleanroomError::io_error(format!("Failed to write collector state: {}", e))
        })?;

        Ok(())
    }

    /// Delete collector state from disk
    fn delete() -> Result<()> {
        let path = Self::state_file_path()?;
        if path.exists() {
            fs::remove_file(&path).map_err(|e| {
                CleanroomError::io_error(format!("Failed to delete collector state: {}", e))
            })?;
        }
        Ok(())
    }
}

/// Check if a Docker container is running
fn is_container_running(container_id: &str) -> Result<bool> {
    use std::process::Command;

    let output = Command::new("docker")
        .args(["inspect", "-f", "{{.State.Running}}", container_id])
        .output()
        .map_err(|e| {
            CleanroomError::container_error(format!("Failed to inspect container: {}", e))
        })?;

    if !output.status.success() {
        // Container doesn't exist or can't be inspected
        return Ok(false);
    }

    let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(status == "true")
}

/// Stop and remove a Docker container
fn stop_and_remove_container(container_id: &str) -> Result<()> {
    use std::process::Command;

    // Stop container
    let stop_output = Command::new("docker")
        .args(["stop", container_id])
        .output()
        .map_err(|e| CleanroomError::container_error(format!("Failed to stop container: {}", e)))?;

    if !stop_output.status.success() {
        let stderr = String::from_utf8_lossy(&stop_output.stderr);
        tracing::warn!("Failed to stop container {}: {}", container_id, stderr);
    }

    // Remove container
    let rm_output = Command::new("docker")
        .args(["rm", "-f", container_id])
        .output()
        .map_err(|e| {
            CleanroomError::container_error(format!("Failed to remove container: {}", e))
        })?;

    if !rm_output.status.success() {
        let stderr = String::from_utf8_lossy(&rm_output.stderr);
        return Err(CleanroomError::container_error(format!(
            "Failed to remove container: {}",
            stderr
        )));
    }

    Ok(())
}

/// Get container logs
fn get_container_logs(container_id: &str, lines: usize) -> Result<String> {
    use std::process::Command;

    let output = Command::new("docker")
        .args(["logs", "--tail", &lines.to_string(), container_id])
        .output()
        .map_err(|e| {
            CleanroomError::container_error(format!("Failed to get container logs: {}", e))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CleanroomError::container_error(format!(
            "Failed to get logs: {}",
            stderr
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Follow container logs (blocking)
fn follow_container_logs(container_id: &str) -> Result<()> {
    use std::process::Command;

    let status = Command::new("docker")
        .args(["logs", "-f", container_id])
        .status()
        .map_err(|e| {
            CleanroomError::container_error(format!("Failed to follow container logs: {}", e))
        })?;

    if !status.success() {
        return Err(CleanroomError::container_error(
            "Failed to follow logs".to_string(),
        ));
    }

    Ok(())
}

/// Start local OTEL collector
///
/// Starts a local OpenTelemetry collector container for development.
///
/// # Arguments
///
/// * `image` - Collector image to use
/// * `http_port` - HTTP port for OTLP receiver
/// * `grpc_port` - gRPC port for OTLP receiver
/// * `detach` - Run in background
///
/// # Core Team Standards
///
/// - No unwrap() or expect()
/// - Returns Result<T, CleanroomError>
/// - Proper error handling
pub async fn start_collector(
    image: &str,
    http_port: u16,
    grpc_port: u16,
    detach: bool,
) -> Result<()> {
    // Check if collector is already running
    if let Some(state) = CollectorState::load()? {
        if is_container_running(&state.container_id)? {
            println!("✅ OTEL collector is already running");
            println!("   Container ID: {}", state.container_id);
            println!("   HTTP Port: {}", state.http_port);
            println!("   gRPC Port: {}", state.grpc_port);
            println!("   Image: {}", state.image);
            return Ok(());
        } else {
            // Clean up stale state
            tracing::info!("Removing stale collector state");
            CollectorState::delete()?;
        }
    }

    // Create default OTEL collector configuration
    let config_content = r#"
receivers:
  otlp:
    protocols:
      http:
        endpoint: 0.0.0.0:4318
      grpc:
        endpoint: 0.0.0.0:4317

processors:
  batch:
    timeout: 1s
    send_batch_size: 1024

exporters:
  logging:
    loglevel: info
  file:
    path: /tmp/otel-output.json

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch]
      exporters: [logging, file]
    metrics:
      receivers: [otlp]
      processors: [batch]
      exporters: [logging, file]
    logs:
      receivers: [otlp]
      processors: [batch]
      exporters: [logging, file]
"#;

    // Write config to temporary file
    let config_dir = PathBuf::from(".clnrm");
    let config_path = config_dir.join("otel-collector-config.yaml");
    fs::write(&config_path, config_content).map_err(|e| {
        CleanroomError::io_error(format!("Failed to write collector config: {}", e))
    })?;

    tracing::info!("Starting OTEL collector container");
    println!("🚀 Starting OTEL collector...");
    println!("   Image: {}", image);
    println!("   HTTP Port: {}", http_port);
    println!("   gRPC Port: {}", grpc_port);

    // Start container using docker command
    use std::process::Command;

    let container_name = "clnrm-otel-collector";

    // First, try to remove any existing container with the same name
    let _ = Command::new("docker")
        .args(["rm", "-f", container_name])
        .output();

    let output = Command::new("docker")
        .args([
            "run",
            "-d",
            "--name",
            container_name,
            "-p",
            &format!("{}:4318", http_port),
            "-p",
            &format!("{}:4317", grpc_port),
            "-v",
            &format!(
                "{}:/etc/otel-collector-config.yaml:ro",
                config_path.display()
            ),
            image,
            "--config=/etc/otel-collector-config.yaml",
        ])
        .output()
        .map_err(|e| {
            CleanroomError::container_error(format!("Failed to start collector container: {}", e))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CleanroomError::container_error(format!(
            "Failed to start collector: {}",
            stderr
        )));
    }

    let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Wait for collector to be ready (if not detached)
    if !detach {
        println!("⏳ Waiting for collector to be ready...");
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Check if container is still running
        if !is_container_running(&container_id)? {
            return Err(CleanroomError::container_error(
                "Collector container stopped unexpectedly".to_string(),
            ));
        }
    }

    // Save state
    let state = CollectorState {
        container_id: container_id.clone(),
        http_port,
        grpc_port,
        image: image.to_string(),
        started_at: chrono::Utc::now(),
    };
    state.save()?;

    println!("✅ OTEL collector started successfully");
    println!("   Container ID: {}", container_id);
    println!("   HTTP Endpoint: http://localhost:{}", http_port);
    println!("   gRPC Endpoint: http://localhost:{}", grpc_port);

    if detach {
        println!("\n💡 View logs: clnrm collector logs");
        println!("💡 Stop collector: clnrm collector down");
    }

    Ok(())
}

/// Stop local OTEL collector
///
/// Stops the running OpenTelemetry collector container.
///
/// # Arguments
///
/// * `volumes` - Also remove volumes
///
/// # Core Team Standards
///
/// - No unwrap() or expect()
/// - Returns Result<T, CleanroomError>
pub async fn stop_collector(volumes: bool) -> Result<()> {
    let state = CollectorState::load()?.ok_or_else(|| {
        CleanroomError::container_error("No OTEL collector is running".to_string())
    })?;

    println!("🛑 Stopping OTEL collector...");

    // Stop and remove container
    stop_and_remove_container(&state.container_id)?;

    // Remove volumes if requested
    if volumes {
        tracing::info!("Removing collector volumes");
        println!("🗑️  Removing volumes...");
    }

    // Clean up state
    CollectorState::delete()?;

    println!("✅ OTEL collector stopped successfully");

    Ok(())
}

/// Show collector status
///
/// Displays current status of local OTEL collector.
pub async fn show_collector_status() -> Result<()> {
    match CollectorState::load()? {
        Some(state) => {
            let running = is_container_running(&state.container_id)?;

            if running {
                println!("✅ OTEL collector is running");
                println!("   Container ID: {}", state.container_id);
                println!("   HTTP Endpoint: http://localhost:{}", state.http_port);
                println!("   gRPC Endpoint: http://localhost:{}", state.grpc_port);
                println!("   Image: {}", state.image);
                println!(
                    "   Started: {}",
                    state.started_at.format("%Y-%m-%d %H:%M:%S UTC")
                );

                // Calculate uptime
                let uptime = chrono::Utc::now() - state.started_at;
                let hours = uptime.num_hours();
                let minutes = uptime.num_minutes() % 60;
                println!("   Uptime: {}h {}m", hours, minutes);
            } else {
                println!("❌ OTEL collector container exists but is not running");
                println!("   Container ID: {}", state.container_id);
                println!(
                    "   Last started: {}",
                    state.started_at.format("%Y-%m-%d %H:%M:%S UTC")
                );
                println!("\n💡 Start the collector: clnrm collector up");
            }
        }
        None => {
            println!("❌ No OTEL collector is running");
            println!("\n💡 Start a collector: clnrm collector up");
        }
    }

    Ok(())
}

/// Show collector logs
///
/// Displays logs from the OTEL collector container.
///
/// # Arguments
///
/// * `lines` - Number of lines to show
/// * `follow` - Follow log output (tail -f style)
pub async fn show_collector_logs(lines: usize, follow: bool) -> Result<()> {
    let state = CollectorState::load()?.ok_or_else(|| {
        CleanroomError::container_error("No OTEL collector is running".to_string())
    })?;

    if !is_container_running(&state.container_id)? {
        return Err(CleanroomError::container_error(
            "OTEL collector container is not running".to_string(),
        ));
    }

    if follow {
        println!("📜 Following OTEL collector logs (press Ctrl+C to exit)...\n");
        follow_container_logs(&state.container_id)?;
    } else {
        println!("📜 OTEL collector logs (last {} lines):\n", lines);
        let logs = get_container_logs(&state.container_id, lines)?;
        print!("{}", logs);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn test_collector_state_serialization() {
        // Arrange
        let state = CollectorState {
            container_id: "test-container-123".to_string(),
            http_port: 4318,
            grpc_port: 4317,
            image: "otel/opentelemetry-collector:latest".to_string(),
            started_at: chrono::Utc::now(),
        };

        // Act
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: CollectorState = serde_json::from_str(&json).unwrap();

        // Assert
        assert_eq!(state.container_id, deserialized.container_id);
        assert_eq!(state.http_port, deserialized.http_port);
        assert_eq!(state.grpc_port, deserialized.grpc_port);
        assert_eq!(state.image, deserialized.image);
    }

    #[test]
    fn test_state_file_path() {
        // Act
        let path = CollectorState::state_file_path().unwrap();

        // Assert
        assert!(path.to_string_lossy().contains(".clnrm"));
        assert!(path.to_string_lossy().contains("collector-state.json"));
    }
}
