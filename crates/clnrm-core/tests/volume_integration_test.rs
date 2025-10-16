//! Integration tests for volume mounting support

use clnrm_core::backend::testcontainer::TestcontainerBackend;
use clnrm_core::backend::{Backend, Cmd};
use clnrm_core::backend::volume::{VolumeMount, VolumeValidator};
use clnrm_core::config::VolumeConfig;
use clnrm_core::error::Result;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_volume_mount_end_to_end() -> Result<()> {
    // Arrange - Create temp directory with test file
    let temp_dir = std::env::temp_dir();
    let host_dir = temp_dir.join("clnrm_volume_test");
    fs::create_dir_all(&host_dir)?;

    let test_file = host_dir.join("test.txt");
    fs::write(&test_file, "Hello from host!")?;

    // Create backend with volume mount
    let backend = TestcontainerBackend::new("alpine:latest")?
        .with_volume(host_dir.to_str().unwrap(), "/data", false)?;

    // Act - Execute command that reads from mounted volume
    let cmd = Cmd::new("cat").arg("/data/test.txt");
    let result = backend.run_cmd(cmd)?;

    // Assert
    assert!(result.success());
    assert_eq!(result.stdout.trim(), "Hello from host!");

    // Cleanup
    fs::remove_file(&test_file)?;
    fs::remove_dir(&host_dir)?;
    Ok(())
}

#[test]
fn test_volume_mount_read_only() -> Result<()> {
    // Arrange - Create temp directory
    let temp_dir = std::env::temp_dir();
    let host_dir = temp_dir.join("clnrm_volume_ro_test");
    fs::create_dir_all(&host_dir)?;

    // Create backend with read-only volume mount
    let backend = TestcontainerBackend::new("alpine:latest")?
        .with_volume_ro(host_dir.to_str().unwrap(), "/data")?;

    // Act - Try to write to read-only mount (should fail)
    let cmd = Cmd::new("sh").args(&["-c", "echo test > /data/readonly.txt"]);
    let result = backend.run_cmd(cmd)?;

    // Assert - Command should fail because volume is read-only
    assert!(!result.success());
    assert!(result.stderr.contains("Read-only") || result.stderr.contains("read-only"));

    // Cleanup
    fs::remove_dir(&host_dir)?;
    Ok(())
}

#[test]
fn test_volume_config_validation() -> Result<()> {
    // Test valid config
    let valid_config = VolumeConfig {
        host_path: "/tmp/valid".to_string(),
        container_path: "/data".to_string(),
        read_only: Some(true),
    };
    assert!(valid_config.validate().is_ok());

    // Test invalid config - empty host path
    let invalid_config = VolumeConfig {
        host_path: "".to_string(),
        container_path: "/data".to_string(),
        read_only: None,
    };
    assert!(invalid_config.validate().is_err());

    // Test invalid config - relative host path
    let relative_config = VolumeConfig {
        host_path: "relative/path".to_string(),
        container_path: "/data".to_string(),
        read_only: None,
    };
    assert!(relative_config.validate().is_err());

    // Test invalid config - relative container path
    let relative_container_config = VolumeConfig {
        host_path: "/tmp/data".to_string(),
        container_path: "relative".to_string(),
        read_only: None,
    };
    assert!(relative_container_config.validate().is_err());

    Ok(())
}

#[test]
fn test_volume_validator_whitelist() -> Result<()> {
    // Arrange
    let temp_dir = std::env::temp_dir();
    let allowed_dir = temp_dir.join("allowed");
    let forbidden_dir = temp_dir.join("forbidden");
    fs::create_dir_all(&allowed_dir)?;
    fs::create_dir_all(&forbidden_dir)?;

    let validator = VolumeValidator::new(vec![allowed_dir.clone()]);

    // Test allowed path
    let allowed_mount = VolumeMount::new(&allowed_dir, "/data", false)?;
    assert!(validator.validate(&allowed_mount).is_ok());

    // Test forbidden path
    let forbidden_mount = VolumeMount::new(&forbidden_dir, "/data", false)?;
    assert!(validator.validate(&forbidden_mount).is_err());

    // Cleanup
    fs::remove_dir(&allowed_dir)?;
    fs::remove_dir(&forbidden_dir)?;
    Ok(())
}

#[test]
fn test_volume_mount_from_config() -> Result<()> {
    // Arrange - Create temp directory
    let temp_dir = std::env::temp_dir();
    let host_dir = temp_dir.join("clnrm_config_test");
    fs::create_dir_all(&host_dir)?;

    let config = VolumeConfig {
        host_path: host_dir.to_string_lossy().to_string(),
        container_path: "/config_data".to_string(),
        read_only: Some(true),
    };

    // Act - Create VolumeMount from config
    let mount = config.to_volume_mount()?;

    // Assert
    assert_eq!(mount.container_path(), PathBuf::from("/config_data"));
    assert!(mount.is_read_only());

    // Cleanup
    fs::remove_dir(&host_dir)?;
    Ok(())
}

#[test]
fn test_multiple_volume_mounts() -> Result<()> {
    // Arrange - Create multiple temp directories
    let temp_dir = std::env::temp_dir();
    let data_dir = temp_dir.join("clnrm_data");
    let config_dir = temp_dir.join("clnrm_config");
    fs::create_dir_all(&data_dir)?;
    fs::create_dir_all(&config_dir)?;

    fs::write(data_dir.join("data.txt"), "data content")?;
    fs::write(config_dir.join("config.txt"), "config content")?;

    // Create backend with multiple volumes
    let backend = TestcontainerBackend::new("alpine:latest")?
        .with_volume(data_dir.to_str().unwrap(), "/data", false)?
        .with_volume(config_dir.to_str().unwrap(), "/config", false)?;

    // Act - Verify both mounts are accessible
    let cmd1 = Cmd::new("cat").arg("/data/data.txt");
    let result1 = backend.run_cmd(cmd1)?;

    let cmd2 = Cmd::new("cat").arg("/config/config.txt");
    let result2 = backend.run_cmd(cmd2)?;

    // Assert
    assert!(result1.success());
    assert_eq!(result1.stdout.trim(), "data content");
    assert!(result2.success());
    assert_eq!(result2.stdout.trim(), "config content");

    // Cleanup
    fs::remove_file(data_dir.join("data.txt"))?;
    fs::remove_file(config_dir.join("config.txt"))?;
    fs::remove_dir(&data_dir)?;
    fs::remove_dir(&config_dir)?;
    Ok(())
}
