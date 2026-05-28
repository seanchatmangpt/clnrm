//! Advanced Rust API Example
//!
//! This demonstrates advanced usage of the CleanroomEnvironment API
//! including service plugins, custom metrics, and error handling.

use clnrm_core::cleanroom::{CleanroomEnvironment, ServicePlugin, ServiceHandle, HealthStatus};
use clnrm_core::error::CleanroomError;
use std::collections::HashMap;

/// Example custom service plugin
struct MockDatabaseService {
    name: String,
}

impl MockDatabaseService {
    fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl ServicePlugin for MockDatabaseService {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle, CleanroomError> {
        println!("   Starting mock database service: {}", self.name);

        Ok(ServiceHandle {
            id: format!("mock-db-{}", self.name),
            service_name: self.name.clone(),
            metadata: HashMap::from([
                ("port".to_string(), "5432".to_string()),
                ("type".to_string(), "postgresql".to_string()),
            ]),
        })
    }

    fn stop(&self, handle: ServiceHandle) -> Result<(), CleanroomError> {
        println!("   Stopping mock database service: {}", handle.service_name);
        Ok(())
    }

    fn health_check(&self, _handle: &ServiceHandle) -> HealthStatus {
        HealthStatus::Healthy
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Advanced Rust API Example");
    println!("============================");

    // Create environment
    println!("\n📋 Creating CleanroomEnvironment...");
    let env = CleanroomEnvironment::new().await?;
    println!("✅ Environment created: {}", env.session_id());

    // Register custom service
    println!("\n📋 Registering custom service plugin...");
    let db_service = MockDatabaseService::new("test-database");
    env.services().write().await.register_plugin(Box::new(db_service));
    println!("✅ Custom service plugin registered");

    // List available services
    println!("\n📋 Available services:");
    let services = env.services().read().await;
    for service_name in services.active_services().keys() {
        println!("   - {}", service_name);
    }

    // Run multiple tests
    println!("\n📋 Running multiple tests...");

    let test_results = vec![
        env.execute_test("database_connection_test", || {
            println!("   Testing database connection...");
            Ok::<String, CleanroomError>("db_connected".to_string())
        }).await?,

        env.execute_test("api_integration_test", || {
            println!("   Testing API integration...");
            Ok::<String, CleanroomError>("api_working".to_string())
        }).await?,

        env.execute_test("performance_test", || {
            println!("   Running performance checks...");
            Ok::<String, CleanroomError>("performance_ok".to_string())
        }).await?,
    ];

    println!("✅ Test results:");
    for (i, result) in test_results.iter().enumerate() {
        println!("   Test {}: {}", i + 1, result);
    }

    // Final metrics
    println!("\n📋 Final metrics:");
    let metrics = env.get_metrics().await?;
    println!("   - Total tests executed: {}", metrics.tests_executed);
    println!("   - Services active: {}", metrics.active_services);
    println!("   - Containers active: {}", metrics.active_containers);

    // Health check all services
    println!("\n📋 Service health check:");
    let health_status = env.check_health().await;
    for (service_name, status) in health_status {
        let status_str = match status {
            HealthStatus::Healthy => "✅ Healthy",
            HealthStatus::Unhealthy => "❌ Unhealthy",
        };
        println!("   {}: {}", service_name, status_str);
    }

    println!("\n🎉 Advanced example completed!");
    println!("💡 This demonstrates:");
    println!("   - Custom service plugins");
    println!("   - Multiple test execution");
    println!("   - Health monitoring");
    println!("   - Metrics collection");
    println!("   - Proper error handling");

    Ok(())
}

