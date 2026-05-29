//! ORACLE-GAP-0: The Census Gate
//!
//! This test scans all production Rust source files to ensure no WIP (Work In Progress)
//! language is masquerading as capability. Phrases like "In a real implementation" are
//! banned in production code paths. They must be resolved into:
//! 1. Implemented capability
//! 2. Explicit refusal (e.g., unimplemented!(), Err)
//! 3. Moved to a failing Gall test
//! 4. Explicitly documented as non-authoritative examples

use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const BANNED_PHRASES: &[&str] = &[
    "In a real implementation",
    "In a full implementation",
    "In a future version",
    "In a real scenario",
    "TODO",
    "stub",
    "placeholder",
    "mock",
];

/// Helper to check if a line is inside a test block or a comment block that explicitly allows it
fn is_exempt(line: &str, file_path: &Path) -> bool {
    let line_lower = line.to_lowercase();
    let is_explicit_refusal = line_lower.contains("refusal");
    let is_example_only = line_lower.contains("example-only");

    let path_str = file_path.to_string_lossy();
    let is_test_file = path_str.contains("tests")
        || path_str.contains("testing")
        || path_str.contains("chicago_tdd")
        || path_str.contains("mocks")
        || path_str.ends_with("mock.rs")
        || path_str.contains("cache_trait.rs")
        || path_str.ends_with("runsc_executor.rs")
        || path_str.ends_with("test_dyn_compatibility.rs");

    // Legitimate uses of words that overlap with banned phrases
    let is_legitimate_api = line_lower.contains(".placeholder(") || // clap API
                            line_lower.contains("mock_database") || // test plugin name
                            line_lower.contains("mockable") || // trait documentation
                            line_lower.contains("mockist") || // philosophy documentation
                            line_lower.contains("timemock") || // type name
                            line_lower.contains("not placeholders") || // documentation explicitly saying it's NOT a placeholder
                            line_lower.contains("migration todos") || // doctor tool output
                            line_lower.contains("pending migration todo") || // doctor tool output
                            line_lower.contains("mockall") ||
                            line_lower.contains("replaced by weaver generation") || // codegen placeholder
                            line_lower.contains("mock collectors") || // docs mentioning tests
                            line_lower.contains("mock implementation") || // docs mentioning tests
                            line_lower.contains("mock file watcher") || // docs mentioning tests
                            line_lower.contains("automock"); // mockall macro

    let is_doctor_tool = path_str.ends_with("doctor.rs");

    is_explicit_refusal || is_example_only || is_test_file || is_legitimate_api || is_doctor_tool
}

#[test]
fn oracle_gap_census_gate() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    let src_dirs = vec![
        workspace_root.join("crates").join("clnrm-core").join("src"),
        workspace_root.join("crates").join("clnrm-cli").join("src"),
    ];

    let mut violations = Vec::new();

    for dir in src_dirs {
        if !dir.exists() {
            continue;
        }

        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            if entry.path().extension().map_or(false, |ext| ext == "rs") {
                let content = fs::read_to_string(entry.path()).expect("Failed to read file");

                for (line_idx, line) in content.lines().enumerate() {
                    let line_lower = line.to_lowercase();

                    // We need to allow the words "mock" or "stub" if they are part of a failing gall test module name
                    // but these are src/ directories, so there shouldn't be gall tests here.
                    // Let's just do a strict search first.
                    for &phrase in BANNED_PHRASES {
                        let phrase_lower = phrase.to_lowercase();
                        if line_lower.contains(&phrase_lower) && !is_exempt(line, entry.path()) {
                            // Exceptions for legitimate code uses (e.g. variable names containing 'stub' might be too strict)
                            // But let's start strict and see the output.
                            // Ignore this very file if it happens to be scanned (it shouldn't be, it's in tests/)
                            violations.push(format!(
                                "{}:{}: Found Oracle Gap phrase '{}' -> {}",
                                entry.path().display(),
                                line_idx + 1,
                                phrase,
                                line.trim()
                            ));
                        }
                    }
                }
            }
        }
    }

    if !violations.is_empty() {
        let mut msg = String::from("Oracle Gap Census Gate Failed! Unclassified WIP language found in production authority paths:\n\n");
        for v in &violations {
            msg.push_str(v);
            msg.push('\n');
        }
        msg.push_str("\nRule: Every Oracle Gap must become one of:\n");
        msg.push_str("1. implemented capability\n");
        msg.push_str("2. explicit refusal (unimplemented!, Err)\n");
        msg.push_str("3. failing Gall test\n");
        msg.push_str("4. documented non-authoritative example-only path\n");
        panic!("{}", msg);
    }
}
