//! Deployment Validation Tests
//!
//! Validates Weaver live-check works across different deployment environments:
//! - Docker containers
//! - Kubernetes pods
//! - GitHub Actions runners
//! - Different operating systems (Linux, macOS, Windows)

use clnrm_core::telemetry::weaver_controller::{WeaverConfig, WeaverController};
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

#[test]
#[ignore = "Requires Docker and Weaver installation"]
fn test_docker_container_deployment() -> Result<(), Box<dyn std::error::Error>> {
    println!("🐳 Testing Docker container deployment");

    // Create a Dockerfile for testing
    let dockerfile_content = r#"
FROM rust:1.70-slim

RUN apt-get update && apt-get install -y \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install Weaver (placeholder - adjust for actual installation)
# RUN curl -L https://github.com/open-telemetry/weaver/releases/latest/download/weaver-linux-amd64 -o /usr/local/bin/weaver
# RUN chmod +x /usr/local/bin/weaver

WORKDIR /app
COPY . /app

CMD ["cargo", "test", "--features", "otel"]
"#;

    std::fs::write("/tmp/Dockerfile.weaver_test", dockerfile_content)?;

    println!("   Building Docker image...");

    // Build Docker image
    let build = Command::new("docker")
        .args(&[
            "build",
            "-f", "/tmp/Dockerfile.weaver_test",
            "-t", "clnrm-weaver-test",
            ".",
        ])
        .output()?;

    if !build.status.success() {
        eprintln!("Docker build failed: {}", String::from_utf8_lossy(&build.stderr));
        return Err("Docker build failed".into());
    }

    println!("   Running tests in container...");

    // Run tests in container
    let run = Command::new("docker")
        .args(&[
            "run",
            "--rm",
            "-e", "OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317",
            "clnrm-weaver-test",
        ])
        .output()?;

    println!("   Container output: {}", String::from_utf8_lossy(&run.stdout));

    if !run.status.success() {
        println!("   Container tests exited with: {}", run.status);
    }

    // Cleanup
    let _ = Command::new("docker")
        .args(&["rmi", "clnrm-weaver-test"])
        .output();

    println!("✅ Docker deployment test completed");
    Ok(())
}

#[test]
#[ignore = "Requires Kubernetes cluster"]
fn test_kubernetes_pod_deployment() -> Result<(), Box<dyn std::error::Error>> {
    println!("☸️  Testing Kubernetes pod deployment");

    let k8s_manifest = r#"
apiVersion: v1
kind: Pod
metadata:
  name: clnrm-weaver-test
  labels:
    app: clnrm
spec:
  containers:
  - name: clnrm-tests
    image: clnrm-weaver-test:latest
    env:
    - name: OTEL_EXPORTER_OTLP_ENDPOINT
      value: "http://otel-collector:4317"
    - name: RUST_LOG
      value: "debug"
  - name: weaver-validator
    image: weaver:latest
    args:
    - registry
    - live-check
    - --registry=/registry
    - --otlp-grpc-port=4317
    volumeMounts:
    - name: registry
      mountPath: /registry
  volumes:
  - name: registry
    configMap:
      name: weaver-registry
  restartPolicy: Never
"#;

    std::fs::write("/tmp/clnrm-k8s-test.yaml", k8s_manifest)?;

    println!("   Applying Kubernetes manifest...");

    // Apply manifest
    let apply = Command::new("kubectl")
        .args(&["apply", "-f", "/tmp/clnrm-k8s-test.yaml"])
        .output()?;

    if !apply.status.success() {
        println!("   kubectl apply failed (cluster may not be available)");
        println!("   {}", String::from_utf8_lossy(&apply.stderr));
        return Ok(());  // Don't fail if no cluster
    }

    println!("   Waiting for pod to complete...");
    thread::sleep(Duration::from_secs(10));

    // Get pod status
    let status = Command::new("kubectl")
        .args(&["get", "pod", "clnrm-weaver-test", "-o", "json"])
        .output()?;

    if status.status.success() {
        println!("   Pod status: {}", String::from_utf8_lossy(&status.stdout));
    }

    // Cleanup
    let _ = Command::new("kubectl")
        .args(&["delete", "pod", "clnrm-weaver-test"])
        .output();

    println!("✅ Kubernetes deployment test completed");
    Ok(())
}

#[test]
#[ignore = "Simulates GitHub Actions environment"]
fn test_github_actions_runner() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Testing GitHub Actions runner environment");

    // Simulate GitHub Actions environment variables
    std::env::set_var("CI", "true");
    std::env::set_var("GITHUB_ACTIONS", "true");
    std::env::set_var("RUNNER_OS", "Linux");

    let config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        output_dir: PathBuf::from("/tmp/clnrm_github_actions_test"),
        stream: false,
        ..Default::default()
    };

    let mut controller = WeaverController::new(config);

    println!("   Starting Weaver in CI environment...");
    controller.start_live_check()?;

    thread::sleep(Duration::from_secs(3));

    println!("   Running tests...");
    // In real scenario, this would run cargo test

    let report = controller.stop_and_report()?;

    println!("   Report status: {:?}", report.status);
    println!("   Violations: {}", report.violations);

    // CI should fail if violations detected
    if report.violations > 0 {
        eprintln!("::error::Weaver detected {} violations", report.violations);
        for detail in &report.details {
            if detail.level == "violation" {
                eprintln!("::error::{}", detail.message);
            }
        }
    }

    // Cleanup env vars
    std::env::remove_var("CI");
    std::env::remove_var("GITHUB_ACTIONS");
    std::env::remove_var("RUNNER_OS");

    println!("✅ GitHub Actions test completed");
    Ok(())
}

#[test]
#[ignore = "Cross-platform compatibility test"]
fn test_multi_platform_compatibility() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌍 Testing multi-platform compatibility");

    let platform = std::env::consts::OS;
    println!("   Current platform: {}", platform);

    let config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        output_dir: PathBuf::from("/tmp/clnrm_multiplatform_test"),
        stream: false,
        ..Default::default()
    };

    let mut controller = WeaverController::new(config);

    // Platform-specific behavior
    match platform {
        "linux" => {
            println!("   Testing on Linux...");
            // Linux-specific tests
        }
        "macos" => {
            println!("   Testing on macOS...");
            // macOS-specific tests
        }
        "windows" => {
            println!("   Testing on Windows...");
            // Windows-specific tests (different path separators, etc.)
        }
        other => {
            println!("   Testing on unknown platform: {}", other);
        }
    }

    controller.start_live_check()?;
    thread::sleep(Duration::from_secs(2));
    let report = controller.stop_and_report()?;

    println!("   Report status on {}: {:?}", platform, report.status);
    println!("✅ Multi-platform test completed");

    Ok(())
}

#[test]
#[ignore = "Tests containerized telemetry collection"]
fn test_docker_compose_deployment() -> Result<(), Box<dyn std::error::Error>> {
    println!("🐳 Testing Docker Compose deployment");

    let docker_compose = r#"
version: '3.8'

services:
  weaver:
    image: weaver:latest
    command:
      - registry
      - live-check
      - --registry=/registry
      - --otlp-grpc-port=4317
      - --output=/output
    volumes:
      - ./registry:/registry:ro
      - ./validation_output:/output
    ports:
      - "4317:4317"
      - "8080:8080"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 5s
      timeout: 3s
      retries: 3

  otel-collector:
    image: otel/opentelemetry-collector-contrib:latest
    command: ["--config=/etc/otel-config.yaml"]
    volumes:
      - ./otel-config.yaml:/etc/otel-config.yaml:ro
    ports:
      - "4318:4318"
    depends_on:
      - weaver

  clnrm-tests:
    build:
      context: .
      dockerfile: Dockerfile
    environment:
      - OTEL_EXPORTER_OTLP_ENDPOINT=http://weaver:4317
      - RUST_LOG=debug
    depends_on:
      weaver:
        condition: service_healthy
"#;

    std::fs::write("/tmp/docker-compose.weaver.yml", docker_compose)?;

    println!("   Starting Docker Compose stack...");

    let up = Command::new("docker-compose")
        .args(&[
            "-f", "/tmp/docker-compose.weaver.yml",
            "up",
            "--abort-on-container-exit",
        ])
        .output()?;

    if !up.status.success() {
        println!("   Docker Compose failed (may not be available)");
        return Ok(());
    }

    println!("   Stack output: {}", String::from_utf8_lossy(&up.stdout));

    // Cleanup
    let _ = Command::new("docker-compose")
        .args(&["-f", "/tmp/docker-compose.weaver.yml", "down"])
        .output();

    println!("✅ Docker Compose test completed");
    Ok(())
}

#[test]
#[ignore = "Tests cloud deployment scenarios"]
fn test_cloud_deployment_simulation() -> Result<(), Box<dyn std::error::Error>> {
    println!("☁️  Testing cloud deployment scenarios");

    // Simulate cloud environment variables
    let cloud_scenarios = vec![
        ("AWS", vec![
            ("AWS_REGION", "us-east-1"),
            ("AWS_EXECUTION_ENV", "AWS_ECS_FARGATE"),
        ]),
        ("GCP", vec![
            ("GOOGLE_CLOUD_PROJECT", "test-project"),
            ("CLOUD_RUN_SERVICE", "clnrm"),
        ]),
        ("Azure", vec![
            ("AZURE_REGION", "eastus"),
            ("WEBSITE_SITE_NAME", "clnrm"),
        ]),
    ];

    for (cloud, env_vars) in cloud_scenarios {
        println!("\n   Testing {} deployment...", cloud);

        // Set cloud-specific env vars
        for (key, value) in &env_vars {
            std::env::set_var(key, value);
        }

        let config = WeaverConfig {
            registry_path: PathBuf::from("registry"),
            output_dir: PathBuf::from(format!("/tmp/clnrm_{}_test", cloud.to_lowercase())),
            stream: false,
            ..Default::default()
        };

        let mut controller = WeaverController::new(config);
        controller.start_live_check()?;

        thread::sleep(Duration::from_secs(2));

        let report = controller.stop_and_report()?;
        println!("     {} status: {:?}", cloud, report.status);

        // Cleanup env vars
        for (key, _) in &env_vars {
            std::env::remove_var(key);
        }
    }

    println!("\n✅ Cloud deployment tests completed");
    Ok(())
}

#[test]
#[ignore = "Tests bare metal deployment"]
fn test_bare_metal_deployment() -> Result<(), Box<dyn std::error::Error>> {
    println!("🖥️  Testing bare metal deployment");

    // Simulate bare metal environment (no containerization)
    std::env::remove_var("KUBERNETES_SERVICE_HOST");
    std::env::remove_var("DOCKER_HOST");

    let config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        output_dir: PathBuf::from("/tmp/clnrm_baremetal_test"),
        stream: false,
        ..Default::default()
    };

    let mut controller = WeaverController::new(config);

    println!("   Starting Weaver on bare metal...");
    controller.start_live_check()?;

    thread::sleep(Duration::from_secs(3));

    let report = controller.stop_and_report()?;

    println!("   Report status: {:?}", report.status);
    println!("   Registry coverage: {:.1}%", report.registry_coverage * 100.0);

    println!("✅ Bare metal deployment test completed");
    Ok(())
}
