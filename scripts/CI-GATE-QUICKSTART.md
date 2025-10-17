# CI Gate Quick Start Guide

## Overview

Configuration-driven quality enforcement for clnrm. Runs 7 comprehensive checks to ensure production quality.

## Quick Usage

```bash
# Run all checks (recommended before commit)
bash scripts/ci-gate.sh

# Run specific check
bash scripts/ci-gate.sh --check critical_patterns

# Stop on first failure
bash scripts/ci-gate.sh --fail-fast

# Help
bash scripts/ci-gate.sh --help
```

## Available Checks

| Check | Description | Typical Runtime |
|-------|-------------|-----------------|
| `critical_patterns` | Detect unwrap, expect, panic | ~5s |
| `core_functions` | Verify required APIs exist | ~1s |
| `linting` | Run clippy with strict rules | ~60s |
| `compilation` | Test all feature combos | ~180s |
| `error_handling` | Verify Result usage | ~10s |
| `documentation` | Check doc coverage | ~45s |
| `coverage` | Ensure 85%+ coverage | ~120s |

## Exit Codes

- `0` - All checks passed
- `1` - General failure
- `2` - Configuration error
- `3` - Check failed

## Reports

After running, check:
- `target/ci-gate-report/report.md` - Human-readable summary
- `target/ci-gate-report/report.json` - Machine-readable results
- `target/ci-gate-report/*.log` - Detailed check logs

## Configuration

Edit `scripts/ci-gate-config.yaml` to customize:

```yaml
quality_gates:
  critical_patterns:
    patterns:
      - pattern: "\.unwrap\(\)"
        severity: "critical"
    exclude_paths:
      - "tests/"
      - "examples/"
```

## Integration

### Pre-commit Hook

Create `.git/hooks/pre-commit`:

```bash
#!/bin/bash
bash scripts/ci-gate.sh --fail-fast --check critical_patterns
bash scripts/ci-gate.sh --fail-fast --check linting
```

### GitHub Actions

Already integrated in `.github/workflows/fast-tests.yml`:

```yaml
- name: Run CI gate checks
  run: bash scripts/ci-gate.sh

- name: Upload CI gate report
  uses: actions/upload-artifact@v4
  with:
    name: ci-gate-report
    path: target/ci-gate-report/
```

## Common Issues

### Issue: "cargo-tarpaulin not installed"
**Solution**: Coverage check is optional. Install with:
```bash
cargo install cargo-tarpaulin
```

### Issue: "Clippy found violations"
**Solution**: Fix issues or add `#[allow(...)]` for false positives:
```rust
#[allow(clippy::unwrap_used)]
fn test_helper() {
    // test code can use unwrap
}
```

### Issue: "Critical patterns detected in tests/"
**Solution**: Add to `exclude_paths` in `ci-gate-config.yaml`:
```yaml
exclude_paths:
  - "tests/"
  - "examples/"
  - "your_test_dir/"
```

## Best Practices

1. **Run before commit**: Catch issues early
2. **Fix critical issues first**: unwrap, expect, panic
3. **Use --fail-fast during dev**: Save time
4. **Review reports**: Learn from violations
5. **Tune config**: Adjust thresholds as needed

## Examples

### Fast pre-push check
```bash
bash scripts/ci-gate.sh \
  --check critical_patterns \
  --check linting \
  --fail-fast
```

### Full validation before release
```bash
bash scripts/ci-gate.sh  # All checks
```

### Debug specific check
```bash
bash scripts/ci-gate.sh --check linting
cat target/ci-gate-report/clippy.log
```

### Custom config for strict mode
```bash
bash scripts/ci-gate.sh \
  --config scripts/ci-gate-strict.yaml
```

## Troubleshooting

**Check fails but report is unclear?**
- Look at detailed logs in `target/ci-gate-report/*.log`
- Run check in isolation: `--check <name>`
- Check exclude paths in config

**Performance issues?**
- Skip slow checks: `--check critical_patterns`
- Use `--fail-fast` to stop early
- Run compilation check separately

**False positives?**
- Add paths to `exclude_paths` in config
- Use `#[allow(...)]` attributes
- Adjust pattern severity to "warning"

## Support

- Full docs: `docs/implementation/ci-gate-implementation.md`
- Config reference: `scripts/ci-gate-config.yaml`
- GitHub workflow: `.github/workflows/fast-tests.yml`
