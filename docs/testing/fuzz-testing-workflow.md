# Fuzz Testing Workflow Guide

## Table of Contents

1. [Introduction](#introduction)
2. [What is Fuzz Testing?](#what-is-fuzz-testing)
3. [Setup and Installation](#setup-and-installation)
4. [Fuzz Targets](#fuzz-targets)
5. [Running Fuzz Tests](#running-fuzz-tests)
6. [Corpus Management](#corpus-management)
7. [Handling Crashes](#handling-crashes)
8. [CI/CD Integration](#cicd-integration)
9. [Best Practices](#best-practices)
10. [Troubleshooting](#troubleshooting)

## Introduction

Fuzz testing (fuzzing) is an automated testing technique that feeds invalid, unexpected, or random data as inputs to discover security vulnerabilities, crashes, memory leaks, and edge cases that traditional testing might miss.

### Why Fuzz Testing?

- **Discover Hidden Bugs**: Find edge cases developers didn't think of
- **Security Vulnerabilities**: Detect buffer overflows, panics, and memory issues
- **Parser Robustness**: Ensure parsers handle invalid inputs gracefully
- **Automated Testing**: Runs continuously without manual test case creation
- **Coverage**: Achieve deep code coverage in complex paths

### CLNRM Fuzz Testing Goals

| Component | Target | Current Status |
|-----------|--------|----------------|
| TOML Parser | 80%+ coverage | Active |
| Scenario DSL | 75%+ coverage | Active |
| CLI Args | 85%+ coverage | Active |
| Error Handling | 90%+ coverage | Active |
| Regex Patterns | 70%+ coverage | Active |

## What is Fuzz Testing?

### Core Concepts

**Fuzzing Process**:
```
1. Generate random or mutated inputs
2. Feed inputs to target function
3. Monitor for crashes, hangs, or errors
4. Save interesting inputs to corpus
5. Mutate corpus inputs for next generation
6. Repeat indefinitely
```

**Coverage-Guided Fuzzing**:
- Tracks code coverage during execution
- Prioritizes inputs that explore new code paths
- Mutates inputs to maximize coverage
- Builds corpus of interesting test cases

**Sanitizers**:
- **AddressSanitizer (ASan)**: Detects memory errors
- **UndefinedBehaviorSanitizer (UBSan)**: Catches undefined behavior
- **ThreadSanitizer (TSan)**: Finds data races

## Setup and Installation

### Prerequisites

```bash
# Install Rust nightly (required for libFuzzer)
rustup install nightly

# Install cargo-fuzz
cargo install cargo-fuzz

# Verify installation
cargo +nightly fuzz --version
```

### Project Structure

```
tests/fuzz/
├── Cargo.toml                  # Fuzz testing dependencies
├── README.md                   # Fuzzing documentation
├── fuzz_targets/               # Fuzz target implementations
│   ├── fuzz_toml_parser.rs
│   ├── fuzz_scenario_dsl.rs
│   ├── fuzz_cli_args.rs
│   ├── fuzz_error_handling.rs
│   └── fuzz_regex_patterns.rs
├── corpus/                     # Seed inputs for each target
│   ├── fuzz_toml_parser/
│   ├── fuzz_scenario_dsl/
│   └── ...
├── artifacts/                  # Crash artifacts (not committed)
└── crash_reproduction_tests.rs # Regression tests for crashes
```

### Initial Setup

```bash
# Navigate to project root
cd /path/to/clnrm

# Create fuzz directory (if not exists)
cargo fuzz init

# Add fuzz target
cargo fuzz add fuzz_toml_parser

# Create corpus directory
mkdir -p tests/fuzz/corpus/fuzz_toml_parser
```

## Fuzz Targets

### 1. TOML Parser Fuzzer

**Purpose**: Test TOML configuration parser robustness

**Location**: `tests/fuzz/fuzz_targets/fuzz_toml_parser.rs`

**Implementation**:
```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use clnrm_core::config::parse_toml_config;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to string
    if let Ok(s) = std::str::from_utf8(data) {
        // Attempt to parse TOML
        // We don't care about Ok/Err, only that it doesn't panic
        let _ = parse_toml_config(s);
    }
});
```

**Attack Surfaces**:
- Malformed TOML syntax
- Deeply nested structures (stack overflow)
- Extremely long strings (memory exhaustion)
- Unicode edge cases
- Duplicate keys
- Invalid escape sequences

**Expected Findings**:
- Parser panics on certain invalid inputs
- Stack overflow with deep nesting
- Memory exhaustion with large strings
- Validation bypass

### 2. Scenario DSL Fuzzer

**Purpose**: Test scenario builder and execution

**Location**: `tests/fuzz/fuzz_targets/fuzz_scenario_dsl.rs`

**Implementation**:
```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use clnrm_core::scenario::Scenario;
use arbitrary::Arbitrary;

#[derive(Debug, Arbitrary)]
struct FuzzScenario {
    name: String,
    steps: Vec<FuzzStep>,
    concurrent: bool,
    timeout_ms: Option<u32>,
}

#[derive(Debug, Arbitrary)]
struct FuzzStep {
    name: String,
    command: Vec<String>,
}

fuzz_target!(|input: FuzzScenario| {
    // Sanitize inputs to prevent actual system damage
    let safe_commands = vec!["echo", "true", "false"];

    let mut scenario = Scenario::new(&input.name);

    for step in input.steps.iter().take(10) {  // Limit steps
        if let Some(cmd) = safe_commands.get(0) {
            scenario = scenario.step(&step.name, vec![cmd.to_string()]);
        }
    }

    // Just test building, not execution
    let _ = scenario.validate();
});
```

**Attack Surfaces**:
- Command injection attempts
- Resource exhaustion (infinite steps)
- Concurrent execution race conditions
- Step ordering issues

**Safety Note**: This fuzzer sanitizes commands to prevent actual system damage during fuzzing.

### 3. CLI Arguments Fuzzer

**Purpose**: Test command-line argument parsing

**Location**: `tests/fuzz/fuzz_targets/fuzz_cli_args.rs`

**Implementation**:
```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use clnrm_core::cli::parse_args;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Split into arguments
        let args: Vec<&str> = s.split_whitespace().collect();

        // Attempt to parse
        let _ = parse_args(&args);
    }
});
```

**Attack Surfaces**:
- Path traversal attempts
- Buffer overflows
- Argument injection
- Unicode handling
- Special character edge cases

### 4. Error Handling Fuzzer

**Purpose**: Test error creation and display

**Location**: `tests/fuzz/fuzz_targets/fuzz_error_handling.rs`

**Implementation**:
```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use clnrm_core::error::CleanroomError;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Test error creation
        let err = CleanroomError::InvalidConfiguration {
            message: s.to_string(),
        };

        // Test Display/Debug traits (shouldn't panic)
        let _ = format!("{}", err);
        let _ = format!("{:?}", err);

        // Test error chaining
        let chained = err.context("Additional context");
        let _ = format!("{}", chained);
    }
});
```

**Attack Surfaces**:
- Format string vulnerabilities
- Stack overflow in error chains
- Serialization issues
- Display/Debug trait panics

### 5. Regex Patterns Fuzzer

**Purpose**: Test regex compilation and matching

**Location**: `tests/fuzz/fuzz_targets/fuzz_regex_patterns.rs`

**Implementation**:
```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use regex::Regex;

fuzz_target!(|data: &[u8]| {
    if let Ok(pattern) = std::str::from_utf8(data) {
        // Attempt to compile regex
        if let Ok(re) = Regex::new(pattern) {
            // Test matching against various inputs
            let test_strings = ["test", "123", "abc-xyz", ""];
            for s in &test_strings {
                let _ = re.is_match(s);
            }
        }
    }
});
```

**Attack Surfaces**:
- ReDoS (Regular Expression Denial of Service)
- Catastrophic backtracking
- Invalid regex syntax
- Memory exhaustion

## Running Fuzz Tests

### Basic Usage

```bash
# Navigate to fuzz directory
cd tests/fuzz

# Run specific fuzz target
cargo +nightly fuzz run fuzz_toml_parser

# Run with time limit (60 seconds)
cargo +nightly fuzz run fuzz_toml_parser -- -max_total_time=60

# Run with memory limit
cargo +nightly fuzz run fuzz_toml_parser -- -rss_limit_mb=2048

# Run with specific number of runs
cargo +nightly fuzz run fuzz_toml_parser -- -runs=1000000
```

### Advanced Options

```bash
# Multiple workers (parallel fuzzing)
cargo +nightly fuzz run fuzz_toml_parser -- -workers=4

# Timeout per input (detect ReDoS)
cargo +nightly fuzz run fuzz_regex_patterns -- -timeout=1

# Dictionary-based fuzzing
cargo +nightly fuzz run fuzz_toml_parser -- -dict=toml.dict

# Minimize corpus while fuzzing
cargo +nightly fuzz run fuzz_toml_parser -- -merge=1

# Print stats every N seconds
cargo +nightly fuzz run fuzz_toml_parser -- -print_final_stats=1
```

### Running All Fuzz Targets

```bash
# Quick smoke test (30 seconds each)
for target in fuzz_targets/*.rs; do
    name=$(basename $target .rs)
    echo "Fuzzing $name..."
    cargo +nightly fuzz run $name -- -max_total_time=30
done

# Comprehensive fuzzing (5 minutes each)
for target in fuzz_targets/*.rs; do
    name=$(basename $target .rs)
    echo "Fuzzing $name..."
    cargo +nightly fuzz run $name -- -max_total_time=300
done
```

## Corpus Management

### Understanding the Corpus

The **corpus** is a collection of interesting inputs that:
- Achieve new code coverage
- Trigger unique execution paths
- Serve as seeds for mutation

### Initial Corpus Setup

```bash
# Create corpus directories
mkdir -p corpus/fuzz_toml_parser

# Add seed files (valid TOML examples)
cat > corpus/fuzz_toml_parser/valid_basic.toml << 'EOF'
[test.metadata]
name = "example"

[[steps]]
name = "step1"
command = ["echo", "test"]
EOF

# Add edge case seeds
cat > corpus/fuzz_toml_parser/edge_empty.toml << 'EOF'
[test.metadata]
name = ""
EOF

# Add malformed seeds
cat > corpus/fuzz_toml_parser/malformed.toml << 'EOF'
[unclosed
key = "value"
EOF
```

### Growing the Corpus

Fuzzing automatically grows the corpus:

```bash
# Fuzz for extended period
cargo +nightly fuzz run fuzz_toml_parser -- -max_total_time=3600

# New interesting inputs saved automatically to:
# corpus/fuzz_toml_parser/
```

### Minimizing the Corpus

Remove redundant inputs that don't increase coverage:

```bash
# Minimize single target corpus
cargo +nightly fuzz cmin fuzz_toml_parser

# Merge multiple corpus directories
cargo +nightly fuzz cmin fuzz_toml_parser -- corpus_backup/

# Minimize all targets
for target in fuzz_targets/*.rs; do
    name=$(basename $target .rs)
    cargo +nightly fuzz cmin $name
done
```

### Corpus Best Practices

1. **Seed with Valid Inputs**: Start with known-good examples
2. **Include Edge Cases**: Empty strings, max values, boundaries
3. **Add Previous Crashes**: Regression testing
4. **Keep Minimal**: Remove redundant inputs
5. **Version Control**: Commit minimized corpus to git

## Handling Crashes

### When Crashes Occur

Fuzzing finds a crash:
```
==1234==ERROR: AddressSanitizer: heap-buffer-overflow
SUMMARY: AddressSanitizer: heap-buffer-overflow
==1234==ABORTING
artifact_prefix='./artifacts/'; Test unit written to ./artifacts/crash-abc123
```

### Crash Workflow

#### 1. Crash Saved

```bash
# Crash artifact saved
ls artifacts/fuzz_toml_parser/
# crash-abc123
# timeout-def456
# oom-ghi789
```

#### 2. Reproduce Crash

```bash
# Reproduce the exact crash
cargo +nightly fuzz run fuzz_toml_parser \
    artifacts/fuzz_toml_parser/crash-abc123

# Debug with more info
RUST_BACKTRACE=1 cargo +nightly fuzz run fuzz_toml_parser \
    artifacts/fuzz_toml_parser/crash-abc123
```

#### 3. Create Regression Test

```rust
// In tests/fuzz/crash_reproduction_tests.rs

#[test]
fn test_crash_abc123_toml_parser_overflow() {
    // Load crash artifact
    let input = include_bytes!("../artifacts/fuzz_toml_parser/crash-abc123");
    let input_str = std::str::from_utf8(input).unwrap();

    // Parse should not panic
    let result = parse_toml_config(input_str);

    // Should return an error, not crash
    assert!(result.is_err());
}
```

#### 4. Fix the Bug

```rust
// Before (crashes)
pub fn parse_toml_config(input: &str) -> Result<Config> {
    let value: toml::Value = toml::from_str(input)?;
    // Missing bounds check causes crash
    Ok(convert_value(value))
}

// After (handles gracefully)
pub fn parse_toml_config(input: &str) -> Result<Config> {
    let value: toml::Value = toml::from_str(input)
        .map_err(|e| CleanroomError::InvalidToml { .. })?;

    // Add validation
    validate_toml_structure(&value)?;

    Ok(convert_value(value))
}
```

#### 5. Verify Fix

```bash
# Run regression test
cargo test test_crash_abc123_toml_parser_overflow

# Re-run fuzzer on artifact
cargo +nightly fuzz run fuzz_toml_parser \
    artifacts/fuzz_toml_parser/crash-abc123

# Add to corpus
mv artifacts/fuzz_toml_parser/crash-abc123 \
   corpus/fuzz_toml_parser/regression_abc123
```

### Crash Types

**Crash (Panic)**:
- Indicates bug in error handling
- Should return Result instead
- High priority fix

**Timeout (Hang)**:
- Potential infinite loop
- ReDoS in regex
- Medium priority, optimize algorithm

**Out of Memory (OOM)**:
- Unbounded allocation
- Memory leak
- Add resource limits

## CI/CD Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/fuzz.yml
name: Fuzz Testing

on:
  schedule:
    - cron: '0 0 * * *'  # Nightly
  workflow_dispatch:      # Manual trigger

jobs:
  fuzz:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target:
          - fuzz_toml_parser
          - fuzz_scenario_dsl
          - fuzz_cli_args
          - fuzz_error_handling
          - fuzz_regex_patterns

    steps:
      - uses: actions/checkout@v3

      - name: Install nightly Rust
        uses: dtolnay/rust-toolchain@nightly

      - name: Install cargo-fuzz
        run: cargo install cargo-fuzz

      - name: Create corpus directory
        run: mkdir -p tests/fuzz/corpus/${{ matrix.target }}

      - name: Run fuzzer
        working-directory: tests/fuzz
        run: |
          cargo +nightly fuzz run ${{ matrix.target }} \
            -- -max_total_time=300 \
            -timeout=10

      - name: Check for crashes
        if: failure()
        run: |
          echo "Crashes found!"
          ls -la tests/fuzz/artifacts/${{ matrix.target }}/

      - name: Upload crash artifacts
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: fuzz-crashes-${{ matrix.target }}
          path: tests/fuzz/artifacts/${{ matrix.target }}/

      - name: Upload corpus
        uses: actions/upload-artifact@v3
        with:
          name: corpus-${{ matrix.target }}
          path: tests/fuzz/corpus/${{ matrix.target }}/
```

### Continuous Fuzzing

For long-term fuzzing, consider:

**OSS-Fuzz Integration**:
- Google's continuous fuzzing service
- Free for open source projects
- 24/7 fuzzing on powerful infrastructure
- Automatic bug reports

**Self-Hosted Fuzzing**:
```bash
# Run fuzzer indefinitely
while true; do
    cargo +nightly fuzz run fuzz_toml_parser -- -max_total_time=3600
done
```

## Best Practices

### 1. Fuzzing Duration

- **Smoke Test**: 30 seconds (quick check)
- **Development**: 5-10 minutes (local testing)
- **CI/CD**: 5-10 minutes per target
- **Nightly**: 1+ hour per target
- **Continuous**: 24/7 on dedicated servers

### 2. Target Selection

**High Priority Targets**:
- Parsers (TOML, JSON, CLI args)
- Input validators
- Regex patterns
- Serialization/deserialization
- Complex algorithms

**Medium Priority Targets**:
- Error handling
- String manipulation
- Format functions

**Low Priority Targets**:
- Simple getters/setters
- Builders (unless complex)
- Display implementations

### 3. Safety Considerations

```rust
// ✅ DO: Limit resource consumption
fuzz_target!(|input: FuzzInput| {
    let limited = input.data.iter().take(1000).collect();  // Limit size
    test_function(limited);
});

// ✅ DO: Sanitize dangerous inputs
fuzz_target!(|input: FuzzCommand| {
    let safe_commands = ["echo", "true", "false"];
    if safe_commands.contains(&input.cmd.as_str()) {
        execute_command(&input.cmd);
    }
});

// ❌ DON'T: Allow arbitrary system commands
fuzz_target!(|input: &[u8]| {
    let cmd = String::from_utf8_lossy(input);
    std::process::Command::new(&cmd).spawn();  // DANGEROUS!
});

// ❌ DON'T: Allow arbitrary file operations
fuzz_target!(|input: &[u8]| {
    let path = String::from_utf8_lossy(input);
    std::fs::remove_file(&path);  // DANGEROUS!
});
```

### 4. Performance Optimization

```bash
# Use multiple workers
cargo +nightly fuzz run target -- -workers=8

# Set memory limits
cargo +nightly fuzz run target -- -rss_limit_mb=2048

# Merge corpus periodically
cargo +nightly fuzz run target -- -merge=1

# Use persistent mode for speed
# (automatically enabled by libFuzzer)
```

### 5. Corpus Management

```bash
# Start with good seed corpus
cp examples/*.toml corpus/fuzz_toml_parser/

# Minimize regularly
cargo +nightly fuzz cmin fuzz_toml_parser

# Backup corpus before long runs
cp -r corpus corpus_backup

# Commit minimized corpus to git
git add corpus/
git commit -m "Update fuzz corpus"
```

## Troubleshooting

### Fuzzer Won't Start

**Error**: "Unable to find a seed corpus"

```bash
# Create corpus directory
mkdir -p corpus/fuzz_toml_parser

# Add seed file
echo '[test]\nname = "test"' > corpus/fuzz_toml_parser/seed.toml

# Or run without corpus
cargo +nightly fuzz run fuzz_toml_parser -- -seed=1
```

### Low Execution Speed

**Problem**: < 1000 exec/sec

**Solutions**:
```bash
# Check if running in debug mode
cargo +nightly fuzz build --release

# Reduce corpus size
cargo +nightly fuzz cmin fuzz_toml_parser

# Simplify target (remove expensive operations)
# Profile target with perf
```

### High Memory Usage

**Problem**: Fuzzer uses too much RAM

```bash
# Set memory limit
cargo +nightly fuzz run target -- -rss_limit_mb=1024

# Reduce max input size
cargo +nightly fuzz run target -- -max_len=4096

# Limit corpus size
cargo +nightly fuzz cmin target
```

### No New Coverage

**Problem**: Fuzzer not finding new paths

**Solutions**:
- Add more diverse seed inputs
- Run longer
- Use dictionary for structured inputs
- Try different mutation strategies
- Review code coverage report

### Crash Artifacts Too Large

**Problem**: Many crash artifacts filling disk

```bash
# Keep only unique crashes
cargo +nightly fuzz run target -- -artifact_prefix=artifacts/

# Periodically clean up
rm artifacts/target/timeout-*  # Remove timeouts
rm artifacts/target/oom-*      # Remove OOMs

# Keep only unique crashes (by stack trace)
```

## Metrics and Monitoring

### Coverage Metrics

```bash
# Generate coverage report
cargo +nightly fuzz coverage fuzz_toml_parser

# View with llvm-cov
cargo cov -- show \
    target/x86_64-unknown-linux-gnu/release/fuzz_toml_parser \
    -instr-profile=coverage/fuzz_toml_parser/coverage.profdata \
    -format=html > coverage.html
```

### Performance Metrics

Monitor:
- Executions per second (target: 10,000+)
- Coverage percentage (target: 80%+)
- Corpus size (keep minimal)
- Unique crashes found
- Time to find crashes

### Example Output

```
#1000: NEW    cov: 245 ft: 567 corp: 23 exec/s: 12450 rss: 128Mb
#2000: pulse  cov: 245 ft: 567 corp: 23 exec/s: 12234 rss: 134Mb
#3000: NEW    cov: 246 ft: 570 corp: 24 exec/s: 12118 rss: 142Mb

Legend:
- NEW: Found new coverage
- pulse: Status update
- cov: Coverage (edge coverage)
- ft: Features (unique paths)
- corp: Corpus size
- exec/s: Executions per second
- rss: Memory usage
```

## Resources

- [libFuzzer Documentation](https://llvm.org/docs/LibFuzzer.html)
- [Rust Fuzz Book](https://rust-fuzz.github.io/book/)
- [cargo-fuzz Guide](https://github.com/rust-fuzz/cargo-fuzz)
- [OSS-Fuzz](https://google.github.io/oss-fuzz/)
- [Fuzzing Examples](https://github.com/rust-fuzz/trophy-case)

---

**Last Updated**: 2025-10-16
**Version**: 1.0.0
**Maintained By**: CLNRM Security Team
