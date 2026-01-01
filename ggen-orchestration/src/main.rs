use ggen_orchestration::{
    GroupConstraints, ServiceDefinition, ServiceGroup, ServiceOrchestrator,
};
use std::collections::HashMap;
use uuid::Uuid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("GGen Service Orchestration Engine\n");
    println!("==================================\n");

    let mut orchestrator = ServiceOrchestrator::new();

    println!("1. Registering service groups...");
    let web_tier_group = create_web_tier_group();
    let group_id_1 = web_tier_group.id.clone();
    orchestrator.register_group(web_tier_group)?;
    println!("   ✓ Web Tier Group registered");

    let data_tier_group = create_data_tier_group();
    let group_id_2 = data_tier_group.id.clone();
    orchestrator.register_group(data_tier_group)?;
    println!("   ✓ Data Tier Group registered\n");

    println!("2. Planning execution of Web Tier...");
    let plan_id_1 = orchestrator.plan_execution(&group_id_1)?;
    let plan_1 = orchestrator.get_plan(&plan_id_1)?;
    println!("   Plan ID: {}", plan_1.id);
    println!("   Total services: {}", plan_1.total_services);
    println!("   Execution stages: {}", plan_1.stages.len());
    for (stage_num, stage) in plan_1.stages.iter().enumerate() {
        println!(
            "     Stage {}: {}",
            stage_num + 1,
            stage.join(", ")
        );
    }
    println!();

    println!("3. Planning execution of Data Tier...");
    let plan_id_2 = orchestrator.plan_execution(&group_id_2)?;
    let plan_2 = orchestrator.get_plan(&plan_id_2)?;
    println!("   Plan ID: {}", plan_2.id);
    println!("   Total services: {}", plan_2.total_services);
    println!("   Execution stages: {}", plan_2.stages.len());
    for (stage_num, stage) in plan_2.stages.iter().enumerate() {
        println!(
            "     Stage {}: {}",
            stage_num + 1,
            stage.join(", ")
        );
    }
    println!();

    println!("4. Executing Web Tier group...");
    let web_instances = orchestrator.execute_group(&group_id_1, &plan_id_1)?;
    println!("   Started {} services", web_instances.len());
    for instance in &web_instances {
        println!(
            "     ✓ {} ({})",
            instance.definition.name, instance.instance_id
        );
    }
    println!();

    println!("5. Executing Data Tier group...");
    let data_instances = orchestrator.execute_group(&group_id_2, &plan_id_2)?;
    println!("   Started {} services", data_instances.len());
    for instance in &data_instances {
        println!(
            "     ✓ {} ({})",
            instance.definition.name, instance.instance_id
        );
    }
    println!();

    println!("6. Listing all running instances...");
    let all_instances = orchestrator.list_instances();
    println!("   Total running: {}", all_instances.len());
    for instance in all_instances {
        println!(
            "     - {} [{}]: {} (port {})",
            instance.definition.name,
            instance.instance_id,
            instance.definition.image,
            instance.definition.port
        );
    }
    println!();

    println!("7. Querying instance details...");
    if let Some(first) = web_instances.first() {
        let instance = orchestrator.get_instance(&first.instance_id)?;
        println!("   Service: {}", instance.definition.name);
        println!("   State: {:?}", instance.state);
        println!("   Image: {}", instance.definition.image);
        println!("   Port: {}", instance.definition.port);
        println!("   Dependencies: {:?}", instance.definition.dependencies);
        println!(
            "   Environment vars: {}",
            instance.definition.environment.len()
        );
    }
    println!();

    println!("8. Stopping Web Tier group...");
    let stopped = orchestrator.stop_group(&group_id_1)?;
    println!("   Stopped {} instances", stopped.len());
    for id in &stopped {
        if let Ok(instance) = orchestrator.get_instance(id) {
            println!("     ✓ {} is now {:?}", instance.definition.name, instance.state);
        }
    }
    println!();

    println!("9. Listing remaining running instances...");
    let remaining = orchestrator.list_instances();
    let still_running: Vec<_> = remaining
        .iter()
        .filter(|i| matches!(i.state, ggen_orchestration::ServiceExecutionState::Running))
        .collect();
    println!("   Total running: {}", still_running.len());
    for instance in still_running {
        println!("     - {} [{}]", instance.definition.name, instance.instance_id);
    }
    println!();

    println!("10. Group statistics...");
    let groups = orchestrator.list_groups();
    println!("   Total groups: {}", groups.len());
    for group in groups {
        println!("     - {}: {} services", group.name, group.services.len());
    }
    println!();

    println!("==================================");
    println!("Orchestration demo complete! ✓");
    Ok(())
}

fn create_web_tier_group() -> ServiceGroup {
    let mut services = HashMap::new();

    services.insert(
        "nginx".to_string(),
        ServiceDefinition {
            id: "nginx".to_string(),
            name: "NGINX Load Balancer".to_string(),
            image: "nginx:alpine".to_string(),
            port: 80,
            dependencies: vec![],
            environment: HashMap::from([
                ("WORKER_PROCESSES".to_string(), "4".to_string()),
                ("WORKER_CONNECTIONS".to_string(), "1024".to_string()),
            ]),
            metadata: HashMap::from([
                ("tier".to_string(), "web".to_string()),
                ("role".to_string(), "gateway".to_string()),
            ]),
        },
    );

    services.insert(
        "api-primary".to_string(),
        ServiceDefinition {
            id: "api-primary".to_string(),
            name: "API Server (Primary)".to_string(),
            image: "api-service:latest".to_string(),
            port: 8080,
            dependencies: vec!["nginx".to_string()],
            environment: HashMap::from([
                ("SERVICE_NAME".to_string(), "api-primary".to_string()),
                ("LOG_LEVEL".to_string(), "info".to_string()),
            ]),
            metadata: HashMap::from([
                ("tier".to_string(), "web".to_string()),
                ("replica".to_string(), "1".to_string()),
            ]),
        },
    );

    services.insert(
        "api-replica".to_string(),
        ServiceDefinition {
            id: "api-replica".to_string(),
            name: "API Server (Replica)".to_string(),
            image: "api-service:latest".to_string(),
            port: 8081,
            dependencies: vec!["nginx".to_string()],
            environment: HashMap::from([
                ("SERVICE_NAME".to_string(), "api-replica".to_string()),
                ("LOG_LEVEL".to_string(), "info".to_string()),
            ]),
            metadata: HashMap::from([
                ("tier".to_string(), "web".to_string()),
                ("replica".to_string(), "2".to_string()),
            ]),
        },
    );

    ServiceGroup {
        id: Uuid::new_v4().to_string(),
        name: "web-tier".to_string(),
        description: "Web tier with load balancer and API servers".to_string(),
        services,
        constraints: GroupConstraints {
            max_parallel_starts: 3,
            startup_timeout_ms: 30000,
            health_check_interval_ms: 5000,
            auto_restart: true,
        },
    }
}

fn create_data_tier_group() -> ServiceGroup {
    let mut services = HashMap::new();

    services.insert(
        "postgres".to_string(),
        ServiceDefinition {
            id: "postgres".to_string(),
            name: "PostgreSQL Master".to_string(),
            image: "postgres:15".to_string(),
            port: 5432,
            dependencies: vec![],
            environment: HashMap::from([
                ("POSTGRES_DB".to_string(), "clnrm".to_string()),
                ("POSTGRES_USER".to_string(), "admin".to_string()),
            ]),
            metadata: HashMap::from([
                ("tier".to_string(), "data".to_string()),
                ("type".to_string(), "primary".to_string()),
            ]),
        },
    );

    services.insert(
        "redis".to_string(),
        ServiceDefinition {
            id: "redis".to_string(),
            name: "Redis Cache".to_string(),
            image: "redis:7-alpine".to_string(),
            port: 6379,
            dependencies: vec![],
            environment: HashMap::new(),
            metadata: HashMap::from([
                ("tier".to_string(), "data".to_string()),
                ("type".to_string(), "cache".to_string()),
            ]),
        },
    );

    services.insert(
        "surrealdb".to_string(),
        ServiceDefinition {
            id: "surrealdb".to_string(),
            name: "SurrealDB Graph".to_string(),
            image: "surrealdb:latest".to_string(),
            port: 8000,
            dependencies: vec!["postgres".to_string()],
            environment: HashMap::from([
                ("SURREALDB_USER".to_string(), "root".to_string()),
                ("SURREALDB_PASS".to_string(), "root".to_string()),
            ]),
            metadata: HashMap::from([
                ("tier".to_string(), "data".to_string()),
                ("type".to_string(), "graph".to_string()),
            ]),
        },
    );

    ServiceGroup {
        id: Uuid::new_v4().to_string(),
        name: "data-tier".to_string(),
        description: "Data tier with database and cache systems".to_string(),
        services,
        constraints: GroupConstraints {
            max_parallel_starts: 2,
            startup_timeout_ms: 60000,
            health_check_interval_ms: 10000,
            auto_restart: true,
        },
    }
}
