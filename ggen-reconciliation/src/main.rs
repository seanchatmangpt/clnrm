use ggen_reconciliation::{
    ReconciliationEngine, ServiceState, DesiredState, ActualState, DriftType,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("GGen Reconciliation Engine\n");
    println!("==========================\n");

    let mut engine = ReconciliationEngine::new();

    println!("1. Setting desired state for services...");
    
    engine.set_desired_state(
        "api-service".to_string(),
        ServiceState {
            service_id: "api-service".to_string(),
            service_name: "API Service".to_string(),
            desired_state: DesiredState::Running,
            actual_state: ActualState::Unknown,
            desired_version: "2.0.0".to_string(),
            actual_version: "1.0.0".to_string(),
            desired_replicas: 5,
            actual_replicas: 3,
            healthy_replicas: 3,
            drift_type: DriftType::VersionMismatch,
            last_observed: 0,
        },
    )?;

    engine.set_desired_state(
        "db-service".to_string(),
        ServiceState {
            service_id: "db-service".to_string(),
            service_name: "Database Service".to_string(),
            desired_state: DesiredState::Running,
            actual_state: ActualState::Unknown,
            desired_version: "15.0".to_string(),
            actual_version: "15.0".to_string(),
            desired_replicas: 1,
            actual_replicas: 1,
            healthy_replicas: 1,
            drift_type: DriftType::None,
            last_observed: 0,
        },
    )?;

    engine.set_desired_state(
        "cache-service".to_string(),
        ServiceState {
            service_id: "cache-service".to_string(),
            service_name: "Cache Service".to_string(),
            desired_state: DesiredState::Running,
            actual_state: ActualState::Unknown,
            desired_version: "7.0".to_string(),
            actual_version: "7.0".to_string(),
            desired_replicas: 3,
            actual_replicas: 3,
            healthy_replicas: 1,
            drift_type: DriftType::HealthMismatch,
            last_observed: 0,
        },
    )?;

    println!("   ✓ API Service (version drift + replicas)");
    println!("   ✓ Database Service (in sync)");
    println!("   ✓ Cache Service (health degraded)\n");

    println!("2. Observing actual states...");

    engine.observe_actual_state(
        "api-service".to_string(),
        ServiceState {
            service_id: "api-service".to_string(),
            service_name: "API Service".to_string(),
            desired_state: DesiredState::Running,
            actual_state: ActualState::Running,
            desired_version: "2.0.0".to_string(),
            actual_version: "1.0.0".to_string(),
            desired_replicas: 5,
            actual_replicas: 3,
            healthy_replicas: 3,
            drift_type: DriftType::None,
            last_observed: 0,
        },
    )?;

    engine.observe_actual_state(
        "db-service".to_string(),
        ServiceState {
            service_id: "db-service".to_string(),
            service_name: "Database Service".to_string(),
            desired_state: DesiredState::Running,
            actual_state: ActualState::Running,
            desired_version: "15.0".to_string(),
            actual_version: "15.0".to_string(),
            desired_replicas: 1,
            actual_replicas: 1,
            healthy_replicas: 1,
            drift_type: DriftType::None,
            last_observed: 0,
        },
    )?;

    engine.observe_actual_state(
        "cache-service".to_string(),
        ServiceState {
            service_id: "cache-service".to_string(),
            service_name: "Cache Service".to_string(),
            desired_state: DesiredState::Running,
            actual_state: ActualState::Degraded,
            desired_version: "7.0".to_string(),
            actual_version: "7.0".to_string(),
            desired_replicas: 3,
            actual_replicas: 3,
            healthy_replicas: 1,
            drift_type: DriftType::None,
            last_observed: 0,
        },
    )?;

    println!("   ✓ Observed actual states from running system\n");

    println!("3. Detecting drift...");

    let api_drift = engine.detect_drift("api-service")?;
    let db_drift = engine.detect_drift("db-service")?;
    let cache_drift = engine.detect_drift("cache-service")?;

    println!("   API Service drift: {:?}", api_drift);
    println!("   DB Service drift: {:?}", db_drift);
    println!("   Cache Service drift: {:?}\n", cache_drift);

    println!("4. Running reconciliation cycle...");

    let cycle = engine.reconcile()?;

    println!("   Cycle ID: {}", cycle.cycle_id);
    println!("   Services observed: {}", cycle.services_observed);
    println!("   Services with drift: {}", cycle.services_with_drift);
    println!("   Actions generated: {}", cycle.actions_generated);

    if !cycle.conflicts_detected.is_empty() {
        println!("   Conflicts detected:");
        for conflict in &cycle.conflicts_detected {
            println!("     - {}", conflict);
        }
    }
    println!();

    println!("5. Examining pending actions...");

    let pending = engine.get_pending_actions();
    println!("   Total pending: {}", pending.len());

    for action in pending {
        println!("   - ID: {}", action.id);
        println!("     Service: {}", action.service_id);
        println!("     Action: {:?}", action.action_type);
        println!("     Reason: {}", action.reason);
    }
    println!();

    println!("6. Executing reconciliation actions...");

    for action in engine.get_pending_actions().to_vec() {
        println!("   Executing action {:?} for {}", action.action_type, action.service_id);
        engine.execute_action(&action.id)?;
    }

    println!("   ✓ All actions executed\n");

    println!("7. Verifying action completion...");

    println!("   Pending actions: {}", engine.get_pending_actions().len());
    println!("   Completed actions: {}", engine.get_completed_actions().len());

    for action in engine.get_completed_actions() {
        println!("     ✓ {} - {:?}", action.service_id, action.action_type);
    }
    println!();

    println!("8. Drift summary...");

    let summary = engine.get_drift_summary()?;
    println!("   Total services: {}", summary.total_services);
    println!("   Services with drift: {}", summary.services_with_drift);
    println!("   Version mismatches: {}", summary.version_mismatches);
    println!("   Replica mismatches: {}", summary.replica_mismatches);
    println!("   Health mismatches: {}", summary.health_mismatches);
    println!("   State conflicts: {}", summary.state_conflicts);
    println!();

    println!("9. Cycle history...");

    let history = engine.get_cycle_history();
    println!("   Total cycles: {}", history.len());

    for cycle in history {
        println!("   Cycle {}:", cycle.cycle_id);
        println!("     - Observed: {}", cycle.services_observed);
        println!("     - Drift: {}", cycle.services_with_drift);
        println!("     - Actions: {}", cycle.actions_generated);
    }
    println!();

    println!("10. Conflict log...");

    let conflicts = engine.get_conflict_log();
    if conflicts.is_empty() {
        println!("   No conflicts detected\n");
    } else {
        for (_time, conflict) in conflicts {
            println!("   - {}", conflict);
        }
        println!();
    }

    println!("==========================");
    println!("Reconciliation complete! ✓");
    Ok(())
}
