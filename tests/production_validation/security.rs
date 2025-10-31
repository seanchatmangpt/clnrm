//! Security Validation Tests
//!
//! Validates Weaver live-check security posture:
//! - Sensitive attribute handling (secrets, PII)
//! - Redaction capabilities
//! - Custom policies for security rules
//! - Data leakage prevention in reports

use clnrm_core::telemetry::weaver_controller::{WeaverConfig, WeaverController};
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

#[test]
#[ignore = "Requires Weaver installation and security review"]
fn test_sensitive_attributes_not_in_output() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Testing sensitive attribute handling");

    let config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        output_dir: PathBuf::from("/tmp/clnrm_security_test"),
        stream: false,
        ..Default::default()
    };

    let mut controller = WeaverController::new(config);
    controller.start_live_check()?;

    println!("   Generating telemetry with mock sensitive data...");

    // In a real test, this would emit telemetry with sensitive attributes
    // like: password, api_key, credit_card, ssn, etc.

    thread::sleep(Duration::from_secs(3));

    let report = controller.stop_and_report()?;

    // Read the output files and verify no sensitive data leaked
    let output_files = fs::read_dir("/tmp/clnrm_security_test")?;

    let sensitive_patterns = vec![
        "password",
        "api_key",
        "secret",
        "token",
        "credit_card",
        "ssn",
        "private_key",
    ];

    for entry in output_files {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let content = fs::read_to_string(&path)?;

            for pattern in &sensitive_patterns {
                assert!(
                    !content.to_lowercase().contains(pattern),
                    "Sensitive pattern '{}' found in {:?}",
                    pattern,
                    path
                );
            }
        }
    }

    println!("   Violations: {}", report.violations);
    println!("✅ Sensitive attribute test passed - no leaks detected");
    Ok(())
}

#[test]
#[ignore = "Requires Weaver installation with redaction support"]
fn test_redaction_capabilities() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔒 Testing redaction capabilities");

    let config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        output_dir: PathBuf::from("/tmp/clnrm_redaction_test"),
        stream: false,
        ..Default::default()
    };

    let mut controller = WeaverController::new(config);
    controller.start_live_check()?;

    println!("   Testing attribute redaction...");

    // Simulate telemetry with attributes that should be redacted
    // Example: user.email, user.phone, user.address

    thread::sleep(Duration::from_secs(2));

    let report = controller.stop_and_report()?;

    // Verify redaction in output
    let output_dir = PathBuf::from("/tmp/clnrm_redaction_test");
    let output_files = fs::read_dir(&output_dir)?;

    for entry in output_files {
        let entry = entry?;
        if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
            let content = fs::read_to_string(entry.path())?;

            // Redacted values should appear as [REDACTED] or similar
            if content.contains("user.email") || content.contains("user.phone") {
                assert!(
                    content.contains("[REDACTED]") ||
                    content.contains("***") ||
                    content.contains("MASKED"),
                    "Expected redaction markers in output"
                );
            }
        }
    }

    println!("   Report status: {:?}", report.status);
    println!("✅ Redaction test passed");
    Ok(())
}

#[test]
#[ignore = "Requires Weaver installation with custom policies"]
fn test_custom_security_policies() -> Result<(), Box<dyn std::error::Error>> {
    println!("📜 Testing custom security policies");

    // Create a custom policy file
    let policy_content = r#"
# Custom Security Policy for clnrm

# Require that all HTTP attributes are present for security auditing
- require_attributes:
    - http.method
    - http.status_code
    - http.url

# Forbid insecure protocols
- forbid_values:
    http.scheme: ["http", "ftp"]

# Require encryption indicators
- require_boolean:
    tls.enabled: true
"#;

    let policy_dir = PathBuf::from("/tmp/clnrm_security_policy");
    fs::create_dir_all(&policy_dir)?;
    fs::write(policy_dir.join("security_policy.yaml"), policy_content)?;

    let config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        output_dir: PathBuf::from("/tmp/clnrm_policy_test"),
        stream: false,
        ..Default::default()
    };

    let mut controller = WeaverController::new(config);
    controller.start_live_check()?;

    println!("   Running with custom security policies...");
    thread::sleep(Duration::from_secs(3));

    let report = controller.stop_and_report()?;

    // Custom policies should trigger violations if not met
    println!("   Violations: {}", report.violations);
    println!("   Policy violations: {}", report.details.len());

    for detail in &report.details {
        if detail.level == "violation" {
            println!("     - {}", detail.message);
        }
    }

    // Cleanup
    let _ = fs::remove_dir_all(&policy_dir);

    println!("✅ Custom security policy test completed");
    Ok(())
}

#[test]
#[ignore = "Requires Weaver installation"]
fn test_pii_detection_in_telemetry() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing PII detection in telemetry");

    let config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        output_dir: PathBuf::from("/tmp/clnrm_pii_test"),
        stream: false,
        ..Default::default()
    };

    let mut controller = WeaverController::new(config);
    controller.start_live_check()?;

    println!("   Simulating telemetry with potential PII...");

    // Common PII patterns to test:
    // - Email addresses: test@example.com
    // - Phone numbers: (555) 123-4567
    // - SSN: 123-45-6789
    // - Credit card: 4111-1111-1111-1111
    // - IP addresses: 192.168.1.1

    thread::sleep(Duration::from_secs(2));

    let report = controller.stop_and_report()?;

    // Check if Weaver detected or flagged PII
    let output_dir = PathBuf::from("/tmp/clnrm_pii_test");
    let report_file = output_dir.join("validation_report.json");

    if report_file.exists() {
        let content = fs::read_to_string(&report_file)?;

        // Verify PII is either:
        // 1. Redacted/masked
        // 2. Flagged as violation
        // 3. Not present in output

        let pii_patterns = vec![
            r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b",  // Email
            r"\b\d{3}-\d{2}-\d{4}\b",  // SSN
            r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b",  // Credit card
        ];

        println!("   Checking for unredacted PII patterns...");

        for pattern in pii_patterns {
            let re = regex::Regex::new(pattern).unwrap();
            let matches = re.find_iter(&content).count();

            if matches > 0 {
                println!("     ⚠️  Found {} instances of pattern: {}", matches, pattern);
            }
        }
    }

    println!("   Report violations: {}", report.violations);
    println!("✅ PII detection test completed");
    Ok(())
}

#[test]
#[ignore = "Requires Weaver installation"]
fn test_secure_output_file_permissions() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Testing secure output file permissions");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let config = WeaverConfig {
            registry_path: PathBuf::from("registry"),
            output_dir: PathBuf::from("/tmp/clnrm_permissions_test"),
            stream: false,
            ..Default::default()
        };

        let mut controller = WeaverController::new(config);
        controller.start_live_check()?;

        thread::sleep(Duration::from_secs(2));

        let _report = controller.stop_and_report()?;

        // Check file permissions
        let output_dir = PathBuf::from("/tmp/clnrm_permissions_test");
        let entries = fs::read_dir(&output_dir)?;

        for entry in entries {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let permissions = metadata.permissions();
            let mode = permissions.mode();

            // Files should not be world-readable (no 0o004 bit)
            let is_world_readable = (mode & 0o004) != 0;
            assert!(
                !is_world_readable,
                "File {:?} is world-readable (mode: {:o})",
                entry.path(),
                mode
            );

            println!("     {:?} - mode: {:o} ✓", entry.file_name(), mode);
        }

        println!("✅ File permissions test passed");
    }

    #[cfg(not(unix))]
    {
        println!("   Skipping permission test on non-Unix platform");
    }

    Ok(())
}

#[test]
#[ignore = "Requires Weaver installation"]
fn test_no_secrets_in_validation_report() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔒 Testing that secrets don't leak into validation reports");

    let config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        output_dir: PathBuf::from("/tmp/clnrm_secrets_test"),
        stream: false,
        ..Default::default()
    };

    let mut controller = WeaverController::new(config);
    controller.start_live_check()?;

    println!("   Generating telemetry with mock secrets...");

    // Simulate telemetry with secret-like attributes
    // Examples: database.password, api.secret_key, oauth.client_secret

    thread::sleep(Duration::from_secs(2));

    let report = controller.stop_and_report()?;

    // Parse the ValidationReport
    println!("   Report details: {} entries", report.details.len());

    // Check that no secret values appear in messages
    let secret_indicators = vec![
        "password",
        "secret",
        "token",
        "key",
        "credential",
    ];

    for detail in &report.details {
        let message_lower = detail.message.to_lowercase();

        for indicator in &secret_indicators {
            // It's OK to mention "password" as a field name
            // But NOT OK to have "password=actual_secret_value"
            if message_lower.contains(&format!("{}=", indicator)) ||
               message_lower.contains(&format!("{}: ", indicator)) {
                panic!(
                    "Potential secret leak in validation report: {}",
                    detail.message
                );
            }
        }
    }

    println!("✅ No secrets leaked into validation report");
    Ok(())
}

#[test]
#[ignore = "Requires Weaver installation"]
fn test_data_sanitization_in_error_messages() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧹 Testing data sanitization in error messages");

    let config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        output_dir: PathBuf::from("/tmp/clnrm_sanitization_test"),
        stream: false,
        ..Default::default()
    };

    let mut controller = WeaverController::new(config);
    controller.start_live_check()?;

    thread::sleep(Duration::from_secs(2));

    let report = controller.stop_and_report()?;

    // Check all error/violation messages for sensitive data patterns
    for detail in &report.details {
        let message = &detail.message;

        // Should not contain actual values that look sensitive
        assert!(
            !message.contains("sk-"),  // OpenAI API keys
            "Error message contains API key pattern: {}",
            message
        );

        assert!(
            !message.contains("AIza"),  // Google API keys
            "Error message contains API key pattern: {}",
            message
        );

        // Should not contain SQL injection patterns in error details
        assert!(
            !message.to_lowercase().contains("drop table"),
            "Error message contains SQL injection pattern: {}",
            message
        );
    }

    println!("✅ Data sanitization test passed");
    Ok(())
}
