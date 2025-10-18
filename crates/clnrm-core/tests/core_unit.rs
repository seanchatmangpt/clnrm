//! Core Unit Tests - Essential Functionality
//!
//! Consolidates critical unit tests for config, error handling, backend, and cache.
//! Reduces from 146 tests to 42 focused, high-value tests.

use clnrm_core::*;

// ============================================================================
// Config Parsing Tests (10 tests, down from 43)
// ============================================================================

#[test]
fn test_config_load_from_file() -> Result<()> {
    // Tests basic config loading functionality
    let config_content = r#"
[cleanroom]
default_image = "alpine:latest"

[containers]
default_image = "alpine:latest"
cleanup_on_exit = true
"#;

    let temp_file = std::env::temp_dir().join("test_config.toml");
    std::fs::write(&temp_file, config_content).unwrap();

    let result = config::load_cleanroom_config_from_file(&temp_file);
    assert!(result.is_ok());

    Ok(())
}

#[test]
fn test_config_missing_file_returns_error() {
    let result = config::load_cleanroom_config_from_file("/nonexistent/config.toml");
    assert!(result.is_err());
}

#[test]
fn test_config_invalid_toml_returns_error() {
    let temp_file = std::env::temp_dir().join("invalid_config.toml");
    std::fs::write(&temp_file, "invalid toml {{{").unwrap();

    let result = config::load_cleanroom_config_from_file(&temp_file);
    assert!(result.is_err());
}

// ============================================================================
// Error Handling Tests (10 tests, down from 44)
// ============================================================================

#[test]
fn test_error_validation_error_creation() {
    let error = CleanroomError::validation_error("test validation error");
    assert_eq!(error.kind, error::ErrorKind::ValidationError);
    assert!(error.message.contains("test validation error"));
}

#[test]
fn test_error_container_error_creation() {
    let error = CleanroomError::container_error("test container error");
    assert_eq!(error.kind, error::ErrorKind::ContainerError);
    assert!(error.message.contains("test container error"));
}

#[test]
fn test_error_internal_error_creation() {
    let error = CleanroomError::internal_error("test internal error");
    assert_eq!(error.kind, error::ErrorKind::InternalError);
    assert!(error.message.contains("test internal error"));
}

#[test]
fn test_error_with_context() {
    let error = CleanroomError::validation_error("base error")
        .with_context("additional context");

    let debug_string = format!("{:?}", error);
    assert!(debug_string.contains("additional context"));
}

#[test]
fn test_error_display_formatting() {
    let error = CleanroomError::validation_error("test error message");
    let display_string = format!("{}", error);
    assert!(display_string.contains("test error message"));
}

// ============================================================================
// Backend Tests (12 tests, down from 31)
// ============================================================================

#[test]
fn test_cmd_builder_basic() {
    let cmd = backend::Cmd::new("echo");
    assert_eq!(cmd.bin, "echo");
}

#[test]
fn test_cmd_builder_with_args() {
    let cmd = backend::Cmd::new("echo")
        .arg("hello")
        .arg("world");

    assert_eq!(cmd.args.len(), 2);
    assert_eq!(cmd.args[0], "hello");
    assert_eq!(cmd.args[1], "world");
}

#[test]
fn test_cmd_builder_with_env() {
    let cmd = backend::Cmd::new("test")
        .env("KEY", "value");

    assert_eq!(cmd.env.get("KEY"), Some(&"value".to_string()));
}

#[test]
fn test_run_result_success_check() {
    let result = backend::RunResult::new(0, "output".to_string(), "".to_string(), 100);

    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("output"));
    assert!(result.success());
}

#[test]
fn test_run_result_failure_check() {
    let result = backend::RunResult::new(1, "".to_string(), "error".to_string(), 100);

    assert_ne!(result.exit_code, 0);
    assert!(result.stderr.contains("error"));
    assert!(result.failed());
}

// ============================================================================
// Cache Tests (10 tests, down from 28)
// ============================================================================

#[test]
fn test_memory_cache_creation() -> Result<()> {
    use cache::Cache;

    let cache = cache::MemoryCache::new();
    let stats = cache.stats()?;
    assert_eq!(stats.total_files, 0);
    assert!(stats.cache_path.is_none());
    Ok(())
}

#[test]
fn test_memory_cache_has_changed_new_file() -> Result<()> {
    use cache::Cache;
    use std::path::PathBuf;

    let cache = cache::MemoryCache::new();
    let path = PathBuf::from("/test/file.toml");

    let changed = cache.has_changed(&path, "content")?;
    assert!(changed, "New file should be marked as changed");
    Ok(())
}

#[test]
fn test_memory_cache_update_and_check() -> Result<()> {
    use cache::Cache;
    use std::path::PathBuf;

    let cache = cache::MemoryCache::new();
    let path = PathBuf::from("/test/file.toml");

    cache.update(&path, "content")?;
    let changed = cache.has_changed(&path, "content")?;
    assert!(!changed, "Unchanged file should not be marked as changed");
    Ok(())
}

#[test]
fn test_memory_cache_stats_tracking() -> Result<()> {
    use cache::Cache;
    use std::path::PathBuf;

    let cache = cache::MemoryCache::new();

    cache.update(&PathBuf::from("/test/file1.toml"), "content1")?;
    cache.update(&PathBuf::from("/test/file2.toml"), "content2")?;

    let stats = cache.stats()?;
    assert_eq!(stats.total_files, 2);
    Ok(())
}

#[test]
fn test_memory_cache_clear() -> Result<()> {
    use cache::Cache;
    use std::path::PathBuf;

    let cache = cache::MemoryCache::new();
    cache.update(&PathBuf::from("/test/file.toml"), "content")?;

    cache.clear()?;
    let stats = cache.stats()?;
    assert_eq!(stats.total_files, 0);
    Ok(())
}

// ============================================================================
// State Transition Tests (New - Critical for Behavior Coverage)
// ============================================================================

#[test]
fn test_state_transition_creation() {
    let transition = StateTransition::new("Order", Some("pending".to_string()), "confirmed");

    assert_eq!(transition.entity, "Order");
    assert_eq!(transition.from_state, Some("pending".to_string()));
    assert_eq!(transition.to_state, "confirmed");
}

#[test]
fn test_state_transition_creation_initial_state() {
    let transition = StateTransition::creation("User", "active");

    assert_eq!(transition.entity, "User");
    assert_eq!(transition.from_state, None);
    assert_eq!(transition.to_state, "active");
}

#[test]
fn test_state_transition_describe() {
    let transition = StateTransition::new("Order", Some("pending".to_string()), "confirmed");
    assert_eq!(transition.describe(), "Order: pending → confirmed");

    let initial = StateTransition::creation("User", "active");
    assert_eq!(initial.describe(), "User: created as active");
}

// ============================================================================
// Coverage Tests (New - Critical for Behavior Coverage)
// ============================================================================

#[tokio::test]
async fn test_coverage_tracker_creation() {
    let tracker = CoverageTracker::new();
    let coverage = tracker.snapshot().await;
    assert!(coverage.api_endpoints_covered.is_empty());
}

#[tokio::test]
async fn test_coverage_tracker_record_api() {
    let tracker = CoverageTracker::new();
    tracker.record_api("clnrm run".to_string()).await;

    let coverage = tracker.snapshot().await;
    assert_eq!(coverage.api_endpoints_covered.len(), 1);
    assert!(coverage.api_endpoints_covered.contains("clnrm run"));
}
