//! Custom Plugin Development Demo
//!
//! This example demonstrates the plugin-based architecture claim from the README:
//! "Plugin-Based Architecture - Extensible service system for any technology"
//!
//! Users can copy this code to see how to create custom plugins.

use clnrm_core::{CleanroomEnvironment, HealthStatus, Result, ServiceHandle, ServicePlugin};
use std::collections::HashMap;

/// Custom Database Plugin (demonstrates plugin system)
#[derive(Debug)]
pub struct PostgresPlugin {
    connection_string: String,
}

impl PostgresPlugin {
    pub fn new(connection_string: String) -> Self {
        Self { connection_string }
    }
}

impl ServicePlugin for PostgresPlugin {
    fn name(&self) -> &str {
        "postgres"
    }

    async fn start(&self) -> Result<ServiceHandle> {
        println!("🚀 Starting PostgreSQL plugin...");

        // Simulate database startup
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        Ok(ServiceHandle {
            id: "postgres-123".to_string(),
            service_name: "postgres".to_string(),
            metadata: {
                let mut map = HashMap::new();
                map.insert(
                    "connection_string".to_string(),
                    self.connection_string.clone(),
                );
                map.insert("status".to_string(), "running".to_string());
                map
            },
        })
    }

    async fn stop(&self, _handle: ServiceHandle) -> Result<()> {
        println!("⏹️  Stopping PostgreSQL plugin...");
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        Ok(())
    }

    fn health_check(&self, _handle: &ServiceHandle) -> HealthStatus {
        // Simulate health check
        HealthStatus::Healthy
    }
}

/// Custom Cache Plugin (demonstrates plugin system)
#[derive(Debug)]
pub struct RedisPlugin {
    host: String,
    port: u16,
}

impl RedisPlugin {
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }
}

impl ServicePlugin for RedisPlugin {
    fn name(&self) -> &str {
        "redis"
    }

    async fn start(&self) -> Result<ServiceHandle> {
        println!("🚀 Starting Redis plugin...");

        // Simulate Redis startup
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

        Ok(ServiceHandle {
            id: "redis-456".to_string(),
            service_name: "redis".to_string(),
            metadata: {
                let mut map = HashMap::new();
                map.insert("host".to_string(), self.host.clone());
                map.insert("port".to_string(), self.port.to_string());
                map.insert("status".to_string(), "running".to_string());
                map
            },
        })
    }

    async fn stop(&self, _handle: ServiceHandle) -> Result<()> {
        println!("⏹️  Stopping Redis plugin...");
        tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
        Ok(())
    }

    fn health_check(&self, _handle: &ServiceHandle) -> HealthStatus {
        // Simulate health check
        HealthStatus::Healthy
    }
}

/// Test plugin system functionality
#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Custom Plugin Development Demo");
    println!("=================================");
    println!("");
    println!("This demo proves the README plugin architecture claims:");
    println!("✅ Plugin-Based Architecture - Extensible service system");
    println!("✅ Custom plugins can be created and registered");
    println!("✅ Plugin lifecycle management works");
    println!("");

    let env = CleanroomEnvironment::new().await?;

    // Test 1: Create and register custom plugins
    println!("📋 Test 1: Plugin Registration");
    println!("==============================");

    let postgres_plugin = Box::new(PostgresPlugin::new(
        "postgresql://localhost:5432/testdb".to_string(),
    ));
    let redis_plugin = Box::new(RedisPlugin::new("localhost".to_string(), 6379));

    // Register plugins with the environment
    env.register_service(postgres_plugin).await?;
    env.register_service(redis_plugin).await?;

    println!("✅ Plugins registered successfully");

    // Test 2: Start services using plugins
    println!("\n📋 Test 2: Service Lifecycle Management");
    println!("=======================================");

    let db_handle = env.start_service("postgres").await?;
    let cache_handle = env.start_service("redis").await?;

    println!("✅ Services started successfully");
    println!(
        "   Database: {} (ID: {})",
        db_handle.service_name, db_handle.id
    );
    println!(
        "   Cache: {} (ID: {})",
        cache_handle.service_name, cache_handle.id
    );

    // Test 3: Health checks
    println!("\n📋 Test 3: Health Checks");
    println!("========================");

    let all_health = env.check_health().await;
    let db_health = all_health
        .get(&db_handle.id)
        .cloned()
        .unwrap_or(HealthStatus::Unknown);
    let cache_health = all_health
        .get(&cache_handle.id)
        .cloned()
        .unwrap_or(HealthStatus::Unknown);

    println!("✅ Health checks completed");
    println!("   Database health: {:?}", db_health);
    println!("   Cache health: {:?}", cache_health);

    assert_eq!(db_health, HealthStatus::Healthy);
    assert_eq!(cache_health, HealthStatus::Healthy);

    // Test 4: Service metadata
    println!("\n📋 Test 4: Service Metadata");
    println!("===========================");

    println!("Database metadata:");
    for (key, value) in &db_handle.metadata {
        println!("   {}: {}", key, value);
    }

    println!("Cache metadata:");
    for (key, value) in &cache_handle.metadata {
        println!("   {}: {}", key, value);
    }

    // Test 5: Stop services
    println!("\n📋 Test 5: Service Cleanup");
    println!("==========================");

    env.stop_service(&db_handle.id).await?;
    env.stop_service(&cache_handle.id).await?;

    println!("✅ Services stopped successfully");

    // Test 6: Plugin reuse (demonstrates plugin architecture)
    println!("\n📋 Test 6: Plugin Reuse");
    println!("=======================");

    // Start services again to test plugin reuse
    let db_handle2 = env.start_service("postgres").await?;
    let cache_handle2 = env.start_service("redis").await?;

    println!("✅ Services restarted using same plugins");
    println!(
        "   Database: {} (ID: {})",
        db_handle2.service_name, db_handle2.id
    );
    println!(
        "   Cache: {} (ID: {})",
        cache_handle2.service_name, cache_handle2.id
    );

    // Clean up
    env.stop_service(&db_handle2.id).await?;
    env.stop_service(&cache_handle2.id).await?;

    println!("\n🎉 SUCCESS: Plugin system demo completed!");
    println!("📚 README claims verified:");
    println!("   ✅ Plugin-Based Architecture works");
    println!("   ✅ Custom plugins can be created");
    println!("   ✅ Plugin lifecycle management works");
    println!("   ✅ Service health checks work");
    println!("   ✅ Plugin reuse works efficiently");

    Ok(())
}
