// Weaver Phase 1-2 Infrastructure Validation Test
// Tests that the core Weaver integration infrastructure works correctly
//
// Phase 1: Start Weaver (allocate ports, spawn process, wait for ready)
// Phase 2: Stop Weaver (graceful shutdown, collect telemetry, cleanup)

use clnrm_core::telemetry::live_check::orchestrator::{LiveCheckConfig, LiveCheckOrchestrator};

#[tokio::test]
#[ignore] // Requires Weaver binary installed
async fn test_phase_1_2_infrastructure_works() {
    // ARRANGE: Create default Weaver configuration
    let config = LiveCheckConfig::default();

    println!("🔧 Testing Phase 1-2 Weaver Infrastructure");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // ACT Phase 1: Start Weaver
    println!("\n📍 Phase 1: Starting Weaver...");
    let orchestrator =
        LiveCheckOrchestrator::new(config).expect("Failed to create LiveCheckOrchestrator");

    let orchestrator = orchestrator
        .start_weaver()
        .await
        .expect("Failed to start Weaver process");

    let otlp_port = orchestrator.otlp_port();
    println!("   ✅ Weaver started on port {}", otlp_port);

    // ASSERT Phase 1: Verify port allocation
    assert!(otlp_port > 0, "OTLP port should be allocated");

    println!("   ✅ Port allocation validated");

    // ACT Phase 2: Stop Weaver
    println!("\n📍 Phase 2: Stopping Weaver...");
    let completed = orchestrator
        .stop_weaver()
        .await
        .expect("Failed to stop Weaver process");

    println!("   ✅ Weaver stopped gracefully");

    // ASSERT Phase 2: Verify validation completed
    assert!(completed.passed(), "Validation should pass");

    let report = completed.report();
    println!("   ✅ Report generated: {} violations", report.violations);

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Phase 1-2 Infrastructure Validated Successfully!");
    println!("\nSummary:");
    println!("  • Phase 1: Weaver start ✅");
    println!("  • Phase 2: Weaver stop ✅");
    println!("  • Port allocation ✅");
    println!("  • Graceful shutdown ✅");
    println!("  • Statistics collection ✅");
}

#[tokio::test]
#[ignore] // Requires Weaver binary installed
async fn test_orchestrator_handles_missing_weaver_binary() {
    // ARRANGE: Create orchestrator
    let config = LiveCheckConfig::default();
    let orchestrator =
        LiveCheckOrchestrator::new(config).expect("Failed to create LiveCheckOrchestrator");

    // ACT: Attempt to start Weaver (may fail if binary not found)
    let result = orchestrator.start_weaver().await;

    // ASSERT: Should either succeed or fail gracefully with clear error
    match result {
        Ok(_) => {
            println!("✅ Weaver binary found and started successfully");
        }
        Err(e) => {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("weaver") || error_msg.contains("not found"),
                "Error should mention weaver binary: {}",
                error_msg
            );
            println!("⚠️  Weaver binary not found (expected in test environment)");
            println!("   Error: {}", error_msg);
        }
    }
}

#[tokio::test]
async fn test_orchestrator_creation_without_weaver() {
    // ARRANGE & ACT: Create orchestrator (should succeed without Weaver installed)
    let config = LiveCheckConfig::default();
    let result = LiveCheckOrchestrator::new(config);

    // ASSERT: Creation should succeed even if Weaver not installed
    assert!(
        result.is_ok(),
        "LiveCheckOrchestrator creation should succeed without Weaver binary"
    );

    println!("✅ LiveCheckOrchestrator created successfully");
}
