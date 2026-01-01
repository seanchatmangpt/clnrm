use ggen_deploy::{
    DeploymentEngine, DeploymentState, ServiceDeploymentSpec,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("GGen Service Deployment Engine\n");
    println!("===============================\n");

    let mut engine = DeploymentEngine::new();

    println!("1. Creating deployment plan...");
    let specs = vec![
        ServiceDeploymentSpec {
            service_id: "api-svc".to_string(),
            service_name: "API Service".to_string(),
            image: "myapp/api:v2.0.0".to_string(),
            version: "2.0.0".to_string(),
            replicas: 3,
            health_check_path: "/health".to_string(),
            startup_timeout_ms: 30000,
        },
        ServiceDeploymentSpec {
            service_id: "worker-svc".to_string(),
            service_name: "Background Worker".to_string(),
            image: "myapp/worker:v2.0.0".to_string(),
            version: "2.0.0".to_string(),
            replicas: 2,
            health_check_path: "/status".to_string(),
            startup_timeout_ms: 45000,
        },
        ServiceDeploymentSpec {
            service_id: "webhook-svc".to_string(),
            service_name: "Webhook Service".to_string(),
            image: "myapp/webhook:v2.0.0".to_string(),
            version: "2.0.0".to_string(),
            replicas: 1,
            health_check_path: "/ready".to_string(),
            startup_timeout_ms: 20000,
        },
    ];

    let deployment_id = engine.create_deployment(
        "production-v2".to_string(),
        "2.0.0".to_string(),
        specs,
    )?;

    println!("   ✓ Deployment created: {}", deployment_id);
    println!("     Services: 3");
    println!("     Total replicas: 6\n");

    println!("2. Starting deployment...");
    engine.start_deployment(&deployment_id)?;
    let dep = engine.get_deployment(&deployment_id)?;
    println!("   ✓ Deployment state: {:?}", dep.state);
    println!();

    println!("3. Executing deployment (pulling images, starting containers)...");
    engine.execute_deployment(&deployment_id)?;
    let dep = engine.get_deployment(&deployment_id)?;
    println!("   ✓ Deployment state: {:?}", dep.state);
    println!();

    println!("4. Deployment progress tracking...");
    let progress = engine.deployment_progress(&deployment_id)?;
    println!("   Services: {}/{}", progress.healthy_services, progress.total_services);
    println!("   Replicas: {}/{}", progress.running_replicas, progress.total_replicas);
    println!("   Progress: {}%", progress.progress_percent);
    println!();

    println!("5. Performing health checks...");
    let healthy = engine.verify_health(&deployment_id)?;
    if healthy {
        println!("   ✓ All services healthy!");
    } else {
        println!("   ⚠ Some services unhealthy");
    }
    let dep = engine.get_deployment(&deployment_id)?;
    println!("   Deployment state: {:?}", dep.state);
    println!();

    println!("6. Detailed deployment status...");
    let dep = engine.get_deployment(&deployment_id)?;
    for (service_id, status) in &dep.deployed_services {
        println!("   {} - {}", service_id, status.service_name);
        println!(
            "     Replicas: {}/{} running, {}/{} healthy",
            status.replicas_running, status.replicas_desired,
            status.healthy_replicas, status.replicas_desired
        );
    }
    println!();

    println!("7. Creating another deployment (canary release)...");
    let canary_specs = vec![
        ServiceDeploymentSpec {
            service_id: "api-svc".to_string(),
            service_name: "API Service (Canary)".to_string(),
            image: "myapp/api:v2.1.0-rc1".to_string(),
            version: "2.1.0-rc1".to_string(),
            replicas: 1,
            health_check_path: "/health".to_string(),
            startup_timeout_ms: 30000,
        },
    ];

    let canary_id = engine.create_deployment(
        "canary-v2.1.0".to_string(),
        "2.1.0-rc1".to_string(),
        canary_specs,
    )?;

    engine.start_deployment(&canary_id)?;
    engine.execute_deployment(&canary_id)?;
    engine.verify_health(&canary_id)?;

    println!("   ✓ Canary deployment created: {}", canary_id);
    let canary = engine.get_deployment(&canary_id)?;
    println!("   State: {:?}", canary.state);
    println!();

    println!("8. Listing all deployments...");
    let deployments = engine.list_deployments();
    println!("   Total: {}", deployments.len());
    for dep in deployments {
        println!("   - {} ({})", dep.id, match dep.state {
            DeploymentState::Active => "✓ Active",
            DeploymentState::Pending => "⏳ Pending",
            DeploymentState::Preparing => "⚙ Preparing",
            DeploymentState::Deploying => "🚀 Deploying",
            DeploymentState::HealthChecking => "💊 Health Checking",
            DeploymentState::Unhealthy => "⚠ Unhealthy",
            DeploymentState::Stopping => "⛔ Stopping",
            DeploymentState::Stopped => "🛑 Stopped",
            DeploymentState::RollingBack => "↩ Rolling Back",
            DeploymentState::Failed => "❌ Failed",
        });
    }
    println!();

    println!("9. Active deployment info...");
    match engine.get_active_deployment() {
        Some(active) => {
            println!("   ID: {}", active.id);
            println!("   State: {:?}", active.state);
            println!("   Services: {}", active.deployed_services.len());
        }
        None => println!("   No active deployment"),
    }
    println!();

    println!("10. Rollback scenario...");
    println!("   Rolling back canary deployment...");
    engine.rollback_deployment(&canary_id)?;
    let canary = engine.get_deployment(&canary_id)?;
    println!("   ✓ State: {:?}", canary.state);
    println!("   Rollback count: {}", canary.rollback_count);
    println!();

    println!("===============================");
    println!("Deployment engine demo complete! ✓");
    Ok(())
}
