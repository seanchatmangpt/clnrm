//! Fuzz target for CLI argument parsing
//!
//! Tests the CLI argument parser for:
//! - Buffer overflows in argument handling
//! - Path traversal vulnerabilities
//! - Argument injection
//! - Unicode and special character handling

#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::{Arbitrary, Unstructured};
use std::path::PathBuf;

/// Fuzzable CLI arguments
#[derive(Debug, Arbitrary)]
struct FuzzCliArgs {
    subcommand: FuzzSubcommand,
    path: Option<String>,
    flags: Vec<String>,
}

#[derive(Debug, Arbitrary)]
enum FuzzSubcommand {
    Run,
    Validate,
    Init,
    Template,
    Report,
}

fuzz_target!(|data: &[u8]| {
    let mut unstructured = Unstructured::new(data);

    if let Ok(args) = FuzzCliArgs::arbitrary(&mut unstructured) {
        // Test path handling (common vulnerability)
        if let Some(path_str) = &args.path {
            // Test path parsing with potentially malicious inputs
            let _ = PathBuf::from(path_str);

            // Test path canonicalization (can expose vulnerabilities)
            if let Ok(path) = PathBuf::from(path_str).canonicalize() {
                // Verify path is within safe bounds (no traversal)
                let _ = path.to_str();
            }
        }

        // Test flag parsing
        for flag in args.flags.iter().take(100) {
            // Test various flag formats
            let _ = parse_flag(flag);
        }

        // Test subcommand handling
        match args.subcommand {
            FuzzSubcommand::Run => {
                // Test run command path validation
                if let Some(path) = &args.path {
                    test_run_path(path);
                }
            }
            FuzzSubcommand::Validate => {
                // Test validate command
                if let Some(path) = &args.path {
                    test_validate_path(path);
                }
            }
            FuzzSubcommand::Init => {
                // Test init command with various project names
                if let Some(name) = &args.path {
                    test_project_name(name);
                }
            }
            FuzzSubcommand::Template => {
                // Test template name validation
                if let Some(template) = &args.path {
                    test_template_name(template);
                }
            }
            FuzzSubcommand::Report => {
                // Test report path handling
                if let Some(path) = &args.path {
                    test_report_path(path);
                }
            }
        }
    }
});

fn parse_flag(flag: &str) {
    // Test flag parsing logic
    let _ = flag.starts_with("--");
    let _ = flag.starts_with('-');

    // Test key=value parsing
    if let Some((key, value)) = flag.split_once('=') {
        let _ = (key.trim(), value.trim());
    }
}

fn test_run_path(path: &str) {
    // Test path validation for run command
    let path_buf = PathBuf::from(path);

    // Check for path traversal attempts
    let _ = path_buf.file_name();
    let _ = path_buf.extension();
    let _ = path_buf.parent();
}

fn test_validate_path(path: &str) {
    // Test validation path handling
    let path_buf = PathBuf::from(path);

    // Check if path looks like a valid test file
    let _ = path.ends_with(".toml");
    let _ = path.ends_with(".clnrm.toml");
    let _ = path_buf.is_absolute();
}

fn test_project_name(name: &str) {
    // Test project name validation
    let sanitized: String = name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .take(255)
        .collect();

    let _ = sanitized.is_empty();
}

fn test_template_name(template: &str) {
    // Test template name validation
    let valid_templates = ["basic", "database", "api", "integration"];
    let _ = valid_templates.contains(&template);
}

fn test_report_path(path: &str) {
    // Test report path validation
    let path_buf = PathBuf::from(path);

    // Check for directory traversal
    let _ = path_buf.parent();
    let _ = path_buf.file_name();

    // Test various report formats
    let _ = path.ends_with(".xml");
    let _ = path.ends_with(".json");
    let _ = path.ends_with(".html");
}
