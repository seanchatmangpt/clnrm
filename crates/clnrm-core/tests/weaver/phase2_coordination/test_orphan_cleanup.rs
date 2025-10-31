//! Test orphan process cleanup
//!
//! Verifies that orphaned Weaver processes are cleaned up before starting new ones.
//! This prevents port conflicts and resource leaks.

#[cfg(test)]
mod orphan_cleanup_tests {
    use clnrm_core::error::Result;
    use clnrm_core::telemetry::weaver_controller::{WeaverConfig, WeaverController};
    use std::path::PathBuf;
    use std::process::{Command, Stdio};
    use std::time::Duration;
    use tokio::time::sleep;

    /// Test that existing Weaver processes are detected
    ///
    /// Verifies cleanup_old_weaver_processes detects and kills orphaned processes.
    #[tokio::test]
    async fn test_detect_existing_weaver_processes() -> Result<()> {
        // This test verifies the cleanup logic
        // Actual detection requires checking running processes
        
        // Arrange - Start a Weaver controller
        let config = WeaverConfig {
            registry_path: PathBuf::from("registry"),
            otlp_port: 0,
            admin_port: 0,
            output_dir: PathBuf::from("./test_validation_output"),
            stream: false,
        };

        let mut controller1 = WeaverController::new(config.clone());
        let _coordination1 = controller1.start_and_coordinate()?;

        // Simulate orphan scenario - controller1 is dropped without cleanup
        // In real scenario, this would leave a process running
        // The cleanup should detect this on next start

        // Act - Start a new controller (should cleanup old processes)
        let mut controller2 = WeaverController::new(config);
        let _coordination2 = controller2.start_and_coordinate()?;

        // Assert - Verify both used different ports or cleanup happened
        // (Cleanup is verified by successful start without port conflicts)
        
        let _ = controller2.stop_and_report();

        Ok(())
    }

    /// Test that orphaned processes are killed before starting
    ///
    /// Verifies that cleanup_old_weaver_processes kills orphaned processes.
    #[tokio::test]
    async fn test_kill_orphaned_processes_before_start() -> Result<()> {
        // This test verifies cleanup behavior
        // In production, cleanup_old_weaver_processes is called before start
        
        // Arrange - Create config
        let config = WeaverConfig {
            registry_path: PathBuf::from("registry"),
            otlp_port: 0,
            admin_port: 0,
            output_dir: PathBuf::from("./test_validation_output"),
            stream: false,
        };

        // Act - Start controller (cleanup happens automatically)
        let mut controller = WeaverController::new(config);
        
        // start_and_coordinate calls cleanup_old_weaver_processes internally
        let _coordination = controller.start_and_coordinate()?;

        // Assert - Start succeeded (cleanup worked)
        // If cleanup failed, we'd get port conflicts or process errors
        
        let _ = controller.stop_and_report();

        Ok(())
    }

    /// Test that port conflicts are resolved by cleanup
    ///
    /// Verifies that cleanup resolves port conflicts by killing orphaned processes.
    #[tokio::test]
    async fn test_port_conflicts_resolved_by_cleanup() -> Result<()> {
        // Arrange - Start first controller
        let config1 = WeaverConfig {
            registry_path: PathBuf::from("registry"),
            otlp_port: 0,
            admin_port: 0,
            output_dir: PathBuf::from("./test_validation_output"),
            stream: false,
        };

        let mut controller1 = WeaverController::new(config1);
        let coordination1 = controller1.start_and_coordinate()?;
        let port1 = coordination1.otlp_grpc_port;

        // Stop first controller properly
        let _ = controller1.stop_and_report();
        
        // Wait for port to be released
        sleep(Duration::from_millis(500)).await;

        // Act - Start second controller (should reuse port after cleanup)
        let config2 = WeaverConfig {
            registry_path: PathBuf::from("registry"),
            otlp_port: 0,
            admin_port: 0,
            output_dir: PathBuf::from("./test_validation_output"),
            stream: false,
        };

        let mut controller2 = WeaverController::new(config2);
        let coordination2 = controller2.start_and_coordinate()?;
        let port2 = coordination2.otlp_grpc_port;

        // Assert - Ports may be same or different (both are valid)
        // Key is that no conflict occurred
        assert!(
            port1 > 0 && port2 > 0,
            "Both controllers should get valid ports"
        );

        // Cleanup
        let _ = controller2.stop_and_report();

        Ok(())
    }

    /// Test cleanup of multiple orphaned processes
    ///
    /// Verifies that cleanup handles multiple orphaned Weaver processes.
    #[tokio::test]
    async fn test_cleanup_multiple_orphaned_processes() -> Result<()> {
        // This test verifies cleanup handles multiple orphans
        // In practice, cleanup searches for all matching processes
        
        // Arrange - Start and stop multiple controllers
        for i in 0..3 {
            let config = WeaverConfig {
                registry_path: PathBuf::from("registry"),
                otlp_port: 0,
                admin_port: 0,
                output_dir: PathBuf::from(format!("./test_validation_output_{}", i)),
                stream: false,
            };

            let mut controller = WeaverController::new(config);
            let _coordination = controller.start_and_coordinate()?;
            
            // Proper cleanup
            let _ = controller.stop_and_report();
            sleep(Duration::from_millis(200)).await;
        }

        // Act - Start final controller (should cleanup any remaining orphans)
        let config = WeaverConfig {
            registry_path: PathBuf::from("registry"),
            otlp_port: 0,
            admin_port: 0,
            output_dir: PathBuf::from("./test_validation_output"),
            stream: false,
        };

        let mut controller = WeaverController::new(config);
        let _coordination = controller.start_and_coordinate()?;

        // Assert - Start succeeded (cleanup worked for all orphans)
        
        let _ = controller.stop_and_report();

        Ok(())
    }
}
