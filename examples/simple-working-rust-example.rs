//! Simple Working Rust Example
//!
//! This demonstrates using the CleanroomEnvironment API directly.
//! Copy and run this code to see the framework in action.

use clnrm_core::cleanroom::CleanroomEnvironment;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Simple Working Rust Example");
    println!("==============================");

    // Create a cleanroom environment
    println!("\n📋 Creating CleanroomEnvironment...");
    let env = CleanroomEnvironment::new().await?;
    println!("✅ Environment created with session ID: {}", env.session_id());

    // Check health
    println!("\n📋 Checking system health...");
    let health = env.check_health().await;
    println!("✅ Health check completed for {} services", health.len());

    // Get metrics
    println!("\n📋 Collecting metrics...");
    let metrics = env.get_metrics().await?;
    println!("✅ Metrics collected:");
    println!("   - Tests executed: {}", metrics.tests_executed);
    println!("   - Active containers: {}", metrics.active_containers);
    println!("   - Active services: {}", metrics.active_services);

    // Execute a simple test
    println!("\n📋 Executing test...");
    let result = env
        .execute_test("simple_test", || {
            println!("   Running test logic...");
            Ok::<String, clnrm_core::CleanroomError>("test_passed".to_string())
        })
        .await?;
    println!("✅ Test result: {}", result);

    // Get container reuse stats
    println!("\n📋 Container reuse statistics...");
    let (created, reused) = env.get_container_reuse_stats().await;
    println!("✅ Container stats: {} created, {} reused", created, reused);

    println!("\n🎉 Example completed successfully!");
    println!("💡 This demonstrates the core CleanroomEnvironment API working correctly.");

    Ok(())
}
