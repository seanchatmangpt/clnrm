//! Cache management for test execution

use crate::cache::{Cache, CacheManager};
use crate::cli::types::CliTestResult;
use crate::error::{CleanroomError, Result};
use std::path::PathBuf;

/// Filter tests that have changed since last cache update
///
/// Returns only test files whose raw content has changed.
/// Note: We use raw content for caching, not rendered templates, because
/// template rendering requires vars from the parsed TOML (chicken-and-egg problem).
pub async fn filter_changed_tests(
    test_files: &[PathBuf],
    cache_manager: &CacheManager,
) -> Result<Vec<PathBuf>> {
    let mut changed_tests = Vec::new();

    for test_file in test_files {
        // Read raw file content (don't render templates)
        let content = std::fs::read_to_string(test_file).map_err(|e| {
            CleanroomError::io_error(format!(
                "Failed to read test file '{}': {}",
                test_file.display(),
                e
            ))
        })?;

        // Check if file has changed based on raw content
        if cache_manager.has_changed(test_file, &content)? {
            changed_tests.push(test_file.clone());
        }
    }

    Ok(changed_tests)
}

/// Update cache for test results
///
/// Updates cache hashes for successfully executed tests using raw content.
pub async fn update_cache_for_results(
    results: &[CliTestResult],
    cache_manager: &CacheManager,
) -> Result<()> {
    for result in results {
        // Only update cache for passed tests
        if result.passed {
            // Reconstruct the file path from test name
            // This assumes test names match file names (which they should)
            let test_path = PathBuf::from(&result.name);

            // Check if file exists and update cache
            if test_path.exists() {
                let content = std::fs::read_to_string(&test_path).map_err(|e| {
                    CleanroomError::io_error(format!(
                        "Failed to read test file '{}': {}",
                        test_path.display(),
                        e
                    ))
                })?;

                // Update cache with raw content
                cache_manager.update(&test_path, &content)?;
            }
        }
    }

    Ok(())
}
