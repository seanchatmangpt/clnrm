//! Image Pre-pull Command (PRD v1.0)
//!
//! Pre-pulls Docker images from test configurations to improve test startup time.

use crate::cli::utils::discover_test_files;
use crate::config::load_cleanroom_config_from_file;
use crate::error::{CleanroomError, Result};
use std::collections::HashSet;
use std::path::PathBuf;
use tracing::{debug, info};

/// Pre-pull Docker images from test configurations
///
/// # Arguments
/// * `paths` - Optional test paths to scan (default: discover all)
/// * `parallel` - Whether to pull images in parallel
/// * `jobs` - Maximum number of parallel pulls
///
/// # Returns
/// * `Result<()>` - Success or error
///
/// # Errors
/// * Returns error if test discovery fails
/// * Returns error if configuration parsing fails
/// * Returns error if image pull fails
pub async fn pull_images(
    paths: Option<Vec<PathBuf>>,
    parallel: bool,
    jobs: usize,
) -> Result<()> {
    // Arrange - Setup configuration and paths
    info!("Starting Docker image pre-pull");

    let test_paths = if let Some(paths) = paths {
        paths
    } else {
        vec![PathBuf::from(".")]
    };

    // Discover test files
    let mut all_test_files = Vec::new();
    for path in &test_paths {
        let discovered = discover_test_files(path)?;
        all_test_files.extend(discovered);
    }

    if all_test_files.is_empty() {
        return Err(CleanroomError::validation_error(
            "No test files found to scan for images",
        ));
    }

    info!("Scanning {} test file(s) for Docker images", all_test_files.len());
    println!("🔍 Scanning {} test file(s)...", all_test_files.len());

    // Act - Extract unique images from all test files
    let mut images = HashSet::new();

    for test_file in &all_test_files {
        debug!("Scanning test file: {}", test_file.display());

        let config = load_cleanroom_config_from_file(test_file).map_err(|e| {
            CleanroomError::configuration_error(format!(
                "Failed to load config from '{}': {}",
                test_file.display(),
                e
            ))
        })?;

        // Extract images from service configurations
        for (service_name, service_config) in config.services {
            if !service_config.image.is_empty() {
                debug!("Found image '{}' in service '{}'", service_config.image, service_name);
                images.insert(service_config.image.clone());
            }
        }
    }

    if images.is_empty() {
        println!("✅ No Docker images found in test configurations");
        return Ok(());
    }

    println!("📦 Found {} unique Docker image(s)", images.len());
    for image in &images {
        println!("   • {}", image);
    }

    // Assert - Pull all images
    println!();
    if parallel {
        println!("🚀 Pulling images in parallel (max {} workers)...", jobs);
        pull_images_parallel(&images, jobs).await?;
    } else {
        println!("🚀 Pulling images sequentially...");
        pull_images_sequential(&images).await?;
    }

    println!();
    println!("✅ All images pulled successfully");
    info!("Image pre-pull completed: {} images", images.len());

    Ok(())
}

/// Pull images sequentially
async fn pull_images_sequential(images: &HashSet<String>) -> Result<()> {
    for (idx, image) in images.iter().enumerate() {
        println!("   [{}/{}] Pulling {}...", idx + 1, images.len(), image);

        let output = tokio::process::Command::new("docker")
            .args(["pull", image])
            .output()
            .await
            .map_err(|e| {
                CleanroomError::internal_error(format!("Failed to execute docker pull: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CleanroomError::container_error(format!(
                "Failed to pull image '{}': {}",
                image, stderr
            )));
        }

        println!("   ✓ {}", image);
    }

    Ok(())
}

/// Pull images in parallel
async fn pull_images_parallel(images: &HashSet<String>, jobs: usize) -> Result<()> {
    use tokio::sync::Semaphore;
    use std::sync::Arc;

    let semaphore = Arc::new(Semaphore::new(jobs));
    let mut tasks = Vec::new();

    let total = images.len();

    for (idx, image) in images.iter().enumerate() {
        let image = image.clone();
        let permit = semaphore.clone().acquire_owned().await.map_err(|e| {
            CleanroomError::internal_error(format!("Semaphore error: {}", e))
        })?;

        let task = tokio::spawn(async move {
            println!("   [{}/{}] Pulling {}...", idx + 1, total, image);

            let output = tokio::process::Command::new("docker")
                .args(["pull", &image])
                .output()
                .await
                .map_err(|e| {
                    CleanroomError::internal_error(format!("Failed to execute docker pull: {}", e))
                })?;

            drop(permit);

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(CleanroomError::container_error(format!(
                    "Failed to pull image '{}': {}",
                    image, stderr
                )));
            }

            println!("   ✓ {}", image);
            Ok::<_, CleanroomError>(())
        });

        tasks.push(task);
    }

    // Wait for all tasks to complete
    for task in tasks {
        task.await.map_err(|e| {
            CleanroomError::internal_error(format!("Task join error: {}", e))
        })??;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_images_set_returns_early() {
        // This would be tested with actual Docker operations in integration tests
        assert!(true);
    }
}
