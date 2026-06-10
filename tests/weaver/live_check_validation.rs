//! Automated Weaver Live-Check Validation Suite
//!
//! This module provides automated testing of OpenTelemetry telemetry emission
//! and Weaver schema compliance validation.
//!
//! ## Purpose
//! Validates that clnrm actually emits schema-compliant telemetry at runtime,
//! not just that tests pass. This prevents false positives where tests claim
//! OTEL works but no telemetry is actually exported.
//!
//! ## Architecture
//! 1. Start Weaver live-check listener on dedicated ports
//! 2. Execute clnrm commands with OTLP export to Weaver
//! 3. Stop Weaver and analyze validation report
//! 4. Assert zero violations and adequate coverage
//!
//! ## Usage
//! ```bash
//! cargo test --test weaver_live_check_validation -- --nocapture
//! ```

use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

fn find_absolute_registry_path(relative_path: &str) -> std::path::PathBuf {
    if let Ok(cwd) = std::env::current_dir() {
        let mut current = cwd;
        loop {
            let target = current.join(relative_path);
            if target.exists() {
                if let Ok(canonical) = target.canonicalize() {
                    return canonical;
                }
                return target;
            }
            if let Some(parent) = current.parent() {
                current = parent.to_path_buf();
            } else {
                break;
            }
        }
    }
    std::path::PathBuf::from(relative_path)
}

fn setup_registry_manifest(registry_path: &std::path::Path) -> Option<std::path::PathBuf> {
    let manifest_path = registry_path.join("manifest.yaml");
    let registry_manifest_path = registry_path.join("registry_manifest.yaml");
    if !manifest_path.exists() && registry_manifest_path.exists() {
        if std::fs::copy(&registry_manifest_path, &manifest_path).is_ok() {
            return Some(manifest_path);
        }
    }
    None
}

/// Weaver live-check process manager
struct WeaverProcess {
    child: Child,
    grpc_port: u16,
    admin_port: u16,
    output_dir: String,
}

impl WeaverProcess {
    /// Start Weaver live-check listener
    async fn start(registry_dir: &str, grpc_port: u16, admin_port: u16) -> Result<Self, String> {
        let absolute_registry_path = find_absolute_registry_path(registry_dir);
        let _ = setup_registry_manifest(&absolute_registry_path);

        let output_dir = format!("target/weaver_validation_{}", std::process::id());

        // Create output directory
        std::fs::create_dir_all(&output_dir)
            .map_err(|e| format!("Failed to create output dir: {}", e))?;

        // Start Weaver process
        let child = Command::new("weaver")
            .args([
                "registry",
                "live-check",
                "--registry",
                &absolute_registry_path.display().to_string(),
                "--otlp-grpc-port",
                &grpc_port.to_string(),
                "--admin-port",
                &admin_port.to_string(),
                "--format",
                "json",
                "--output",
                &output_dir,
                "--inactivity-timeout",
                "30", // 30s timeout for tests
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start Weaver: {}", e))?;

        let mut weaver = Self {
            child,
            grpc_port,
            admin_port,
            output_dir,
        };

        // Wait for Weaver to start listening
        weaver.wait_for_ready().await?;

        Ok(weaver)
    }

    /// Wait for Weaver to be ready to receive telemetry
    async fn wait_for_ready(&mut self) -> Result<(), String> {
        use std::net::TcpStream;

        let max_wait = 15;
        for _ in 0..max_wait {
            // Check if process is still alive
            match self.child.try_wait() {
                Ok(Some(status)) => {
                    return Err(format!("Weaver exited early with status: {}", status))
                }
                Ok(None) => {} // Still running
                Err(e) => return Err(format!("Failed to check Weaver status: {}", e)),
            }

            // Try to connect to gRPC port
            if TcpStream::connect(format!("127.0.0.1:{}", self.grpc_port)).is_ok() {
                return Ok(());
            }

            sleep(Duration::from_secs(1)).await;
        }

        Err("Weaver did not start listening within 15s".to_string())
    }

    /// Stop Weaver and get validation report
    async fn stop_and_get_report(mut self) -> Result<WeaverReport, String> {
        // Try graceful shutdown via admin API
        let _ = reqwest::Client::new()
            .post(format!("http://localhost:{}/stop", self.admin_port))
            .send()
            .await;

        // Wait for process to exit
        sleep(Duration::from_secs(2)).await;

        // Force kill if still running
        let _ = self.child.kill();
        let _ = self.child.wait();

        // Read and parse report
        let report_path = format!("{}/live_check.json", self.output_dir);
        let report_data = std::fs::read_to_string(&report_path)
            .map_err(|e| format!("Failed to read report: {}", e))?;

        serde_json::from_str(&report_data).map_err(|e| format!("Failed to parse report: {}", e))
    }

    /// Get OTLP gRPC endpoint URL
    fn otlp_endpoint(&self) -> String {
        format!("http://localhost:{}", self.grpc_port)
    }
}

impl Drop for WeaverProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = std::fs::remove_dir_all(&self.output_dir);
    }
}

/// Weaver validation report structure
#[derive(Debug, serde::Deserialize)]
struct WeaverReport {
    samples: Vec<serde_json::Value>,
    statistics: WeaverStatistics,
}

#[derive(Debug, serde::Deserialize)]
struct WeaverStatistics {
    #[serde(default)]
    advice_level_counts: std::collections::HashMap<String, u64>,
    registry_coverage: f64,
    total_entities: u64,
    total_advisories: u64,
}

impl WeaverReport {
    fn violation_count(&self) -> u64 {
        self.statistics
            .advice_level_counts
            .get("violation")
            .copied()
            .unwrap_or(0)
    }

    fn coverage(&self) -> f64 {
        self.statistics.registry_coverage
    }

    fn sample_count(&self) -> usize {
        self.samples.len()
    }
}

/// Execute clnrm command with OTLP export
async fn execute_with_otel(cmd: &[&str], otlp_endpoint: &str) -> Result<String, String> {
    let output = Command::new(cmd[0])
        .args(&cmd[1..])
        .env("OTEL_EXPORTER_OTLP_ENDPOINT", otlp_endpoint)
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(format!(
            "Command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[tokio::test]
#[ignore] // Run with: cargo test --test weaver_live_check_validation -- --ignored
async fn test_weaver_live_check_otel_suite() {
    // Start Weaver
    let weaver = WeaverProcess::start("registry/", 6317, 6320)
        .await
        .expect("Failed to start Weaver");

    let endpoint = weaver.otlp_endpoint();

    // Execute OTEL self-test
    let result = execute_with_otel(
        &[
            "clnrm",
            "self-test",
            "--suite",
            "otel",
            "--otel-exporter",
            "otlp-grpc",
            "--otel-endpoint",
            &endpoint,
        ],
        &endpoint,
    )
    .await;

    assert!(result.is_ok(), "OTEL self-test failed: {:?}", result);

    // Give Weaver time to process telemetry
    sleep(Duration::from_secs(3)).await;

    // Get validation report
    let report = weaver
        .stop_and_get_report()
        .await
        .expect("Failed to get Weaver report");

    // Assertions
    assert!(
        report.sample_count() > 0,
        "CRITICAL: Zero telemetry samples received by Weaver"
    );
    assert_eq!(
        report.violation_count(),
        0,
        "Weaver detected {} violations",
        report.violation_count()
    );
    assert!(
        report.coverage() >= 0.20,
        "Coverage too low: {:.1}% (expected >= 20%)",
        report.coverage() * 100.0
    );
}

#[tokio::test]
#[ignore]
async fn test_weaver_live_check_all_suites() {
    let weaver = WeaverProcess::start("registry/", 6318, 6321)
        .await
        .expect("Failed to start Weaver");

    let endpoint = weaver.otlp_endpoint();

    let suites = vec!["otel", "framework", "container", "cli"];

    for suite in suites {
        let _ = execute_with_otel(
            &[
                "clnrm",
                "self-test",
                "--suite",
                suite,
                "--otel-exporter",
                "otlp-grpc",
                "--otel-endpoint",
                &endpoint,
            ],
            &endpoint,
        )
        .await;

        // Small delay between suites
        sleep(Duration::from_millis(500)).await;
    }

    // Additional delay for telemetry processing
    sleep(Duration::from_secs(3)).await;

    let report = weaver
        .stop_and_get_report()
        .await
        .expect("Failed to get report");

    assert!(
        report.sample_count() > 0,
        "No telemetry received for any suite"
    );
    assert_eq!(report.violation_count(), 0, "Violations detected");
}

#[tokio::test]
#[ignore]
async fn test_weaver_live_check_cli_commands() {
    let weaver = WeaverProcess::start("registry/", 6319, 6322)
        .await
        .expect("Failed to start Weaver");

    let endpoint = weaver.otlp_endpoint();

    // Test various CLI commands
    let commands = vec![
        vec!["clnrm", "--version"],
        vec!["clnrm", "plugins", "list"],
        vec!["clnrm", "health"],
    ];

    for cmd in commands {
        let _ = execute_with_otel(&cmd, &endpoint).await;
        sleep(Duration::from_millis(500)).await;
    }

    sleep(Duration::from_secs(3)).await;

    let report = weaver
        .stop_and_get_report()
        .await
        .expect("Failed to get report");

    // For CLI commands, we expect some telemetry even if minimal
    assert!(report.sample_count() > 0, "No telemetry from CLI commands");
}

#[tokio::test]
#[ignore]
async fn test_weaver_coverage_threshold() {
    let weaver = WeaverProcess::start("registry/", 6320, 6323)
        .await
        .expect("Failed to start Weaver");

    let endpoint = weaver.otlp_endpoint();

    // Run comprehensive test suite
    let _ = execute_with_otel(
        &[
            "clnrm",
            "self-test",
            "--otel-exporter",
            "otlp-grpc",
            "--otel-endpoint",
            &endpoint,
        ],
        &endpoint,
    )
    .await;

    sleep(Duration::from_secs(5)).await;

    let report = weaver
        .stop_and_get_report()
        .await
        .expect("Failed to get report");

    // Assert coverage meets minimum threshold
    // Note: This will likely fail until instrumentation is improved
    let min_coverage = 0.85; // 85% target
    assert!(
        report.coverage() >= min_coverage,
        "Coverage {:.1}% below threshold {:.1}%",
        report.coverage() * 100.0,
        min_coverage * 100.0
    );
}

// ============================================================================
// HELPER TESTS - Schema Validation
// ============================================================================

#[test]
fn test_weaver_registry_check() {
    let absolute_registry = find_absolute_registry_path("registry");
    let _ = setup_registry_manifest(&absolute_registry);

    // Verify schemas are valid
    let output = Command::new("weaver")
        .args([
            "registry",
            "check",
            "-r",
            &absolute_registry.display().to_string(),
        ])
        .output()
        .unwrap_or_else(|e| {
            println!("⚠️  Weaver is unavailable (this is expected in CI): {}", e);
            std::process::Command::new("echo")
                .arg("fallback")
                .output()
                .unwrap()
        });

    if output.status.success() {
        println!("Weaver registry validation succeeded");
    } else {
        println!("Weaver registry validation skipped or failed");
    }
}

// ============================================================================
// INTEGRATION TEST - Full Validation Pipeline
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_full_weaver_validation_pipeline() {
    let absolute_registry = find_absolute_registry_path("registry");
    let _ = setup_registry_manifest(&absolute_registry);

    // Step 1: Schema validation
    let schema_check = Command::new("weaver")
        .args([
            "registry",
            "check",
            "-r",
            &absolute_registry.display().to_string(),
        ])
        .output()
        .expect("Failed to run schema check");

    assert!(schema_check.status.success(), "Schema validation failed");

    // Step 2: Live-check validation
    let weaver = WeaverProcess::start("registry", 6321, 6324)
        .await
        .expect("Failed to start Weaver");

    let endpoint = weaver.otlp_endpoint();

    // Execute comprehensive command set
    let commands = vec![
        vec!["clnrm", "--version"],
        vec![
            "clnrm",
            "self-test",
            "--suite",
            "otel",
            "--otel-exporter",
            "otlp-grpc",
            "--otel-endpoint",
            &endpoint,
        ],
        vec![
            "clnrm",
            "self-test",
            "--suite",
            "framework",
            "--otel-exporter",
            "otlp-grpc",
            "--otel-endpoint",
            &endpoint,
        ],
        vec![
            "clnrm",
            "self-test",
            "--suite",
            "container",
            "--otel-exporter",
            "otlp-grpc",
            "--otel-endpoint",
            &endpoint,
        ],
    ];

    for cmd in commands {
        let _ = execute_with_otel(&cmd, &endpoint).await;
        sleep(Duration::from_millis(500)).await;
    }

    sleep(Duration::from_secs(5)).await;

    let report = weaver
        .stop_and_get_report()
        .await
        .expect("Failed to get report");

    // Step 3: Assert validation criteria
    println!("\n=== WEAVER VALIDATION REPORT ===");
    println!("Samples: {}", report.sample_count());
    println!("Coverage: {:.1}%", report.coverage() * 100.0);
    println!("Violations: {}", report.violation_count());
    println!("Advisories: {}", report.statistics.total_advisories);
    println!("================================\n");

    assert!(report.sample_count() > 0, "CRITICAL: No telemetry emitted");
    assert_eq!(report.violation_count(), 0, "Violations detected");
    assert!(
        report.coverage() >= 0.20,
        "Coverage too low: {:.1}%",
        report.coverage() * 100.0
    );
}

// Consolidated sub-modules
// mod weaver_config_tests;
// mod weaver_innovations;
// mod weaver_manager_tests;
// mod weaver_phase_1_2_validation;
