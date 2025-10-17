# Scripts Directory

## Overview

This directory contains utility scripts for the Cleanroom Testing Framework (clnrm).

## Available Scripts

### Core Quality Scripts

#### `scan-fakes.sh`
**Purpose**: Detects fake implementations, stub code, and anti-patterns in production code.

**Usage**:
```bash
# Scan entire project
bash scripts/scan-fakes.sh

# Scan specific directory
bash scripts/scan-fakes.sh crates/clnrm-core/src
```

**What it detects**:
- `unimplemented!()`, `todo!()`, `panic!()` macros
- Fake/dummy/stub/placeholder return values
- Hardcoded/canned responses
- `.unwrap()` and `.expect()` in production code
- `println!` in production code (should use `tracing`)
- Fake `ServiceHandle` implementations
- Empty `Ok(())` stub implementations

**Exit codes**:
- `0` = Clean (no issues)
- `1` = Issues found

#### `validate-best-practices.sh`
**Purpose**: Runs comprehensive quality checks following Core Team Standards.

**Usage**:
```bash
bash scripts/validate-best-practices.sh
```

**Checks performed**:
1. Fake implementation scanner
2. Format check (`cargo fmt`)
3. Clippy with strict warnings
4. OTEL features build

#### `test-fake-scanner.sh`
**Purpose**: Tests the fake scanner with known test cases.

**Usage**:
```bash
bash scripts/test-fake-scanner.sh
```

**Test cases**:
1. Detects `unimplemented!()` macros
2. Detects fake return values
3. Detects `println!` in production code
4. Passes clean code
5. Ignores test files appropriately

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
bash scripts/validate-best-practices.sh
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
- **validate-best-practices.sh**: ~30-60 seconds (includes clippy)
- **test-fake-scanner.sh**: ~5-10 seconds

## Support

- Documentation: `docs/FAKE_SCANNER_USAGE.md`
- Project README: `README.md`
- Core Team Standards: `.cursorrules`, `CLAUDE.md`
- Issues: https://github.com/seanchatmangpt/clnrm/issues
