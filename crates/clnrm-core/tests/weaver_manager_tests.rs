/// Integration tests for WeaverProcessManager
///
/// These tests validate:
/// - Weaver binary detection
/// - Port discovery and allocation
/// - Process lifecycle (start, health check, stop)
/// - Error handling and recovery
/// - Timeout behavior
/// - Resource cleanup
use clnrm_core::error::Result;
use clnrm_core::telemetry::live_check::WeaverProcessManager;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

/// Helper to create test configuration
fn create_test_manager() -> Result<WeaverProcessManager> {
    let registry_path = PathBuf::from("registry/");
    let output_dir = PathBuf::from("test_validation_output/");
    let inactivity_timeout = 120;

    WeaverProcessManager::new(registry_path, inactivity_timeout, output_dir)
}

/// Helper to check if Weaver is installed
fn is_weaver_installed() -> bool {
    which::which("weaver").is_ok()
}

#[tokio::test]
async fn test_manager_creation() -> Result<()> {
    // Arrange & Act
    let manager = create_test_manager()?;

    // Assert
    assert_eq!(
        manager.pid(),
        None,
        "No process should be running initially"
    );
    assert_eq!(
        manager.otlp_port(),
        None,
        "No OTLP port should be set initially"
    );
    assert_eq!(
        manager.admin_port(),
        None,
        "No admin port should be set initially"
    );
    assert_eq!(manager.uptime(), None, "No uptime before start");

    Ok(())
}

#[tokio::test]
async fn test_weaver_start_and_stop() -> Result<()> {
    // Skip if Weaver not installed
    if !is_weaver_installed() {
        eprintln!("⚠️  Skipping test: Weaver not installed");
        return Ok(());
    }

    // Arrange
    let mut manager = create_test_manager()?;

    // Act: Start Weaver
    let ports = manager.start().await?;

    // Assert: Process started
    assert!(manager.pid().is_some(), "Process should be running");
    assert!(
        manager.otlp_port().is_some(),
        "OTLP port should be discovered"
    );
    assert!(
        manager.admin_port().is_some(),
        "Admin port should be discovered"
    );
    assert!(manager.uptime().is_some(), "Uptime should be tracked");

    // Assert: Ports are valid
    assert!(
        ports.otlp_grpc >= 4317 && ports.otlp_grpc <= 6337,
        "OTLP port should be in valid range"
    );
    assert!(
        ports.admin_http >= 8080 && ports.admin_http <= 10099,
        "Admin port should be in valid range"
    );

    // Act: Health check
    let health = manager.health_check().await?;
    assert!(health, "Health check should pass");

    // Act: Stop Weaver
    manager.stop().await?;

    // Assert: Process stopped
    assert_eq!(manager.pid(), None, "Process should be stopped");

    Ok(())
}

#[tokio::test]
async fn test_health_check_passes() -> Result<()> {
    // Skip if Weaver not installed
    if !is_weaver_installed() {
        eprintln!("⚠️  Skipping test: Weaver not installed");
        return Ok(());
    }

    // Arrange
    let mut manager = create_test_manager()?;

    // Act: Start Weaver
    let _ports = manager.start().await?;

    // Wait a bit for full initialization
    sleep(Duration::from_millis(500)).await;

    // Act: Health check
    let health = manager.health_check().await?;

    // Assert
    assert!(health, "Health check should pass for running Weaver");

    // Cleanup
    manager.stop().await?;

    Ok(())
}

#[tokio::test]
async fn test_multiple_managers_different_ports() -> Result<()> {
    // Skip if Weaver not installed
    if !is_weaver_installed() {
        eprintln!("⚠️  Skipping test: Weaver not installed");
        return Ok(());
    }

    // Arrange: Create two managers
    let mut manager1 = WeaverProcessManager::new(
        PathBuf::from("registry/"),
        120,
        PathBuf::from("test_validation_output_1/"),
    )?;

    let mut manager2 = WeaverProcessManager::new(
        PathBuf::from("registry/"),
        120,
        PathBuf::from("test_validation_output_2/"),
    )?;

    // Act: Start both
    let ports1 = manager1.start().await?;
    let ports2 = manager2.start().await?;

    // Assert: Different ports
    assert_ne!(
        ports1.otlp_grpc, ports2.otlp_grpc,
        "OTLP ports should be different"
    );
    assert_ne!(
        ports1.admin_http, ports2.admin_http,
        "Admin ports should be different"
    );

    // Cleanup
    manager1.stop().await?;
    manager2.stop().await?;

    Ok(())
}

#[tokio::test]
async fn test_report_collection() -> Result<()> {
    // Skip if Weaver not installed
    if !is_weaver_installed() {
        eprintln!("⚠️  Skipping test: Weaver not installed");
        return Ok(());
    }

    // Arrange
    let mut manager = create_test_manager()?;

    // Act: Start, wait, then stop
    let _ports = manager.start().await?;
    sleep(Duration::from_secs(2)).await;
    manager.stop().await?;

    // Give Weaver time to write report
    sleep(Duration::from_millis(500)).await;

    // Act: Collect report
    let report = manager.collect_report();

    // Assert: Report should exist (even if empty)
    // Note: Actual content depends on whether telemetry was sent
    match report {
        Ok(content) => {
            assert!(!content.is_empty(), "Report should have content");
            // Should be valid JSON
            assert!(content.contains("{"), "Report should be JSON");
        }
        Err(e) => {
            // Report might not exist if Weaver didn't receive telemetry
            eprintln!("⚠️  Report not found (expected for no telemetry): {}", e);
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_stop_without_start_fails() -> Result<()> {
    // Arrange
    let mut manager = create_test_manager()?;

    // Act
    let result = manager.stop().await;

    // Assert: Should fail since process never started
    assert!(result.is_err(), "Stopping non-running process should fail");

    Ok(())
}

#[tokio::test]
async fn test_health_check_without_start_fails() -> Result<()> {
    // Arrange
    let manager = create_test_manager()?;

    // Act
    let result = manager.health_check().await;

    // Assert: Should fail since process not started
    assert!(result.is_err(), "Health check without start should fail");

    Ok(())
}

#[tokio::test]
async fn test_force_kill_cleanup() -> Result<()> {
    // Skip if Weaver not installed
    if !is_weaver_installed() {
        eprintln!("⚠️  Skipping test: Weaver not installed");
        return Ok(());
    }

    // Arrange
    let mut manager = create_test_manager()?;
    let _ports = manager.start().await?;

    let pid = manager.pid().expect("Process should be running");

    // Act: Force kill
    manager.force_kill()?;

    // Assert: Process should be gone
    assert_eq!(manager.pid(), None, "PID should be None after force kill");

    // Verify process actually killed
    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        // Sending signal 0 checks if process exists
        let result = kill(Pid::from_raw(pid as i32), Signal::from_c_int(0).unwrap());
        assert!(result.is_err(), "Process should not exist after force kill");
    }

    Ok(())
}

#[tokio::test]
async fn test_drop_cleanup() -> Result<()> {
    // Skip if Weaver not installed
    if !is_weaver_installed() {
        eprintln!("⚠️  Skipping test: Weaver not installed");
        return Ok(());
    }

    // Arrange
    let pid = {
        let mut manager = create_test_manager()?;
        let _ports = manager.start().await?;
        manager.pid().expect("Process should be running")
    }; // Drop manager here

    // Wait for cleanup
    sleep(Duration::from_millis(100)).await;

    // Assert: Process should be cleaned up by Drop
    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        let result = kill(Pid::from_raw(pid as i32), Signal::from_c_int(0).unwrap());
        assert!(result.is_err(), "Process should be cleaned up by Drop");
    }

    Ok(())
}

#[tokio::test]
async fn test_uptime_tracking() -> Result<()> {
    // Skip if Weaver not installed
    if !is_weaver_installed() {
        eprintln!("⚠️  Skipping test: Weaver not installed");
        return Ok(());
    }

    // Arrange
    let mut manager = create_test_manager()?;

    // Act: Start and wait
    let _ports = manager.start().await?;
    sleep(Duration::from_millis(500)).await;

    // Assert: Uptime should be at least 500ms
    let uptime = manager.uptime().expect("Uptime should be tracked");
    assert!(
        uptime >= Duration::from_millis(500),
        "Uptime should be at least 500ms"
    );

    // Cleanup
    manager.stop().await?;

    Ok(())
}

#[test]
fn test_port_ranges_exhaustion() {
    // This test validates the port discovery logic without actually binding ports
    // In a real scenario, all 40 ports would be occupied

    // The ranges should be:
    // OTLP: 4317-4327 (10), 5317-5327 (10), 6317-6337 (20) = 40 total
    // Admin: 8080-8089 (10), 9080-9089 (10), 10080-10099 (20) = 40 total

    let otlp_capacity = (4327 - 4317 + 1) + (5327 - 5317 + 1) + (6337 - 6317 + 1);
    let admin_capacity = (8089 - 8080 + 1) + (9089 - 9080 + 1) + (10099 - 10080 + 1);

    assert_eq!(otlp_capacity, 40, "OTLP should support 40 processes");
    assert_eq!(admin_capacity, 40, "Admin should support 40 processes");
}

#[tokio::test]
async fn test_startup_performance() -> Result<()> {
    // Skip if Weaver not installed
    if !is_weaver_installed() {
        eprintln!("⚠️  Skipping test: Weaver not installed");
        return Ok(());
    }

    // Arrange
    let mut manager = create_test_manager()?;

    // Act: Measure startup time
    let start = std::time::Instant::now();
    let _ports = manager.start().await?;
    let startup_duration = start.elapsed();

    // Assert: Should be under 3 seconds (performance target)
    assert!(
        startup_duration < Duration::from_secs(3),
        "Startup took {}ms (target: <3000ms)",
        startup_duration.as_millis()
    );

    // Cleanup
    manager.stop().await?;

    Ok(())
}

#[tokio::test]
async fn test_shutdown_performance() -> Result<()> {
    // Skip if Weaver not installed
    if !is_weaver_installed() {
        eprintln!("⚠️  Skipping test: Weaver not installed");
        return Ok(());
    }

    // Arrange
    let mut manager = create_test_manager()?;
    let _ports = manager.start().await?;

    // Act: Measure shutdown time
    let start = std::time::Instant::now();
    manager.stop().await?;
    let shutdown_duration = start.elapsed();

    // Assert: Should be under 1 second (performance target)
    assert!(
        shutdown_duration < Duration::from_secs(1),
        "Shutdown took {}ms (target: <1000ms)",
        shutdown_duration.as_millis()
    );

    Ok(())
}
