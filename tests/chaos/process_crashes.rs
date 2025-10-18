//! Process Crash Simulation Tests
//!
//! Tests system resilience against random process crashes,
//! panic handling, and ungraceful terminations.

use clnrm_core::services::chaos_engine::{ChaosEnginePlugin, ChaosScenario};
use clnrm_core::error::Result;
use std::time::Instant;

/// Test panic recovery
#[tokio::test]
async fn test_panic_recovery() -> Result<()> {
    let engine = ChaosEnginePlugin::new("panic_test");

    // Test panic handling with catch_unwind
    let result = std::panic::catch_unwind(|| {
        // This should panic
        if rand::random::<f64>() < 0.5 {
            panic!("Simulated panic!");
        }
        "completed"
    });

    match result {
        Ok(val) => println!("Operation completed: {}", val),
        Err(_) => println!("Panic caught and recovered"),
    }

    // Verify system is still functional after panic
    let test_scenario = ChaosScenario::RandomFailures {
        duration_secs: 1,
        failure_rate: 0.3,
    };

    engine.run_scenario(&test_scenario).await?;
    println!("System recovered successfully after panic");

    Ok(())
}

/// Test async task cancellation
#[tokio::test]
async fn test_task_cancellation() -> Result<()> {
    let handle = tokio::spawn(async {
        // Long-running task
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        "completed"
    });

    // Simulate crash by aborting task
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    handle.abort();

    match handle.await {
        Ok(_) => println!("Task completed normally"),
        Err(e) if e.is_cancelled() => {
            println!("Task cancelled successfully");
        }
        Err(e) => {
            println!("Task failed with error: {}", e);
        }
    }

    Ok(())
}

/// Test process failure simulation
#[tokio::test]
async fn test_process_failure_simulation() -> Result<()> {
    let engine = ChaosEnginePlugin::new("process_failure")
        .with_failure_rate(0.7);

    let processes = vec![
        "worker_1", "worker_2", "worker_3", "worker_4", "worker_5"
    ];

    let mut crashed = 0;
    let mut running = 0;

    for process in &processes {
        if engine.inject_failure(process).await? {
            crashed += 1;
            println!("Process '{}' crashed - restarting...", process);

            // Simulate restart delay
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        } else {
            running += 1;
            println!("Process '{}' running normally", process);
        }
    }

    println!("Process crash simulation:");
    println!("  Crashed: {}", crashed);
    println!("  Running: {}", running);

    Ok(())
}

/// Test supervisor tree resilience
#[tokio::test]
async fn test_supervisor_tree() -> Result<()> {
    let engine = ChaosEnginePlugin::new("supervisor");

    // Simulate supervisor managing child processes
    let mut active_workers = vec!["worker_1", "worker_2", "worker_3"];
    let max_restarts = 3;
    let mut restart_counts = std::collections::HashMap::new();

    for iteration in 0..10 {
        println!("\nIteration {}: Active workers: {:?}", iteration, active_workers);

        for worker in active_workers.clone() {
            if engine.inject_failure(worker).await? {
                println!("Worker '{}' crashed", worker);

                let restarts = restart_counts.entry(worker).or_insert(0);
                *restarts += 1;

                if *restarts >= max_restarts {
                    println!("Worker '{}' exceeded max restarts, removing", worker);
                    active_workers.retain(|&w| w != worker);
                } else {
                    println!("Restarting worker '{}' (attempt {})", worker, restarts);
                }
            }
        }

        if active_workers.is_empty() {
            println!("All workers failed, system degraded");
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    Ok(())
}

/// Test graceful shutdown vs crash
#[tokio::test]
async fn test_graceful_vs_crash_shutdown() -> Result<()> {
    let engine = ChaosEnginePlugin::new("shutdown");

    // Test graceful shutdown
    let graceful_task = tokio::spawn(async {
        println!("Graceful shutdown initiated");
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        println!("Cleanup completed");
        "graceful_shutdown"
    });

    // Test crash (ungraceful)
    let crash_task = tokio::spawn(async {
        println!("Process running");
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        if rand::random::<f64>() < 0.5 {
            panic!("Simulated crash!");
        }
        "normal_completion"
    });

    // Wait for tasks with different handling
    match graceful_task.await {
        Ok(result) => println!("Graceful task: {}", result),
        Err(e) => println!("Graceful task error: {}", e),
    }

    match crash_task.await {
        Ok(result) => println!("Crash task: {}", result),
        Err(e) => println!("Crash task error (expected): {}", e),
    }

    Ok(())
}

/// Test memory leak leading to crash
#[tokio::test]
async fn test_memory_leak_crash() -> Result<()> {
    let engine = ChaosEnginePlugin::new("memory_leak");

    // Simulate gradual memory leak
    let mut leaked_data: Vec<Vec<u8>> = Vec::new();
    let leak_iterations = 10;

    for i in 0..leak_iterations {
        // Allocate memory that "leaks" (not freed)
        let allocation = vec![0u8; 1024 * 100]; // 100KB per iteration
        leaked_data.push(allocation);

        println!("Iteration {}: Leaked ~{}KB total",
            i, (i + 1) * 100);

        // Check if we should crash due to memory pressure
        if engine.inject_failure(&format!("memory_check_{}", i)).await? {
            println!("Memory pressure critical, simulating OOM");
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    // Cleanup (simulating recovery after crash)
    leaked_data.clear();
    println!("Memory reclaimed after crash recovery");

    Ok(())
}

/// Test deadlock detection and recovery
#[tokio::test]
async fn test_deadlock_detection() -> Result<()> {
    use tokio::sync::Mutex;
    use std::sync::Arc;

    let lock1 = Arc::new(Mutex::new(0));
    let lock2 = Arc::new(Mutex::new(0));

    let timeout = tokio::time::Duration::from_secs(1);

    // Try to acquire locks with timeout to avoid actual deadlock
    let task1 = {
        let lock1 = Arc::clone(&lock1);
        let lock2 = Arc::clone(&lock2);
        tokio::spawn(async move {
            let _l1 = lock1.lock().await;
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            // Try to acquire second lock with timeout
            match tokio::time::timeout(timeout, lock2.lock()).await {
                Ok(_) => println!("Task 1: Acquired both locks"),
                Err(_) => println!("Task 1: Deadlock detected, backing off"),
            }
        })
    };

    let task2 = {
        let lock1 = Arc::clone(&lock1);
        let lock2 = Arc::clone(&lock2);
        tokio::spawn(async move {
            let _l2 = lock2.lock().await;
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            match tokio::time::timeout(timeout, lock1.lock()).await {
                Ok(_) => println!("Task 2: Acquired both locks"),
                Err(_) => println!("Task 2: Deadlock detected, backing off"),
            }
        })
    };

    tokio::try_join!(task1, task2)?;

    println!("Deadlock detection test completed");

    Ok(())
}

/// Test zombie process cleanup
#[tokio::test]
async fn test_zombie_process_cleanup() -> Result<()> {
    let engine = ChaosEnginePlugin::new("zombie");

    let mut zombie_processes = Vec::new();

    // Create processes that might become zombies
    for i in 0..5 {
        let handle = tokio::spawn(async move {
            if rand::random::<f64>() < 0.3 {
                // Simulate zombie state (task completed but not awaited)
                println!("Process {} entered zombie state", i);
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }
            i
        });

        zombie_processes.push(handle);
    }

    // Cleanup zombie processes
    let mut cleaned_up = 0;
    for (i, handle) in zombie_processes.into_iter().enumerate() {
        match handle.await {
            Ok(result) => {
                println!("Process {} cleaned up (result: {})", i, result);
                cleaned_up += 1;
            }
            Err(e) => println!("Process {} cleanup error: {}", i, e),
        }
    }

    println!("Zombie cleanup: {} processes cleaned", cleaned_up);

    Ok(())
}

/// Test cascading process failures
#[tokio::test]
async fn test_cascading_process_failures() -> Result<()> {
    let engine = ChaosEnginePlugin::new("cascading_crash");

    let scenario = ChaosScenario::CascadingFailures {
        trigger_service: "primary_process".to_string(),
        propagation_delay_ms: 100,
    };

    let start = Instant::now();
    engine.run_scenario(&scenario).await?;

    let metrics = engine.get_metrics().await;

    println!("Cascading process failures:");
    println!("  Initial failure: primary_process");
    println!("  Total processes affected: {}", metrics.affected_services.len());
    println!("  Cascade duration: {:?}", start.elapsed());

    assert!(metrics.affected_services.len() > 1,
        "Cascading failure should affect multiple processes");

    Ok(())
}

/// Test process restart with state recovery
#[tokio::test]
async fn test_process_restart_with_state() -> Result<()> {
    use std::sync::Arc;
    use tokio::sync::RwLock;

    // Shared state that survives crashes
    let state = Arc::new(RwLock::new(vec![1, 2, 3, 4, 5]));

    let engine = ChaosEnginePlugin::new("stateful_restart");

    for iteration in 0..5 {
        let state = Arc::clone(&state);

        let task = tokio::spawn(async move {
            let mut data = state.write().await;
            data.push(iteration + 6);

            if rand::random::<f64>() < 0.4 {
                return Err("Process crashed");
            }

            Ok(data.len())
        });

        match task.await {
            Ok(Ok(len)) => {
                println!("Iteration {}: Process completed (state size: {})", iteration, len);
            }
            Ok(Err(e)) => {
                println!("Iteration {}: Process crashed - {}", iteration, e);
                println!("Restarting with preserved state...");
            }
            Err(e) => {
                println!("Iteration {}: Task join error - {}", iteration, e);
            }
        }
    }

    let final_state = state.read().await;
    println!("Final state after crashes: {} items", final_state.len());

    Ok(())
}
