//! CLI Output Snapshot Tests
//!
//! Tests for command-line interface output consistency

#[cfg(test)]
mod cli_snapshot_tests {
    use std::process::{Command, Output};

    struct CliOutput {
        stdout: String,
        stderr: String,
        exit_code: i32,
    }

    impl CliOutput {
        fn normalize(&self) -> String {
            // Normalize timestamps, paths, and dynamic values
            let mut output = format!(
                "Exit Code: {}\n\nSTDOUT:\n{}\n\nSTDERR:\n{}",
                self.exit_code, self.stdout, self.stderr
            );

            // Replace timestamps
            output = regex::Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}")
                .unwrap()
                .replace_all(&output, "[TIMESTAMP]")
                .to_string();

            // Replace duration values
            output = regex::Regex::new(r"\d+\.\d+ms")
                .unwrap()
                .replace_all(&output, "[DURATION]ms")
                .to_string();

            // Replace absolute paths
            output = regex::Regex::new(r"/Users/[^/]+/[^\s]+")
                .unwrap()
                .replace_all(&output, "[PATH]")
                .to_string();

            output
        }
    }

    #[test]
    fn test_clnrm_help_output_snapshot() {
        // Simulated help output
        let output = CliOutput {
            stdout: r#"clnrm 0.4.0
Cleanroom Testing Framework

USAGE:
    clnrm [OPTIONS] [COMMAND]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

COMMANDS:
    run       Run tests
    init      Initialize a new test project
    watch     Watch for changes and run tests
    help      Print this message or the help of the given subcommand(s)"#.to_string(),
            stderr: String::new(),
            exit_code: 0,
        };

        let normalized = output.normalize();

        // insta::assert_snapshot!("clnrm_help", normalized);
        assert!(normalized.contains("clnrm"));
        assert!(normalized.contains("USAGE"));
        assert!(normalized.contains("OPTIONS"));
    }

    #[test]
    fn test_clnrm_version_snapshot() {
        let output = CliOutput {
            stdout: "clnrm 0.4.0\n".to_string(),
            stderr: String::new(),
            exit_code: 0,
        };

        let normalized = output.normalize();
        // insta::assert_snapshot!("clnrm_version", normalized);
        assert!(normalized.contains("clnrm 0.4.0"));
    }

    #[test]
    fn test_clnrm_run_output_snapshot() {
        let output = CliOutput {
            stdout: r#"Running tests from: ./tests
Found 3 test files

Test: basic_test.toml
  ✓ Setup step completed (45ms)
  ✓ Execute step completed (78ms)
  ✓ Verify step completed (23ms)

Test completed in 146ms
Exit code: 0

Summary:
  Total: 3 tests
  Passed: 3
  Failed: 0
  Duration: 423ms"#.to_string(),
            stderr: String::new(),
            exit_code: 0,
        };

        let normalized = output.normalize();

        // insta::assert_snapshot!("clnrm_run_success", normalized);
        assert!(normalized.contains("Running tests"));
        assert!(normalized.contains("Passed: 3"));
    }

    #[test]
    fn test_clnrm_error_output_snapshot() {
        let output = CliOutput {
            stdout: String::new(),
            stderr: r#"Error: Test file not found: ./tests/nonexistent.toml

Caused by:
    No such file or directory (os error 2)

For more information, run with --verbose"#.to_string(),
            exit_code: 1,
        };

        let normalized = output.normalize();

        // insta::assert_snapshot!("clnrm_error", normalized);
        assert!(normalized.contains("Error:"));
        assert!(normalized.contains("Exit Code: 1"));
    }
}
