//! Services command implementation
//!
//! Handles service management including status, logs, and restart operations.

use crate::error::{CleanroomError, Result};
use crate::cleanroom::CleanroomEnvironment;
use tracing::{info, debug, warn};

/// Show service status
pub async fn show_service_status() -> Result<()> {
    println!("📊 Service Status:");
    
    // Create a temporary environment to check for any active services
    let environment = CleanroomEnvironment::new().await
        .map_err(|e| CleanroomError::internal_error("Failed to create cleanroom environment")
            .with_context("Service status command initialization")
            .with_source(e.to_string()))?;
    let services = environment.services().await;
    
    if services.active_services().is_empty() {
        println!("✅ No services currently running");
        println!("💡 Run 'clnrm run <test_file>' to start services");
    } else {
        println!("Active Services: {}", services.active_services().len());
        for (_handle_id, handle) in services.active_services() {
            println!("Service: {} (ID: {})", handle.service_name, handle.id);
            if !handle.metadata.is_empty() {
                for (key, value) in &handle.metadata {
                    println!("  {}: {}", key, value);
                }
            }
        }
    }
    
    Ok(())
}

/// Show service logs
pub async fn show_service_logs(service: &str, lines: usize) -> Result<()> {
    println!("📄 Service Logs for '{}':", service);
    
    // Create a temporary environment to check for services
    let environment = CleanroomEnvironment::new().await
        .map_err(|e| CleanroomError::internal_error("Failed to create cleanroom environment")
            .with_context("Service logs command initialization")
            .with_source(e.to_string()))?;
    let services = environment.services().await;
    
    // Find the service by name
    let service_handle = services.active_services()
        .values()
        .find(|handle| handle.service_name == service);
    
    match service_handle {
        Some(handle) => {
            println!("Service found: {} (ID: {})", handle.service_name, handle.id);
            
            // Try to retrieve logs from the service
            match environment.get_service_logs(&handle.id, lines).await {
                Ok(logs) => {
                    if logs.is_empty() {
                        println!("📄 No logs available for service '{}'", service);
                    } else {
                        println!("📄 Recent logs (last {} lines):", lines);
                        for log_line in logs {
                            println!("  {}", log_line);
                        }
                    }
                }
                Err(e) => {
                    println!("⚠️  Could not retrieve logs: {}", e);
                    println!("💡 Service '{}' is running but log access may not be available", service);
                }
            }
            
            if !handle.metadata.is_empty() {
                println!("Metadata:");
                for (key, value) in &handle.metadata {
                    println!("  {}: {}", key, value);
                }
            }
        }
        None => {
            println!("❌ Service '{}' not found in active services", service);
            println!("Available services:");
            for (_, handle) in services.active_services() {
                println!("  - {}", handle.service_name);
            }
            if services.active_services().is_empty() {
                println!("No services currently running");
                println!("Run 'clnrm run <test_file>' to start services");
            }
        }
    }
    
    Ok(())
}

/// Restart a service
pub async fn restart_service(service: &str) -> Result<()> {
    println!("🔄 Restarting service '{}':", service);
    
    // Create a temporary environment to check for services
    let environment = CleanroomEnvironment::new().await
        .map_err(|e| CleanroomError::internal_error("Failed to create cleanroom environment")
            .with_context("Service restart command initialization")
            .with_source(e.to_string()))?;
    let services = environment.services().await;
    
    // Find the service by name
    let service_handle = services.active_services()
        .values()
        .find(|handle| handle.service_name == service);
    
    match service_handle {
        Some(handle) => {
            println!("Service found: {} (ID: {})", handle.service_name, handle.id);
            
            // Stop the service
            println!("Stopping service...");
            environment.stop_service(&handle.id).await
                .map_err(|e| CleanroomError::internal_error("Failed to stop service")
                    .with_context(format!("Service: {}", service))
                    .with_source(e.to_string()))?;
            println!("Service stopped");
            
            // Wait a moment for cleanup
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            
            // Start the service again
            println!("Starting service...");
            let new_handle = environment.start_service(service).await
                .map_err(|e| CleanroomError::internal_error("Failed to restart service")
                    .with_context(format!("Service: {}", service))
                    .with_source(e.to_string()))?;
            println!("Service restarted");
            println!("New service ID: {}", new_handle.id);
            
            println!("✅ Service '{}' restarted successfully", service);
        }
        None => {
            println!("❌ Service '{}' not found in active services", service);
            println!("Available services:");
            for (_, handle) in services.active_services() {
                println!("  - {}", handle.service_name);
            }
            if services.active_services().is_empty() {
                println!("No services currently running");
                println!("Run 'clnrm run <test_file>' to start services");
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_show_service_status() -> Result<()> {
        // Act
        let result = show_service_status().await;
        
        // Assert
        assert!(result.is_ok());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_show_service_logs() -> Result<()> {
        // Act
        let result = show_service_logs("test_service", 10).await;
        
        // Assert
        assert!(result.is_ok());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_show_service_logs_with_different_line_counts() -> Result<()> {
        // Test different line count values
        let line_counts = vec![1, 10, 50, 100];
        
        for lines in line_counts {
            // Act
            let result = show_service_logs("test_service", lines).await;
            
            // Assert
            assert!(result.is_ok(), "Should handle {} lines", lines);
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_show_service_logs_empty_service_name() -> Result<()> {
        // Act
        let result = show_service_logs("", 10).await;
        
        // Assert
        assert!(result.is_ok());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_restart_service() -> Result<()> {
        // Act
        let result = restart_service("test_service").await;
        
        // Assert
        assert!(result.is_ok());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_restart_service_empty_name() -> Result<()> {
        // Act
        let result = restart_service("").await;
        
        // Assert
        assert!(result.is_ok());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_restart_service_nonexistent_service() -> Result<()> {
        // Act
        let result = restart_service("nonexistent_service").await;
        
        // Assert
        assert!(result.is_ok());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_service_functions_with_special_characters() -> Result<()> {
        // Test service names with special characters
        let service_names = vec![
            "service-with-dashes",
            "service_with_underscores",
            "service.with.dots",
            "service with spaces",
            "service@with#special$chars",
        ];
        
        for service_name in service_names {
            // Test show_service_logs
            let logs_result = show_service_logs(service_name, 10).await;
            assert!(logs_result.is_ok(), "show_service_logs should handle service name: {}", service_name);
            
            // Test restart_service
            let restart_result = restart_service(service_name).await;
            assert!(restart_result.is_ok(), "restart_service should handle service name: {}", service_name);
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_service_functions_with_unicode() -> Result<()> {
        // Test service names with unicode characters
        let service_names = vec![
            "service-测试",
            "service-🚀",
            "service-émojis",
            "service-中文",
        ];
        
        for service_name in service_names {
            // Test show_service_logs
            let logs_result = show_service_logs(service_name, 10).await;
            assert!(logs_result.is_ok(), "show_service_logs should handle unicode service name: {}", service_name);
            
            // Test restart_service
            let restart_result = restart_service(service_name).await;
            assert!(restart_result.is_ok(), "restart_service should handle unicode service name: {}", service_name);
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_show_service_status_creates_environment() -> Result<()> {
        // Act
        let result = show_service_status().await;
        
        // Assert
        assert!(result.is_ok());
        
        // The function should complete without panicking when creating CleanroomEnvironment
        Ok(())
    }

    #[tokio::test]
    async fn test_show_service_logs_creates_environment() -> Result<()> {
        // Act
        let result = show_service_logs("test", 10).await;
        
        // Assert
        assert!(result.is_ok());
        
        // The function should complete without panicking when creating CleanroomEnvironment
        Ok(())
    }

    #[tokio::test]
    async fn test_restart_service_creates_environment() -> Result<()> {
        // Act
        let result = restart_service("test").await;
        
        // Assert
        assert!(result.is_ok());
        
        // The function should complete without panicking when creating CleanroomEnvironment
        Ok(())
    }
}
