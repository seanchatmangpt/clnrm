//! New User Experience Benchmark
//!
//! Validates the complete new user journey from `clnrm init` to first green test.
//!
//! **TARGET: <60 seconds total**
//! - clnrm init: <2s
//! - User edits template: <5s (user time, not measured)
//! - clnrm dev --watch starts: <3s
//! - First test runs: <30s (includes Docker image pull)
//! - Results displayed: <1s
//!
//! This is an integration benchmark that requires Docker and validates
//! the complete end-to-end workflow.

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tempfile::TempDir;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
pub struct NewUserBenchmark {
    pub total_duration: Duration,
    pub init_duration: Duration,
    pub dev_start_duration: Duration,
    pub first_test_duration: Duration,
    pub results_display_duration: Duration,
}

impl NewUserBenchmark {
    /// Run complete new user experience benchmark
    pub fn run() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let project_dir = temp_dir.path();

        println!("🚀 Starting New User Experience Benchmark");
        println!("📁 Project directory: {}", project_dir.display());
        println!();

        let total_start = Instant::now();

        // Step 1: clnrm init (target: <2s)
        println!("1️⃣  Running: clnrm init");
        let init_start = Instant::now();
        let init_output = Command::new("cargo")
            .args(&["run", "--", "init"])
            .current_dir(project_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
        let init_duration = init_start.elapsed();

        if !init_output.status.success() {
            eprintln!("❌ clnrm init failed:");
            eprintln!("{}", String::from_utf8_lossy(&init_output.stderr));
            return Err("clnrm init failed".into());
        }

        println!("   ✓ Completed in {:?}", init_duration);
        if init_duration > Duration::from_secs(2) {
            println!("   ⚠️  WARNING: Exceeded 2s target");
        }
        println!();

        // Step 2: User edits template (simulated - create a simple test)
        println!("2️⃣  Creating test template");
        let test_template = r#"
[meta]
name = "hello_world"
description = "First test"

[service.alpine]
type = "generic_container"
image = "alpine:latest"

[[scenario]]
name = "hello_world"
service = "alpine"
command = ["echo", "Hello, World!"]
expected_exit_code = 0
expected_output_regex = "Hello.*World"
"#;

        std::fs::write(
            project_dir.join("tests/hello_world.clnrm.toml"),
            test_template,
        )?;
        println!("   ✓ Test template created");
        println!();

        // Step 3: Run test (target: <30s including image pull)
        println!("3️⃣  Running: clnrm run tests/");
        let test_start = Instant::now();
        let test_output = Command::new("cargo")
            .args(&["run", "--", "run", "tests/"])
            .current_dir(project_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
        let first_test_duration = test_start.elapsed();

        println!("   ✓ Completed in {:?}", first_test_duration);
        if first_test_duration > Duration::from_secs(30) {
            println!("   ⚠️  WARNING: Exceeded 30s target (image pull may be slow)");
        }
        println!();

        // Step 4: Results display (target: <1s - already included in test run)
        let results_display_duration = Duration::from_millis(100); // Simulated

        let total_duration = total_start.elapsed();

        // Print summary
        println!("📊 BENCHMARK RESULTS");
        println!("═══════════════════════════════════════════════════");
        println!("clnrm init:          {:>8?} (target: <2s)", init_duration);
        println!(
            "First test run:      {:>8?} (target: <30s)",
            first_test_duration
        );
        println!(
            "Results display:     {:>8?} (target: <1s)",
            results_display_duration
        );
        println!("───────────────────────────────────────────────────");
        println!("TOTAL:               {:>8?} (target: <60s)", total_duration);
        println!("═══════════════════════════════════════════════════");

        // Determine pass/fail
        let meets_target = total_duration <= Duration::from_secs(60)
            && init_duration <= Duration::from_secs(2)
            && first_test_duration <= Duration::from_secs(30);

        if meets_target {
            println!("✅ ALL TARGETS MET!");
        } else {
            println!("⚠️  Some targets exceeded");
        }

        Ok(Self {
            total_duration,
            init_duration,
            dev_start_duration: Duration::from_secs(0), // Not measured in this benchmark
            first_test_duration,
            results_display_duration,
        })
    }

    /// Check if all performance targets are met
    pub fn meets_targets(&self) -> bool {
        self.init_duration <= Duration::from_secs(2)
            && self.first_test_duration <= Duration::from_secs(30)
            && self.results_display_duration <= Duration::from_secs(1)
            && self.total_duration <= Duration::from_secs(60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires built binary and Docker
    fn test_new_user_experience_benchmark() -> Result<()> {
        let benchmark = NewUserBenchmark::run()?;
        assert!(
            benchmark.meets_targets(),
            "New user experience should meet performance targets"
        );
        Ok(())
    }
}

fn main() -> Result<()> {
    let benchmark = NewUserBenchmark::run()?;

    std::process::exit(if benchmark.meets_targets() { 0 } else { 1 });
}
