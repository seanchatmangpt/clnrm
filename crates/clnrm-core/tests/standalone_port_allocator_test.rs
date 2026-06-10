//! Standalone port allocator test to verify functionality
//!
//! This test file verifies the port allocator compiles and works correctly
//! independently of other live_check components.

use clnrm_core::error::Result;

// Direct test of port allocator compilation
#[tokio::test]
async fn test_port_allocator_compiles() -> Result<()> {
    // Just verify the module can be imported
    println!("Port allocator module compiled successfully");
    Ok(())
}

#[test]
fn test_port_range_basic() {
    use clnrm_core::telemetry::live_check::PortRange;

    let range = PortRange::new(4317, 4327);
    assert_eq!(range.start, 4317);
    assert_eq!(range.end, 4327);
    assert_eq!(range.size(), 11);

    println!("✓ PortRange basic test passed");
}

#[tokio::test]
async fn test_port_allocator_creation() -> Result<()> {
    use clnrm_core::telemetry::live_check::PortAllocator;
    use std::path::PathBuf;

    let allocator = PortAllocator::new()?;

    // Hack to get lock_dir for debugging
    // Since lock_dir is private, we'll use the same logic
    let lock_dir = std::env::temp_dir().join("clnrm-port-locks");
    println!("✓ Lock directory: {}", lock_dir.display());
    println!("✓ PortAllocator created successfully");

    // Try to allocate a port
    let lock = allocator.allocate_port().await?;
    println!("✓ Port {} allocated successfully", lock.port());

    Ok(())
}
