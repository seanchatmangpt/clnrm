//! Cache management for test execution

use crate::cache::{Cache, CacheManager};
use crate::cli::types::CliTestResult;
use crate::error::{CleanroomError, Result};
use crate::template::TemplateRenderer;
use std::path::PathBuf;

/// Filter tests that have changed since last cache update
///
/// Returns only test files whose rendered content has changed.
pub async fn filter_changed_tests(
    test_files: &[PathBuf],
    cache_manager: &CacheManager,
) -> Result<Vec<PathBuf>> {
    let mut renderer = TemplateRenderer::new()?;
    let mut changed_tests = Vec::new();

    for test_file in test_files {
        // Read and render the test file
        let content = std::fs::read_to_string(test_file).map_err(|e| {
            CleanroomError::io_error(format!(
                "Failed to read test file '{}': {}",
                test_file.display(),
                e
            ))
        })?;

        // Render template to get final content for hashing
        let template_name = test_file.to_str().unwrap_or("unknown");
        let rendered_content = renderer.render_str(&content, template_name)?;

        // Check if file has changed
        if cache_manager.has_changed(test_file, &rendered_content)? {
            changed_tests.push(test_file.clone());
        }
    }

    Ok(changed_tests)
}

/// Update cache for test results
///
/// Updates cache hashes for successfully executed tests.
pub async fn update_cache_for_results(
    results: &[CliTestResult],
    cache_manager: &CacheManager,
) -> Result<()> {
    let mut renderer = TemplateRenderer::new()?;

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

                let template_name = test_path.to_str().unwrap_or("unknown");
                let rendered_content = renderer.render_str(&content, template_name)?;
                cache_manager.update(&test_path, &rendered_content)?;
            }
        }
    }

    Ok(())
}
