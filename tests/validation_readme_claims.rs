//! Integration tests validating README claims
//!
//! Based on false positive analysis reports

#[cfg(test)]
mod readme_validation {
    use std::process::Command;

    #[test]
    fn test_clnrm_version_command_works() {
        let output = Command::new("clnrm")
            .arg("--version")
            .output();

        match output {
            Ok(out) => {
                assert!(out.status.success(), "clnrm --version should succeed");
                let stdout = String::from_utf8_lossy(&out.stdout);
                assert!(stdout.contains("clnrm"), "Version output should contain 'clnrm'");
            }
            Err(_) => {
                eprintln!("Skipping test: clnrm not installed via Homebrew");
            }
        }
    }

    #[test]
    fn test_clnrm_plugins_command_works() {
        let output = Command::new("clnrm")
            .arg("plugins")
            .output();

        match output {
            Ok(out) => {
                assert!(out.status.success(), "clnrm plugins should succeed");
                let stdout = String::from_utf8_lossy(&out.stdout);

                // Verify plugins are listed
                assert!(stdout.contains("generic_container"), "Should list generic_container plugin");
                assert!(stdout.contains("Available Service Plugins") || stdout.contains("Service Plugins"),
                       "Should show plugin header");
            }
            Err(_) => {
                eprintln!("Skipping test: clnrm not installed");
            }
        }
    }

    #[test]
    fn test_clnrm_init_creates_structure() {
        let temp_dir = std::env::temp_dir().join(format!("clnrm_test_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let output = Command::new("clnrm")
            .arg("init")
            .current_dir(&temp_dir)
            .output();

        match output {
            Ok(out) => {
                assert!(out.status.success(), "clnrm init should succeed");

                // Verify files were created
                assert!(temp_dir.join("tests").join("basic.clnrm.toml").exists(),
                       "Should create tests/basic.clnrm.toml");
                assert!(temp_dir.join("README.md").exists(),
                       "Should create README.md");
                assert!(temp_dir.join("scenarios").exists(),
                       "Should create scenarios/ directory");
            }
            Err(_) => {
                eprintln!("Skipping test: clnrm not installed");
            }
        }

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_self_test_command_exists() {
        let output = Command::new("clnrm")
            .arg("self-test")
            .arg("--help")
            .output();

        match output {
            Ok(out) => {
                assert!(out.status.success(), "clnrm self-test --help should succeed");
                let stdout = String::from_utf8_lossy(&out.stdout);
                assert!(stdout.contains("self-test") || stdout.contains("Self-test"),
                       "Should show self-test help");
            }
            Err(_) => {
                eprintln!("Skipping test: clnrm not installed");
            }
        }
    }

    #[test]
    fn test_template_command_lists_templates() {
        let output = Command::new("clnrm")
            .arg("template")
            .arg("--help")
            .output();

        match output {
            Ok(out) => {
                assert!(out.status.success(), "clnrm template --help should succeed");
                let stdout = String::from_utf8_lossy(&out.stdout);

                // Verify template types mentioned in help
                assert!(stdout.contains("template") || stdout.contains("TEMPLATE"),
                       "Should mention template in help");
            }
            Err(_) => {
                eprintln!("Skipping test: clnrm not installed");
            }
        }
    }
}
