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
    let runtime_config = parser.to_runtime_config(&oci_config, None, Some(&policy)).unwrap();

    // Assert
    // GALL GAP: The parser generates a `linux` config block but doesn't actually
    // wire the `policy.resources` into the `linux.resources` cgroups structure.
    let linux_config = runtime_config.linux.expect("Gall Gap: Linux config block should be generated for cgroups");
    
    panic!("Gall Gap: OCI Resource Constraint Gap. SecurityPolicy memory limits are not injected into the OCI config.json linux.resources cgroups");
}