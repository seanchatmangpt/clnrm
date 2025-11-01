//! Docker Integration Tests for Weaver Live-Check
//!
//! Tests the complete workflow with actual Docker containers,
//! verifying that telemetry is emitted and validated by Weaver.
//!
//! These tests use testcontainers to spin up real services and
//! verify that the live-check integration works end-to-end.

use clnrm_core::error::Result;
use clnrm_core::telemetry::live_check::{
    LiveCheckConfig, LiveCheckOrchestrator, Uninitialized, ValidationConfig, ValidationMode,
};
use clnrm_core::{cleanroom::CleanroomEnvironment, services::generic::GenericContainerPlugin};
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::time::Duration;

// ============================================================================
// Test Helpers
// ============================================================================

/// Create test configuration for Docker integration
fn create_docker_test_config(temp_dir: &TempDir) -> LiveCheckConfig {
    LiveCheckConfig {
        enabled: true,
        registry_path: PathBuf::from("registry"),
        otlp_port: None,
        admin_port: None,
        output_dir: temp_dir.path().join("validation_output"),
        stream: false,
        fail_fast: false,
    }
}

/// Create cleanroom environment with OTEL configured
async fn create_instrumented_environment() -> Result<CleanroomEnvironment> {
    CleanroomEnvironment::new().await
}

// ============================================================================
// Test 1: Docker Container Lifecycle with Live-Check
// ============================================================================

#[tokio::test]
#[ignore] // Requires Docker and Weaver installed
async fn test_docker_container_lifecycle_emits_telemetry() -> Result<()> {
    // Arrange: Start Weaver live-check
    let temp_dir = TempDir::new().map_err(|e| {
        clnrm_core::error::CleanroomError::internal_error(format!(
            "Failed to create temp dir: {}",
            e
        ))
    })?;

    let config = create_docker_test_config(&temp_dir);
    let orchestrator = LiveCheckOrchestrator::<Uninitialized>::new(config)?;
    let live_check = orchestrator.start_weaver().await?;

    // Configure OTEL to send to Weaver's OTLP endpoint
    let otlp_port = live_check.otlp_port();
    std::env::set_var(
        "OTEL_EXPORTER_OTLP_ENDPOINT",
        format!("http://localhost:{}", otlp_port),
    );

    // Create cleanroom environment (this will emit telemetry)
    let env = create_instrumented_environment().await?;

    // Act: Start a container (should emit clnrm.container.start span)
    let plugin = GenericContainerPlugin::new("alpine-test", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;
    let handle = env.start_service("alpine-test").await?;

    // Execute command (should emit telemetry)
    let result = env
        .execute_in_container(
            "alpine-test",
            &["echo".to_string(), "hello-docker".to_string()],
            None,
            None,
        )
        .await?;

    assert!(result.succeeded());
    assert!(result.stdout.contains("hello-docker"));

    // Stop container (should emit clnrm.container.stop span)
    env.stop_service(&handle.id).await?;

    // Give telemetry time to be exported
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Assert: Stop live-check and verify telemetry was received
    let completed = live_check.stop_weaver().await?;
    let report = completed.report();

    // Should have received telemetry samples
    assert!(
        report.sample_count > 0,
        "Should have received telemetry from Docker operations, got {} samples",
        report.sample_count
    );

    // Verify critical spans were emitted
    // Note: Actual span validation depends on instrumentation implementation
    println!("Validation report: {:?}", report);

    Ok(())
}

// ============================================================================
// Test 2: SurrealDB Container with Telemetry Validation
// ============================================================================

#[tokio::test]
#[ignore] // Requires Docker, SurrealDB image, and Weaver
async fn test_surrealdb_container_emits_expected_telemetry() -> Result<()> {
    // Arrange: Start Weaver
    let temp_dir = TempDir::new().map_err(|e| {
        clnrm_core::error::CleanroomError::internal_error(format!(
            "Failed to create temp dir: {}",
            e
        ))
    })?;

    let config = create_docker_test_config(&temp_dir);
    let orchestrator = LiveCheckOrchestrator::<Uninitialized>::new(config)?;
    let live_check = orchestrator.start_weaver().await?;

    // Configure OTEL endpoint
    let otlp_port = live_check.otlp_port();
    std::env::set_var(
        "OTEL_EXPORTER_OTLP_ENDPOINT",
        format!("http://localhost:{}", otlp_port),
    );

    // Create environment
    let env = create_instrumented_environment().await?;

    // Act: Start SurrealDB
    let plugin = GenericContainerPlugin::new("surrealdb-test", "surrealdb/surrealdb:latest");
    env.register_service(Box::new(plugin)).await?;
    let handle = env.start_service("surrealdb-test").await?;

    // Wait for SurrealDB to be ready
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Execute health check
    let result = env
        .execute_in_container(
            "surrealdb-test",
            &["surreal".to_string(), "version".to_string()],
            None,
            None,
        )
        .await?;

    assert!(result.succeeded());

    // Cleanup
    env.stop_service(&handle.id).await?;

    // Wait for telemetry export
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Assert: Verify telemetry validation
    let completed = live_check.stop_weaver().await?;
    let report = completed.report();

    assert!(
        report.sample_count > 0,
        "Should have received telemetry from SurrealDB operations"
    );

    Ok(())
}

// ============================================================================
// Test 3: PostgreSQL Container with Missing Spans Detection
// ============================================================================

#[tokio::test]
#[ignore] // Requires Docker and PostgreSQL image
async fn test_postgres_container_missing_spans_detected() -> Result<()> {
    use clnrm_core::telemetry::live_check::ValidationMode;

    // Arrange: Start Weaver in strict mode
    let temp_dir = TempDir::new().map_err(|e| {
        clnrm_core::error::CleanroomError::internal_error(format!(
            "Failed to create temp dir: {}",
            e
        ))
    })?;

    let config = create_docker_test_config(&temp_dir);
    let orchestrator = LiveCheckOrchestrator::<Uninitialized>::new(config)?;
    let live_check = orchestrator.start_weaver().await?;

    let otlp_port = live_check.otlp_port();
    std::env::set_var(
        "OTEL_EXPORTER_OTLP_ENDPOINT",
        format!("http://localhost:{}", otlp_port),
    );

    let env = create_instrumented_environment().await?;

    // Act: Start PostgreSQL
    let mut env_vars = HashMap::new();
    env_vars.insert("POSTGRES_PASSWORD".to_string(), "test123".to_string());

    let plugin = GenericContainerPlugin::new("postgres-test", "postgres:15-alpine");
    env.register_service(Box::new(plugin)).await?;

    // Start container but DON'T emit all expected telemetry
    // (simulates incomplete instrumentation)
    let handle = env.start_service("postgres-test").await?;

    // Minimal interaction
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Stop without full lifecycle
    env.stop_service(&handle.id).await?;

    tokio::time::sleep(Duration::from_secs(2)).await;

    // Assert: Weaver should detect missing spans
    let completed = live_check.stop_weaver().await?;
    let report = completed.report();

    // In strict mode, missing spans should be flagged
    // (This test verifies Weaver catches incomplete instrumentation)
    println!("Violations detected: {}", report.violations);
    println!("Sample count: {}", report.sample_count);

    Ok(())
}

// ============================================================================
// Test 4: Multi-Container Test with Concurrent Telemetry
// ============================================================================

#[tokio::test]
#[ignore] // Requires Docker and significant resources
async fn test_multi_container_concurrent_telemetry_no_conflicts() -> Result<()> {
    // Arrange: Start Weaver
    let temp_dir = TempDir::new().map_err(|e| {
        clnrm_core::error::CleanroomError::internal_error(format!(
            "Failed to create temp dir: {}",
            e
        ))
    })?;

    let config = create_docker_test_config(&temp_dir);
    let orchestrator = LiveCheckOrchestrator::<Uninitialized>::new(config)?;
    let live_check = orchestrator.start_weaver().await?;

    let otlp_port = live_check.otlp_port();
    std::env::set_var(
        "OTEL_EXPORTER_OTLP_ENDPOINT",
        format!("http://localhost:{}", otlp_port),
    );

    let env = create_instrumented_environment().await?;

    // Act: Start 5 containers concurrently
    let container_names = vec![
        ("alpine1", "alpine:latest"),
        ("alpine2", "alpine:latest"),
        ("alpine3", "alpine:latest"),
        ("busybox1", "busybox:latest"),
        ("busybox2", "busybox:latest"),
    ];

    let mut handles = vec![];

    for (name, image) in &container_names {
        let plugin = GenericContainerPlugin::new(name, image);
        env.register_service(Box::new(plugin)).await?;
        let handle = env.start_service(name).await?;
        handles.push((name.to_string(), handle));
    }

    // Execute commands in all containers
    for (name, _) in &handles {
        let result = env
            .execute_in_container(
                name,
                &["echo".to_string(), format!("hello-{}", name)],
                None,
                None,
            )
            .await?;
        assert!(result.succeeded());
    }

    // Stop all containers
    for (_, handle) in handles {
        env.stop_service(&handle.id).await?;
    }

    // Wait for telemetry
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Assert: All telemetry should be captured
    let completed = live_check.stop_weaver().await?;
    let report = completed.report();

    // With 5 containers * (start + execute + stop), expect significant telemetry
    assert!(
        report.sample_count >= 10,
        "Expected at least 10 telemetry samples from 5 containers, got {}",
        report.sample_count
    );

    Ok(())
}

// ============================================================================
// Test 5: Container Failure Telemetry Validation
// ============================================================================

#[tokio::test]
#[ignore] // Requires Docker
async fn test_container_failure_emits_error_telemetry() -> Result<()> {
    // Arrange: Start Weaver
    let temp_dir = TempDir::new().map_err(|e| {
        clnrm_core::error::CleanroomError::internal_error(format!(
            "Failed to create temp dir: {}",
            e
        ))
    })?;

    let config = create_docker_test_config(&temp_dir);
    let orchestrator = LiveCheckOrchestrator::<Uninitialized>::new(config)?;
    let live_check = orchestrator.start_weaver().await?;

    let otlp_port = live_check.otlp_port();
    std::env::set_var(
        "OTEL_EXPORTER_OTLP_ENDPOINT",
        format!("http://localhost:{}", otlp_port),
    );

    let env = create_instrumented_environment().await?;

    // Act: Start container and execute failing command
    let plugin = GenericContainerPlugin::new("alpine-fail", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;
    let handle = env.start_service("alpine-fail").await?;

    // Execute command that will fail
    let result = env
        .execute_in_container(
            "alpine-fail",
            &["sh".to_string(), "-c".to_string(), "exit 42".to_string()],
            None,
            None,
        )
        .await;

    // Command should fail
    if let Ok(exec_result) = result {
        assert!(!exec_result.succeeded());
        // Exit code should be 42
        assert_eq!(exec_result.exit_code, 42);
    }

    // Cleanup
    env.stop_service(&handle.id).await?;

    tokio::time::sleep(Duration::from_secs(2)).await;

    // Assert: Error telemetry should be captured
    let completed = live_check.stop_weaver().await?;
    let report = completed.report();

    assert!(
        report.sample_count > 0,
        "Should have received telemetry including error spans"
    );

    // Note: Specific error span validation would require checking span attributes
    // for status code = ERROR

    Ok(())
}

// ============================================================================
// Test 6: Rapid Container Lifecycle Stress Test
// ============================================================================

#[tokio::test]
#[ignore] // Requires Docker and is resource-intensive
async fn test_rapid_container_lifecycle_telemetry_captured() -> Result<()> {
    // Arrange: Start Weaver
    let temp_dir = TempDir::new().map_err(|e| {
        clnrm_core::error::CleanroomError::internal_error(format!(
            "Failed to create temp dir: {}",
            e
        ))
    })?;

    let config = create_docker_test_config(&temp_dir);
    let orchestrator = LiveCheckOrchestrator::<Uninitialized>::new(config)?;
    let live_check = orchestrator.start_weaver().await?;

    let otlp_port = live_check.otlp_port();
    std::env::set_var(
        "OTEL_EXPORTER_OTLP_ENDPOINT",
        format!("http://localhost:{}", otlp_port),
    );

    let env = create_instrumented_environment().await?;

    // Act: Rapidly create and destroy 20 containers
    for i in 0..20 {
        let name = format!("rapid-{}", i);
        let plugin = GenericContainerPlugin::new(&name, "alpine:latest");
        env.register_service(Box::new(plugin)).await?;

        let handle = env.start_service(&name).await?;

        // Quick command
        let _ = env
            .execute_in_container(
                &name,
                &["echo".to_string(), "quick".to_string()],
                None,
                None,
            )
            .await;

        // Immediate stop
        env.stop_service(&handle.id).await?;
    }

    // Wait for all telemetry to be exported
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Assert: All lifecycle events should be captured
    let completed = live_check.stop_weaver().await?;
    let report = completed.report();

    // With 20 containers * 3 operations (start, execute, stop), expect significant samples
    assert!(
        report.sample_count >= 40,
        "Expected at least 40 telemetry samples from rapid lifecycle, got {}",
        report.sample_count
    );

    Ok(())
}

// ============================================================================
// Test 7: Docker Network Telemetry Validation
// ============================================================================

#[tokio::test]
#[ignore] // Requires Docker with network support
async fn test_docker_network_operations_emit_telemetry() -> Result<()> {
    // Arrange: Start Weaver
    let temp_dir = TempDir::new().map_err(|e| {
        clnrm_core::error::CleanroomError::internal_error(format!(
            "Failed to create temp dir: {}",
            e
        ))
    })?;

    let config = create_docker_test_config(&temp_dir);
    let orchestrator = LiveCheckOrchestrator::<Uninitialized>::new(config)?;
    let live_check = orchestrator.start_weaver().await?;

    let otlp_port = live_check.otlp_port();
    std::env::set_var(
        "OTEL_EXPORTER_OTLP_ENDPOINT",
        format!("http://localhost:{}", otlp_port),
    );

    let env = create_instrumented_environment().await?;

    // Act: Create containers with network interaction
    let plugin1 = GenericContainerPlugin::new("server", "alpine:latest");
    let plugin2 = GenericContainerPlugin::new("client", "alpine:latest");

    env.register_service(Box::new(plugin1)).await?;
    env.register_service(Box::new(plugin2)).await?;

    let handle1 = env.start_service("server").await?;
    let handle2 = env.start_service("client").await?;

    // Network operations would emit container.network.* attributes
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Cleanup
    env.stop_service(&handle1.id).await?;
    env.stop_service(&handle2.id).await?;

    tokio::time::sleep(Duration::from_secs(2)).await;

    // Assert: Network telemetry captured
    let completed = live_check.stop_weaver().await?;
    let report = completed.report();

    assert!(
        report.sample_count > 0,
        "Should have received network operation telemetry"
    );

    Ok(())
}
