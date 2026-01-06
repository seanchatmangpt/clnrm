use ggen_control::{
    ControlLoop, Policy, PolicyType, ViolationSeverity, ScheduledAction, ActionState,
};
use std::collections::HashMap;
use uuid::Uuid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("GGen Control Loop & Policy Engine\n");
    println!("==================================\n");

    let mut control_loop = ControlLoop::new();

    println!("1. Creating policies...");

    let max_concurrent_policy = Policy {
        id: Uuid::new_v4().to_string(),
        policy_type: PolicyType::MaxConcurrentActions,
        name: "Max Concurrent Actions".to_string(),
        enabled: true,
        parameters: HashMap::from([
            ("max_concurrent".to_string(), "2".to_string()),
        ]),
        violation_severity: ViolationSeverity::Warning,
    };

    let dependency_policy = Policy {
        id: Uuid::new_v4().to_string(),
        policy_type: PolicyType::DependencyOrdering,
        name: "Dependency Ordering".to_string(),
        enabled: true,
        parameters: HashMap::new(),
        violation_severity: ViolationSeverity::Critical,
    };

    let ha_policy = Policy {
        id: Uuid::new_v4().to_string(),
        policy_type: PolicyType::HighAvailability,
        name: "High Availability".to_string(),
        enabled: true,
        parameters: HashMap::from([
            ("min_replicas".to_string(), "2".to_string()),
        ]),
        violation_severity: ViolationSeverity::Critical,
    };

    control_loop.add_policy(max_concurrent_policy.clone())?;
    control_loop.add_policy(dependency_policy.clone())?;
    control_loop.add_policy(ha_policy)?;

    println!("   ✓ Max Concurrent Actions policy");
    println!("   ✓ Dependency Ordering policy");
    println!("   ✓ High Availability policy\n");

    println!("2. Scheduling actions...");

    let action1 = ScheduledAction {
        id: Uuid::new_v4().to_string(),
        action_type: "Upgrade".to_string(),
        target_service: "api-service".to_string(),
        dependencies: vec![],
        priority: 1,
        state: ActionState::Pending,
        reason: "Version mismatch".to_string(),
        timestamp: 0,
    };

    let action2 = ScheduledAction {
        id: Uuid::new_v4().to_string(),
        action_type: "ScaleUp".to_string(),
        target_service: "cache-service".to_string(),
        dependencies: vec![],
        priority: 2,
        state: ActionState::Pending,
        reason: "Health degradation".to_string(),
        timestamp: 0,
    };

    let mut action3 = ScheduledAction {
        id: Uuid::new_v4().to_string(),
        action_type: "Restart".to_string(),
        target_service: "worker-service".to_string(),
        dependencies: vec![action1.id.clone()],
        priority: 3,
        state: ActionState::Pending,
        reason: "State conflict resolution".to_string(),
        timestamp: 0,
    };

    control_loop.schedule_action(action1)?;
    control_loop.schedule_action(action2)?;
    control_loop.schedule_action(action3.clone())?;

    println!("   ✓ Upgrade api-service (no deps)");
    println!("   ✓ ScaleUp cache-service (no deps)");
    println!("   ✓ Restart worker-service (depends on action1)\n");

    println!("3. Running first control cycle...");

    let cycle1 = control_loop.execute_cycle()?;
    println!("   Cycle ID: {}", cycle1.cycle_id);
    println!("   Actions executed: {}", cycle1.actions_executed);
    println!("   Actions remaining: {}", cycle1.actions_scheduled);
    println!("   Policy violations: {}", cycle1.policy_violations);
    for decision in &cycle1.control_decisions {
        println!("     - {}", decision);
    }
    println!();

    println!("4. Checking violations...");

    let violations = control_loop.get_violations();
    println!("   Total violations: {}", violations.len());
    for violation in violations {
        println!("   - {}: {}", violation.policy_id, violation.message);
        println!("     Severity: {:?}", violation.severity);
    }
    println!();

    println!("5. Examining action queue...");

    let queue = control_loop.get_action_queue();
    println!("   Pending actions: {}", queue.len());
    for action in queue {
        println!("   - {} on {} [{:?}]", action.action_type, action.target_service, action.state);
        if !action.dependencies.is_empty() {
            println!("     Dependencies: {:?}", action.dependencies);
        }
    }
    println!();

    println!("6. Resolving dependency violation...");

    if !violations.is_empty() {
        let violation_id = violations[0].id.clone();
        control_loop.resolve_violation(&violation_id)?;
        println!("   ✓ Resolved violation: {}", violation_id);
    }
    println!();

    println!("7. Running second control cycle...");

    let cycle2 = control_loop.execute_cycle()?;
    println!("   Cycle ID: {}", cycle2.cycle_id);
    println!("   Actions executed: {}", cycle2.actions_executed);
    println!("   Actions remaining: {}", cycle2.actions_scheduled);
    println!("   Policy violations: {}", cycle2.policy_violations);
    for decision in &cycle2.control_decisions {
        println!("     - {}", decision);
    }
    println!();

    println!("8. Completed actions...");

    let completed = control_loop.get_completed_actions();
    println!("   Total: {}", completed.len());
    for action in completed {
        println!("   ✓ {} on {}", action.action_type, action.target_service);
    }
    println!();

    println!("9. Active policies...");

    let policies = control_loop.get_policies();
    println!("   Total: {}", policies.len());
    for policy in policies {
        let status = if policy.enabled { "enabled" } else { "disabled" };
        println!("   - {} [{}]", policy.name, status);
        println!("     Type: {:?}", policy.policy_type);
        println!("     Severity: {:?}", policy.violation_severity);
    }
    println!();

    println!("10. Control cycle history...");

    let history = control_loop.get_cycle_history();
    println!("   Total cycles: {}", history.len());
    for cycle in history {
        println!("   Cycle {}:", cycle.cycle_id);
        println!("     - Executed: {}", cycle.actions_executed);
        println!("     - Scheduled: {}", cycle.actions_scheduled);
        println!("     - Violations: {}", cycle.policy_violations);
    }
    println!();

    println!("==================================");
    println!("Control loop execution complete! ✓");
    Ok(())
}
