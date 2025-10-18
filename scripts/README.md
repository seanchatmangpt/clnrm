# Scripts Directory

## Overview

This directory contains utility scripts for the Cleanroom Testing Framework (clnrm). All scripts follow Rust core team best practices and the 80/20 principle, focusing on essential functionality for daily development workflow.

## Script Categories

### 🔍 Quality Assurance Scripts
Scripts for ensuring code quality, detecting anti-patterns, and enforcing best practices.

### 📊 Testing & Coverage Scripts
Scripts for running tests, generating coverage reports, and validating test suites.

### 🌐 Observability Scripts
Scripts for telemetry, logging, and OpenTelemetry integration validation.

### 🤖 AI & Development Tools
Scripts for AI-assisted development and coding assistance.

### 📚 Libraries & Examples
Reusable libraries and demonstration examples.

---

## Available Scripts

#### `scan-fakes.sh`
**Purpose**: Advanced fake implementation scanner that detects production code anti-patterns and enforces cleanroom testing principles.

**Core Functionality**:
- **Anti-pattern Detection**: Identifies `unimplemented!()`, `todo!()`, `panic!()` macros in production code
- **Fake Implementation Detection**: Finds dummy, stub, placeholder, and mock return values
- **Production Logging Issues**: Detects `println!` usage in production (should use `tracing`)
- **Error Handling Violations**: Identifies `.unwrap()` and `.expect()` in production code
- **CLNRM-Specific Checks**: Validates proper ServiceHandle implementations and `Ok(())` stubs

**Architecture**:
- **Safe Pattern Matching**: Uses ripgrep with timeouts and retry logic
- **Input Validation**: Sanitizes paths and prevents directory traversal
- **Structured Output**: Consistent error reporting with file locations
- **Performance Optimized**: Fast scanning with proper exclusions

**Usage**:
```bash
# Scan entire project (recommended for CI)
bash scripts/scan-fakes.sh

# Scan specific directory
bash scripts/scan-fakes.sh crates/clnrm-core/src

# Scan with custom timeout (for large projects)
TIMEOUT=30s bash scripts/scan-fakes.sh
```

**What it detects**:
- `unimplemented!()`, `todo!()`, `panic!()` macros
- Fake/dummy/stub/placeholder return values
- Hardcoded/canned responses
- `.unwrap()` and `.expect()` in production code
- `println!` in production code (should use `tracing`)
- Fake ServiceHandle implementations
- Empty `Ok(())` stub implementations

**Exit codes**:
- `0` = Clean (no issues found)
- `1` = Issues found (blocking)

**Performance**: ~1-3 seconds for full clnrm codebase

#### `check-best-practices.sh`
**Purpose**: Comprehensive best practices validation enforcing FAANG-level code standards.

**Core Functionality**:
- **Error Handling**: No `unwrap/expect` in production code (src/ only)
- **Trait Compatibility**: No async trait methods (maintains dyn compatibility)
- **Linting**: Zero clippy warnings with strict rules (-D warnings)
- **Test Patterns**: AAA (Arrange, Act, Assert) test compliance
- **Implementation Honesty**: No false green `Ok(())` stubs
- **Error Types**: Proper `Result<T, CleanroomError>` usage
- **Formatting**: Code formatting compliance

**Architecture**:
- **Modular Design**: Separate check functions for each validation
- **CI Integration**: Dedicated CI mode with strict error handling
- **Auto-fix Support**: Can automatically fix formatting and some clippy issues
- **Detailed Reporting**: Structured output with violation counts and locations

**Usage**:
```bash
# Run all checks (recommended for development)
bash scripts/check-best-practices.sh

# Auto-fix formatting and clippy issues
bash scripts/check-best-practices.sh --fix

# CI mode (strict, no colors, fail on warnings)
bash scripts/check-best-practices.sh --ci

# Verbose output for debugging
bash scripts/check-best-practices.sh --verbose
```

**Exit codes**:
- `0` = All checks passed
- `1` = Critical violations found
- `2` = Warning violations found (CI mode only)

**Performance**: ~30-60 seconds (includes clippy compilation)

#### `test-fake-scanner.sh`
**Purpose**: Unit tests for the fake scanner to ensure it correctly identifies anti-patterns.

**Core Functionality**:
- **Test Case Generation**: Creates temporary Rust projects with known anti-patterns
- **Validation Testing**: Verifies scanner correctly detects each pattern type
- **Regression Testing**: Ensures clean code passes validation
- **Test Isolation**: Proper cleanup of temporary test files

**Architecture**:
- **Isolated Testing**: Creates temporary directory structure for each test
- **Pattern Verification**: Tests each anti-pattern detection individually
- **Clean Code Validation**: Ensures legitimate code passes without false positives
- **Test File Handling**: Verifies test files are properly excluded from scanning

**Usage**:
```bash
# Run all scanner tests
bash scripts/test-fake-scanner.sh
```

**Test cases**:
1. Detects `unimplemented!()` macros in production code
2. Detects fake return values (`fake_handle()`, etc.)
3. Detects `println!` in production code
4. Passes clean code implementations
5. Ignores patterns in test files appropriately

**Exit codes**:
- `0` = All tests passed
- `1` = Some tests failed

**Performance**: ~5-10 seconds

## CI Integration

The fake scanner runs automatically in GitHub Actions as part of the `fast-tests.yml` workflow:

```yaml
fake-scanner:
  name: Fake Implementation Scanner
  runs-on: ubuntu-latest
  steps:
    - run: bash scripts/scan-fakes.sh
```

Failures upload artifacts for debugging.

## Requirements

### System Dependencies

- **ripgrep** (`rg`) - Fast pattern matching
  ```bash
  # macOS
  brew install ripgrep

  # Ubuntu/Debian
  sudo apt-get install ripgrep

  # From source
  cargo install ripgrep
  ```

### Rust Dependencies

- Rust 1.70+ toolchain
- `cargo` and `rustup`

## Development Workflow

### Pre-commit Validation

Run before committing changes:

```bash
bash scripts/check-best-practices.sh
```

### Adding New Checks

To add a new pattern to the fake scanner:

1. Edit `scripts/scan-fakes.sh`
2. Add pattern to `main()` function:
   ```bash
   if ! scan_pattern_safely \
       'your_pattern' \
       "Description of what you're checking" \
       "$VALIDATED_ROOT" \
       "$SCRIPT_TIMEOUT"; then
       exit_code=1
   fi
   ```
3. Add test case to `scripts/test-fake-scanner.sh`
4. Run tests to verify

## Core Team Standards Compliance

All scripts follow Core Team Standards:

- **Error handling**: Proper exit codes and error messages
- **Logging**: Structured output with severity levels
- **Input validation**: Sanitize and validate all inputs
- **Timeouts**: Protect against hanging operations
- **Retry logic**: Handle transient failures
- **Clean code**: Self-documenting with clear variable names

## Troubleshooting

### Scanner Times Out

Increase timeout in `scan-fakes.sh`:
```bash
readonly SCRIPT_TIMEOUT="30s"
```

### False Positives

Add exclusions to patterns:
```bash
exclude_args+=(
    --glob '!path/to/exclude/**'
)
```

### Scanner Not Finding Issues

1. Check ripgrep is installed: `which rg`
2. Verify file extensions are correct (`.rs` files)
3. Test pattern manually: `rg 'pattern' directory/`
4. Check default exclusions in `scan_pattern_safely()`

### CI Failures

1. Check uploaded artifacts in GitHub Actions
2. Run scanner locally: `bash scripts/scan-fakes.sh`
3. Review scan output for specific file/line numbers
4. Fix issues and re-run validation

## Performance

- **scan-fakes.sh**: ~1-3 seconds for full clnrm codebase
- **check-best-practices.sh**: ~30-60 seconds (includes clippy)
- **test-fake-scanner.sh**: ~5-10 seconds

## Support

- Documentation: `docs/FAKE_SCANNER_USAGE.md`
- Project README: `README.md`
- Core Team Standards: `.cursorrules`, `CLAUDE.md`
- Issues: https://github.com/seanchatmangpt/clnrm/issues
