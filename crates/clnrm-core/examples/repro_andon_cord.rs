use clnrm_core::cleanroom::CleanroomEnvironment;
use clnrm_core::error::Result;
use clnrm_core::telemetry::init::init_otlp;
use clnrm_core::telemetry::live_check::{LiveCheckConfig, LiveCheckOrchestrator};
use std::time::Duration;
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Start Weaver (LiveCheckOrchestrator)
    println!("Starting LiveCheckOrchestrator...");
    let config = LiveCheckConfig {
        enabled: true,
        registry_path: "registry".into(),
        ..Default::default()
    };

    let orchestrator = LiveCheckOrchestrator::new(config)?;
    let running = orchestrator.start_weaver().await?;
    let otlp_port = running.otlp_port();
    let weaver_pid = running.pid().expect("Weaver PID missing");

    println!(
        "Weaver started on port {} with PID {}",
        otlp_port, weaver_pid
    );

    // 2. Initialize OTEL
    println!("Initializing OTEL pointing to Weaver...");
    let endpoint = format!("http://localhost:{}", otlp_port);
    let _telemetry_handle = init_otlp(&endpoint)?;

    // 3. Create CleanroomEnvironment
    println!("Creating CleanroomEnvironment...");
    let env = CleanroomEnvironment::new().await?;

    // 4. Execute a test and KILL Weaver during it
    println!("Executing a test and KILLING Weaver during it...");
    let result = env
        .execute_test("andon_cord_test", || {
            println!("Test started, waiting 1s then killing Weaver...");
            std::thread::sleep(Duration::from_secs(1));

            println!("KILLING Weaver (PID {})", weaver_pid);
            let _ = std::process::Command::new("kill")
                .arg("-9")
                .arg(weaver_pid.to_string())
                .status();

            println!("Weaver killed. Continuing test for 2 more seconds...");
            std::thread::sleep(Duration::from_secs(2));

            println!("Test finishing...");
            Ok(())
        })
        .await?;

    println!("Test result: {:?}", result);

    // 5. Check health of Weaver via orchestrator
    println!("Checking Weaver health via orchestrator...");
    let is_healthy = running.health_check().await?;
    println!("Weaver healthy? {}", is_healthy);

    if !is_healthy {
        println!("Orchestrator detected Weaver failure! But did it STOP THE LINE?");
    }

    // 6. Final report (if we can even get it)
    println!("Attempting to stop Weaver and get report...");
    let completed_result = running.stop_weaver().await;
    match completed_result {
        Ok(completed) => {
            println!("Got report! Passed: {}", completed.passed());
            println!("{}", completed.summary());
        }
        Err(e) => {
            println!("Failed to get report (expected since we killed it): {}", e);
        }
    }

    println!("Execution finished. If you see this, the 'Andon Cord' did not automatically stop the process.");

    Ok(())
}
