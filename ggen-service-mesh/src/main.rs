use ggen_service_mesh::{HealthMonitor, HealthStatus, ServiceEndpoint, ServiceRegistry};
use std::collections::HashMap;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("GGen Service Mesh\n");
    println!("=================\n");

    let registry = Arc::new(ServiceRegistry::new());

    println!("1. Registering services...");
    let api_id = registry.register(
        "api-service",
        ServiceEndpoint {
            host: "api.local".to_string(),
            port: 8080,
            protocol: "http".to_string(),
        },
        "1.0.0".to_string(),
        vec!["web".to_string(), "api".to_string()],
        HashMap::from([
            ("tier".to_string(), "web".to_string()),
            ("region".to_string(), "us-east-1".to_string()),
        ]),
    )?;
    println!("   ✓ API Service: {}", api_id);

    let db_id = registry.register(
        "postgres",
        ServiceEndpoint {
            host: "db.local".to_string(),
            port: 5432,
            protocol: "postgres".to_string(),
        },
        "15.0".to_string(),
        vec!["data".to_string(), "database".to_string()],
        HashMap::from([
            ("tier".to_string(), "data".to_string()),
            ("replica".to_string(), "primary".to_string()),
        ]),
    )?;
    println!("   ✓ PostgreSQL: {}", db_id);

    let cache_id = registry.register(
        "redis",
        ServiceEndpoint {
            host: "cache.local".to_string(),
            port: 6379,
            protocol: "redis".to_string(),
        },
        "7.0".to_string(),
        vec!["data".to_string(), "cache".to_string()],
        HashMap::new(),
    )?;
    println!("   ✓ Redis Cache: {}", cache_id);

    let queue_id = registry.register(
        "rabbitmq",
        ServiceEndpoint {
            host: "queue.local".to_string(),
            port: 5672,
            protocol: "amqp".to_string(),
        },
        "3.12".to_string(),
        vec!["messaging".to_string()],
        HashMap::new(),
    )?;
    println!("   ✓ RabbitMQ: {}\n", queue_id);

    println!("2. Performing initial health checks...");
    let monitor = HealthMonitor::new(registry.clone());
    let health_results = monitor.perform_full_check()?;

    for (service_name, status) in health_results {
        let status_str = match status {
            HealthStatus::Healthy => "✓ Healthy",
            HealthStatus::Degraded => "⚠ Degraded",
            HealthStatus::Unhealthy => "✗ Unhealthy",
            HealthStatus::Unknown => "? Unknown",
        };
        println!("   {} - {}", service_name, status_str);
    }
    println!();

    println!("3. Discovering services by name...");
    let api_instances = registry.discover("api-service")?;
    println!("   api-service instances: {}", api_instances.len());
    for instance in &api_instances {
        println!(
            "     - {} [{}]: {}",
            instance.registration.service_name,
            instance.registration.service_id,
            instance.registration.endpoint.url()
        );
    }
    println!();

    println!("4. Listing services by tag...");
    let data_services = registry.list_by_tag("data");
    println!("   Services tagged 'data': {}", data_services.len());
    for service in &data_services {
        println!(
            "     - {} ({})",
            service.registration.service_name, service.registration.version
        );
    }
    println!();

    println!("5. Simulating request traffic and metrics...");
    registry.update_metrics(&api_id, 45, true)?;
    registry.update_metrics(&api_id, 52, true)?;
    registry.update_metrics(&api_id, 250, false)?;
    registry.update_metrics(&api_id, 38, true)?;

    registry.update_metrics(&db_id, 15, true)?;
    registry.update_metrics(&db_id, 18, true)?;
    registry.update_metrics(&db_id, 20, true)?;

    registry.update_metrics(&cache_id, 5, true)?;
    registry.update_metrics(&cache_id, 4, true)?;
    registry.update_metrics(&cache_id, 6, true)?;
    registry.update_metrics(&cache_id, 500, false)?;

    let api_service = registry.get_service(&api_id)?;
    println!("   API Service Metrics:");
    println!(
        "     Total requests: {}",
        api_service.metrics.total_requests
    );
    println!(
        "     Successful: {}, Failed: {}",
        api_service.metrics.successful_requests, api_service.metrics.failed_requests
    );
    println!(
        "     Avg latency: {} ms",
        api_service.metrics.latency_ms
    );
    println!();

    println!("6. Updating health status based on metrics...");
    registry.update_health(&api_id, HealthStatus::Degraded)?;
    registry.update_health(&db_id, HealthStatus::Healthy)?;
    registry.update_health(&cache_id, HealthStatus::Degraded)?;
    registry.update_health(&queue_id, HealthStatus::Healthy)?;

    let stats = registry.get_registry_stats();
    println!("   Total services: {}", stats.total_services);
    println!("   Healthy: {}", stats.healthy_services);
    println!("   Degraded: {}", stats.degraded_services);
    println!("   Unhealthy: {}", stats.unhealthy_services);
    println!();

    println!("7. Finding healthy instances...");
    match registry.get_healthy_instances("api-service") {
        Ok(healthy) => {
            println!("   ✓ Found {} healthy instances", healthy.len());
        }
        Err(e) => {
            println!("   ⚠ {}", e);
        }
    }

    match registry.get_healthy_instances("postgres") {
        Ok(healthy) => {
            println!("   ✓ Found {} healthy instances", healthy.len());
        }
        Err(e) => {
            println!("   ⚠ {}", e);
        }
    }
    println!();

    println!("8. Listing all registered services...");
    let all_services = registry.list_all();
    println!("   Total: {}", all_services.len());
    for service in all_services {
        let status = match service.metrics.health_status {
            HealthStatus::Healthy => "✓",
            HealthStatus::Degraded => "⚠",
            HealthStatus::Unhealthy => "✗",
            HealthStatus::Unknown => "?",
        };
        println!(
            "     {} {} - {} ({})",
            status,
            service.registration.service_name,
            service.registration.endpoint.url(),
            service.registration.version
        );
    }
    println!();

    println!("9. Service metadata...");
    let api_service = registry.get_service(&api_id)?;
    println!("   API Service Metadata:");
    println!("     Tier: {}", api_service.registration.metadata.get("tier").unwrap_or(&"N/A".to_string()));
    println!("     Region: {}", api_service.registration.metadata.get("region").unwrap_or(&"N/A".to_string()));
    println!("     Tags: {}", api_service.registration.tags.join(", "));
    println!();

    println!("10. Deregistering a service...");
    println!("   Removing: {}", queue_id);
    registry.deregister(&queue_id)?;
    println!("   ✓ Deregistered successfully");
    println!("   Remaining services: {}", registry.list_all().len());
    println!();

    println!("=================");
    println!("Service mesh demo complete! ✓");
    Ok(())
}
