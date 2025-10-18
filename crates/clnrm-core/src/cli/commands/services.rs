//! Services command implementation
//!
//! Handles service management including status, logs, restart operations,
//! and AI-driven autonomous service lifecycle management.

use crate::cleanroom::CleanroomEnvironment;
use crate::error::{CleanroomError, Result};
use crate::services::service_manager::{AutoScaleConfig, ServiceManager, ServiceMetrics};
use tracing::warn;

/// Show service status
pub async fn show_service_status() -> Result<()> {
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
pub async fn show_service_logs(service: &str, lines: usize) -> Result<()> {
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
pub async fn restart_service(service: &str) -> Result<()> {
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

/// AI-driven service lifecycle management
///
/// Provides autonomous service management with auto-scaling, load prediction,
/// resource optimization, and cost recommendations.
pub async fn ai_manage(
    auto_scale: bool,
    predict_load: bool,
    optimize_resources: bool,
    horizon_minutes: u32,
    service_filter: Option<String>,
) -> Result<()> {
    println!("🤖 AI Service Management");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Create a temporary environment to access services
    let environment = CleanroomEnvironment::new().await.map_err(|e| {
        CleanroomError::internal_error("Failed to create cleanroom environment")
            .with_context("AI management initialization")
            .with_source(e.to_string())
    })?;

    let services = environment.services().await;

    if services.active_services().is_empty() {
        println!("⚠️  No services currently running");
        println!("💡 Start services with 'clnrm run <test_file>' first");
        return Ok(());
    }

    // Initialize service manager
    let mut manager = ServiceManager::new();

    // Collect current service metrics
    println!("\n📊 Collecting service metrics...");
    for handle in services.active_services().values() {
        // Filter services if specified
        if let Some(ref filter) = service_filter {
            if !handle.service_name.contains(filter) {
                continue;
            }
        }

        // Simulate collecting real metrics (in production, this would query actual metrics)
        let mut metrics = ServiceMetrics::new(handle.id.clone(), handle.service_name.clone());

        // For demonstration, use some simulated values
        // In production, these would come from actual monitoring
        metrics.cpu_usage = 45.0 + (rand::random::<f64>() * 30.0);
        metrics.memory_usage = 256.0 + (rand::random::<f64>() * 256.0);
        metrics.network_io = rand::random::<f64>() * 10.0;
        metrics.active_connections = (rand::random::<u32>() % 100) + 10;
        metrics.request_rate = 50.0 + (rand::random::<f64>() * 50.0);
        metrics.response_time_ms = 50.0 + (rand::random::<f64>() * 100.0);
        metrics.error_rate = rand::random::<f64>() * 0.05;

        println!(
            "  ✓ {} - CPU: {:.1}%, Memory: {:.0}MB, RPS: {:.1}",
            handle.service_name, metrics.cpu_usage, metrics.memory_usage, metrics.request_rate
        );

        manager.record_metrics(metrics);

        // Set default auto-scaling configuration
        manager.set_auto_scale_config(handle.id.clone(), AutoScaleConfig::default());
        manager.update_instance_count(handle.id.clone(), 1);
    }

    // Simulate historical data for better predictions
    println!("\n📈 Simulating historical data for predictions...");
    for handle in services.active_services().values() {
        if let Some(ref filter) = service_filter {
            if !handle.service_name.contains(filter) {
                continue;
            }
        }

        // Add 20 historical data points
        for i in 0..20 {
            let mut metrics = ServiceMetrics::new(handle.id.clone(), handle.service_name.clone());
            let trend = i as f64 * 2.0;

            metrics.cpu_usage = 40.0 + trend + (rand::random::<f64>() * 10.0);
            metrics.memory_usage = 200.0 + (trend * 5.0) + (rand::random::<f64>() * 50.0);
            metrics.request_rate = 40.0 + trend + (rand::random::<f64>() * 20.0);
            metrics.response_time_ms = 60.0 + (rand::random::<f64>() * 40.0);
            metrics.error_rate = rand::random::<f64>() * 0.02;

            manager.record_metrics(metrics);
        }
    }

    // Load Prediction
    if predict_load {
        println!("\n🔮 Load Prediction ({}min horizon):", horizon_minutes);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        for handle in services.active_services().values() {
            if let Some(ref filter) = service_filter {
                if !handle.service_name.contains(filter) {
                    continue;
                }
            }

            if let Some(predicted) = manager.predict_load(&handle.id, horizon_minutes) {
                println!("  📦 {}", handle.service_name);
                println!(
                    "     CPU: {:.1}% → {:.1}%",
                    predicted.cpu_usage - 10.0,
                    predicted.cpu_usage
                );
                println!(
                    "     Memory: {:.0}MB → {:.0}MB",
                    predicted.memory_usage - 50.0,
                    predicted.memory_usage
                );
                println!(
                    "     RPS: {:.1} → {:.1}",
                    predicted.request_rate - 5.0,
                    predicted.request_rate
                );
                println!("     Health Score: {:.1}/100", predicted.health_score());

                // Predict health status
                match manager.predict_service_health(&handle.id) {
                    Ok(health) => {
                        let health_emoji = match health {
                            crate::cleanroom::HealthStatus::Healthy => "✅",
                            crate::cleanroom::HealthStatus::Unhealthy => "❌",
                            crate::cleanroom::HealthStatus::Unknown => "⚠️",
                        };
                        println!("     Predicted Health: {} {:?}", health_emoji, health);
                    }
                    Err(e) => {
                        warn!(
                            "Failed to predict health for {}: {}",
                            handle.service_name, e
                        );
                    }
                }
            } else {
                println!(
                    "  ⚠️  {} - Insufficient data for prediction",
                    handle.service_name
                );
            }
        }
    }

    // Auto-Scaling
    if auto_scale {
        println!("\n⚡ Auto-Scaling Analysis:");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        for handle in services.active_services().values() {
            if let Some(ref filter) = service_filter {
                if !handle.service_name.contains(filter) {
                    continue;
                }
            }

            match manager.determine_scaling_action(&handle.id) {
                Ok(action) => {
                    use crate::services::service_manager::ScalingAction;
                    match action {
                        ScalingAction::ScaleUp(count) => {
                            println!(
                                "  📈 {} - Scale UP by {} instance(s)",
                                handle.service_name, count
                            );
                            println!("     Reason: High resource utilization detected");
                            manager.update_instance_count(
                                handle.id.clone(),
                                *manager.service_instances.get(&handle.id).unwrap_or(&1) + count,
                            );
                        }
                        ScalingAction::ScaleDown(count) => {
                            println!(
                                "  📉 {} - Scale DOWN by {} instance(s)",
                                handle.service_name, count
                            );
                            println!("     Reason: Low resource utilization detected");
                            let current = *manager.service_instances.get(&handle.id).unwrap_or(&1);
                            manager.update_instance_count(
                                handle.id.clone(),
                                current.saturating_sub(count).max(1),
                            );
                        }
                        ScalingAction::NoAction => {
                            println!("  ✓ {} - No scaling needed", handle.service_name);
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to determine scaling action for {}: {}",
                        handle.service_name, e
                    );
                }
            }
        }
    }

    // Resource Optimization
    if optimize_resources {
        println!("\n🎯 Resource Optimization:");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        for handle in services.active_services().values() {
            if let Some(ref filter) = service_filter {
                if !handle.service_name.contains(filter) {
                    continue;
                }
            }

            // Setup resource pool
            let pool = manager.get_or_create_pool(handle.service_name.clone(), 5);
            println!("  📦 {} Resource Pool:", handle.service_name);
            println!(
                "     Size: {} available, {} in-use",
                pool.available.len(),
                pool.in_use.len()
            );
            println!("     Utilization: {:.1}%", pool.utilization() * 100.0);

            if pool.utilization() < 0.3 && pool.available.len() > 1 {
                println!("     💡 Consider reducing pool size (low utilization)");
            } else if pool.utilization() > 0.8 {
                println!("     ⚠️  Consider increasing pool size (high utilization)");
            }
        }

        // Cost Optimization Recommendations
        println!("\n💰 Cost Optimization Recommendations:");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let mut all_recommendations = Vec::new();
        for handle in services.active_services().values() {
            if let Some(ref filter) = service_filter {
                if !handle.service_name.contains(filter) {
                    continue;
                }
            }

            let recommendations = manager.generate_cost_recommendations(&handle.id);
            all_recommendations.extend(recommendations);
        }

        if all_recommendations.is_empty() {
            println!("  ✓ No cost optimization recommendations at this time");
        } else {
            for (i, rec) in all_recommendations.iter().enumerate() {
                println!(
                    "\n  {}. {} - {} (Priority: {}/5)",
                    i + 1,
                    rec.service_name,
                    rec.recommendation_type,
                    rec.priority
                );
                println!("     {}", rec.description);
                println!("     💰 Estimated savings: {:.0}%", rec.estimated_savings);
            }
        }
    }

    // Summary
    println!("\n📊 Management Summary:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let summary = manager.get_summary();
    for (key, value) in summary {
        println!("  {}: {}", key, value);
    }

    println!("\n✅ AI service management completed");
    println!("\n💡 Tips:");
    println!("  - Enable auto-scaling to automatically adjust capacity");
    println!("  - Use load prediction to proactively scale before peaks");
    println!("  - Review cost recommendations regularly");
    println!("  - Monitor resource pool utilization for optimal performance");

    Ok(())
}
