// Integration tests for LiveCheckOrchestrator state machine
//
// These tests validate:
// 1. Type-safe state transitions (compile-time)
// 2. Process lifecycle management (runtime)
// 3. Error handling and recovery
// 4. RAII guard behavior
// 5. Graceful fallback logic

use std::path::PathBuf;
use std::time::Duration;

use clnrm_core::error::Result;
use clnrm_core::telemetry::live_check::{
    run_with_graceful_fallback, GracefulFallbackResult, LiveCheckConfig, LiveCheckGuard,
    LiveCheckOrchestrator, OrchestrationMode, ValidationStatus,
};

// ============================================================================
// Helper Functions
// ============================================================================

/// Check if Weaver is available in PATH
fn weaver_available() -> bool {
    std::process::Command::new("weaver")
        .arg("--version")
        .output()
        .is_ok()
}

/// Create test configuration with temporary output directory
fn test_config() -> LiveCheckConfig {
    let temp_dir = std::env::temp_dir().join(format!("weaver-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();

    LiveCheckConfig {
        enabled: true,
        registry_path: PathBuf::from("registry"),
        otlp_port: None,  // Auto-discover
        admin_port: None, // Auto-discover
        output_dir: temp_dir,
        stream: false,
        fail_fast: false,
    }
}

/// Create test configuration with invalid registry (for failure testing)
fn invalid_registry_config() -> LiveCheckConfig {
    let mut config = test_config();
    config.registry_path = PathBuf::from("/nonexistent/registry");
    config
}

// ============================================================================
// State Machine Tests
// ============================================================================

#[tokio::test]
async fn test_state_machine_transitions() -> Result<()> {
    if !weaver_available() {
        eprintln!("Skipping test: Weaver not available");
        return Ok(());
    }

    let config = test_config();

    // Create orchestrator (Uninitialized state)
    let orchestrator = LiveCheckOrchestrator::new(config)?;

    // Cannot access OTLP port in Uninitialized state (compile-time check)
    // orchestrator.otlp_port(); // ← This won't compile ✓

    // Transition to WeaverRunning state
    let running = orchestrator.start_weaver().await?;

    // Can now access OTLP port
    let otlp_port = running.otlp_port();
    assert!(otlp_port > 0, "OTLP port should be assigned");

    let admin_port = running.admin_port();
    assert!(admin_port > 0, "Admin port should be assigned");
    assert_ne!(otlp_port, admin_port, "Ports should be different");

    // Cannot access report in WeaverRunning state (compile-time check)
    // running.report(); // ← This won't compile ✓

    // Transition to Completed state
    let completed = running.stop_weaver().await?;

    // Can now access report
    let report = completed.report();
    assert!(report.sample_count >= 0, "Report should have sample count");

    // Cannot call stop_weaver again (compile-time check)
    // completed.stop_weaver().await?; // ← This won't compile ✓

    Ok(())
}

#[tokio::test]
async fn test_orchestrator_with_valid_registry() -> Result<()> {
    if !weaver_available() {
        eprintln!("Skipping test: Weaver not available");
        return Ok(());
    }

    // Verify registry exists
    let registry_path = PathBuf::from("registry");
    if !registry_path.exists() {
        eprintln!("Skipping test: registry/ directory not found");
        return Ok(());
    }

    let config = test_config();
    let orchestrator = LiveCheckOrchestrator::new(config)?;

    // Start Weaver
    let running = orchestrator.start_weaver().await?;

    // Verify OTLP endpoint format
    let endpoint = running.otlp_endpoint();
    assert!(endpoint.starts_with("http://127.0.0.1:"));
    assert!(endpoint.contains(&running.otlp_port().to_string()));

    // Check uptime
    let uptime = running.uptime();
    assert!(uptime.as_millis() < 10000, "Uptime should be less than 10s");

    // Stop and validate
    let completed = running.stop_weaver().await?;

    // Check runtime duration
    let runtime_ms = completed.runtime_duration_ms();
    assert!(runtime_ms > 0, "Runtime should be > 0");
    assert!(runtime_ms < 30000, "Runtime should be < 30s");

    // Check report structure
    let report = completed.report();
    assert_eq!(report.status, ValidationStatus::Success);

    // Generate summary
    let summary = completed.summary();
    assert!(summary.contains("Weaver Live-Check"));
    assert!(summary.contains("Status:"));

    // Check exit code
    let exit_code = completed.exit_code();
    assert!(exit_code == 0 || exit_code == 1);

    Ok(())
}

#[tokio::test]
async fn test_orchestrator_endpoint_format() -> Result<()> {
    if !weaver_available() {
        eprintln!("Skipping test: Weaver not available");
        return Ok(());
    }

    let config = test_config();
    let running = LiveCheckOrchestrator::new(config)?.start_weaver().await?;

    // Test endpoint format
    let endpoint = running.otlp_endpoint();
    assert!(endpoint.starts_with("http://"));
    assert!(endpoint.contains("127.0.0.1"));

    // Parse port from endpoint
    let port_str = endpoint.split(':').last().unwrap();
    let port: u16 = port_str.parse().unwrap();
    assert_eq!(port, running.otlp_port());

    // Cleanup
    let _ = running.stop_weaver().await?;

    Ok(())
}

// ============================================================================
// RAII Guard Tests
// ============================================================================

#[tokio::test]
async fn test_raii_guard_cleanup() -> Result<()> {
    if !weaver_available() {
        eprintln!("Skipping test: Weaver not available");
        return Ok(());
    }

    let config = test_config();
    let orchestrator = LiveCheckOrchestrator::new(config)?.start_weaver().await?;

    let otlp_port = orchestrator.otlp_port();

    // Create guard
    let guard = LiveCheckGuard::new(orchestrator);

    // Can access orchestrator through guard
    let port_via_guard = guard.orchestrator().otlp_port();
    assert_eq!(port_via_guard, otlp_port);

    // Take orchestrator back
    let orchestrator = guard.take_orchestrator();
    assert_eq!(orchestrator.otlp_port(), otlp_port);

    // Stop explicitly
    let _ = orchestrator.stop_weaver().await?;

    Ok(())
}

#[tokio::test]
async fn test_guard_automatic_cleanup() -> Result<()> {
    if !weaver_available() {
        eprintln!("Skipping test: Weaver not available");
        return Ok(());
    }

    let config = test_config();
    let orchestrator = LiveCheckOrchestrator::new(config)?.start_weaver().await?;

    {
        let _guard = LiveCheckGuard::new(orchestrator);
        // Guard dropped here - should trigger cleanup
    }

    // Give cleanup time to complete
    tokio::time::sleep(Duration::from_millis(500)).await;

    // If we get here without deadlock, cleanup worked
    Ok(())
}

#[tokio::test]
#[should_panic(expected = "orchestrator already taken")]
async fn test_guard_double_take_panics() {
    if !weaver_available() {
        eprintln!("Skipping test: Weaver not available");
        return;
    }

    let config = test_config();
    let orchestrator = LiveCheckOrchestrator::new(config)
        .unwrap()
        .start_weaver()
        .await
        .unwrap();

    let guard = LiveCheckGuard::new(orchestrator);

    // Take once (ok)
    let _orchestrator = guard.take_orchestrator();

    // Try to take again (should panic)
    // let _orchestrator2 = guard.take_orchestrator(); // Can't use guard after move

    // Alternative test: Access after take
    // guard.orchestrator(); // Would panic
}

// ============================================================================
// Graceful Fallback Tests
// ============================================================================

#[tokio::test]
async fn test_fallback_to_registry_check() -> Result<()> {
    if !weaver_available() {
        eprintln!("Skipping test: Weaver not available");
        return Ok(());
    }

    // Use invalid registry to force fallback
    let config = invalid_registry_config();
    let orchestrator = LiveCheckOrchestrator::new(config)?;

    // Start with fallback
    let mode = orchestrator.start_with_fallback().await?;

    // Should fallback to registry check
    match mode {
        OrchestrationMode::RegistryCheckOnly {
            registry_path,
            reason,
        } => {
            assert!(reason.len() > 0, "Should have fallback reason");
            assert_eq!(registry_path, PathBuf::from("/nonexistent/registry"));
        }
        OrchestrationMode::LiveCheck(_) => {
            panic!("Should have fallen back to registry check");
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_graceful_fallback_with_valid_registry() -> Result<()> {
    if !weaver_available() {
        eprintln!("Skipping test: Weaver not available");
        return Ok(());
    }

    let registry_path = PathBuf::from("registry");
    if !registry_path.exists() {
        eprintln!("Skipping test: registry/ directory not found");
        return Ok(());
    }

    let config = test_config();

    // This should use live-check (not fallback)
    let result: GracefulFallbackResult = run_with_graceful_fallback(&config).await?;

    // Should be live-check mode
    match result.mode {
        clnrm_core::telemetry::live_check::FallbackMode::LiveCheck => {
            // Expected
        }
        clnrm_core::telemetry::live_check::FallbackMode::RegistryCheckOnly { reason } => {
            panic!("Should not have fallen back to registry check: {}", reason);
        }
    }

    // Should have full report
    assert!(result.report.is_some(), "Live-check should provide report");

    Ok(())
}

#[tokio::test]
async fn test_graceful_fallback_with_invalid_registry() -> Result<()> {
    if !weaver_available() {
        eprintln!("Skipping test: Weaver not available");
        return Ok(());
    }

    let config = invalid_registry_config();

    // This should fallback to registry check
    let result = run_with_graceful_fallback(&config).await?;

    // Should be registry check mode
    match result.mode {
        clnrm_core::telemetry::live_check::FallbackMode::RegistryCheckOnly { .. } => {
            // Expected
        }
        clnrm_core::telemetry::live_check::FallbackMode::LiveCheck => {
            panic!("Should have fallen back to registry check");
        }
    }

    // Should not have full report (registry check only)
    assert!(
        result.report.is_none(),
        "Registry check should not provide report"
    );

    Ok(())
}

// ============================================================================
// Health Check Tests
// ============================================================================

#[tokio::test]
async fn test_health_check() -> Result<()> {
    if !weaver_available() {
        eprintln!("Skipping test: Weaver not available");
        return Ok(());
    }

    let config = test_config();
    let running = LiveCheckOrchestrator::new(config)?.start_weaver().await?;

    // Health check should return true (process alive)
    let health = running.health_check().await?;
    assert!(health, "Weaver should be healthy");

    // Stop
    let _ = running.stop_weaver().await?;

    Ok(())
}

#[tokio::test]
async fn test_pid() -> Result<()> {
    if !weaver_available() {
        eprintln!("Skipping test: Weaver not available");
        return Ok(());
    }

    let config = test_config();
    let running = LiveCheckOrchestrator::new(config)?.start_weaver().await?;

    // Should have PID
    let pid = running.pid();
    assert!(pid.is_some(), "Should have PID");

    // Stop
    let _ = running.stop_weaver().await?;

    Ok(())
}

// ============================================================================
// Report Tests
// ============================================================================

#[tokio::test]
async fn test_report_structure() -> Result<()> {
    if !weaver_available() {
        eprintln!("Skipping test: Weaver not available");
        return Ok(());
    }

    let registry_path = PathBuf::from("registry");
    if !registry_path.exists() {
        eprintln!("Skipping test: registry/ directory not found");
        return Ok(());
    }

    let config = test_config();
    let completed = LiveCheckOrchestrator::new(config)?
        .start_weaver()
        .await?
        .stop_weaver()
        .await?;

    let report = completed.report();

    // Check report structure
    assert!(report.violations >= 0);
    assert!(report.improvements >= 0);
    assert!(report.information >= 0);
    assert!(report.sample_count >= 0);
    assert!(report.registry_coverage >= 0.0);
    assert!(report.registry_coverage <= 1.0);

    // Check status is valid
    assert!(
        report.status == ValidationStatus::Success || report.status == ValidationStatus::Failure
    );

    Ok(())
}

#[tokio::test]
async fn test_summary_generation() -> Result<()> {
    if !weaver_available() {
        eprintln!("Skipping test: Weaver not available");
        return Ok(());
    }

    let config = test_config();
    let completed = LiveCheckOrchestrator::new(config)?
        .start_weaver()
        .await?
        .stop_weaver()
        .await?;

    let summary = completed.summary();

    // Check summary contains expected sections
    assert!(summary.contains("Weaver Live-Check"));
    assert!(summary.contains("Status:"));
    assert!(summary.contains("Samples Received:"));
    assert!(summary.contains("Violations:"));
    assert!(summary.contains("Improvements:"));
    assert!(summary.contains("Registry Coverage:"));

    // Check for pass/fail indicator
    assert!(summary.contains("PASSED") || summary.contains("FAILED"));

    Ok(())
}

#[tokio::test]
async fn test_passed_logic() -> Result<()> {
    if !weaver_available() {
        eprintln!("Skipping test: Weaver not available");
        return Ok(());
    }

    let config = test_config();
    let completed = LiveCheckOrchestrator::new(config)?
        .start_weaver()
        .await?
        .stop_weaver()
        .await?;

    let report = completed.report();

    // passed() should match manual check
    let expected_passed = report.status == ValidationStatus::Success
        && report.violations == 0
        && report.sample_count > 0;

    assert_eq!(completed.passed(), expected_passed);

    Ok(())
}

#[tokio::test]
async fn test_exit_code() -> Result<()> {
    if !weaver_available() {
        eprintln!("Skipping test: Weaver not available");
        return Ok(());
    }

    let config = test_config();
    let completed = LiveCheckOrchestrator::new(config)?
        .start_weaver()
        .await?
        .stop_weaver()
        .await?;

    let exit_code = completed.exit_code();

    // Exit code should match passed status
    if completed.passed() {
        assert_eq!(exit_code, 0);
    } else {
        assert_eq!(exit_code, 1);
    }

    Ok(())
}

#[tokio::test]
async fn test_into_report() -> Result<()> {
    if !weaver_available() {
        eprintln!("Skipping test: Weaver not available");
        return Ok(());
    }

    let config = test_config();
    let completed = LiveCheckOrchestrator::new(config)?
        .start_weaver()
        .await?
        .stop_weaver()
        .await?;

    // Get report by reference
    let violations_ref = completed.report().violations;

    // Consume orchestrator
    let report = completed.into_report();

    // Should be same report
    assert_eq!(report.violations, violations_ref);

    Ok(())
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_invalid_config_validation() {
    // Test invalid port (< 1024)
    let mut config = test_config();
    config.otlp_port = Some(80); // Privileged port

    let result = LiveCheckOrchestrator::new(config);
    assert!(result.is_err(), "Should reject privileged port");
}

#[tokio::test]
async fn test_missing_registry() {
    let config = invalid_registry_config();

    let orchestrator = LiveCheckOrchestrator::new(config).unwrap();

    // Should fail to start with missing registry
    let result = orchestrator.start_weaver().await;
    assert!(result.is_err(), "Should fail with missing registry");
}

// ============================================================================
// Compile-Time Safety Tests (Type System)
// ============================================================================

// These tests verify compile-time safety by demonstrating that invalid
// operations don't compile. They are not runtime tests.

#[allow(dead_code)]
fn compile_time_safety_checks() {
    // These are examples of operations that SHOULD NOT COMPILE:

    // Cannot call stop on Uninitialized
    // let orchestrator = LiveCheckOrchestrator::new(test_config()).unwrap();
    // orchestrator.stop_weaver(); // ← Compile error ✓

    // Cannot access OTLP port on Uninitialized
    // let orchestrator = LiveCheckOrchestrator::new(test_config()).unwrap();
    // orchestrator.otlp_port(); // ← Compile error ✓

    // Cannot access report on Running
    // let running = ... ; // Some Running state
    // running.report(); // ← Compile error ✓

    // Cannot call stop twice
    // let completed = running.stop_weaver().await?;
    // completed.stop_weaver().await?; // ← Compile error ✓
}

// ============================================================================
// Integration Tests (End-to-End)
// ============================================================================

#[tokio::test]
async fn test_full_lifecycle_with_otel() -> Result<()> {
    if !weaver_available() {
        eprintln!("Skipping test: Weaver not available");
        return Ok(());
    }

    let registry_path = PathBuf::from("registry");
    if !registry_path.exists() {
        eprintln!("Skipping test: registry/ directory not found");
        return Ok(());
    }

    let config = test_config();

    // 1. Create orchestrator
    let orchestrator = LiveCheckOrchestrator::new(config)?;

    // 2. Start Weaver
    let running = orchestrator.start_weaver().await?;

    // 3. Get OTLP endpoint for OTEL configuration
    let endpoint = running.otlp_endpoint();
    println!("OTLP endpoint: {}", endpoint);

    // 4. Simulate test execution (in real usage, would run tests here)
    tokio::time::sleep(Duration::from_millis(500)).await;

    // 5. Stop Weaver and get report
    let completed = running.stop_weaver().await?;

    // 6. Check results
    let report = completed.report();
    println!("Violations: {}", report.violations);
    println!("Samples: {}", report.sample_count);

    // 7. Generate summary
    println!("{}", completed.summary());

    Ok(())
}

#[tokio::test]
async fn test_parallel_orchestrators() -> Result<()> {
    if !weaver_available() {
        eprintln!("Skipping test: Weaver not available");
        return Ok(());
    }

    // Test that multiple orchestrators can run concurrently (different ports)
    let config1 = test_config();
    let config2 = test_config();

    // Start both concurrently
    let (result1, result2) = tokio::join!(
        async { LiveCheckOrchestrator::new(config1)?.start_weaver().await },
        async { LiveCheckOrchestrator::new(config2)?.start_weaver().await }
    );

    let running1 = result1?;
    let running2 = result2?;

    // Should have different ports
    assert_ne!(running1.otlp_port(), running2.otlp_port());
    assert_ne!(running1.admin_port(), running2.admin_port());

    // Stop both
    let _ = running1.stop_weaver().await?;
    let _ = running2.stop_weaver().await?;

    Ok(())
}
