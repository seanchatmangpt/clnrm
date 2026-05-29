//! Gall Test Suite for OCI Resource Constraints
//!
//! Exposes the gap where SecurityPolicy resource limits (CPU/Memory)
//! are not translated into OCI config.json cgroups constraints.

use clnrm_core::backend::oci::config_parser::{ConfigParser, RuntimeConfig};
use clnrm_core::backend::oci::OciImageConfig;
use clnrm_core::policy::{Policy, SecurityLevel};
use serde_json::json;

#[test]
fn gall_gap_test_oci_cgroups_limits() {
    // Arrange
    let mut policy = Policy::with_security_level(SecurityLevel::High);
    // Enforce a strict 100MB memory limit
    policy.resources.max_memory_usage_bytes = 100 * 1024 * 1024;

    let parser = ConfigParser;
    let dummy_image_config_data = json!({
        "architecture": "amd64",
        "os": "linux",
        "rootfs": {
            "type": "layers",
            "diff_ids": []
        },
        "config": {
            "Env": ["PATH=/usr/bin"],
            "Cmd": ["sh"]
        }
    });
    let oci_config: OciImageConfig = serde_json::from_value(dummy_image_config_data).unwrap();

    // Act
    let runtime_config = parser
        .to_runtime_config(&oci_config, None, Some(&policy))
        .unwrap();

    // Assert
    // The parser generates a `linux` config block and wires the `policy.resources`
    // into the `linux.resources` cgroups structure.
    let linux_config = runtime_config
        .linux
        .expect("Linux config block should be generated for cgroups");

    let resources = linux_config
        .resources
        .expect("Resources block should be generated");
    let memory = resources.memory.expect("Memory block should be generated");
    assert_eq!(
        memory.limit,
        Some(100 * 1024 * 1024),
        "SecurityPolicy memory limits should be injected into the OCI config.json linux.resources cgroups"
    );
}
