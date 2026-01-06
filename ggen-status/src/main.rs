use ggen_status::{SystemStatus, ComponentStatus, ComponentHealth, AlertSeverity};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("GGen System Status Monitor\n");
    println!("==========================\n");

    let mut system = SystemStatus::new();

    println!("1. Registering integration dependencies...");

    system.register_integration(
        "orchestration".to_string(),
        vec!["service-mesh".to_string(), "config".to_string()],
    )?;

    system.register_integration(
        "deployment".to_string(),
        vec!["orchestration".to_string(), "control".to_string()],
    )?;

    system.register_integration(
        "control".to_string(),
        vec!["reconciliation".to_string()],
    )?;

    println!("   ✓ orchestration → [service-mesh, config]");
    println!("   ✓ deployment → [orchestration, control]");
    println!("   ✓ control → [reconciliation]\n");

    println!("2. Collecting component statuses...");

    let mut components = HashMap::new();

    components.insert(
        "codegen".to_string(),
        ComponentStatus {
            component_id: "codegen".to_string(),
            component_name: "Code Generator".to_string(),
            health: ComponentHealth::Healthy,
            ready: true,
            version: "0.1.0".to_string(),
            uptime_ms: 3600000,
            error_count: 0,
            last_update: 0,
        },
    );

    components.insert(
        "service-mesh".to_string(),
        ComponentStatus {
            component_id: "mesh".to_string(),
            component_name: "Service Mesh".to_string(),
            health: ComponentHealth::Healthy,
            ready: true,
            version: "0.1.0".to_string(),
            uptime_ms: 3600000,
            error_count: 2,
            last_update: 0,
        },
    );

    components.insert(
        "config".to_string(),
        ComponentStatus {
            component_id: "config".to_string(),
            component_name: "Configuration".to_string(),
            health: ComponentHealth::Degraded,
            ready: true,
            version: "0.1.0".to_string(),
            uptime_ms: 3600000,
            error_count: 5,
            last_update: 0,
        },
    );

    components.insert(
        "orchestration".to_string(),
        ComponentStatus {
            component_id: "orch".to_string(),
            component_name: "Orchestration".to_string(),
            health: ComponentHealth::Healthy,
            ready: true,
            version: "0.1.0".to_string(),
            uptime_ms: 3600000,
            error_count: 0,
            last_update: 0,
        },
    );

    components.insert(
        "deployment".to_string(),
        ComponentStatus {
            component_id: "deploy".to_string(),
            component_name: "Deployment".to_string(),
            health: ComponentHealth::Healthy,
            ready: true,
            version: "0.1.0".to_string(),
            uptime_ms: 3600000,
            error_count: 1,
            last_update: 0,
        },
    );

    components.insert(
        "observability".to_string(),
        ComponentStatus {
            component_id: "obs".to_string(),
            component_name: "Observability".to_string(),
            health: ComponentHealth::Healthy,
            ready: true,
            version: "0.1.0".to_string(),
            uptime_ms: 3600000,
            error_count: 0,
            last_update: 0,
        },
    );

    components.insert(
        "reconciliation".to_string(),
        ComponentStatus {
            component_id: "reconcile".to_string(),
            component_name: "Reconciliation".to_string(),
            health: ComponentHealth::Healthy,
            ready: true,
            version: "0.1.0".to_string(),
            uptime_ms: 3600000,
            error_count: 0,
            last_update: 0,
        },
    );

    components.insert(
        "control".to_string(),
        ComponentStatus {
            component_id: "control".to_string(),
            component_name: "Control Loop".to_string(),
            health: ComponentHealth::Healthy,
            ready: true,
            version: "0.1.0".to_string(),
            uptime_ms: 3600000,
            error_count: 0,
            last_update: 0,
        },
    );

    println!("   ✓ {} components collected\n", components.len());

    println!("3. Running status collection cycle...");

    let snapshot = system.collect_status(components)?;

    println!("   Snapshot ID: {}", snapshot.snapshot_id);
    println!("   Total components: {}", snapshot.components.len());
    println!();

    println!("4. Component health report...");

    for component in snapshot.components.values() {
        let health_icon = match component.health {
            ComponentHealth::Healthy => "✓",
            ComponentHealth::Degraded => "⚠",
            ComponentHealth::Unhealthy => "✗",
            ComponentHealth::Unknown => "?",
        };

        println!("   {} {} - {:?} (ready: {})", health_icon, component.component_name, component.health, component.ready);
        if component.error_count > 0 {
            println!("      Errors: {}", component.error_count);
        }
    }
    println!();

    println!("5. System coherence analysis...");

    let coherence = &snapshot.coherence;
    println!("   Assertions: {}/{} passed", coherence.passed_assertions, coherence.total_assertions);
    println!("   Coherent: {}", if coherence.coherent { "Yes ✓" } else { "No ✗" });

    if !coherence.failed_assertions.is_empty() {
        println!("   Failed assertions:");
        for assertion in &coherence.failed_assertions {
            println!("     - {}", assertion);
        }
    }
    println!();

    println!("6. Integration status...");

    for (source, status) in &snapshot.integration_status {
        let status_icon = match status {
            ggen_status::IntegrationStatus::Connected => "✓",
            ggen_status::IntegrationStatus::PartiallyConnected => "⚠",
            ggen_status::IntegrationStatus::Disconnected => "✗",
            ggen_status::IntegrationStatus::Unknown => "?",
        };

        println!("   {} {} → {:?}", status_icon, source, status);
    }
    println!();

    println!("7. Active alerts...");

    if snapshot.alerts.is_empty() {
        println!("   No alerts");
    } else {
        for alert in &snapshot.alerts {
            let severity_icon = match alert.severity {
                AlertSeverity::Info => "ℹ",
                AlertSeverity::Warning => "⚠",
                AlertSeverity::Critical => "🔴",
            };

            println!("   {} [{}] {}: {}", severity_icon, alert.component, alert.alert_id, alert.message);
        }
    }
    println!();

    println!("8. Verifying system coherence...");

    match system.verify_coherence() {
        Ok(_) => println!("   ✓ System is coherent"),
        Err(e) => println!("   ✗ Coherence check failed: {}", e),
    }
    println!();

    println!("9. Broken integrations...");

    let broken = system.get_broken_integrations();
    if broken.is_empty() {
        println!("   None");
    } else {
        for (source, status) in broken {
            println!("   - {} is {:?}", source, status);
        }
    }
    println!();

    println!("10. Unhealthy components...");

    let unhealthy = system.get_unhealthy_components();
    if unhealthy.is_empty() {
        println!("   None");
    } else {
        for component in unhealthy {
            println!("   - {} (health: {:?})", component.component_name, component.health);
        }
    }
    println!();

    println!("==========================");
    println!("Status report complete! ✓");
    Ok(())
}
