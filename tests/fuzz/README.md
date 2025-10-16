# Fuzz Testing Infrastructure

This directory contains comprehensive fuzz testing infrastructure for the CLNRM project, targeting critical components like parsers, input handlers, and complex logic.

## Overview

Fuzz testing (fuzzing) is an automated testing technique that provides invalid, unexpected, or random data as inputs to a computer program. The goal is to discover security vulnerabilities, crashes, memory leaks, and edge cases that traditional testing might miss.

## Quick Start

### Prerequisites

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Ensure you have nightly Rust (required for libFuzzer)
rustup install nightly
```

### Running Fuzz Tests

```bash
# Navigate to fuzz directory
cd tests/fuzz

# Run a specific fuzz target
cargo +nightly fuzz run fuzz_toml_parser

# Run with custom options
cargo +nightly fuzz run fuzz_toml_parser -- -max_total_time=60

# Run all fuzz targets (one at a time)
cargo +nightly fuzz run fuzz_toml_parser -- -max_total_time=30
cargo +nightly fuzz run fuzz_scenario_dsl -- -max_total_time=30
cargo +nightly fuzz run fuzz_cli_args -- -max_total_time=30
cargo +nightly fuzz run fuzz_error_handling -- -max_total_time=30
cargo +nightly fuzz run fuzz_regex_patterns -- -max_total_time=30
```

## Fuzz Targets

### 1. `fuzz_toml_parser`

**Target**: TOML configuration parser (`config.rs`, `validate.rs`)

**Attack Surfaces**:
- Malformed TOML syntax
- Deeply nested structures
- Extremely long strings
- Unicode edge cases
- Duplicate keys
- Invalid escape sequences

**Expected Findings**:
- Parser panics
- Stack overflows
- Memory exhaustion
- Validation bypass

### 2. `fuzz_scenario_dsl`

**Target**: Scenario DSL builder (`scenario.rs`)

**Attack Surfaces**:
- Command injection attempts
- Resource exhaustion (infinite steps)
- Concurrent execution edge cases
- Step ordering issues

**Safety**: This fuzzer sanitizes commands to prevent actual system damage

### 3. `fuzz_cli_args`

**Target**: CLI argument parsing

**Attack Surfaces**:
- Path traversal vulnerabilities
- Buffer overflows in argument handling
- Argument injection
- Unicode and special character handling

### 4. `fuzz_error_handling`

**Target**: Error creation, chaining, and display

**Attack Surfaces**:
- Format string vulnerabilities
- Stack overflow in error chains
- Serialization issues
- Display/Debug trait panics

### 5. `fuzz_regex_patterns`

**Target**: Regex compilation and matching

**Attack Surfaces**:
- ReDoS (Regular Expression Denial of Service)
- Catastrophic backtracking
- Invalid regex syntax
- Memory exhaustion

## Corpus Management

### Initial Corpus

The `corpus/` directory contains seed inputs for each fuzz target:

```
corpus/
├── fuzz_toml_parser/
│   ├── valid_basic.toml
│   ├── valid_with_service.toml
│   ├── edge_empty_name.toml
│   ├── edge_unicode.toml
│   ├── malformed_unclosed_bracket.toml
│   └── ...
├── fuzz_regex_patterns/
│   ├── basic_pattern.txt
│   ├── redos_pattern.txt
│   └── ...
└── ...
```

### Growing the Corpus

Fuzzing automatically grows the corpus:

```bash
# Fuzz for longer to discover more interesting inputs
cargo +nightly fuzz run fuzz_toml_parser -- -max_total_time=3600

# The fuzzer will save interesting inputs to:
# tests/fuzz/corpus/fuzz_toml_parser/
```

### Managing Corpus Size

```bash
# Minimize corpus (remove redundant inputs)
cargo +nightly fuzz cmin fuzz_toml_parser

# Merge multiple corpus directories
cargo +nightly fuzz cmin -M corpus/fuzz_toml_parser corpus_backup/
```

## Crash Artifacts

When a crash is discovered:

1. **Artifact saved**: `fuzz/artifacts/fuzz_toml_parser/crash-<hash>`

2. **Reproduce crash**:
```bash
cargo +nightly fuzz run fuzz_toml_parser fuzz/artifacts/fuzz_toml_parser/crash-<hash>
```

3. **Create regression test**:
   - Add test to `crash_reproduction_tests.rs`
   - Document the crash scenario
   - Fix the underlying issue
   - Verify test passes

4. **Example**:
```rust
#[test]
fn test_crash_<issue_number>() {
    let input = include_bytes!("../artifacts/fuzz_toml_parser/crash-<hash>");
    let result = parse_toml_config(&String::from_utf8_lossy(input));
    // Should not panic
    let _ = result;
}
```

## Continuous Fuzzing Integration

### GitHub Actions

See `.github/workflows/fuzz.yml` for CI configuration:

```yaml
- name: Run fuzz tests
  run: |
    cd tests/fuzz
    cargo +nightly fuzz run fuzz_toml_parser -- -max_total_time=300
    cargo +nightly fuzz run fuzz_cli_args -- -max_total_time=300
```

### OSS-Fuzz Integration (Recommended)

For long-term continuous fuzzing, integrate with [OSS-Fuzz](https://github.com/google/oss-fuzz):

1. Submit project to OSS-Fuzz
2. Configure build scripts
3. Receive automatic crash reports
4. Get coverage reports

## Coverage Reports

Generate coverage from fuzzing:

```bash
# Generate coverage
cargo +nightly fuzz coverage fuzz_toml_parser

# View coverage report
cargo cov -- show fuzz/target/*/release/fuzz_toml_parser \
    -instr-profile=fuzz/coverage/fuzz_toml_parser/coverage.profdata \
    -format=html > coverage.html
```

## Best Practices

### 1. Fuzzing Duration

- **Quick smoke test**: 30 seconds per target
- **CI/CD integration**: 5-10 minutes per target
- **Thorough fuzzing**: 1+ hours per target
- **Continuous fuzzing**: Run 24/7 on dedicated servers

### 2. Memory Limits

```bash
# Limit memory to prevent OOM
cargo +nightly fuzz run fuzz_toml_parser -- -rss_limit_mb=2048
```

### 3. Timeout Detection

```bash
# Detect slow inputs (potential ReDoS)
cargo +nightly fuzz run fuzz_regex_patterns -- -timeout=1
```

### 4. Sanitizers

```bash
# Run with AddressSanitizer (default)
cargo +nightly fuzz run fuzz_toml_parser

# Run with UndefinedBehaviorSanitizer
RUSTFLAGS="-Zsanitizer=undefined" cargo +nightly fuzz run fuzz_toml_parser
```

## Interpreting Results

### No Crashes

- **Good**: Code appears robust
- **Action**: Run longer, expand corpus, try different fuzz targets

### Crashes Found

- **Parse errors**: Usually acceptable (invalid input rejected)
- **Panics**: Need investigation and fixes
- **Timeouts**: Potential ReDoS or performance issues
- **Memory issues**: Memory leaks or unbounded allocation

### Coverage Metrics

- **Target**: 80%+ code coverage from fuzzing
- **Focus**: Cover error handling paths
- **Priority**: High-risk code (parsers, validators)

## Troubleshooting

### "Unable to find a seed corpus"

```bash
# Ensure corpus directory exists
mkdir -p corpus/fuzz_toml_parser

# Add seed file
echo '[test.metadata]\nname = "test"\n\n[[steps]]\nname = "s"\ncommand = ["echo"]' > corpus/fuzz_toml_parser/seed1.toml
```

### "Sanitizer not found"

```bash
# Ensure nightly Rust with sanitizer support
rustup component add rust-src --toolchain nightly
```

### High Memory Usage

```bash
# Limit memory
cargo +nightly fuzz run <target> -- -rss_limit_mb=1024
```

## Performance Benchmarks

Expected fuzzing performance (approximate):

- **TOML parser**: 50,000-100,000 exec/sec
- **CLI args**: 200,000-500,000 exec/sec
- **Regex patterns**: 10,000-50,000 exec/sec
- **Error handling**: 500,000+ exec/sec

Lower performance may indicate:
- Complex parsing logic
- Slow validation
- Inefficient corpus

## Security Disclosure

If fuzzing discovers a security vulnerability:

1. **Do not** commit crash artifacts to public repo
2. Create private security advisory on GitHub
3. Follow responsible disclosure timeline
4. Coordinate with maintainers for patch

## References

- [libFuzzer Documentation](https://llvm.org/docs/LibFuzzer.html)
- [Rust Fuzz Book](https://rust-fuzz.github.io/book/)
- [cargo-fuzz Guide](https://github.com/rust-fuzz/cargo-fuzz)
- [OSS-Fuzz](https://google.github.io/oss-fuzz/)

## License

Same as parent project (see root LICENSE file)
