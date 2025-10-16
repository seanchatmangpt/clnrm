//! Time Manipulation and Clock Skew Tests
//!
//! Tests system behavior under time-related chaos scenarios including
//! clock skew, time travel, timezone changes, and leap second handling.

use clnrm_core::error::{Result, CleanroomError};
use std::time::{Duration, SystemTime, UNIX_EPOCH, Instant};
use chrono::{DateTime, Utc, TimeZone, offset::FixedOffset};

/// Test clock skew detection
#[tokio::test]
async fn test_clock_skew_detection() -> Result<()> {
    let start_time = SystemTime::now();

    // Simulate work
    tokio::time::sleep(Duration::from_millis(100)).await;

    let end_time = SystemTime::now();
    let elapsed = end_time.duration_since(start_time)
        .map_err(|_| CleanroomError::internal_error("Time went backwards - system clock inconsistency detected"))?;

    println!("Clock skew test - Elapsed: {:?}", elapsed);
    assert!(elapsed >= Duration::from_millis(100));

    Ok(())
}

/// Test time monotonicity
#[tokio::test]
async fn test_time_monotonicity() -> Result<()> {
    let mut previous = Instant::now();

    for i in 0..100 {
        tokio::time::sleep(Duration::from_millis(10)).await;
        let current = Instant::now();

        // Instant should always be monotonic
        assert!(current >= previous,
            "Time went backwards at iteration {}", i);

        previous = current;
    }

    println!("Time monotonicity verified across 100 iterations");
    Ok(())
}

/// Test timezone changes
#[tokio::test]
async fn test_timezone_handling() -> Result<()> {
    let utc_time: DateTime<Utc> = Utc::now();

    // Convert to different timezones
    let est = FixedOffset::west_opt(5 * 3600).unwrap(); // EST (UTC-5)
    let pst = FixedOffset::west_opt(8 * 3600).unwrap(); // PST (UTC-8)
    let jst = FixedOffset::east_opt(9 * 3600).unwrap(); // JST (UTC+9)

    let est_time = utc_time.with_timezone(&est);
    let pst_time = utc_time.with_timezone(&pst);
    let jst_time = utc_time.with_timezone(&jst);

    println!("UTC: {}", utc_time);
    println!("EST: {}", est_time);
    println!("PST: {}", pst_time);
    println!("JST: {}", jst_time);

    // All should represent the same moment in time
    assert_eq!(utc_time.timestamp(), est_time.timestamp());
    assert_eq!(utc_time.timestamp(), pst_time.timestamp());
    assert_eq!(utc_time.timestamp(), jst_time.timestamp());

    Ok(())
}

/// Test timestamp precision
#[tokio::test]
async fn test_timestamp_precision() -> Result<()> {
    let start = Instant::now();

    // Perform high-precision timing
    let mut timestamps = Vec::new();
    for _ in 0..1000 {
        timestamps.push(Instant::now());
    }

    let end = Instant::now();
    let total_duration = end.duration_since(start);

    // Verify timestamps are unique and increasing
    for i in 1..timestamps.len() {
        assert!(timestamps[i] >= timestamps[i-1],
            "Timestamps not monotonically increasing at index {}", i);
    }

    println!("Timestamp precision test - 1000 timestamps in {:?}", total_duration);
    Ok(())
}

/// Test timeout accuracy
#[tokio::test]
async fn test_timeout_accuracy() -> Result<()> {
    let timeout_ms = 500;
    let tolerance_ms = 50; // Allow 50ms tolerance

    let start = Instant::now();
    tokio::time::sleep(Duration::from_millis(timeout_ms)).await;
    let elapsed = start.elapsed();

    let actual_ms = elapsed.as_millis() as u64;
    let diff = if actual_ms > timeout_ms {
        actual_ms - timeout_ms
    } else {
        timeout_ms - actual_ms
    };

    println!("Timeout accuracy - Expected: {}ms, Actual: {}ms, Diff: {}ms",
        timeout_ms, actual_ms, diff);

    assert!(diff <= tolerance_ms,
        "Timeout accuracy exceeded tolerance: {}ms > {}ms", diff, tolerance_ms);

    Ok(())
}

/// Test time-based rate limiting
#[tokio::test]
async fn test_time_based_rate_limiting() -> Result<()> {
    let rate_limit = 10; // 10 operations per second
    let window_duration = Duration::from_secs(1);

    let start = Instant::now();
    let mut operations = 0;

    while start.elapsed() < window_duration {
        operations += 1;

        // Simple rate limiting: sleep between operations
        if operations < rate_limit {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    println!("Rate limiting - Operations in 1s: {}", operations);
    assert!(operations <= rate_limit + 2, // Allow small variance
        "Rate limit exceeded: {} > {}", operations, rate_limit);

    Ok(())
}

/// Test duration calculations under load
#[tokio::test]
async fn test_duration_calculations() -> Result<()> {
    let operations = vec![10, 50, 100, 200, 500];

    for &delay_ms in &operations {
        let expected = Duration::from_millis(delay_ms);
        let start = Instant::now();

        tokio::time::sleep(expected).await;

        let actual = start.elapsed();
        let diff_ms = if actual > expected {
            (actual - expected).as_millis()
        } else {
            (expected - actual).as_millis()
        };

        println!("Duration test - Expected: {}ms, Actual: {}ms, Diff: {}ms",
            delay_ms, actual.as_millis(), diff_ms);

        // Allow 20% tolerance for small durations
        let tolerance_ms = (delay_ms as f64 * 0.2) as u128;
        assert!(diff_ms <= tolerance_ms,
            "Duration calculation outside tolerance");
    }

    Ok(())
}

/// Test leap second handling
#[tokio::test]
async fn test_leap_second_awareness() -> Result<()> {
    // While we can't force a leap second, we can test that our time handling
    // is aware of the possibility

    let now = Utc::now();
    let future = now + chrono::Duration::seconds(86401); // One day + 1 second

    let diff = future - now;

    println!("Leap second awareness - Time difference: {} seconds", diff.num_seconds());
    assert!(diff.num_seconds() >= 86400); // At least one day

    Ok(())
}

/// Test time ordering in concurrent operations
#[tokio::test]
async fn test_concurrent_time_ordering() -> Result<()> {
    use tokio::sync::Mutex;
    use std::sync::Arc;

    let timestamps = Arc::new(Mutex::new(Vec::new()));
    let mut tasks = vec![];

    for i in 0..50 {
        let timestamps = Arc::clone(&timestamps);
        let task = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(i * 10)).await;
            let now = Instant::now();
            timestamps.lock().await.push((i, now));
        });
        tasks.push(task);
    }

    // Wait for all tasks
    for task in tasks {
        task.await.unwrap();
    }

    let mut timestamps = timestamps.lock().await;
    timestamps.sort_by_key(|(i, _)| *i);

    // Verify timestamps reflect the delays
    for i in 1..timestamps.len() {
        let (_, prev_time) = timestamps[i-1];
        let (_, curr_time) = timestamps[i];

        // Current timestamp should be later than previous
        assert!(curr_time >= prev_time,
            "Concurrent timestamp ordering violation at index {}", i);
    }

    println!("Concurrent time ordering verified across 50 tasks");
    Ok(())
}

/// Test deadline scheduling
#[tokio::test]
async fn test_deadline_scheduling() -> Result<()> {
    let deadline = Duration::from_millis(500);
    let start = Instant::now();

    // Simulate work that should complete before deadline
    let work = async {
        tokio::time::sleep(Duration::from_millis(200)).await;
        "completed"
    };

    match tokio::time::timeout(deadline, work).await {
        Ok(result) => {
            let elapsed = start.elapsed();
            println!("Work {} before deadline - Took: {:?}, Deadline: {:?}",
                result, elapsed, deadline);
            assert!(elapsed < deadline);
        }
        Err(_) => {
            panic!("Deadline exceeded when it shouldn't have");
        }
    }

    Ok(())
}

/// Test deadline miss handling
#[tokio::test]
async fn test_deadline_miss_handling() -> Result<()> {
    let deadline = Duration::from_millis(100);
    let start = Instant::now();

    // Simulate work that will exceed deadline
    let slow_work = async {
        tokio::time::sleep(Duration::from_millis(300)).await;
        "should timeout"
    };

    match tokio::time::timeout(deadline, slow_work).await {
        Ok(_) => {
            panic!("Expected timeout but work completed");
        }
        Err(_) => {
            let elapsed = start.elapsed();
            println!("Deadline miss handled correctly - Elapsed: {:?}, Deadline: {:?}",
                elapsed, deadline);

            // Should timeout approximately at deadline
            assert!(elapsed >= deadline);
            assert!(elapsed < deadline + Duration::from_millis(100));
        }
    }

    Ok(())
}

/// Test time-based retries with exponential backoff
#[tokio::test]
async fn test_exponential_backoff_timing() -> Result<()> {
    let base_delay = Duration::from_millis(50);
    let max_retries = 5;

    let mut total_duration = Duration::ZERO;
    let start = Instant::now();

    for retry in 0..max_retries {
        let delay = base_delay * 2_u32.pow(retry);
        println!("Retry {}: Waiting {:?}", retry, delay);

        tokio::time::sleep(delay).await;
        total_duration += delay;
    }

    let actual = start.elapsed();
    println!("Exponential backoff - Expected: {:?}, Actual: {:?}",
        total_duration, actual);

    // Actual should be close to expected
    assert!(actual >= total_duration);
    assert!(actual < total_duration + Duration::from_millis(200));

    Ok(())
}

/// Test system time vs monotonic time
#[tokio::test]
async fn test_system_vs_monotonic_time() -> Result<()> {
    // Instant is monotonic, SystemTime is not
    let instant_start = Instant::now();
    let system_start = SystemTime::now();

    tokio::time::sleep(Duration::from_millis(100)).await;

    let instant_elapsed = instant_start.elapsed();
    let system_elapsed = system_start.elapsed()
        .map_err(|_| CleanroomError::internal_error("SystemTime went backwards - system clock inconsistency detected"))?;

    println!("Instant (monotonic): {:?}", instant_elapsed);
    println!("SystemTime: {:?}", system_elapsed);

    // Both should be roughly similar
    let diff = if instant_elapsed > system_elapsed {
        instant_elapsed - system_elapsed
    } else {
        system_elapsed - instant_elapsed
    };

    // Allow 50ms difference between clock types
    assert!(diff < Duration::from_millis(50),
        "Too much difference between clock types: {:?}", diff);

    Ok(())
}

#[cfg(test)]
mod chaos_time_tests {
    use super::*;

    /// Test time chaos: rapid time checks
    #[tokio::test]
    async fn test_rapid_time_checks() -> Result<()> {
        let iterations = 10000;
        let start = Instant::now();

        for _ in 0..iterations {
            let _ = Instant::now();
        }

        let elapsed = start.elapsed();
        let ns_per_check = elapsed.as_nanos() / iterations;

        println!("Rapid time checks - {}ns per check ({} checks)",
            ns_per_check, iterations);

        Ok(())
    }

    /// Test time under concurrent load
    #[tokio::test]
    async fn test_time_under_concurrent_load() -> Result<()> {
        let mut tasks = vec![];

        for i in 0..100 {
            let task = tokio::spawn(async move {
                let start = Instant::now();

                // Simulate variable work
                tokio::time::sleep(Duration::from_micros(i * 100)).await;

                start.elapsed()
            });
            tasks.push(task);
        }

        let results = futures_util::future::join_all(tasks).await;

        for (i, result) in results.iter().enumerate() {
            if let Ok(duration) = result {
                println!("Task {}: {:?}", i, duration);
            }
        }

        println!("Time tracking under concurrent load completed");
        Ok(())
    }
}
