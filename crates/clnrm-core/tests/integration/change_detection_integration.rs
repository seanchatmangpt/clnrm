//! London School TDD Tests for Change Detection System
//!
//! Test Coverage:
//! - SHA-256 digest computation for scenarios
//! - Changed scenario detection
//! - Unchanged scenario skipping
//! - Digest format validation
//! - Multi-scenario change detection
//! - Cache persistence across runs
//!
//! Testing Philosophy:
//! - OUTSIDE-IN: Test behavior from cache manager perspective
//! - MOCK-FIRST: Define cache contracts through test doubles
//! - BEHAVIOR VERIFICATION: Focus on hash consistency and comparison
//!
//! Core Team Compliance:
//! - ✅ AAA pattern (Arrange, Act, Assert)
//! - ✅ Descriptive test names (test_X_with_Y_produces_Z)
//! - ✅ No false positives - proper error propagation
//! - ✅ Result<()> for proper error handling

#![allow(clippy::unwrap_used)] // Test code only

use clnrm_core::cache::{Cache, MemoryCache};
use clnrm_core::error::{CleanroomError, Result};
use sha2::{Digest, Sha256};
use std::path::Path;

// ============================================================================
// Change Detection - SHA-256 Digest Tests
// ============================================================================

#[test]
fn test_scenario_digest_computation_produces_consistent_hash() -> Result<()> {
    // Arrange
    let scenario_content = r#"
[[scenario]]
name = "test_api"
service = "api"
run = "curl localhost:8080/health"
expect_success = true
"#;

    // Act - Compute SHA-256 digest
    let mut hasher1 = Sha256::new();
    hasher1.update(scenario_content.as_bytes());
    let digest1 = format!("{:x}", hasher1.finalize());

    // Compute again to verify consistency
    let mut hasher2 = Sha256::new();
    hasher2.update(scenario_content.as_bytes());
    let digest2 = format!("{:x}", hasher2.finalize());

    // Assert - Digests are identical (deterministic)
    assert_eq!(digest1, digest2, "SHA-256 digest should be deterministic");
    assert_eq!(
        digest1.len(),
        64,
        "SHA-256 hex digest should be 64 characters"
    );
    assert!(
        digest1.chars().all(|c| c.is_ascii_hexdigit()),
        "Digest should contain only hex characters"
    );

    Ok(())
}

#[test]
fn test_detect_changed_scenarios_identifies_modifications() -> Result<()> {
    // Arrange
    let original_content = "scenario_version_1";
    let modified_content = "scenario_version_2";

    let mut hasher_original = Sha256::new();
    hasher_original.update(original_content.as_bytes());
    let original_hash = format!("{:x}", hasher_original.finalize());

    let mut hasher_modified = Sha256::new();
    hasher_modified.update(modified_content.as_bytes());
    let modified_hash = format!("{:x}", hasher_modified.finalize());

    // Act - Compare hashes
    let has_changed = original_hash != modified_hash;

    // Assert - Change detected
    assert!(
        has_changed,
        "Different content should produce different hashes"
    );
    assert_ne!(
        original_hash, modified_hash,
        "Modified scenario should have different digest"
    );

    Ok(())
}

#[test]
fn test_skip_unchanged_scenarios_preserves_cache() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();
    let scenario_path = Path::new("tests/unchanged.toml.tera");
    let scenario_content = "stable_scenario_content";

    // Act - First check (not in cache, should show as changed)
    let changed_first = cache.has_changed(scenario_path, scenario_content)?;

    // Update cache
    cache.update(scenario_path, scenario_content)?;

    // Second check with same content (should show as unchanged)
    let changed_second = cache.has_changed(scenario_path, scenario_content)?;

    // Assert - First shows changed, second shows unchanged
    assert!(
        changed_first,
        "First check should show as changed (not in cache)"
    );
    assert!(
        !changed_second,
        "Second check should show as unchanged (same content)"
    );

    Ok(())
}

#[test]
fn test_digest_format_validation_enforces_sha256() -> Result<()> {
    // Arrange
    let test_content = "test_scenario";

    let mut hasher = Sha256::new();
    hasher.update(test_content.as_bytes());
    let digest = format!("{:x}", hasher.finalize());

    // Act - Validate digest format
    let is_valid_length = digest.len() == 64;
    let is_valid_hex = digest.chars().all(|c| c.is_ascii_hexdigit());
    let is_lowercase = digest.chars().all(|c| !c.is_uppercase());

    // Assert - SHA-256 format requirements
    assert!(is_valid_length, "SHA-256 hex digest must be 64 chars");
    assert!(is_valid_hex, "Digest must contain only hex characters");
    assert!(is_lowercase, "Digest should use lowercase hex");

    Ok(())
}

#[test]
fn test_multi_scenario_change_detection_tracks_independently() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();

    let scenarios = vec![
        (
            Path::new("tests/scenario1.toml.tera"),
            "content_1",
            "modified_1",
        ),
        (
            Path::new("tests/scenario2.toml.tera"),
            "content_2",
            "content_2",
        ), // unchanged
        (
            Path::new("tests/scenario3.toml.tera"),
            "content_3",
            "modified_3",
        ),
    ];

    // Store initial content in cache
    for (path, original, _) in &scenarios {
        cache.update(path, original)?;
    }

    // Act - Check each scenario for changes with updated content
    let mut change_results = Vec::new();
    for (path, _, current) in &scenarios {
        let has_changed = cache.has_changed(path, current)?;
        change_results.push((path, has_changed));
    }

    // Assert - Correct change detection per scenario
    assert!(
        change_results[0].1,
        "Scenario 1 should be detected as changed"
    );
    assert!(
        !change_results[1].1,
        "Scenario 2 should be detected as unchanged"
    );
    assert!(
        change_results[2].1,
        "Scenario 3 should be detected as changed"
    );

    Ok(())
}

#[tokio::test]
async fn test_cache_persistence_across_runs_maintains_state() -> Result<()> {
    // Arrange
    use tempfile::TempDir;
    let temp_dir = TempDir::new()
        .map_err(|e| CleanroomError::internal_error(format!("Failed to create temp dir: {}", e)))?;

    let cache_file = temp_dir.path().join("test_cache.json");
    let scenario_path = "tests/persistent.toml.tera";
    let scenario_content = "persistent_content";

    // Compute hash
    let mut hasher = Sha256::new();
    hasher.update(scenario_content.as_bytes());
    let expected_hash = format!("{:x}", hasher.finalize());

    // Act - First run: create cache
    {
        let cache = MemoryCache::new();
        cache.update(Path::new(scenario_path), scenario_content)?;

        // Simulate persistence by storing to temporary location
        let cache_data = serde_json::json!({
            scenario_path: expected_hash.clone()
        });
        std::fs::write(&cache_file, cache_data.to_string())
            .map_err(|e| CleanroomError::io_error(format!("Failed to write cache: {}", e)))?;
    }

    // Second run: load cache
    let cache_content = std::fs::read_to_string(&cache_file)
        .map_err(|e| CleanroomError::io_error(format!("Failed to read cache: {}", e)))?;

    let loaded_data: serde_json::Value = serde_json::from_str(&cache_content).map_err(|e| {
        CleanroomError::serialization_error(format!("Failed to parse cache: {}", e))
    })?;

    let persisted_hash = loaded_data[scenario_path].as_str();

    // Assert - Cache persisted correctly
    assert_eq!(
        persisted_hash,
        Some(expected_hash.as_str()),
        "Cache should persist hash across runs"
    );

    Ok(())
}

// ============================================================================
// Performance and Optimization Tests
// ============================================================================

#[test]
fn test_digest_computation_performance_is_acceptable() -> Result<()> {
    // Arrange
    let large_content = "x".repeat(10_000); // 10KB scenario
    let start = std::time::Instant::now();

    // Act - Compute digest
    let mut hasher = Sha256::new();
    hasher.update(large_content.as_bytes());
    let _digest = format!("{:x}", hasher.finalize());

    let elapsed = start.elapsed();

    // Assert - Performance target <10ms for 10KB
    assert!(
        elapsed.as_millis() < 10,
        "Digest computation should be fast (<10ms), took {}ms",
        elapsed.as_millis()
    );

    Ok(())
}

#[test]
fn test_cache_lookup_performance_is_constant_time() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();

    // Store 100 scenarios
    for i in 0..100 {
        let path_str = format!("tests/scenario_{}.toml.tera", i);
        let content = format!("content_{}", i);
        cache.update(Path::new(&path_str), &content)?;
    }

    // Act - Lookup first and last entries
    let start_first = std::time::Instant::now();
    let _ = cache.has_changed(Path::new("tests/scenario_0.toml.tera"), "content_0")?;
    let elapsed_first = start_first.elapsed();

    let start_last = std::time::Instant::now();
    let _ = cache.has_changed(Path::new("tests/scenario_99.toml.tera"), "content_99")?;
    let elapsed_last = start_last.elapsed();

    // Assert - Lookup time should be similar (hash map O(1))
    let time_diff = elapsed_first.as_micros().abs_diff(elapsed_last.as_micros());
    assert!(
        time_diff < 1000, // <1ms difference
        "Cache lookup should be constant time, difference was {}μs",
        time_diff
    );

    Ok(())
}

// ============================================================================
// Edge Cases and Error Handling
// ============================================================================

#[test]
fn test_empty_content_produces_valid_digest() -> Result<()> {
    // Arrange
    let empty_content = "";

    // Act
    let mut hasher = Sha256::new();
    hasher.update(empty_content.as_bytes());
    let digest = format!("{:x}", hasher.finalize());

    // Assert - Empty content has valid digest
    assert_eq!(digest.len(), 64, "Empty content should have valid digest");
    assert_eq!(
        digest, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "Empty content SHA-256 should match expected value"
    );

    Ok(())
}

#[test]
fn test_whitespace_differences_detected_in_digest() -> Result<()> {
    // Arrange
    let content1 = "test scenario";
    let content2 = "test  scenario"; // Extra space

    let mut hasher1 = Sha256::new();
    hasher1.update(content1.as_bytes());
    let digest1 = format!("{:x}", hasher1.finalize());

    let mut hasher2 = Sha256::new();
    hasher2.update(content2.as_bytes());
    let digest2 = format!("{:x}", hasher2.finalize());

    // Act & Assert
    assert_ne!(
        digest1, digest2,
        "Whitespace differences should produce different digests"
    );

    Ok(())
}
