use ggen_integration_example::{ServiceRegistry, HealthStatus};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("GGen Integration Example\n");
    println!("======================\n");

    let mut registry = ServiceRegistry::new();

    println!("1. Loading services from ggen instances...");
    registry.load_ggen_services()?;
    println!("   ✓ Loaded successfully\n");

    println!("2. Available services:");
    for service_name in registry.list_services() {
        println!("   - {}", service_name);
    }
    println!();

    println!("3. Starting SurrealDB service...");
    let handle = registry.start_service("surrealdb")?;
    println!("   ✓ Started with ID: {}", handle.id);
    println!("   Metadata:");
    for (k, v) in &handle.metadata {
        println!("     {}: {}", k, v);
    }
    println!();

    println!("4. Checking health...");
    let plugin = registry.plugins.get("surrealdb").unwrap();
    let health = plugin.health_check(&handle);
    println!("   Health status: {:?}", health);
    assert_eq!(health, HealthStatus::Healthy);
    println!("   ✓ Service is healthy\n");

    println!("5. Stopping SurrealDB service...");
    registry.stop_service(&handle.id)?;
    println!("   ✓ Stopped successfully\n");

    println!("6. Testing multiple services concurrently...");
    let services_to_test = vec!["postgres", "ollama"];

    let mut handles = Vec::new();
    for service_name in &services_to_test {
        match registry.start_service(service_name) {
            Ok(h) => {
                println!("   ✓ Started: {}", service_name);
                handles.push(h);
            }
            Err(e) => println!("   ✗ Failed to start {}: {}", service_name, e),
        }
    }
    println!();

    println!("7. Checking health of all running services...");
    for handle in &handles {
        let plugin = registry.plugins.get(&handle.service_name).unwrap();
        let health = plugin.health_check(handle);
        println!("   - {}: {:?}", handle.service_name, health);
    }
    println!();

    println!("8. Stopping all services...");
    for handle in handles {
        registry.stop_service(&handle.id)?;
        println!("   ✓ Stopped: {}", handle.service_name);
    }
    println!();

    println!("9. Full service lifecycle test...");
    for service_name in ["surrealdb", "postgres", "ollama"] {
        print!("   Testing {} ... ", service_name);

        let h = registry.start_service(service_name)?;
        let plugin = registry.plugins.get(service_name).unwrap();

        assert_eq!(plugin.health_check(&h), HealthStatus::Healthy);

        registry.stop_service(&h.id)?;
        println!("✓");
    }
    println!();

    println!("======================");
    println!("All tests passed! ✓\n");

    Ok(())
}
