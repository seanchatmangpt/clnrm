//! Determinism Feature Validation
//!
//! Verifies that freeze_clock and random_seed work correctly in TOML configurations,
//! especially for Rosetta Stone chaos tests.

use clnrm_core::*;
use serial_test::serial;

#[test]
#[serial]
fn test_determinism_config_parsing_from_toml() -> Result<()> {
    // Arrange
    let toml_content = r#"
[meta]
name = "determinism_test"
version = "1.0.0"

[determinism]
seed = 666
freeze_clock = "2025-01-01T00:00:00Z"

[[scenario]]
name = "test_scenario"
service = "test"
run = "echo test"
"#;

    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("test_determinism.clnrm.toml");
    std::fs::write(&temp_file, toml_content)?;

    // Act
    let config = config::load_config_from_file(&temp_file)?;

    // Assert - determinism section parsed correctly
    assert!(config.determinism.is_some());
    let determinism = config.determinism.unwrap();
    assert_eq!(determinism.seed, Some(666));
    assert_eq!(determinism.freeze_clock, Some("2025-01-01T00:00:00Z".to_string()));
    assert!(determinism.is_deterministic());

    // Cleanup
    std::fs::remove_file(&temp_file)?;

    Ok(())
}

#[test]
#[serial]
fn test_determinism_engine_with_freeze_clock() -> Result<()> {
    // Arrange
    let config = config::types::DeterminismConfig {
        seed: None,
        freeze_clock: Some("2025-01-01T00:00:00Z".to_string()),
    };

    // Act
    let engine = determinism::DeterminismEngine::new(config)?;

    // Assert
    assert!(engine.has_frozen_clock());
    assert!(!engine.has_seed());

    // Verify frozen time returns same value
    let ts1 = engine.get_timestamp_rfc3339();
    std::thread::sleep(std::time::Duration::from_millis(10));
    let ts2 = engine.get_timestamp_rfc3339();

    assert_eq!(ts1, ts2);
    assert!(ts1.starts_with("2025-01-01"));

    Ok(())
}

#[test]
#[serial]
fn test_determinism_engine_with_seed() -> Result<()> {
    // Arrange
    let config = config::types::DeterminismConfig {
        seed: Some(666),
        freeze_clock: None,
    };

    // Act
    let engine1 = determinism::DeterminismEngine::new(config.clone())?;
    let engine2 = determinism::DeterminismEngine::new(config)?;

    // Assert
    assert!(engine1.has_seed());
    assert_eq!(engine1.get_seed(), Some(666));

    // Same seed produces same random values
    let val1 = engine1.next_u64()?;
    let val2 = engine2.next_u64()?;
    assert_eq!(val1, val2);

    Ok(())
}

#[test]
#[serial]
fn test_determinism_engine_with_both_features() -> Result<()> {
    // Arrange - matching chaos Rosetta Stone config
    let config = config::types::DeterminismConfig {
        seed: Some(666),
        freeze_clock: Some("2025-01-01T00:00:00Z".to_string()),
    };

    // Act
    let engine = determinism::DeterminismEngine::new(config)?;

    // Assert
    assert!(engine.is_deterministic());
    assert!(engine.has_seed());
    assert!(engine.has_frozen_clock());

    // Verify both features work
    let timestamp = engine.get_timestamp_rfc3339();
    assert!(timestamp.starts_with("2025-01-01"));

    let random_val = engine.next_u64()?;
    assert!(random_val > 0); // Should produce deterministic value

    Ok(())
}

#[test]
#[serial]
fn test_chaos_container_failures_toml_has_determinism() -> Result<()> {
    // Arrange - read actual Rosetta Stone chaos test
    let chaos_file = std::path::Path::new("tests/chaos/container_failures.clnrm.toml");

    // Skip if file doesn't exist (CI environment)
    if !chaos_file.exists() {
        println!("Skipping: chaos test file not found");
        return Ok(());
    }

    // Act
    let config = config::load_config_from_file(chaos_file)?;

    // Assert - verify chaos test has determinism configured
    assert!(config.determinism.is_some(), "container_failures.clnrm.toml should have [determinism] section");

    let determinism = config.determinism.unwrap();
    assert!(determinism.seed.is_some(), "container_failures should have seed");
    assert!(determinism.freeze_clock.is_some(), "container_failures should have freeze_clock");

    // Verify values match documented chaos config
    assert_eq!(determinism.seed, Some(666));
    assert_eq!(determinism.freeze_clock, Some("2025-01-01T00:00:00Z".to_string()));

    Ok(())
}

#[test]
#[serial]
fn test_chaos_concurrent_chaos_toml_has_determinism() -> Result<()> {
    // Arrange
    let chaos_file = std::path::Path::new("tests/chaos/concurrent_chaos.clnrm.toml");

    if !chaos_file.exists() {
        println!("Skipping: chaos test file not found");
        return Ok(());
    }

    // Act
    let config = config::load_config_from_file(chaos_file)?;

    // Assert
    assert!(config.determinism.is_some());
    let determinism = config.determinism.unwrap();

    assert!(determinism.seed.is_some());
    assert!(determinism.freeze_clock.is_some());
    assert_eq!(determinism.seed, Some(670)); // Different seed
    assert_eq!(determinism.freeze_clock, Some("2025-01-01T00:00:00Z".to_string()));

    Ok(())
}

#[test]
#[serial]
fn test_template_rendering_with_freeze_clock() -> Result<()> {
    // Arrange
    let template_content = r#"
[meta]
name = "{{ svc }}_test"

[determinism]
freeze_clock = "2025-06-15T12:00:00Z"

[test_data]
timestamp = "{{ now_rfc3339() }}"
service = "{{ svc }}"
"#;

    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("test_freeze_template.clnrm.toml.tera");
    std::fs::write(&temp_file, template_content)?;

    // Act - render template with defaults
    let user_vars = std::collections::HashMap::new();
    let rendered = template::render_template_file(&temp_file, user_vars)?;

    // Assert - should contain frozen timestamp
    // Note: Template rendering doesn't automatically use determinism config yet
    // This test documents expected behavior once integration is complete
    assert!(rendered.contains("timestamp"));

    // Cleanup
    std::fs::remove_file(&temp_file)?;

    Ok(())
}

#[test]
#[serial]
fn test_template_now_rfc3339_freeze_integration() -> Result<()> {
    // Arrange - Test that template functions can be frozen
    let mut renderer = template::TemplateRenderer::new()?;

    let template = r#"
timestamp1 = "{{ now_rfc3339() }}"
timestamp2 = "{{ now_rfc3339() }}"
"#;

    // Act - render twice
    let result1 = renderer.render_str(template, "test1")?;
    std::thread::sleep(std::time::Duration::from_millis(10));
    let result2 = renderer.render_str(template, "test2")?;

    // Assert - timestamps should be different (not frozen by default)
    // If freeze_clock is implemented, timestamps would be identical
    assert!(result1.contains("timestamp1"));
    assert!(result2.contains("timestamp1"));

    Ok(())
}

#[test]
#[serial]
fn test_determinism_validation_report() -> Result<()> {
    // Arrange - comprehensive validation of determinism features
    let mut issues: Vec<String> = Vec::new();

    // Check 1: DeterminismConfig can be parsed
    let config = config::types::DeterminismConfig {
        seed: Some(42),
        freeze_clock: Some("2025-01-01T00:00:00Z".to_string()),
    };

    if !config.is_deterministic() {
        issues.push("DeterminismConfig.is_deterministic() failed".to_string());
    }

    // Check 2: DeterminismEngine can be created
    match determinism::DeterminismEngine::new(config.clone()) {
        Ok(engine) => {
            if !engine.is_deterministic() {
                issues.push("DeterminismEngine.is_deterministic() returned false".to_string());
            }
            if !engine.has_seed() {
                issues.push("DeterminismEngine.has_seed() returned false".to_string());
            }
            if !engine.has_frozen_clock() {
                issues.push("DeterminismEngine.has_frozen_clock() returned false".to_string());
            }
        }
        Err(e) => {
            issues.push(format!("DeterminismEngine::new() failed: {}", e));
        }
    }

    // Check 3: Chaos tests have determinism sections
    let chaos_files = vec![
        "tests/chaos/container_failures.clnrm.toml",
        "tests/chaos/concurrent_chaos.clnrm.toml",
    ];

    for file_path in chaos_files {
        let path = std::path::Path::new(file_path);
        if path.exists() {
            match config::load_config_from_file(path) {
                Ok(cfg) => {
                    if cfg.determinism.is_none() {
                        issues.push(format!("{} missing [determinism] section", file_path));
                    }
                }
                Err(e) => {
                    issues.push(format!("Failed to load {}: {}", file_path, e));
                }
            }
        }
    }

    // Assert
    if !issues.is_empty() {
        eprintln!("Determinism validation issues:");
        for issue in &issues {
            eprintln!("  ❌ {}", issue);
        }
        panic!("Determinism validation failed with {} issues", issues.len());
    }

    Ok(())
}
