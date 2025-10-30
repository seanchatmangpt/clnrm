//! Integration tests for error case validation
//!
//! Tests that error cases actually fail and don't produce false positives

#[cfg(test)]
mod error_validation {
    use std::process::Command;

    #[test]
    fn test_invalid_toml_file_fails() {
        // Create invalid TOML file
        let invalid_toml = r#"
[test.metadata]
name = "invalid"
this is not valid toml syntax {{{
"#;

        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("invalid_test.clnrm.toml");
        std::fs::write(&test_file, invalid_toml).unwrap();

        // Run clnrm validate - should fail
        let output = Command::new("clnrm")
            .arg("validate")
            .arg(&test_file)
            .output();

        match output {
            Ok(out) => {
                assert!(
                    !out.status.success(),
                    "Invalid TOML should fail validation - FALSE POSITIVE!"
                );
            }
            Err(_) => {
                // clnrm not installed, skip test
                eprintln!("Skipping test: clnrm not installed");
            }
        }

        // Cleanup
        let _ = std::fs::remove_file(&test_file);
    }

    #[test]
    fn test_nonexistent_file_fails() {
        let output = Command::new("clnrm")
            .arg("validate")
            .arg("/nonexistent/path/to/test.clnrm.toml")
            .output();

        match output {
            Ok(out) => {
                assert!(
                    !out.status.success(),
                    "Nonexistent file should fail - FALSE POSITIVE!"
                );
            }
            Err(_) => {
                eprintln!("Skipping test: clnrm not installed");
            }
        }
    }

    #[test]
    fn test_invalid_command_fails() {
        let output = Command::new("clnrm")
            .arg("nonexistent-command")
            .output();

        match output {
            Ok(out) => {
                assert!(
                    !out.status.success(),
                    "Invalid command should fail - FALSE POSITIVE!"
                );
            }
            Err(_) => {
                eprintln!("Skipping test: clnrm not installed");
            }
        }
    }
}
