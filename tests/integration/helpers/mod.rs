//! Test helpers for integration tests
//!
//! This module provides common utilities and helpers for setting up
//! and tearing down test environments.

use std::path::PathBuf;
use std::sync::Once;
use tempfile::TempDir;

static INIT: Once = Once::new();

/// Initialize test environment (logging, tracing, etc.)
/// This is called once per test run
pub fn init_test_environment() {
    INIT.call_once(|| {
        // Initialize logging
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();

        // Initialize tracing
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_env_filter("debug")
            .try_init();
    });
}

/// Test context that provides isolated environment for each test
pub struct TestContext {
    pub temp_dir: TempDir,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
}

impl TestContext {
    /// Create a new test context with isolated directories
    pub fn new() -> anyhow::Result<Self> {
        init_test_environment();

        let temp_dir = TempDir::new()?;
        let config_dir = temp_dir.path().join("config");
        let data_dir = temp_dir.path().join("data");

        std::fs::create_dir_all(&config_dir)?;
        std::fs::create_dir_all(&data_dir)?;

        Ok(Self {
            temp_dir,
            config_dir,
            data_dir,
        })
    }

    /// Get path to temporary directory
    pub fn temp_path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }

    /// Create a test file with content
    pub fn create_file(&self, relative_path: &str, content: &str) -> anyhow::Result<PathBuf> {
        let file_path = self.temp_path().join(relative_path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&file_path, content)?;
        Ok(file_path)
    }

    /// Read a test file
    pub fn read_file(&self, relative_path: &str) -> anyhow::Result<String> {
        let file_path = self.temp_path().join(relative_path);
        Ok(std::fs::read_to_string(file_path)?)
    }
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| panic!("Failed to create test context - this indicates a critical system configuration issue"))
    }
}

/// Check if Docker is available
pub fn docker_available() -> bool {
    use std::process::Command;

    Command::new("docker")
        .args(&["ps"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Skip test if Docker is not available
#[macro_export]
macro_rules! skip_if_no_docker {
    () => {
        if !$crate::helpers::docker_available() {
            println!("Docker not available, skipping test");
            println!("To run Docker tests:");
            println!("  1. Start Docker Desktop or Docker daemon");
            println!("  2. Run: docker ps (to verify Docker is working)");
            println!("  3. Re-run the tests");
            return;
        }
    };
}

/// Wait for a condition to be true with timeout
pub async fn wait_for<F>(mut condition: F, timeout_secs: u64) -> bool
where
    F: FnMut() -> bool,
{
    use tokio::time::{sleep, Duration};

    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    while start.elapsed() < timeout {
        if condition() {
            return true;
        }
        sleep(Duration::from_millis(100)).await;
    }

    false
}

/// Guard that runs cleanup function when dropped
pub struct TestGuard<F: FnOnce()> {
    cleanup: Option<F>,
}

impl<F: FnOnce()> TestGuard<F> {
    pub fn new(cleanup: F) -> Self {
        Self {
            cleanup: Some(cleanup),
        }
    }
}

impl<F: FnOnce()> Drop for TestGuard<F> {
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup();
        }
    }
}
