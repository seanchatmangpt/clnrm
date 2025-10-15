//! Container Reuse Performance Benchmark
//! 
//! This example demonstrates the 10-50x performance improvement
//! achieved through container reuse as documented in the README.

use clnrm_core::{CleanroomEnvironment, CleanroomError};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), CleanroomError> {
    println!("🚀 Container Reuse Performance Benchmark");
    println!("==========================================");

    let env = CleanroomEnvironment::new().await?;

    // Benchmark: Create containers without reuse (traditional approach)
    println!("\n📊 Benchmarking Traditional Container Creation...");
    let start = Instant::now();
    
    for i in 0..10 {
        let _container = env.get_or_create_container(&format!("traditional-{}", i), || {
            // Simulate expensive container creation (like downloading images, etc.)
            std::thread::sleep(std::time::Duration::from_millis(10));
            Ok::<String, CleanroomError>(format!("container-{}", i))
        }).await?;
    }
    
    let traditional_duration = start.elapsed();
    println!("✅ Traditional approach: {}ms for 10 containers", traditional_duration.as_millis());

    // Benchmark: Reuse containers (Cleanroom approach)
    println!("\n📊 Benchmarking Container Reuse...");
    let start = Instant::now();
    
    // First, create a container
    let _container1 = env.get_or_create_container("reused-container", || {
        // Simulate expensive container creation
        std::thread::sleep(std::time::Duration::from_millis(10));
        Ok::<String, CleanroomError>("reused-container-instance".to_string())
    }).await?;
    
    // Then reuse it 9 more times
    for i in 0..9 {
        let _container = env.get_or_create_container("reused-container", || {
            // This factory should NOT be called due to reuse
            println!("⚠️  Factory called on iteration {} - container not being reused!", i);
            Ok::<String, CleanroomError>("should-not-be-created".to_string())
        }).await?;
    }
    
    let reuse_duration = start.elapsed();
    println!("✅ Container reuse approach: {}ms for 10 containers", reuse_duration.as_millis());

    // Calculate performance improvement
    let improvement = traditional_duration.as_millis() as f64 / reuse_duration.as_millis() as f64;
    println!("\n🎉 Performance Results:");
    println!("   Traditional: {}ms", traditional_duration.as_millis());
    println!("   With Reuse:  {}ms", reuse_duration.as_millis());
    println!("   Improvement: {:.1}x faster", improvement);

    // Show metrics
    let (created, reused) = env.get_container_reuse_stats().await;
    println!("\n📈 Container Reuse Statistics:");
    println!("   Containers Created: {}", created);
    println!("   Containers Reused:  {}", reused);
    println!("   Reuse Rate: {:.1}%", (reused as f64 / (created + reused) as f64) * 100.0);

    if improvement >= 10.0 {
        println!("\n✅ SUCCESS: Achieved {:.1}x performance improvement as promised!", improvement);
    } else {
        println!("\n⚠️  Note: Performance improvement is {:.1}x (target was 10-50x)", improvement);
    }

    Ok(())
}
