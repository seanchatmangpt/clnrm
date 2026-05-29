//! Gall Test Suite for CLI Subsystems
//!
//! Exposes the gap where foundational management commands are stubbed or dead code.

use clnrm_core::cli::types::Commands;

#[tokio::test]
async fn gall_gap_test_cli_management_commands() {
    // Arrange
    let cmd = Commands::Run {
        paths: None,
        parallel: false,
        jobs: 1,
        fail_fast: false,
        watch: false,
        force: false,
        shard: None,
        digest: false,
        report_junit: None,
        validate: false,
        otel_exporter: "none".to_string(),
        otel_endpoint: None,
        live_check: false,
        validation_mode: None,
        registry_path: None,
        otlp_port: 0,
        admin_port: 0,
        diagnostic_format: "text".to_string(),
        stop_timeout: 0,
    }; // Example command
    
    // Act
    let result = cmd.run(false).await;

    // Assert
    // The command dispatch now correctly refuses orphaned commands explicitly.
    let err = result.unwrap_err();
    assert!(err.to_string().contains("CLI-GALL-1 Refusal"));
}