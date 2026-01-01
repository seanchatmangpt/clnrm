use ggen_test_runner::{ServiceRegistry, TestExecutor, TestStep, HealthStatus};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("GGen Test Runner\n");
    println!("================\n");

    let mut registry = ServiceRegistry::new();
    registry.load_ggen_services()?;

    println!("1. Loaded services from ggen:");
    for plugin_name in registry.plugins.keys() {
        println!("   - {}", plugin_name);
    }
    println!();

    println!("2. Starting services...");
    let h_sdb = registry.start_service("surrealdb")?;
    let h_pg = registry.start_service("postgres")?;
    println!("   ✓ SurrealDB: {}", h_sdb.id);
    println!("   ✓ PostgreSQL: {}", h_pg.id);
    println!();

    println!("3. Verifying service state:");
    if let Some(service) = registry.get_service_status(&h_sdb.id) {
        println!("   Service: {}", service.service_name);
        println!("   State: {:?}", service.state);
        println!("   Image: {}", service.metadata.get("image").unwrap());
        println!("   Port: {}", service.metadata.get("port").unwrap());
    }
    println!();

    println!("4. Health checks:");
    let sdb_plugin = registry.plugins.get("surrealdb").unwrap();
    let pg_plugin = registry.plugins.get("postgres").unwrap();

    let sdb_health = sdb_plugin.health_check(&h_sdb);
    let pg_health = pg_plugin.health_check(&h_pg);

    println!("   SurrealDB: {:?}", sdb_health);
    println!("   PostgreSQL: {:?}", pg_health);
    assert_eq!(sdb_health, HealthStatus::Healthy);
    assert_eq!(pg_health, HealthStatus::Healthy);
    println!();

    println!("5. Running test scenarios...");
    let mut executor = TestExecutor::new(registry);

    let test_steps = vec![
        TestStep {
            name: "surrealdb-startup".to_string(),
            command: vec!["curl".to_string(), "http://localhost:8000".to_string()],
            expected_output: Some("surrealdb".to_string()),
            timeout_ms: 5000,
            retries: 3,
        },
        TestStep {
            name: "postgres-startup".to_string(),
            command: vec!["psql".to_string(), "-U".to_string(), "postgres".to_string()],
            expected_output: Some("postgres".to_string()),
            timeout_ms: 5000,
            retries: 3,
        },
        TestStep {
            name: "surrealdb-query".to_string(),
            command: vec!["surreal".to_string(), "query".to_string()],
            expected_output: Some("surrealdb".to_string()),
            timeout_ms: 5000,
            retries: 1,
        },
        TestStep {
            name: "postgres-query".to_string(),
            command: vec!["psql".to_string(), "-c".to_string(), "SELECT 1".to_string()],
            expected_output: Some("postgres".to_string()),
            timeout_ms: 5000,
            retries: 1,
        },
    ];

    let start = Instant::now();
    let results = executor.execute_steps(test_steps);
    let duration = start.elapsed();

    println!("   Tests executed: {}", results.len());
    for result in &results {
        let status = if result.passed { "✓" } else { "✗" };
        println!("   {} {} ({} ms)", status, result.name, result.duration_ms);
    }
    println!();

    let (passed, total, total_duration) = executor.summary();
    println!("6. Test summary:");
    println!("   Passed: {}/{}", passed, total);
    println!("   Total duration: {} ms", total_duration);
    println!("   Execution time: {:?}", duration);
    println!();

    println!("7. Running services:");
    let running = executor.list_running_services();
    for (name, handle) in running {
        println!("   - {} ({})", name, handle.id);
    }
    println!();

    println!("8. Cleanup - stopping all services...");
    executor.stop_all()?;
    println!("   ✓ All services stopped");
    println!();

    println!("================");
    println!("Test run complete! ✓");
    Ok(())
}
