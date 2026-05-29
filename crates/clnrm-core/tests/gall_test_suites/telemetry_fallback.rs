//! Gall Test Suite for Telemetry Emission Fallback
//!
//! Exposes the gap where traces are lost if the external weaver daemon is not running.

use clnrm_core::cleanroom::CleanroomEnvironment;

#[tokio::test]
async fn gall_gap_test_telemetry_direct_to_disk_fallback() {
    // Arrange
    // CleanroomEnvironment initialization logic.
    let _env = CleanroomEnvironment::with_config(None).await.unwrap();

    // Act
    // In a system without weaver running, we need a native primitive to write 
    // the OTLP JSON directly to a fallback directory.
    
    // Assert
    // GALL GAP: The environment currently has no offline telemetry writer fallback.
    // Traces are simply dropped if the grpc OTLP collector is unreachable.
    panic!("Gall Gap: Telemetry Emission Gap. No native fallback exists to write OTLP JSON traces to disk when Weaver daemon is absent.");
}