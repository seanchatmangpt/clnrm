# Fake Implementation Scanner - Usage Guide

## Overview

The Fake Implementation Scanner is a code quality tool adapted from KCura's scanner to detect stub implementations, incomplete code, and anti-patterns in the Cleanroom Testing Framework.

## What It Detects

### 1. Unimplemented Code
- `unimplemented!()` macros
- `todo!()` macros
- `panic!()` calls (outside tests)

### 2. Fake Return Values
- Functions returning dummy/fake/stub/placeholder values
- Mock responses in production code
- Hardcoded/canned responses

### 3. Poor Error Handling
- `.unwrap()` in production code
- `.expect()` in production code

### 4. Clnrm-Specific Anti-Patterns
- `println!` in production code (should use `tracing` macros)
- Fake `ServiceHandle` implementations
- Empty `Ok(())` stub implementations

## Usage

### Run Standalone

```bash
# Scan entire project
bash scripts/scan-fakes.sh

# Scan specific directory
bash scripts/scan-fakes.sh crates/clnrm-core/src
```

### Run Full Validation Suite

```bash
# Run all best practices checks (format, clippy, fake scanner, OTEL build)
bash scripts/validate-best-practices.sh
```

### CI Integration

The fake scanner runs automatically in GitHub Actions:

```yaml
# .github/workflows/fast-tests.yml
fake-scanner:
  name: Fake Implementation Scanner
  runs-on: ubuntu-latest
  steps:
    - run: bash scripts/scan-fakes.sh
```

## Exit Codes

- `0` - No fake implementations detected (PASS)
- `1` - Fake implementations found (FAIL)

## Output Format

```
[INFO] Starting Cleanroom Fake Scanner v1.0
[INFO] Scanning: Unimplemented/TODO/panic macros
[SUCCESS] Clean: Unimplemented/TODO/panic macros
[INFO] Scanning: Fake/dummy/stub return values
[ERROR] Fake pattern detected: Fake/dummy/stub return values
----------------------------------------
crates/clnrm-core/src/example.rs:42:    Ok(fake_value)
----------------------------------------
[ERROR] Scan completed with issues
```

## Core Team Standards

### ❌ WRONG - Will Be Detected

```rust
// Fake return values
pub fn execute_test(&self) -> Result<()> {
    println!("Test executed");
    Ok(())  // Stub implementation!
}

// Using unwrap
let result = operation().unwrap();

// Incomplete code
pub fn start_service(&self) -> Result<ServiceHandle> {
    unimplemented!("needs implementation")
}
```

### ✅ CORRECT - Clean Code

```rust
// Proper error handling
pub fn execute_test(&self) -> Result<()> {
    let container = self.backend.create_container()?;
    container.execute_command(&self.steps)?;
    Ok(())
}

// Proper error propagation
let result = operation().map_err(|e| {
    CleanroomError::internal_error(format!("Operation failed: {}", e))
})?;

// Use tracing instead of println
tracing::info!("Starting service", service_name = %name);
```

## Configuration

### Timeout and Retries

Edit `scripts/scan-fakes.sh`:

```bash
readonly SCRIPT_TIMEOUT="10s"  # Timeout per pattern scan
readonly MAX_RETRIES=2         # Retry count on timeout
```

### Exclusions

Patterns automatically exclude:
- `scripts/**` - Utility scripts
- `target/**` - Build artifacts
- `.git/**` - Git metadata
- `*.md` - Documentation
- Test files for most checks

## Troubleshooting

### Scanner Times Out

Increase timeout:
```bash
readonly SCRIPT_TIMEOUT="30s"
```

### False Positives

Add pattern exclusions:
```bash
exclude_args+=(
    --glob '!path/to/exclude/**'
)
```

### Missing ripgrep

Install ripgrep:
```bash
# macOS
brew install ripgrep

# Ubuntu/Debian
sudo apt-get install ripgrep

# From source
cargo install ripgrep
```

## Integration with Definition of Done

Before code is production-ready, ALL must be true:

- [ ] `bash scripts/scan-fakes.sh` passes (no fake implementations)
- [ ] `cargo clippy -- -D warnings` shows zero issues
- [ ] `cargo fmt -- --check` passes
- [ ] `cargo build --release --features otel` succeeds
- [ ] `cargo test` passes completely
- [ ] No `.unwrap()` or `.expect()` in production code
- [ ] Proper `Result<T, CleanroomError>` error handling
- [ ] Tests follow AAA pattern with descriptive names
- [ ] No `println!` in production code (use `tracing` macros)

## Pre-commit Hook (Optional)

Add to `.git/hooks/pre-commit`:

```bash
#!/usr/bin/env bash
if ! bash scripts/scan-fakes.sh; then
    echo "❌ Fake implementations detected - commit blocked"
    exit 1
fi
```

## Performance

- **Scan time**: ~1-3 seconds for entire clnrm codebase
- **Memory**: Minimal (ripgrep is highly efficient)
- **Parallelization**: Scans run sequentially for clear output

## Support

- Report issues: https://github.com/seanchatmangpt/clnrm/issues
- See also: `docs/TESTING.md`, `CLAUDE.md`, `.cursorrules`
