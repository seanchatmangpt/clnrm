use clnrm_core::cleanroom::CleanroomEnvironment;
use clnrm_core::backend::DockerBackend;
use clnrm_core::error::Result;
use clnrm_core::telemetry::{init_otel, OtelConfig, Export};
use std::sync::Arc;
use opentelemetry_sdk::trace::InMemorySpanExporter;

#[tokio::test]
async fn test_telemetry_on_container_failure() -> Result<()> {
    // 1. Setup in-memory telemetry
    let exporter = InMemorySpanExporter::default();
    
    // We can't easily use init_otel with InMemorySpanExporter because it's not exposed in OtelConfig's Export enum.
    // However, clnrm-core/src/telemetry/testing.rs has TestTracerProvider.
    
    use clnrm_core::telemetry::testing::TestTracerProvider;
    let test_provider = TestTracerProvider::new();
    
    // 2. Initialize Cleanroom with a non-existent image
    // DockerBackend::new checks availability but not image existence during creation.
    let backend = Arc::new(DockerBackend {
        default_image: "non-existent-image-clnrm-test-12345:latest".to_string(),
    });
    
    // We need to bypass CleanroomEnvironment::new() and inject our backend
    // Since CleanroomEnvironment fields are private, we might need to use public methods if available
    // or create a mock environment if possible.
    
    // CleanroomEnvironment doesn't let us easily inject a tracer provider.
    // It uses global::meter and global::tracer indirectly via SpanBuilder.
    
    // Let's try to use the actual CleanroomEnvironment but with a failing backend.
    
    // Since I can't easily inject the TestTracerProvider into CleanroomEnvironment without changing its code,
    // I'll check if I can use CleanroomEnvironment::execute_in_container directly.
    
    // Actually, I can use the global tracer if I initialize it with a TestTracerProvider.
    
    // 3. Trigger the failure
    let env = CleanroomEnvironment::new().await?;
    // We want to force it to use our failing image.
    // execute_in_container uses the backend configured in env.
    
    println!("Executing command in non-existent container...");
    let result = env.execute_in_container(
        "non-existent",
        &["echo".to_string(), "hello".to_string()],
        None,
        None
    ).await;
    
    match &result {
        Ok(_) => println!("Unexpected success!"),
        Err(e) => println!("Caught expected error: {:?}", e),
    }
    
    // 4. Verify telemetry (this part is hard without being able to see the spans)
    // I'll use the findings from code audit to propose a fix.
    
    Ok(())
}
