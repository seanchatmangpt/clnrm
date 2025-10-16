# Fuzz Testing Guide for CLNRM

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Architecture](#architecture)
4. [Fuzz Targets](#fuzz-targets)
5. [Running Locally](#running-locally)
6. [CI Integration](#ci-integration)
7. [Handling Crashes](#handling-crashes)
8. [Best Practices](#best-practices)
9. [Advanced Topics](#advanced-topics)

## Overview

Fuzz testing is a critical component of CLNRM's security and reliability strategy. Our fuzzing infrastructure targets high-risk components where untrusted input is processed:

- **TOML Configuration Parsing**: Prevents malformed configs from crashing the system
- **CLI Argument Handling**: Protects against injection attacks and path traversal
- **Scenario DSL**: Ensures safe command execution and resource management
- **Error Handling**: Validates robust error propagation without panics
- **Regex Patterns**: Prevents ReDoS (Regular Expression Denial of Service) attacks

### Why Fuzz Testing?

Traditional unit tests validate known scenarios, but fuzz testing:

- **Discovers edge cases** that developers didn't anticipate
- **Finds security vulnerabilities** before attackers do
- **Improves code coverage** by exploring unusual code paths
- **Prevents regressions** through continuous automated testing
- **Validates assumptions** about input handling and error cases

## Quick Start

### Prerequisites

```bash
# Install cargo-fuzz (one-time setup)
cargo install cargo-fuzz

# Install Rust nightly toolchain
rustup install nightly
```

### Run All Fuzz Tests (Quick)

```bash
cd tests/fuzz
./run_local_fuzz.sh all 30
```

This runs all fuzz targets for 30 seconds each - perfect for local development.

### Run Specific Target

```bash
cd tests/fuzz
cargo +nightly fuzz run fuzz_toml_parser -- -max_total_time=60
```

## Architecture

### Directory Structure

```
tests/fuzz/
├── Cargo.toml                    # Fuzz project configuration
├── fuzz_targets/                 # Fuzz target implementations
│   ├── fuzz_toml_parser.rs       # TOML parser fuzzer
│   ├── fuzz_scenario_dsl.rs      # Scenario DSL fuzzer
│   ├── fuzz_cli_args.rs          # CLI argument fuzzer
│   ├── fuzz_error_handling.rs    # Error handling fuzzer
│   └── fuzz_regex_patterns.rs    # Regex fuzzer
├── corpus/                       # Seed inputs for fuzzing
│   ├── fuzz_toml_parser/
│   │   ├── valid_basic.toml
│   │   ├── edge_unicode.toml
│   │   └── malformed_*.toml
│   └── ...
├── artifacts/                    # Crash artifacts (generated)
├── crash_reproduction_tests.rs   # Regression tests for crashes
├── run_local_fuzz.sh            # Local fuzzing helper script
└── README.md                     # Detailed documentation
```

### Technology Stack

- **libFuzzer**: LLVM's coverage-guided fuzzer (industry standard)
- **cargo-fuzz**: Rust integration for libFuzzer
- **arbitrary**: Generate structured data from raw fuzzer bytes
- **sanitizers**: AddressSanitizer, UndefinedBehaviorSanitizer

## Fuzz Targets

### 1. TOML Parser Fuzzer

**File**: `fuzz_targets/fuzz_toml_parser.rs`

**Purpose**: Validates robustness of TOML configuration parsing

**Attack Vectors Tested**:
- Deeply nested structures (stack overflow)
- Extremely long strings (memory exhaustion)
- Malformed syntax (parser panics)
- Unicode edge cases (encoding issues)
- Duplicate keys (validation bypass)

**Example Run**:
```bash
cargo +nightly fuzz run fuzz_toml_parser -- -max_total_time=300
```

**Expected Findings**:
- ✅ Parse errors (acceptable - invalid input rejected)
- ❌ Panics (need fixes)
- ❌ Hangs/timeouts (need optimization)
- ❌ Memory leaks (need investigation)

### 2. Scenario DSL Fuzzer

**File**: `fuzz_targets/fuzz_scenario_dsl.rs`

**Purpose**: Tests scenario builder for command injection and resource exhaustion

**Safety Features**:
- Sanitizes commands to prevent system damage
- Limits step count to prevent infinite scenarios
- Bounds timeout values

**Attack Vectors Tested**:
- Command injection attempts
- Infinite step loops
- Concurrent execution edge cases
- Resource exhaustion

**Example Run**:
```bash
cargo +nightly fuzz run fuzz_scenario_dsl -- -max_total_time=300
```

### 3. CLI Arguments Fuzzer

**File**: `fuzz_targets/fuzz_cli_args.rs`

**Purpose**: Validates CLI argument parsing for security vulnerabilities

**Attack Vectors Tested**:
- Path traversal (`../../../etc/passwd`)
- Buffer overflows in argument handling
- Argument injection
- Unicode and special characters

**Example Run**:
```bash
cargo +nightly fuzz run fuzz_cli_args -- -max_total_time=300
```

### 4. Error Handling Fuzzer

**File**: `fuzz_targets/fuzz_error_handling.rs`

**Purpose**: Tests error creation, chaining, and display for panics

**Attack Vectors Tested**:
- Format string vulnerabilities
- Stack overflow in error chains
- Serialization edge cases
- Display/Debug trait panics

**Example Run**:
```bash
cargo +nightly fuzz run fuzz_error_handling -- -max_total_time=300
```

### 5. Regex Pattern Fuzzer

**File**: `fuzz_targets/fuzz_regex_patterns.rs`

**Purpose**: Prevents ReDoS (Regular Expression Denial of Service) attacks

**Attack Vectors Tested**:
- Catastrophic backtracking patterns: `(a+)+b`
- Invalid regex syntax
- Memory exhaustion in matching
- Extremely long input strings

**Example Run**:
```bash
cargo +nightly fuzz run fuzz_regex_patterns -- -max_total_time=300 -timeout=1
```

## Running Locally

### Quick Development Workflow

```bash
# 1. Make code changes to core library
vim crates/clnrm-core/src/config.rs

# 2. Run quick fuzz test (30 seconds)
cd tests/fuzz
./run_local_fuzz.sh fuzz_toml_parser 30

# 3. If crashes found, reproduce
cargo +nightly fuzz run fuzz_toml_parser artifacts/fuzz_toml_parser/crash-xyz

# 4. Fix the issue and re-run
./run_local_fuzz.sh fuzz_toml_parser 30
```

### Comprehensive Local Fuzzing

```bash
# Run all targets for 10 minutes each (thorough)
cd tests/fuzz
./run_local_fuzz.sh all 600

# Or run individually with custom settings
cargo +nightly fuzz run fuzz_toml_parser -- \
    -max_total_time=3600 \
    -rss_limit_mb=4096 \
    -jobs=4
```

### Monitoring Fuzzing Progress

```bash
# Terminal 1: Run fuzzer
cargo +nightly fuzz run fuzz_toml_parser

# Terminal 2: Monitor coverage and stats
watch -n 5 'cat fuzz-*.log | tail -20'
```

### Tips for Effective Local Fuzzing

1. **Start small**: 30-60 seconds for quick feedback
2. **Incremental**: Run after every significant change
3. **Focus**: Target the code you just modified
4. **Parallelize**: Use `-jobs=N` for faster coverage
5. **Memory limits**: Use `-rss_limit_mb` to prevent OOM

## CI Integration

### GitHub Actions Workflow

Our CI runs fuzzing automatically:

- **Pull Requests**: 30-second smoke test per target
- **Daily Schedule**: 30-minute thorough fuzzing
- **Manual Trigger**: Configurable duration

**Configuration**: `.github/workflows/fuzz.yml`

### Interpreting CI Results

#### ✅ All Checks Passed

No crashes found - code appears robust for the tested duration.

**Action**: None required, but consider running longer locally.

#### ❌ Fuzz Target Failed

Crashes were discovered during fuzzing.

**Actions**:
1. Download crash artifacts from CI
2. Reproduce locally: `cargo +nightly fuzz run <target> <artifact>`
3. Add regression test to `crash_reproduction_tests.rs`
4. Fix the underlying issue
5. Verify fix with local fuzzing

#### 📊 Coverage Report Available

Shows which code paths were exercised during fuzzing.

**Action**: Review uncovered paths and add corpus inputs.

## Handling Crashes

### Crash Discovery Workflow

1. **Fuzzer finds crash**:
   ```
   ==1234==ERROR: AddressSanitizer: heap-buffer-overflow
   SUMMARY: AddressSanitizer: heap-buffer-overflow
   ```

2. **Artifact saved**: `tests/fuzz/artifacts/fuzz_toml_parser/crash-abc123`

3. **Reproduce crash**:
   ```bash
   cargo +nightly fuzz run fuzz_toml_parser \
       tests/fuzz/artifacts/fuzz_toml_parser/crash-abc123
   ```

4. **Analyze crash**:
   ```bash
   # View crash input (if text)
   cat artifacts/fuzz_toml_parser/crash-abc123

   # Hex dump (if binary)
   hexdump -C artifacts/fuzz_toml_parser/crash-abc123
   ```

5. **Create regression test**:
   ```rust
   // In crash_reproduction_tests.rs
   #[test]
   fn test_crash_issue_123() {
       let input = include_bytes!("../artifacts/.../crash-abc123");
       let result = parse_toml_config(&String::from_utf8_lossy(input));
       // Should not panic after fix
       let _ = result;
   }
   ```

6. **Fix the issue** in source code

7. **Verify fix**:
   ```bash
   # Run regression test
   cargo test test_crash_issue_123

   # Re-run fuzzer
   cargo +nightly fuzz run fuzz_toml_parser -- -max_total_time=300
   ```

8. **Document in PR**:
   - Include crash artifact (if not sensitive)
   - Explain root cause
   - Link to regression test

### Common Crash Types

#### Panic / Unwrap on None

```rust
// Before (vulnerable)
let value = map.get("key").unwrap();

// After (safe)
let value = map.get("key")
    .ok_or_else(|| Error::missing_key("key"))?;
```

#### Stack Overflow

```rust
// Before (vulnerable to deep nesting)
fn parse_nested(value: &Value) -> Result<T> {
    if let Some(inner) = value.as_array() {
        parse_nested(&inner[0])  // Unbounded recursion
    }
}

// After (safe with depth limit)
fn parse_nested(value: &Value, depth: usize) -> Result<T> {
    if depth > MAX_DEPTH {
        return Err(Error::too_deep());
    }
    if let Some(inner) = value.as_array() {
        parse_nested(&inner[0], depth + 1)
    }
}
```

#### Memory Exhaustion

```rust
// Before (vulnerable to large allocations)
let mut buffer = Vec::with_capacity(input.len());

// After (safe with limits)
let max_size = 10 * 1024 * 1024; // 10MB
if input.len() > max_size {
    return Err(Error::too_large());
}
let mut buffer = Vec::with_capacity(input.len().min(max_size));
```

## Best Practices

### 1. Corpus Management

**Good corpus characteristics**:
- Mix of valid and invalid inputs
- Edge cases (empty, maximum size, unicode)
- Known problematic patterns
- Real-world examples

**Maintaining corpus**:
```bash
# Minimize corpus (remove redundant inputs)
cargo +nightly fuzz cmin fuzz_toml_parser

# Add custom seed
echo "..." > corpus/fuzz_toml_parser/custom_seed.toml
```

### 2. Fuzzing Duration

- **Development**: 30-60 seconds (quick feedback)
- **PR validation**: 5 minutes (catch obvious issues)
- **Nightly CI**: 30-60 minutes (thorough testing)
- **Continuous**: 24/7 on dedicated servers (OSS-Fuzz)

### 3. Resource Limits

Always set limits to prevent runaway fuzzing:

```bash
cargo +nightly fuzz run <target> -- \
    -max_total_time=300 \      # 5 minutes
    -rss_limit_mb=2048 \       # 2GB RAM
    -timeout=5 \               # 5s per input
    -max_len=100000            # 100KB input size
```

### 4. Sanitizer Selection

```bash
# AddressSanitizer (default, catches memory errors)
cargo +nightly fuzz run <target>

# UndefinedBehaviorSanitizer (catches UB)
RUSTFLAGS="-Zsanitizer=undefined" cargo +nightly fuzz run <target>

# MemorySanitizer (catches uninitialized memory)
RUSTFLAGS="-Zsanitizer=memory" cargo +nightly fuzz run <target>
```

### 5. Integration with Development

```bash
# Pre-commit hook (.git/hooks/pre-commit)
#!/bin/bash
cd tests/fuzz
./run_local_fuzz.sh all 30 || {
    echo "Fuzzing found issues. Fix before committing."
    exit 1
}
```

## Advanced Topics

### OSS-Fuzz Integration

For long-term continuous fuzzing, integrate with Google's OSS-Fuzz:

1. Submit project: https://github.com/google/oss-fuzz
2. Configure `project.yaml`
3. Receive daily crash reports
4. Get coverage reports

### Coverage-Guided Corpus Generation

```bash
# Generate coverage report
cargo +nightly fuzz coverage fuzz_toml_parser

# View coverage with llvm-cov
cargo cov -- show target/x86_64-unknown-linux-gnu/release/fuzz_toml_parser \
    --format=html > coverage.html
```

### Differential Fuzzing

Compare two implementations:

```rust
fuzz_target!(|data: &[u8]| {
    let input = String::from_utf8_lossy(data);

    let result1 = old_parser(&input);
    let result2 = new_parser(&input);

    // Both should agree or both should fail
    assert_eq!(result1.is_ok(), result2.is_ok());
});
```

### Structure-Aware Fuzzing

Use `arbitrary` crate for complex structures:

```rust
#[derive(Arbitrary, Debug)]
struct FuzzConfig {
    name: String,
    steps: Vec<FuzzStep>,
    timeout: Option<u64>,
}

fuzz_target!(|config: FuzzConfig| {
    // Fuzzer generates structurally valid configs
    test_config(config);
});
```

## Troubleshooting

### "No instrumentation found"

```bash
# Ensure using nightly
rustup default nightly

# Clean and rebuild
cargo clean
cargo +nightly fuzz run <target>
```

### High Memory Usage

```bash
# Limit RSS
cargo +nightly fuzz run <target> -- -rss_limit_mb=1024
```

### Slow Fuzzing Performance

- Simplify target (avoid expensive operations)
- Use faster corpus minimization
- Run with multiple jobs: `-jobs=4`
- Profile with `perf` to find bottlenecks

## References

- [Rust Fuzz Book](https://rust-fuzz.github.io/book/)
- [libFuzzer Documentation](https://llvm.org/docs/LibFuzzer.html)
- [cargo-fuzz GitHub](https://github.com/rust-fuzz/cargo-fuzz)
- [OSS-Fuzz](https://google.github.io/oss-fuzz/)

---

**Questions or Issues?** Open an issue on GitHub or contact the security team.
