//! Gall Test Suite for Security Policy Rules
//!
//! Validates `SecurityPolicy` constraints shift correctly depending on `SecurityLevel`.

use clnrm_core::policy::{SecurityLevel, SecurityPolicy};

#[test]
fn gall_test_policy_low_security_disables_isolation() {
    // Arrange / Act
    let policy = SecurityPolicy::with_security_level(SecurityLevel::Low);

    // Assert
    assert!(
        !policy.enable_network_isolation,
        "Low security should disable network isolation"
    );
    assert!(
        !policy.enable_filesystem_isolation,
        "Low security should disable FS isolation"
    );
    assert!(
        !policy.enable_process_isolation,
        "Low security should disable process isolation"
    );
    assert!(
        !policy.enable_data_redaction,
        "Low security should disable redaction"
    );
}

#[test]
fn gall_test_policy_high_security_enforces_isolation() {
    // Arrange / Act
    let policy = SecurityPolicy::with_security_level(SecurityLevel::High);

    // Assert
    assert!(
        policy.enable_network_isolation,
        "High security should enforce network isolation"
    );
    assert!(
        policy.enable_filesystem_isolation,
        "High security should enforce FS isolation"
    );
    assert!(
        policy.enable_process_isolation,
        "High security should enforce process isolation"
    );
    assert!(
        policy.enable_data_redaction,
        "High security should enforce redaction"
    );
}
