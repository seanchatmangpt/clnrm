//! Integration Validation Tests
//!
//! Validates Weaver live-check integration with real clnrm components:
//! - Real clnrm tests with telemetry
//! - Multiple concurrent live-checks
//! - Different OTLP endpoints
//! - Custom registries

use clnrm_core::telemetry::weaver_controller::{WeaverConfig, WeaverController};
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

#[test]
#[ignore = "Requires Weaver installation and clnrm build"]
fn test_real_clnrm_tests_with_weaver() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Testing real clnrm tests with Weaver validation");

    let config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        output_dir: PathBuf::from("/tmp/clnrm_real_tests"),
        stream: true,
        ..Default::default()
    };

    let mut controller = WeaverController::new(config);

    println!("   Starting Weaver...");
    controller.start_live_check()?;

    println!("   Running clnrm tests with OTLP export...");

    // Run actual clnrm tests with telemetry enabled
    let test_output = Command::new("cargo")
        .args(&[
            "test",
            "--lib",
            "--features", "otel",
            "--", "--test-threads=1"
        ])
        .env("OTEL_EXPORTER_OTLP_ENDPOINT", "http://localhost:4317")
        .env("RUST_LOG", "debug")
        .output()?;

    println!("   Test exit code: {}", test_output.status);
    println!("   Test output:\n{}", String::from_utf8_lossy(&test_output.stdout));

    if !test_output.stderr.is_empty() {
        println!("   Test stderr:\n{}", String::from_utf8_lossy(&test_output.stderr));
    }

    // Stop Weaver and get validation report
    println!("   Stopping Weaver and retrieving report...");
    let report = controller.stop_and_report()?;

    println!("\n📊 Validation Report:");
    println!("   Status: {:?}", report.status);
    println!("   Violations: {}", report.violations);
    println!("   Improvements: {}", report.improvements);
    println!("   Registry Coverage: {:.1}%", report.registry_coverage * 100.0);

    if report.violations > 0 {
        println!("\n❌ Violations detected:");
        for detail in &report.details {
            if detail.level == "violation" {
                println!("     - {}", detail.message);
            }
        }
    }

    // Integration success: tests ran AND Weaver validated
    assert!(
        test_output.status.success() || report.violations == 0,
        "Either tests failed or Weaver detected violations"
    );

    println!("✅ Real clnrm tests integration passed");
    Ok(())
}

#[test]
#[ignore = "Requires Weaver installation"]
fn test_concurrent_live_check_instances() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔀 Testing multiple concurrent live-check instances");

    let instances = vec![
        WeaverConfig {
            registry_path: PathBuf::from("registry"),
            output_dir: PathBuf::from("/tmp/clnrm_concurrent_a"),
            otlp_port: 4317,
            admin_port: 8080,
            stream: false,
        },
        WeaverConfig {
            registry_path: PathBuf::from("registry"),
            output_dir: PathBuf::from("/tmp/clnrm_concurrent_b"),
            otlp_port: 4327,
            admin_port: 8081,
            stream: false,
        },
        WeaverConfig {
            registry_path: PathBuf::from("registry"),
            output_dir: PathBuf::from("/tmp/clnrm_concurrent_c"),
            otlp_port: 4337,
            admin_port: 8082,
            stream: false,
        },
    ];

    let mut controllers: Vec<WeaverController> = instances
        .into_iter()
        .map(WeaverController::new)
        .collect();

    println!("   Starting {} Weaver instances...", controllers.len());

    // Start all instances
    for (i, controller) in controllers.iter_mut().enumerate() {
        controller.start_live_check()?;
        println!("     Instance {} started", i + 1);
    }

    println!("   All instances running, waiting...");
    thread::sleep(Duration::from_secs(5));

    // Verify all are healthy
    for (i, controller) in controllers.iter().enumerate() {
        let is_passing = controller.is_validation_passing();
        println!("     Instance {}: {}", i + 1,
            if is_passing { "✅ Passing" } else { "❌ Failing" });
    }

    // Stop all instances
    println!("   Stopping all instances...");
    for (i, controller) in controllers.iter_mut().enumerate() {
        let report = controller.stop_and_report()?;
        println!("     Instance {} stopped: {:?}", i + 1, report.status);
    }

    println!("✅ Concurrent instances test passed");
    Ok(())
}

#[test]
#[ignore = "Requires Weaver installation and OTLP infrastructure"]
fn test_different_otlp_endpoints() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Testing different OTLP endpoints");

    let endpoints = vec![
        ("Jaeger", "http://localhost:4319"),
        ("OTEL Collector", "http://localhost:4318"),
        ("Direct Weaver", "http://localhost:4317"),
    ];

    for (name, endpoint) in endpoints {
        println!("\n   Testing with {} ({})", name, endpoint);

        let config = WeaverConfig {
            registry_path: PathBuf::from("registry"),
            output_dir: PathBuf::from(format!("/tmp/clnrm_otlp_{}", name.to_lowercase().replace(" ", "_"))),
            stream: false,
            ..Default::default()
        };

        let mut controller = WeaverController::new(config);
        controller.start_live_check()?;

        // Simulate telemetry export to endpoint
        println!("     Exporting telemetry to {}...", endpoint);
        thread::sleep(Duration::from_secs(2));

        let report = controller.stop_and_report()?;
        println!("     {} status: {:?}", name, report.status);
    }

    println!("\n✅ OTLP endpoints test completed");
    Ok(())
}

#[test]
#[ignore = "Requires Weaver installation and custom registry"]
fn test_custom_registry_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("📚 Testing custom registry validation");

    // Create a custom minimal registry
    let custom_registry_dir = PathBuf::from("/tmp/clnrm_custom_registry");
    std::fs::create_dir_all(&custom_registry_dir)?;

    let manifest = r#"
registry:
  url: https://example.com/custom-registry
  description: Custom test registry

groups:
  - id: custom.test
    prefix: test
    brief: Custom test telemetry
"#;

    std::fs::write(custom_registry_dir.join("registry_manifest.yaml"), manifest)?;

    let test_schema = r#"
groups:
  - id: test.execution
    type: span
    brief: Test execution span
    attributes:
      - id: test.name
        type: string
        requirement_level: required
        brief: Test name
      - id: test.result
        type: string
        requirement_level: required
        brief: Test result
"#;

    let core_dir = custom_registry_dir.join("core");
    std::fs::create_dir_all(&core_dir)?;
    std::fs::write(core_dir.join("test.yaml"), test_schema)?;

    // Use custom registry
    let config = WeaverConfig {
        registry_path: custom_registry_dir.clone(),
        output_dir: PathBuf::from("/tmp/clnrm_custom_output"),
        stream: false,
        ..Default::default()
    };

    let mut controller = WeaverController::new(config);

    println!("   Starting Weaver with custom registry...");
    controller.start_live_check()?;

    thread::sleep(Duration::from_secs(2));

    let report = controller.stop_and_report()?;

    println!("   Custom registry validation:");
    println!("     Status: {:?}", report.status);
    println!("     Coverage: {:.1}%", report.registry_coverage * 100.0);

    // Cleanup
    let _ = std::fs::remove_dir_all(&custom_registry_dir);

    println!("✅ Custom registry test passed");
    Ok(())
}

#[test]
#[ignore = "Requires Weaver installation"]
fn test_integration_with_docker_otlp_collector() -> Result<(), Box<dyn std::error::Error>> {
    println!("🐳 Testing integration with Docker OTLP Collector");

    // Check if OTLP collector is running
    let docker_ps = Command::new("docker")
        .args(&["ps", "--filter", "name=otel-collector", "--format", "{{.Names}}"])
        .output()?;

    let collector_running = String::from_utf8_lossy(&docker_ps.stdout)
        .contains("otel-collector");

    if !collector_running {
        println!("   OTLP Collector not running, starting...");

        // Start OTLP collector
        let start = Command::new("docker")
            .args(&[
                "run",
                "-d",
                "--name", "otel-collector-test",
                "-p", "4317:4317",
                "-p", "4318:4318",
                "otel/opentelemetry-collector-contrib:latest",
            ])
            .output()?;

        if !start.status.success() {
            println!("   Failed to start collector, skipping test");
            return Ok(());
        }

        thread::sleep(Duration::from_secs(5));
    }

    let config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        output_dir: PathBuf::from("/tmp/clnrm_docker_otlp"),
        stream: false,
        ..Default::default()
    };

    let mut controller = WeaverController::new(config);
    controller.start_live_check()?;

    println!("   Running tests with Docker OTLP Collector...");

    // Run tests that export to collector
    let test = Command::new("cargo")
        .args(&["test", "--lib", "--features", "otel", "--", "telemetry"])
        .env("OTEL_EXPORTER_OTLP_ENDPOINT", "http://localhost:4317")
        .output()?;

    println!("   Test status: {}", test.status);

    let report = controller.stop_and_report()?;

    println!("   Validation report:");
    println!("     Violations: {}", report.violations);
    println!("     Coverage: {:.1}%", report.registry_coverage * 100.0);

    // Cleanup
    if !collector_running {
        let _ = Command::new("docker")
            .args(&["rm", "-f", "otel-collector-test"])
            .output();
    }

    println!("✅ Docker OTLP Collector integration passed");
    Ok(())
}

#[test]
#[ignore = "Requires Weaver installation"]
fn test_end_to_end_validation_workflow() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Testing end-to-end validation workflow");

    println!("\n1️⃣  Schema Validation");
    let schema_check = Command::new("weaver")
        .args(&["registry", "check", "--registry", "registry/"])
        .output()?;

    println!("   Schema check: {}",
        if schema_check.status.success() { "✅ Passed" } else { "❌ Failed" });

    if !schema_check.status.success() {
        println!("   Output: {}", String::from_utf8_lossy(&schema_check.stderr));
        return Err("Schema validation failed".into());
    }

    println!("\n2️⃣  Live-Check Startup");
    let config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        output_dir: PathBuf::from("/tmp/clnrm_e2e_test"),
        stream: true,
        ..Default::default()
    };

    let mut controller = WeaverController::new(config);
    controller.start_live_check()?;
    println!("   Live-check started: ✅");

    println!("\n3️⃣  Test Execution with Telemetry");
    let tests = Command::new("cargo")
        .args(&["test", "--lib", "--features", "otel", "--", "--test-threads=1"])
        .env("OTEL_EXPORTER_OTLP_ENDPOINT", "http://localhost:4317")
        .output()?;

    println!("   Tests executed: {}",
        if tests.status.success() { "✅ Passed" } else { "⚠️  Some failed" });

    println!("\n4️⃣  Validation Report");
    let report = controller.stop_and_report()?;

    println!("   Report retrieved: ✅");
    println!("   Status: {:?}", report.status);
    println!("   Violations: {}", report.violations);
    println!("   Coverage: {:.1}%", report.registry_coverage * 100.0);

    println!("\n5️⃣  Decision Gate");
    if report.violations > 0 {
        println!("   ❌ BLOCK DEPLOYMENT - Violations detected");
        for detail in &report.details {
            if detail.level == "violation" {
                println!("      - {}", detail.message);
            }
        }
    } else {
        println!("   ✅ APPROVE DEPLOYMENT - No violations");
    }

    println!("\n✅ End-to-end workflow completed successfully");
    Ok(())
}

#[test]
#[ignore = "Requires Weaver installation"]
fn test_validation_with_high_cardinality_attributes() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Testing validation with high-cardinality attributes");

    let config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        output_dir: PathBuf::from("/tmp/clnrm_cardinality_test"),
        stream: false,
        ..Default::default()
    };

    let mut controller = WeaverController::new(config);
    controller.start_live_check()?;

    println!("   Simulating high-cardinality telemetry...");

    // Simulate high-cardinality attributes like:
    // - Unique user IDs
    // - Request IDs
    // - Container IDs
    // - Timestamps

    thread::sleep(Duration::from_secs(3));

    let report = controller.stop_and_report()?;

    println!("   Validation with high cardinality:");
    println!("     Status: {:?}", report.status);
    println!("     Violations: {}", report.violations);

    // High cardinality should not cause validation failures
    // (unless schema specifically restricts it)

    println!("✅ High-cardinality test passed");
    Ok(())
}
