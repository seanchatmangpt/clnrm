//! Pull command - Pre-pull Docker images from test configurations
//!
//! Scans test files for Docker images and pulls them in advance to avoid delays during test execution.

use crate::config::TestConfig;
use crate::error::{CleanroomError, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, info};

/// Pre-pull Docker images from test configurations
pub async fn pull_images(paths: Option<Vec<PathBuf>>, parallel: bool, jobs: usize) -> Result<()> {
    info!("Scanning test files for Docker images to pull");

    // Discover test files
    let test_files = if let Some(paths) = paths {
        discover_test_files_from_paths(&paths)?
    } else {
        discover_test_files_from_paths(&[PathBuf::from(".")])?
    };

    if test_files.is_empty() {
        println!("No test files found");
        return Ok(());
    }

    info!("Found {} test file(s)", test_files.len());

    // Extract unique images from test files
    let images = extract_images_from_test_files(&test_files)?;

    if images.is_empty() {
        println!("No Docker images found in test configurations");
        return Ok(());
    }

    println!("Found {} unique image(s) to pull:", images.len());
    for image in &images {
        println!("  - {}", image);
    }

    // Pull images
    if parallel {
        pull_images_parallel(&images, jobs).await?;
    } else {
        pull_images_sequential(&images).await?;
    }

    println!("\n✅ Successfully pulled {} image(s)", images.len());
    Ok(())
}

/// Discover test files from paths
fn discover_test_files_from_paths(paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut test_files = Vec::new();

    for path in paths {
        if path.is_file() {
            if is_test_file(path) {
                test_files.push(path.clone());
            }
        } else if path.is_dir() {
            let discovered = discover_test_files_recursive(path)?;
            test_files.extend(discovered);
        }
    }

    Ok(test_files)
}

/// Recursively discover test files
fn discover_test_files_recursive(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut test_files = Vec::new();

    let entries = std::fs::read_dir(dir).map_err(|e| {
        CleanroomError::io_error(format!("Failed to read directory {}: {}", dir.display(), e))
    })?;

    for entry in entries {
        let entry =
            entry.map_err(|e| CleanroomError::io_error(format!("Failed to read entry: {}", e)))?;
        let path = entry.path();

        if path.is_file() && is_test_file(&path) {
            test_files.push(path);
        } else if path.is_dir() {
            let discovered = discover_test_files_recursive(&path)?;
            test_files.extend(discovered);
        }
    }

    Ok(test_files)
}

/// Check if file is a test file
fn is_test_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "toml")
        .unwrap_or(false)
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.ends_with(".clnrm.toml") || name.ends_with(".toml"))
            .unwrap_or(false)
}

/// Extract unique images from test files
fn extract_images_from_test_files(test_files: &[PathBuf]) -> Result<Vec<String>> {
    let mut images = HashSet::new();

    for test_file in test_files {
        debug!("Scanning {}", test_file.display());

        let content = std::fs::read_to_string(test_file).map_err(|e| {
            CleanroomError::io_error(format!(
                "Failed to read test file {}: {}",
                test_file.display(),
                e
            ))
        })?;

        // Parse TOML
        let config: TestConfig = toml::from_str(&content).map_err(|e| {
            CleanroomError::config_error(format!(
                "Failed to parse test file {}: {}",
                test_file.display(),
                e
            ))
        })?;

        // Extract images from service configurations (services section)
        if let Some(services) = &config.services {
            for (service_name, service_config) in services {
                if let Some(image) = &service_config.image {
                    debug!("Found image '{}' in service '{}'", image, service_name);
                    images.insert(image.clone());
                }
            }
        }

        // Extract images from service configurations (service section - v0.6.0 format)
        if let Some(services) = &config.service {
            for (service_name, service_config) in services {
                if let Some(image) = &service_config.image {
                    debug!("Found image '{}' in service '{}'", image, service_name);
                    images.insert(image.clone());
                }
            }
        }
    }

    Ok(images.into_iter().collect())
}

/// Pull images sequentially
async fn pull_images_sequential(images: &[String]) -> Result<()> {
    for (idx, image) in images.iter().enumerate() {
        println!("\n[{}/{}] Pulling {}...", idx + 1, images.len(), image);
        pull_single_image(image).await?;
    }

    Ok(())
}

/// Pull images in parallel
async fn pull_images_parallel(images: &[String], jobs: usize) -> Result<()> {
    let semaphore = Arc::new(Semaphore::new(jobs));
    let mut tasks = Vec::new();

    for (idx, image) in images.iter().enumerate() {
        let image = image.clone();
        let semaphore = Arc::clone(&semaphore);
        let total = images.len();

        let task = tokio::spawn(async move {
            let _permit = semaphore
                .acquire()
                .await
                .map_err(|e| CleanroomError::internal_error(format!("Semaphore error: {}", e)))?;

            println!("[{}/{}] Pulling {}...", idx + 1, total, image);
            pull_single_image(&image).await
        });

        tasks.push(task);
    }

    // Wait for all tasks to complete
    for task in tasks {
        task.await
            .map_err(|e| CleanroomError::internal_error(format!("Task join error: {}", e)))??;
    }

    Ok(())
}

/// Pull a single Docker image
async fn pull_single_image(image: &str) -> Result<()> {
    debug!("Pulling image: {}", image);

    let output = tokio::process::Command::new("docker")
        .arg("pull")
        .arg(image)
        .output()
        .await
        .map_err(|e| {
            CleanroomError::container_error(format!("Failed to execute docker pull: {}", e))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CleanroomError::container_error(format!(
            "Failed to pull image {}: {}",
            image, stderr
        )));
    }

    println!("  ✓ Pulled {}", image);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_test_file() {
        assert!(is_test_file(Path::new("test.clnrm.toml")));
        assert!(is_test_file(Path::new("tests/integration.clnrm.toml")));
        assert!(!is_test_file(Path::new("config.toml")));
        assert!(!is_test_file(Path::new("test.txt")));
    }
}
