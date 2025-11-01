//! Services command implementation using noun-verb pattern

use crate::cleanroom::CleanroomEnvironment;
use crate::error::{CleanroomError, Result};
use clap_noun_verb::{noun, verb, NounVerbError, VerbArgs};

/// Create the services noun command
pub fn services_command() -> impl clap_noun_verb::NounCommand {
    noun!(
        "services",
        "Manage application services",
        [
            verb!(
                "status",
                "Show status of all services",
                |_args: &VerbArgs| {
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            show_service_status()
                                .await
                                .map_err(|e| NounVerbError::ExecutionError {
                                    message: e.to_string(),
                                })
                        })
                    })
                }
            ),
            verb!("logs", "Show logs for a service", |_args: &VerbArgs| {
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        // Get service name from args - in a real implementation, this would come from clap args
                        let service = "default-service"; // This should be extracted from args.matches
                        let lines = 50; // This should be extracted from args.matches
                        show_service_logs(service, lines).await.map_err(|e| {
                            NounVerbError::ExecutionError {
                                message: e.to_string(),
                            }
                        })
                    })
                })
            }),
            verb!("restart", "Restart a service", |_args: &VerbArgs| {
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        // Get service name from args - in a real implementation, this would come from clap args
                        let service = "default-service"; // This should be extracted from args.matches
                        restart_service(service)
                            .await
                            .map_err(|e| NounVerbError::ExecutionError {
                                message: e.to_string(),
                            })
                    })
                })
            }),
        ]
    )
}

/// Show service status
async fn show_service_status() -> Result<()> {
    println!("📊 Service Status:");

    // Create a temporary environment to check for any active services
    let environment = CleanroomEnvironment::new().await.map_err(|e| {
        CleanroomError::internal_error("Failed to create cleanroom environment")
            .with_context("Service status command initialization")
            .with_source(e.to_string())
    })?;
    let services = environment.services().await;

    if services.active_services().is_empty() {
        println!("✅ No services currently running");
        println!("💡 Run 'clnrm run <test_file>' to start services");
    } else {
        println!("Active Services: {}", services.active_services().len());
        for handle in services.active_services().values() {
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
async fn show_service_logs(service: &str, lines: usize) -> Result<()> {
    println!("📄 Service Logs for '{}':", service);

    // Create a temporary environment to check for services
    let environment = CleanroomEnvironment::new().await.map_err(|e| {
        CleanroomError::internal_error("Failed to create cleanroom environment")
            .with_context("Service logs command initialization")
            .with_source(e.to_string())
    })?;
    let services = environment.services().await;

    // Find the service by name
    let service_handle = services
        .active_services()
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
                    println!(
                        "💡 Service '{}' is running but log access may not be available",
                        service
                    );
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
            for handle in services.active_services().values() {
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
async fn restart_service(service: &str) -> Result<()> {
    println!("🔄 Restarting service '{}':", service);

    // Create a temporary environment to check for services
    let environment = CleanroomEnvironment::new().await.map_err(|e| {
        CleanroomError::internal_error("Failed to create cleanroom environment")
            .with_context("Service restart command initialization")
            .with_source(e.to_string())
    })?;
    let services = environment.services().await;

    // Find the service by name
    let service_handle = services
        .active_services()
        .values()
        .find(|handle| handle.service_name == service);

    match service_handle {
        Some(handle) => {
            println!("Service found: {} (ID: {})", handle.service_name, handle.id);

            // Stop the service
            println!("Stopping service...");
            environment.stop_service(&handle.id).await.map_err(|e| {
                CleanroomError::internal_error("Failed to stop service")
                    .with_context(format!("Service: {}", service))
                    .with_source(e.to_string())
            })?;
            println!("Service stopped");

            // Wait a moment for cleanup
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            // Start the service again
            println!("Starting service...");
            let new_handle = environment.start_service(service).await.map_err(|e| {
                CleanroomError::internal_error("Failed to restart service")
                    .with_context(format!("Service: {}", service))
                    .with_source(e.to_string())
            })?;
            println!("Service restarted");
            println!("New service ID: {}", new_handle.id);

            println!("✅ Service '{}' restarted successfully", service);
        }
        None => {
            println!("❌ Service '{}' not found in active services", service);
            println!("Available services:");
            for handle in services.active_services().values() {
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
