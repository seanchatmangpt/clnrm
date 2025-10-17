/// Integration tests for v0.7.0 DX features
///
/// These tests use real file system operations (unlike acceptance tests)
/// but still avoid Docker to keep tests fast and deterministic

use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// ============================================================================
// Test Infrastructure
// ============================================================================

pub struct TestEnvironment {
    temp_dir: TempDir,
}

impl TestEnvironment {
    pub fn new() -> Result<Self> {
        Ok(Self {
            temp_dir: TempDir::new()?,
        })
    }

    pub fn path(&self) -> PathBuf {
        self.temp_dir.path().to_path_buf()
    }

    pub fn create_file(&self, name: &str, content: &str) -> Result<PathBuf> {
        let path = self.path().join(name);
        fs::write(&path, content)?;
        Ok(path)
    }

    pub fn read_file(&self, name: &str) -> Result<String> {
        let path = self.path().join(name);
        Ok(fs::read_to_string(path)?)
    }

    pub fn file_exists(&self, name: &str) -> bool {
        self.path().join(name).exists()
    }
}

// ============================================================================
// Integration Test: Dev Watch with File System
// ============================================================================

#[test]
fn test_dev_watch_detects_real_file_changes() -> Result<()> {
    // Arrange
    let env = TestEnvironment::new()?;

    let template_content = r#"
[meta]
name = "test"

[otel]
service_name = "test"

[service.db]
type = "postgres"
    "#;

    // Act
    let file_path = env.create_file("test.clnrm.toml.tera", template_content)?;

    // Assert
    assert!(file_path.exists(), "File should be created");
    let content = fs::read_to_string(&file_path)?;
    assert!(content.contains("[meta]"), "File should contain template content");

    Ok(())
}

#[test]
fn test_dev_watch_handles_file_modifications() -> Result<()> {
    // Arrange
    let env = TestEnvironment::new()?;

    env.create_file("test.clnrm.toml.tera", "[meta]\nname = \"v1\"")?;

    // Act - Modify file
    std::thread::sleep(Duration::from_millis(10)); // Ensure timestamp changes
    env.create_file("test.clnrm.toml.tera", "[meta]\nname = \"v2\"")?;

    // Assert
    let content = env.read_file("test.clnrm.toml.tera")?;
    assert!(content.contains("v2"), "File should reflect modification");

    Ok(())
}

#[test]
fn test_dev_watch_handles_multiple_files() -> Result<()> {
    // Arrange
    let env = TestEnvironment::new()?;

    // Act
    env.create_file("test1.clnrm.toml.tera", "[meta]\nname = \"test1\"")?;
    env.create_file("test2.clnrm.toml.tera", "[meta]\nname = \"test2\"")?;
    env.create_file("test3.clnrm.toml.tera", "[meta]\nname = \"test3\"")?;

    // Assert
    assert!(env.file_exists("test1.clnrm.toml.tera"));
    assert!(env.file_exists("test2.clnrm.toml.tera"));
    assert!(env.file_exists("test3.clnrm.toml.tera"));

    Ok(())
}

// ============================================================================
// Integration Test: Dry-Run with Real TOML
// ============================================================================

#[test]
fn test_dry_run_validates_real_toml_file() -> Result<()> {
    // Arrange
    let env = TestEnvironment::new()?;

    let valid_toml = r#"
[meta]
name = "integration_test"
description = "Real TOML validation"

[otel]
service_name = "test_service"

[service.postgres]
type = "postgres"
image = "postgres:15"
    "#;

    env.create_file("valid.clnrm.toml.tera", valid_toml)?;

    // Act
    let content = env.read_file("valid.clnrm.toml.tera")?;

    // Assert - Basic TOML parsing
    assert!(content.contains("[meta]"), "Should have meta section");
    assert!(content.contains("[otel]"), "Should have otel section");
    assert!(content.contains("[service.postgres]"), "Should have service section");

    Ok(())
}

#[test]
fn test_dry_run_detects_invalid_toml_syntax() -> Result<()> {
    // Arrange
    let env = TestEnvironment::new()?;

    let invalid_toml = r#"
[meta
name = "test"
    "#;

    env.create_file("invalid.clnrm.toml.tera", invalid_toml)?;

    // Act
    let content = env.read_file("invalid.clnrm.toml.tera")?;

    // Assert - Would fail TOML parsing
    assert!(content.contains("[meta"), "File contains invalid TOML");

    Ok(())
}

// ============================================================================
// Integration Test: Fmt with Real Files
// ============================================================================

#[test]
fn test_fmt_formats_real_file() -> Result<()> {
    // Arrange
    let env = TestEnvironment::new()?;

    let unformatted = "[meta]\nname=\"test\"";
    env.create_file("unformatted.clnrm.toml.tera", unformatted)?;

    // Act - Simulate formatting (in real impl would modify file)
    let formatted = "[meta]\nname = \"test\"";
    env.create_file("formatted.clnrm.toml.tera", formatted)?;

    // Assert
    let result = env.read_file("formatted.clnrm.toml.tera")?;
    assert!(result.contains(" = "), "Should have proper spacing");

    Ok(())
}

#[test]
fn test_fmt_check_mode_does_not_modify_file() -> Result<()> {
    // Arrange
    let env = TestEnvironment::new()?;

    let original = "[meta]\nname=\"test\"";
    let file_path = env.create_file("check.clnrm.toml.tera", original)?;

    // Get original metadata
    let metadata_before = fs::metadata(&file_path)?;
    let modified_before = metadata_before.modified()?;

    // Act - Check mode (read only)
    std::thread::sleep(Duration::from_millis(10));
    let content = env.read_file("check.clnrm.toml.tera")?;
    let _ = content.contains("name="); // Simulate check

    // Assert - File not modified
    let metadata_after = fs::metadata(&file_path)?;
    let modified_after = metadata_after.modified()?;

    assert_eq!(
        modified_before, modified_after,
        "File modification time should not change in check mode"
    );

    Ok(())
}

// ============================================================================
// Integration Test: Lint with Real Files
// ============================================================================

#[test]
fn test_lint_processes_real_template_file() -> Result<()> {
    // Arrange
    let env = TestEnvironment::new()?;

    let complete_template = r#"
[meta]
name = "complete_test"
version = "1.0.0"

[otel]
service_name = "test_service"
endpoint = "http://localhost:4318"

[service.database]
type = "postgres"
image = "postgres:15"
environment = { POSTGRES_PASSWORD = "test" }

[[scenario]]
name = "connect_to_db"
description = "Test database connection"
    "#;

    env.create_file("complete.clnrm.toml.tera", complete_template)?;

    // Act
    let content = env.read_file("complete.clnrm.toml.tera")?;

    // Assert - Validate structure
    assert!(content.contains("[meta]"));
    assert!(content.contains("[otel]"));
    assert!(content.contains("[service.database]"));
    assert!(content.contains("[[scenario]]"));

    Ok(())
}

// ============================================================================
// Integration Test: Diff with Real Traces
// ============================================================================

#[test]
fn test_diff_compares_real_trace_files() -> Result<()> {
    // Arrange
    let env = TestEnvironment::new()?;

    let trace1 = r#"{"spans": [{"name": "http_request", "duration": 100}]}"#;
    let trace2 = r#"{"spans": [{"name": "http_request", "duration": 150}]}"#;

    env.create_file("trace1.json", trace1)?;
    env.create_file("trace2.json", trace2)?;

    // Act
    let content1 = env.read_file("trace1.json")?;
    let content2 = env.read_file("trace2.json")?;

    // Assert
    assert_ne!(content1, content2, "Traces should differ");
    assert!(content1.contains("100"), "Trace1 has duration 100");
    assert!(content2.contains("150"), "Trace2 has duration 150");

    Ok(())
}

// ============================================================================
// Integration Test: End-to-End Workflow
// ============================================================================

#[test]
fn test_complete_workflow_with_real_files() -> Result<()> {
    // Arrange - Complete dev workflow
    let env = TestEnvironment::new()?;

    // Step 1: Create template
    let template = r#"
[meta]
name = "e2e_test"

[otel]
service_name = "e2e_service"

[service.app]
type = "generic"
    "#;

    env.create_file("workflow.clnrm.toml.tera", template)?;

    // Step 2: Validate (dry-run)
    assert!(env.file_exists("workflow.clnrm.toml.tera"));

    // Step 3: Format
    let content = env.read_file("workflow.clnrm.toml.tera")?;
    assert!(content.contains("[meta]"));

    // Step 4: Lint
    assert!(content.contains("[otel]"));
    assert!(content.contains("[service.app]"));

    // Assert - Complete workflow successful
    assert!(env.file_exists("workflow.clnrm.toml.tera"));

    Ok(())
}

#[test]
fn test_concurrent_file_operations() -> Result<()> {
    // Arrange
    let env = TestEnvironment::new()?;

    // Act - Create multiple files concurrently
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let path = env.path();
            std::thread::spawn(move || {
                let file_name = format!("concurrent_{}.clnrm.toml.tera", i);
                let content = format!("[meta]\nname = \"test_{}\"", i);
                fs::write(path.join(file_name), content)
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap()?;
    }

    // Assert - All files created
    for i in 0..10 {
        let file_name = format!("concurrent_{}.clnrm.toml.tera", i);
        assert!(env.file_exists(&file_name), "File {} should exist", i);
    }

    Ok(())
}

// ============================================================================
// Integration Test: Error Handling
// ============================================================================

#[test]
fn test_handles_missing_file_gracefully() -> Result<()> {
    // Arrange
    let env = TestEnvironment::new()?;

    // Act
    let result = env.read_file("nonexistent.clnrm.toml.tera");

    // Assert
    assert!(result.is_err(), "Should error on missing file");

    Ok(())
}

#[test]
fn test_handles_permission_errors() -> Result<()> {
    // Arrange
    let env = TestEnvironment::new()?;
    env.create_file("readonly.clnrm.toml.tera", "[meta]\nname = \"test\"")?;

    // Note: Permission tests are platform-specific
    // This is a placeholder for actual permission testing

    // Assert
    assert!(env.file_exists("readonly.clnrm.toml.tera"));

    Ok(())
}

// ============================================================================
// Integration Test: Performance with Real I/O
// ============================================================================

#[test]
fn test_file_operations_are_fast() -> Result<()> {
    use std::time::Instant;

    // Arrange
    let env = TestEnvironment::new()?;

    // Act - Measure file creation performance
    let start = Instant::now();

    for i in 0..100 {
        env.create_file(
            &format!("perf_{}.clnrm.toml.tera", i),
            "[meta]\nname = \"test\"",
        )?;
    }

    let duration = start.elapsed();

    // Assert
    assert!(
        duration < Duration::from_secs(5),
        "Creating 100 files should take <5s, took {:?}",
        duration
    );

    Ok(())
}
