//! Race Condition Trigger Tests
//!
//! Tests to expose and validate handling of race conditions,
//! data races, concurrent access violations, and synchronization issues.

use clnrm_core::error::Result;
use std::sync::{Arc, atomic::{AtomicUsize, AtomicBool, Ordering}};
use tokio::sync::{Mutex, RwLock, Semaphore};
use std::time::Duration;

/// Test basic race condition on shared counter
#[tokio::test]
async fn test_shared_counter_race() -> Result<()> {
    let counter = Arc::new(AtomicUsize::new(0));
    let mut tasks = vec![];

    // Spawn many concurrent tasks incrementing counter
    for _ in 0..100 {
        let counter = Arc::clone(&counter);
        let task = tokio::spawn(async move {
            for _ in 0..100 {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        });
        tasks.push(task);
    }

    futures_util::future::join_all(tasks).await;

    let final_count = counter.load(Ordering::SeqCst);
    let expected = 100 * 100;

    println!("Shared counter race test:");
    println!("  Expected: {}", expected);
    println!("  Actual: {}", final_count);
    println!("  Match: {}", final_count == expected);

    assert_eq!(final_count, expected,
        "AtomicUsize should prevent race conditions");

    Ok(())
}

/// Test mutex contention
#[tokio::test]
async fn test_mutex_contention() -> Result<()> {
    let data = Arc::new(Mutex::new(0));
    let mut tasks = vec![];

    for _ in 0..50 {
        let data = Arc::clone(&data);
        let task = tokio::spawn(async move {
            for _ in 0..20 {
                let mut guard = data.lock().await;
                *guard += 1;
                tokio::time::sleep(Duration::from_micros(10)).await;
            }
        });
        tasks.push(task);
    }

    futures_util::future::join_all(tasks).await;

    let final_value = *data.lock().await;
    let expected = 50 * 20;

    println!("Mutex contention test:");
    println!("  Expected: {}", expected);
    println!("  Actual: {}", final_value);

    assert_eq!(final_value, expected,
        "Mutex should prevent race conditions");

    Ok(())
}

/// Test read-write lock contention
#[tokio::test]
async fn test_rwlock_contention() -> Result<()> {
    let data = Arc::new(RwLock::new(vec![0; 100]));
    let mut tasks = vec![];

    // Spawn readers
    for i in 0..30 {
        let data = Arc::clone(&data);
        let task = tokio::spawn(async move {
            for _ in 0..10 {
                let guard = data.read().await;
                let _sum: i32 = guard.iter().sum();
                tokio::time::sleep(Duration::from_micros(50)).await;
            }
            i
        });
        tasks.push(task);
    }

    // Spawn writers
    for i in 0..10 {
        let data = Arc::clone(&data);
        let task = tokio::spawn(async move {
            for _ in 0..5 {
                let mut guard = data.write().await;
                guard[i] += 1;
                tokio::time::sleep(Duration::from_micros(100)).await;
            }
            i
        });
        tasks.push(task);
    }

    futures_util::future::join_all(tasks).await;

    let guard = data.read().await;
    let total: i32 = guard.iter().sum();

    println!("RwLock contention test:");
    println!("  Total writes: {}", total);
    println!("  Expected: 50 (10 writers * 5 iterations)");

    assert_eq!(total, 50,
        "RwLock should handle concurrent reads and writes correctly");

    Ok(())
}

/// Test double-checked locking pattern
#[tokio::test]
async fn test_double_checked_locking() -> Result<()> {
    let initialized = Arc::new(AtomicBool::new(false));
    let lock = Arc::new(Mutex::new(()));
    let mut tasks = vec![];

    for i in 0..20 {
        let initialized = Arc::clone(&initialized);
        let lock = Arc::clone(&lock);

        let task = tokio::spawn(async move {
            // First check (without lock)
            if !initialized.load(Ordering::Acquire) {
                // Acquire lock
                let _guard = lock.lock().await;

                // Second check (with lock)
                if !initialized.load(Ordering::Acquire) {
                    println!("Task {} initializing", i);
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    initialized.store(true, Ordering::Release);
                    return true; // This task did initialization
                }
            }
            false // Already initialized
        });

        tasks.push(task);
    }

    let results = futures_util::future::join_all(tasks).await;

    let initializers = results.iter()
        .filter_map(|r| r.as_ref().ok())
        .filter(|&&init| init)
        .count();

    println!("Double-checked locking test:");
    println!("  Initializers: {} (expected: 1)", initializers);

    assert_eq!(initializers, 1,
        "Only one task should perform initialization");

    Ok(())
}

/// Test semaphore-based resource limiting
#[tokio::test]
async fn test_semaphore_limiting() -> Result<()> {
    let semaphore = Arc::new(Semaphore::new(5)); // Max 5 concurrent
    let active_count = Arc::new(AtomicUsize::new(0));
    let max_observed = Arc::new(AtomicUsize::new(0));

    let mut tasks = vec![];

    for i in 0..50 {
        let semaphore = Arc::clone(&semaphore);
        let active_count = Arc::clone(&active_count);
        let max_observed = Arc::clone(&max_observed);

        let task = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();

            let current = active_count.fetch_add(1, Ordering::SeqCst) + 1;

            // Update max observed
            max_observed.fetch_max(current, Ordering::SeqCst);

            println!("Task {}: {} active", i, current);

            tokio::time::sleep(Duration::from_millis(50)).await;

            active_count.fetch_sub(1, Ordering::SeqCst);
        });

        tasks.push(task);
    }

    futures_util::future::join_all(tasks).await;

    let max = max_observed.load(Ordering::SeqCst);

    println!("Semaphore limiting test:");
    println!("  Max concurrent: {} (limit: 5)", max);

    assert!(max <= 5,
        "Should never exceed semaphore limit");

    Ok(())
}

/// Test check-then-act race condition
#[tokio::test]
async fn test_check_then_act_race() -> Result<()> {
    let balance = Arc::new(Mutex::new(1000));
    let mut tasks = vec![];

    // Multiple tasks trying to withdraw
    for i in 0..20 {
        let balance = Arc::clone(&balance);

        let task = tokio::spawn(async move {
            let withdraw_amount = 100;

            let mut guard = balance.lock().await;

            // Check-then-act pattern (atomic within lock)
            if *guard >= withdraw_amount {
                *guard -= withdraw_amount;
                println!("Task {}: Withdrew {} (balance: {})", i, withdraw_amount, *guard);
                true
            } else {
                println!("Task {}: Insufficient funds (balance: {})", i, *guard);
                false
            }
        });

        tasks.push(task);
    }

    let results = futures_util::future::join_all(tasks).await;

    let successful = results.iter()
        .filter_map(|r| r.as_ref().ok())
        .filter(|&&success| success)
        .count();

    let final_balance = *balance.lock().await;

    println!("Check-then-act test:");
    println!("  Successful withdrawals: {}", successful);
    println!("  Final balance: {}", final_balance);
    println!("  Expected balance: {}", 1000 - (successful * 100));

    assert_eq!(final_balance, 1000 - (successful * 100),
        "Balance should be consistent");

    Ok(())
}

/// Test lost update problem
#[tokio::test]
async fn test_lost_update_prevention() -> Result<()> {
    let value = Arc::new(Mutex::new(0));
    let mut tasks = vec![];

    for _ in 0..100 {
        let value = Arc::clone(&value);

        let task = tokio::spawn(async move {
            let mut guard = value.lock().await;

            // Read-modify-write operation
            let current = *guard;
            tokio::time::sleep(Duration::from_micros(10)).await;
            *guard = current + 1;
        });

        tasks.push(task);
    }

    futures_util::future::join_all(tasks).await;

    let final_value = *value.lock().await;

    println!("Lost update test:");
    println!("  Expected: 100");
    println!("  Actual: {}", final_value);

    assert_eq!(final_value, 100,
        "Mutex should prevent lost updates");

    Ok(())
}

/// Test ABA problem with atomic operations
#[tokio::test]
async fn test_aba_problem() -> Result<()> {
    let value = Arc::new(AtomicUsize::new(0));
    let mut tasks = vec![];

    // Task 1: Increments from 0 to 1
    let value1 = Arc::clone(&value);
    let task1 = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(10)).await;
        value1.store(1, Ordering::SeqCst);
        println!("Task 1: Set to 1");
    });
    tasks.push(task1);

    // Task 2: Sets back to 0
    let value2 = Arc::clone(&value);
    let task2 = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(20)).await;
        value2.store(0, Ordering::SeqCst);
        println!("Task 2: Set to 0");
    });
    tasks.push(task2);

    // Task 3: Uses compare-and-swap
    let value3 = Arc::clone(&value);
    let task3 = tokio::spawn(async move {
        let initial = value3.load(Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(30)).await;

        // Try to CAS from initial value
        let result = value3.compare_exchange(
            initial,
            42,
            Ordering::SeqCst,
            Ordering::SeqCst
        );

        println!("Task 3: CAS result: {:?}", result);
        result.is_ok()
    });
    tasks.push(task3);

    let results = futures_util::future::join_all(tasks).await;

    println!("ABA problem test completed");
    println!("Task 3 CAS success: {}", results[2].as_ref().unwrap());

    Ok(())
}

/// Test reader-writer starvation
#[tokio::test]
async fn test_reader_writer_starvation() -> Result<()> {
    let data = Arc::new(RwLock::new(0));
    let reader_count = Arc::new(AtomicUsize::new(0));
    let writer_count = Arc::new(AtomicUsize::new(0));

    let mut tasks = vec![];

    // Many readers
    for _ in 0..100 {
        let data = Arc::clone(&data);
        let reader_count = Arc::clone(&reader_count);

        let task = tokio::spawn(async move {
            let _guard = data.read().await;
            reader_count.fetch_add(1, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_micros(100)).await;
        });

        tasks.push(task);
    }

    // Few writers
    for _ in 0..5 {
        let data = Arc::clone(&data);
        let writer_count = Arc::clone(&writer_count);

        let task = tokio::spawn(async move {
            let mut guard = data.write().await;
            *guard += 1;
            writer_count.fetch_add(1, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_micros(100)).await;
        });

        tasks.push(task);
    }

    futures_util::future::join_all(tasks).await;

    println!("Reader-writer starvation test:");
    println!("  Readers completed: {}", reader_count.load(Ordering::SeqCst));
    println!("  Writers completed: {}", writer_count.load(Ordering::SeqCst));

    assert_eq!(writer_count.load(Ordering::SeqCst), 5,
        "All writers should complete (no starvation)");

    Ok(())
}

/// Test concurrent hash map access
#[tokio::test]
async fn test_concurrent_hashmap_access() -> Result<()> {
    use std::collections::HashMap;

    let map = Arc::new(Mutex::new(HashMap::new()));
    let mut tasks = vec![];

    // Concurrent insertions
    for i in 0..50 {
        let map = Arc::clone(&map);

        let task = tokio::spawn(async move {
            let mut guard = map.lock().await;
            guard.insert(i, format!("value_{}", i));
        });

        tasks.push(task);
    }

    // Concurrent reads
    for i in 0..50 {
        let map = Arc::clone(&map);

        let task = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            let guard = map.lock().await;
            guard.get(&i).cloned()
        });

        tasks.push(task);
    }

    futures_util::future::join_all(tasks).await;

    let final_size = map.lock().await.len();

    println!("Concurrent HashMap test:");
    println!("  Final size: {} (expected: 50)", final_size);

    assert_eq!(final_size, 50,
        "All insertions should be visible");

    Ok(())
}

/// Test message passing race conditions
#[tokio::test]
async fn test_message_passing_race() -> Result<()> {
    use tokio::sync::mpsc;

    let (tx, mut rx) = mpsc::channel(100);
    let mut senders = vec![];

    // Multiple senders
    for i in 0..10 {
        let tx = tx.clone();
        let task = tokio::spawn(async move {
            for j in 0..10 {
                tx.send((i, j)).await.unwrap();
            }
        });
        senders.push(task);
    }

    drop(tx); // Drop original sender

    // Receiver
    let receiver = tokio::spawn(async move {
        let mut count = 0;
        while let Some((sender_id, msg_id)) = rx.recv().await {
            count += 1;
        }
        count
    });

    futures_util::future::join_all(senders).await;
    let received = receiver.await.unwrap();

    println!("Message passing race test:");
    println!("  Messages received: {} (expected: 100)", received);

    assert_eq!(received, 100,
        "All messages should be received");

    Ok(())
}
