use ggen_config::{ConfigManager, Environment, GroupConstraints, ResourceConfig, ServiceConfig, ServiceGroupConfig};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("GGen Configuration Manager\n");
    println!("==========================\n");

    let config_dir = "./ggen-configs";
    let manager = ConfigManager::new(config_dir)?;

    println!("1. Creating service configurations...");

    let api_service = ServiceConfig {
        name: "API Service".to_string(),
        image: "api:v1.2.0".to_string(),
        port: 8080,
        version: "1.2.0".to_string(),
        dependencies: vec!["postgres".to_string(), "redis".to_string()],
        environment: HashMap::from([
            ("LOG_LEVEL".to_string(), "info".to_string()),
            ("CACHE_ENABLED".to_string(), "true".to_string()),
        ]),
        resources: ResourceConfig {
            cpu_cores: 1.0,
            memory_mb: 512,
            disk_gb: 5,
        },
        metadata: HashMap::from([
            ("team".to_string(), "backend".to_string()),
            ("tier".to_string(), "web".to_string()),
        ]),
    };

    let postgres_service = ServiceConfig {
        name: "PostgreSQL".to_string(),
        image: "postgres:15".to_string(),
        port: 5432,
        version: "15.0".to_string(),
        dependencies: vec![],
        environment: HashMap::from([
            ("POSTGRES_DB".to_string(), "clnrm".to_string()),
            ("POSTGRES_USER".to_string(), "admin".to_string()),
        ]),
        resources: ResourceConfig {
            cpu_cores: 2.0,
            memory_mb: 1024,
            disk_gb: 20,
        },
        metadata: HashMap::from([
            ("team".to_string(), "data".to_string()),
            ("tier".to_string(), "data".to_string()),
        ]),
    };

    let redis_service = ServiceConfig {
        name: "Redis".to_string(),
        image: "redis:7-alpine".to_string(),
        port: 6379,
        version: "7.0".to_string(),
        dependencies: vec![],
        environment: HashMap::new(),
        resources: ResourceConfig {
            cpu_cores: 0.5,
            memory_mb: 256,
            disk_gb: 2,
        },
        metadata: HashMap::from([
            ("team".to_string(), "data".to_string()),
            ("tier".to_string(), "cache".to_string()),
        ]),
    };

    println!("   ✓ API Service configured");
    println!("   ✓ PostgreSQL configured");
    println!("   ✓ Redis configured\n");

    println!("2. Creating service group for development...");
    let dev_group = ServiceGroupConfig {
        name: "backend-services".to_string(),
        description: "Backend services for development environment".to_string(),
        environment: "development".to_string(),
        services: HashMap::from([
            ("api".to_string(), api_service.clone()),
            ("postgres".to_string(), postgres_service.clone()),
            ("redis".to_string(), redis_service.clone()),
        ]),
        constraints: GroupConstraints {
            max_parallel_starts: 2,
            startup_timeout_ms: 30000,
            health_check_interval_ms: 5000,
            auto_restart: true,
        },
    };

    println!("   ✓ Development group created");
    println!("     Services: {}", dev_group.services.len());
    println!("     Environment: {}", dev_group.environment);
    println!();

    println!("3. Saving development configuration...");
    manager.save_group_config(&dev_group)?;
    println!("   ✓ Saved: backend-services-development.json");
    println!();

    println!("4. Creating production configuration (with overrides)...");
    let mut prod_api = api_service.clone();
    prod_api.image = "api:v1.2.0-prod".to_string();
    prod_api.resources = ResourceConfig {
        cpu_cores: 4.0,
        memory_mb: 2048,
        disk_gb: 20,
    };
    prod_api.environment.insert("LOG_LEVEL".to_string(), "warn".to_string());

    let mut prod_postgres = postgres_service.clone();
    prod_postgres.resources = ResourceConfig {
        cpu_cores: 8.0,
        memory_mb: 4096,
        disk_gb: 100,
    };

    let prod_group = ServiceGroupConfig {
        name: "backend-services".to_string(),
        description: "Backend services for production environment".to_string(),
        environment: "production".to_string(),
        services: HashMap::from([
            ("api".to_string(), prod_api),
            ("postgres".to_string(), prod_postgres),
            ("redis".to_string(), redis_service),
        ]),
        constraints: GroupConstraints {
            max_parallel_starts: 3,
            startup_timeout_ms: 60000,
            health_check_interval_ms: 10000,
            auto_restart: true,
        },
    };

    manager.save_group_config(&prod_group)?;
    println!("   ✓ Saved: backend-services-production.json");
    println!();

    println!("5. Listing all saved configurations...");
    let configs = manager.list_configs()?;
    for config in &configs {
        println!("   - {}", config);
    }
    println!();

    println!("6. Loading development configuration...");
    let loaded_dev = manager.load_group_config("backend-services", Environment::Development)?;
    println!("   ✓ Loaded: {}", loaded_dev.name);
    println!("     Services: {}", loaded_dev.services.len());
    println!("     Constraints:");
    println!(
        "       Max parallel starts: {}",
        loaded_dev.constraints.max_parallel_starts
    );
    println!(
        "       Startup timeout: {} ms",
        loaded_dev.constraints.startup_timeout_ms
    );
    println!();

    println!("7. Comparing configurations...");
    println!("   Development API Service:");
    if let Some(api) = loaded_dev.services.get("api") {
        println!("     Image: {}", api.image);
        println!("     Port: {}", api.port);
        println!("     Resources: {}CPU, {}MB RAM, {}GB disk",
                 api.resources.cpu_cores, api.resources.memory_mb, api.resources.disk_gb);
    }

    println!("   Production API Service:");
    if let Some(api) = prod_group.services.get("api") {
        println!("     Image: {}", api.image);
        println!("     Port: {}", api.port);
        println!("     Resources: {}CPU, {}MB RAM, {}GB disk",
                 api.resources.cpu_cores, api.resources.memory_mb, api.resources.disk_gb);
    }
    println!();

    println!("8. Merging configurations...");
    let merged = manager.merge_config(&loaded_dev, &prod_group)?;
    println!("   ✓ Merged development and production configs");
    println!("     Result environment: {}", merged.environment);
    println!("     Services: {}", merged.services.len());
    println!();

    println!("9. Service details from configuration...");
    if let Some(postgres) = loaded_dev.services.get("postgres") {
        println!("   PostgreSQL Configuration:");
        println!("     Name: {}", postgres.name);
        println!("     Image: {}", postgres.image);
        println!("     Port: {}", postgres.port);
        println!("     Dependencies: {} (none)", postgres.dependencies.len());
        println!("     Environment variables:");
        for (key, value) in &postgres.environment {
            println!("       {} = {}", key, value);
        }
        println!("     Metadata:");
        for (key, value) in &postgres.metadata {
            println!("       {} = {}", key, value);
        }
    }
    println!();

    println!("10. Configuration validation...");
    match manager.validate_group(&loaded_dev) {
        Ok(_) => println!("   ✓ Development group is valid"),
        Err(e) => println!("   ✗ Validation error: {}", e),
    }
    println!();

    println!("==========================");
    println!("Configuration manager demo complete! ✓");
    Ok(())
}
