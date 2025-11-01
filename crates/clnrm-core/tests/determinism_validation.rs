//! Determinism validation tests
//!
//! These tests verify that deterministic execution features work correctly
//! by running tests multiple times and checking for consistency.

use clnrm_core::backend::{Backend, Cmd, TestcontainerBackend};
use clnrm_core::config::DeterminismConfig;
use clnrm_core::determinism::DeterminismEngine;
use clnrm_core::error::Result;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

/// Run a command multiple times and collect outputs
fn run_multiple_times(
    backend: &TestcontainerBackend,
    cmd: &Cmd,
    iterations: usize,
) -> Result<Vec<String>> {
    let mut outputs = Vec::new();

    for _ in 0..iterations {
        let result = backend.run_cmd(cmd.clone())?;
        outputs.push(result.stdout.clone());
    }

    Ok(outputs)
}

#[test]
fn test_deterministic_random_seed() -> Result<()> {
    // Create determinism config with seed
    let config = DeterminismConfig {
        seed: Some(42),
        freeze_clock: None,
        deterministic_ports: None,
        deterministic_volumes: None,
    };

    let engine = Arc::new(DeterminismEngine::new(config)?);
    let backend = TestcontainerBackend::new("alpine:latest")?.with_determinism(engine.clone());

    // Command to read RANDOM env var
    let cmd = Cmd {
        bin: "sh".to_string(),
        args: vec!["-c".to_string(), "echo $RANDOM".to_string()],
        env: Default::default(),
        workdir: None,
        policy: Default::default(),
    };

    // Run 5 times
    let outputs = run_multiple_times(&backend, &cmd, 5)?;

    // All outputs should be identical when using same seed
    let unique_outputs: HashSet<_> = outputs.iter().collect();
    assert_eq!(
        unique_outputs.len(),
        1,
        "Expected all RANDOM values to be identical with same seed, got: {:?}",
        outputs
    );

    Ok(())
}

#[test]
fn test_deterministic_random_seed_different_values() -> Result<()> {
    // Create two engines with different seeds
    let config1 = DeterminismConfig {
        seed: Some(42),
        freeze_clock: None,
        deterministic_ports: None,
        deterministic_volumes: None,
    };

    let config2 = DeterminismConfig {
        seed: Some(43),
        freeze_clock: None,
        deterministic_ports: None,
        deterministic_volumes: None,
    };

    let engine1 = Arc::new(DeterminismEngine::new(config1)?);
    let engine2 = Arc::new(DeterminismEngine::new(config2)?);

    let backend1 = TestcontainerBackend::new("alpine:latest")?.with_determinism(engine1);

    let backend2 = TestcontainerBackend::new("alpine:latest")?.with_determinism(engine2);

    let cmd = Cmd {
        bin: "sh".to_string(),
        args: vec!["-c".to_string(), "echo $RANDOM".to_string()],
        env: Default::default(),
        workdir: None,
        policy: Default::default(),
    };

    let result1 = backend1.run_cmd(cmd.clone())?;
    let result2 = backend2.run_cmd(cmd)?;

    // Different seeds should produce different RANDOM values
    assert_ne!(
        result1.stdout.trim(),
        result2.stdout.trim(),
        "Different seeds should produce different RANDOM values"
    );

    Ok(())
}

#[test]
fn test_deterministic_ports_env() -> Result<()> {
    // Create determinism config with deterministic ports
    let config = DeterminismConfig {
        seed: None,
        freeze_clock: None,
        deterministic_ports: Some(true),
        deterministic_volumes: None,
    };

    let engine = Arc::new(DeterminismEngine::new(config)?);
    let backend = TestcontainerBackend::new("alpine:latest")?.with_determinism(engine);

    let cmd = Cmd {
        bin: "sh".to_string(),
        args: vec![
            "-c".to_string(),
            "echo $CLEANROOM_ALLOWED_PORTS".to_string(),
        ],
        env: Default::default(),
        workdir: None,
        policy: Default::default(),
    };

    // Run 5 times
    let outputs = run_multiple_times(&backend, &cmd, 5)?;

    // All outputs should be identical
    let unique_outputs: HashSet<_> = outputs.iter().collect();
    assert_eq!(
        unique_outputs.len(),
        1,
        "Expected CLEANROOM_ALLOWED_PORTS to be identical across runs, got: {:?}",
        outputs
    );

    // Verify ports are set (should be "5432,6379,8080,9090,3000,5000,8000,9000")
    let ports_str = outputs[0].trim();
    assert!(
        !ports_str.is_empty(),
        "CLEANROOM_ALLOWED_PORTS should not be empty"
    );
    assert!(
        ports_str.contains("5432"),
        "Should contain default port 5432"
    );

    Ok(())
}

#[test]
fn test_port_allocator_deterministic() -> Result<()> {
    // Create determinism config
    let config = DeterminismConfig {
        seed: Some(42),
        freeze_clock: None,
        deterministic_ports: Some(true),
        deterministic_volumes: None,
    };

    let engine = DeterminismEngine::new(config)?;

    // Allocate ports in sequence
    let port1 = engine.allocate_port()?;
    let port2 = engine.allocate_port()?;
    let port3 = engine.allocate_port()?;

    // Create new engine with same config
    let engine2 = DeterminismEngine::new(DeterminismConfig {
        seed: Some(42),
        freeze_clock: None,
        deterministic_ports: Some(true),
        deterministic_volumes: None,
    })?;

    // Should get same ports in same order
    let port1_again = engine2.allocate_port()?;
    let port2_again = engine2.allocate_port()?;
    let port3_again = engine2.allocate_port()?;

    assert_eq!(port1, port1_again);
    assert_eq!(port2, port2_again);
    assert_eq!(port3, port3_again);

    Ok(())
}

#[test]
fn test_volume_naming_deterministic() -> Result<()> {
    // Create determinism config with seed
    let config = DeterminismConfig {
        seed: Some(42),
        freeze_clock: None,
        deterministic_ports: None,
        deterministic_volumes: Some(true),
    };

    let engine = DeterminismEngine::new(config)?;

    // Generate volume names
    let vol_name1 = engine.generate_volume_name("test1");
    let vol_name2 = engine.generate_volume_name("test1");

    // Same test name with same seed should produce same volume name
    assert_eq!(vol_name1, vol_name2);

    // Different test name should produce different volume name
    let vol_name3 = engine.generate_volume_name("test2");
    assert_ne!(vol_name1, vol_name3);

    Ok(())
}

#[test]
fn test_container_naming_deterministic() -> Result<()> {
    // Create determinism config with seed
    let config = DeterminismConfig {
        seed: Some(42),
        freeze_clock: None,
        deterministic_ports: None,
        deterministic_volumes: Some(true),
    };

    let engine = DeterminismEngine::new(config)?;

    // Generate container names
    let name1 = engine.generate_container_name("my_test", "step1");
    let name2 = engine.generate_container_name("my_test", "step1");

    // Same inputs should produce same name
    assert_eq!(name1, name2);

    // Different step should produce different name
    let name3 = engine.generate_container_name("my_test", "step2");
    assert_ne!(name1, name3);

    Ok(())
}

#[test]
#[ignore] // Requires libfaketime installed in alpine container
fn test_clock_freezing() -> Result<()> {
    // NOTE: This test is ignored by default because it requires:
    // 1. libfaketime to be installed in the container image
    // 2. A custom alpine image with libfaketime pre-installed
    //
    // To run this test:
    // 1. Build alpine image with: apk add libfaketime
    // 2. Run with: cargo test test_clock_freezing -- --ignored

    let config = DeterminismConfig {
        seed: None,
        freeze_clock: Some("2024-01-01T12:00:00Z".to_string()),
        deterministic_ports: None,
        deterministic_volumes: None,
    };

    let engine = Arc::new(DeterminismEngine::new(config)?);
    let backend = TestcontainerBackend::new("alpine:latest")?.with_determinism(engine);

    let cmd = Cmd {
        bin: "sh".to_string(),
        args: vec!["-c".to_string(), "date +%s".to_string()],
        env: Default::default(),
        workdir: None,
        policy: Default::default(),
    };

    // Run 5 times
    let outputs = run_multiple_times(&backend, &cmd, 5)?;

    // All timestamps should be identical when clock is frozen
    let unique_outputs: HashSet<_> = outputs.iter().collect();
    assert_eq!(
        unique_outputs.len(),
        1,
        "Expected all timestamps to be identical with frozen clock, got: {:?}",
        outputs
    );

    // Should be frozen at 1704110400 (2024-01-01 12:00:00 UTC)
    let timestamp: i64 = outputs[0].trim().parse().expect("Invalid timestamp");
    assert_eq!(
        timestamp, 1704110400,
        "Timestamp should be frozen at 1704110400"
    );

    Ok(())
}

#[test]
fn test_full_determinism_config() -> Result<()> {
    // Test all determinism features together
    let config = DeterminismConfig {
        seed: Some(42),
        freeze_clock: Some("2024-01-01T12:00:00Z".to_string()),
        deterministic_ports: Some(true),
        deterministic_volumes: Some(true),
    };

    let engine = DeterminismEngine::new(config)?;

    // Verify all features are enabled
    assert!(engine.has_seed());
    assert!(engine.has_frozen_clock());
    assert!(engine.config().has_deterministic_ports());
    assert!(engine.config().has_deterministic_volumes());

    // Verify determinism is detected
    assert!(engine.is_deterministic());

    Ok(())
}
