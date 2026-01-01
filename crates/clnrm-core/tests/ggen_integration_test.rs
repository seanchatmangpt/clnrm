use clnrm_core::{GenGenServiceLoader, GenGenConfigBuilder, ServiceRegistry};

#[test]
fn test_ggen_service_loader() {
    let services = GenGenServiceLoader::load_services().expect("Failed to load services");

    assert!(!services.is_empty());

    let service_names: Vec<&str> = services.iter().map(|s| s.name()).collect();
    assert!(service_names.contains(&"surrealdb"));
    assert!(service_names.contains(&"ollama"));
    assert!(service_names.contains(&"vllm"));
    assert!(service_names.contains(&"tgi"));
    assert!(service_names.contains(&"postgres"));
    assert!(service_names.contains(&"otel-collector"));
}

#[test]
fn test_ggen_surrealdb_service() {
    let services = GenGenServiceLoader::load_services().expect("Failed to load services");
    let surrealdb = services.iter().find(|s| s.name() == "surrealdb").unwrap();

    let handle = surrealdb.start().expect("Failed to start service");
    assert_eq!(handle.service_name, "surrealdb");
    assert!(handle.metadata.contains_key("image"));
    assert!(handle.metadata.contains_key("port"));

    surrealdb.stop(handle).expect("Failed to stop service");
}

#[test]
fn test_ggen_service_registry() {
    let registry = ServiceRegistry::new();
    let registry = registry.with_ggen_plugins().expect("Failed to load ggen plugins");

    assert!(!registry.plugins.is_empty());
}

#[test]
fn test_ggen_simple_echo_config() {
    let config = GenGenConfigBuilder::build_simple_echo_test()
        .expect("Failed to build simple echo test");

    assert_eq!(config.scenarios.len(), 1);
    let scenario = &config.scenarios[0];
    assert_eq!(scenario.name, "simple-echo");
    assert_eq!(scenario.steps.len(), 1);

    let step = &scenario.steps[0];
    assert_eq!(step.name, "echo-hello");
    assert_eq!(step.command[0], "echo");
    assert_eq!(step.expected_output, Some("Hello from cleanroom".to_string()));
}

#[test]
fn test_ggen_comprehensive_test_config() {
    let config = GenGenConfigBuilder::build_comprehensive_test()
        .expect("Failed to build comprehensive test");

    assert_eq!(config.scenarios.len(), 1);
    let scenario = &config.scenarios[0];
    assert_eq!(scenario.name, "comprehensive-integration-test");
    assert_eq!(scenario.steps.len(), 5);

    assert_eq!(scenario.steps[0].name, "check-db-health");
    assert_eq!(scenario.steps[1].name, "start-api");
    assert_eq!(scenario.steps[2].name, "api-health-check");
    assert_eq!(scenario.steps[3].name, "test-query");
    assert_eq!(scenario.steps[4].name, "cleanup");
}

#[test]
fn test_ggen_database_test_config() {
    let config = GenGenConfigBuilder::build_database_test()
        .expect("Failed to build database test");

    assert_eq!(config.scenarios.len(), 1);
    let scenario = &config.scenarios[0];
    assert_eq!(scenario.name, "database-integration");
    assert_eq!(scenario.steps.len(), 3);
    assert_eq!(scenario.service, Some("postgres".to_string()));
}

#[test]
fn test_ggen_service_health_check() {
    let services = GenGenServiceLoader::load_services().expect("Failed to load services");

    for service in services {
        let handle = service.start().expect("Failed to start service");
        let health = service.health_check(&handle);

        assert_eq!(health, clnrm_core::HealthStatus::Healthy);

        service.stop(handle).expect("Failed to stop service");
    }
}

#[test]
fn test_multiple_services_concurrent() {
    let services = GenGenServiceLoader::load_services().expect("Failed to load services");

    let mut handles = Vec::new();

    for service in services.iter() {
        let handle = service.start().expect("Failed to start service");
        handles.push((service.name().to_string(), handle));
    }

    assert_eq!(handles.len(), 6);

    for (name, handle) in handles {
        let service = services.iter().find(|s| s.name() == name).unwrap();
        service.stop(handle).expect("Failed to stop service");
    }
}

#[test]
fn test_ggen_config_with_registry() {
    let config = GenGenConfigBuilder::build_simple_echo_test()
        .expect("Failed to build config");

    let registry = ServiceRegistry::new();
    let registry = registry.with_ggen_plugins().expect("Failed to load plugins");

    assert!(!config.scenarios.is_empty());
    assert!(!registry.plugins.is_empty());
}
