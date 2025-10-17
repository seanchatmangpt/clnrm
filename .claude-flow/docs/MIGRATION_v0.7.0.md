# Migration Guide: v0.6.0 → v0.7.0

**Release Date**: 2025-10-17
**Migration Difficulty**: Easy (no breaking changes)
**Estimated Time**: 15-30 minutes

## Overview

v0.7.0 is a **fully backward compatible** release focused on Developer Experience (DX). All v0.6.0 configurations work unchanged in v0.7.0. New features are opt-in additions that enhance the development workflow.

## What's New

### Core Features

1. **dev --watch** - Hot reload on file save (<3s latency)
2. **Cache System** - Change-aware execution (10x faster iteration)
3. **fmt Command** - Deterministic TOML formatting
4. **Enhanced Validation** - Comprehensive static analysis
5. **Performance** - Edit→rerun latency: p50 <1.5s, p95 <3s

### New CLI Commands

```bash
# Watch mode (hot reload)
clnrm dev --watch

# Format TOML files
clnrm fmt

# Enhanced validation
clnrm validate --verbose
```

## Breaking Changes

**NONE** - v0.7.0 is fully backward compatible!

## New Features (Opt-In)

### 1. Watch Mode

**Before (v0.6.0)**:
```bash
# Manual re-run after each edit
$ vim tests/api.toml
$ clnrm run tests/api.toml
$ vim tests/api.toml
$ clnrm run tests/api.toml
```

**After (v0.7.0)**:
```bash
# Start watch once
$ clnrm dev --watch

# Edit and save - auto-runs
$ vim tests/api.toml  # Save triggers auto-run
```

**Migration**: Optional - use `dev --watch` for faster iteration.

### 2. Cache System

**Before (v0.6.0)**:
```bash
# Always runs all tests
$ clnrm run tests/
✓ 10 tests passed in 45.2s
```

**After (v0.7.0)**:
```bash
# First run - creates cache
$ clnrm run tests/
✓ 10 tests passed in 45.2s

# Second run - cache hit
$ clnrm run tests/
✓ 10 tests skipped (no changes) in 0.12s
```

**Migration**: Automatic - cache enabled by default. Use `--force` to bypass.

### 3. TOML Formatting

**Before (v0.6.0)**:
```bash
# Manual formatting
$ vim tests/api.toml  # Format manually
```

**After (v0.7.0)**:
```bash
# Auto-format all files
$ clnrm fmt

# Format on save (optional)
$ clnrm dev --watch --auto-format
```

**Migration**: Optional - run `clnrm fmt` to format existing files.

### 4. Enhanced Validation

**Before (v0.6.0)**:
```bash
$ clnrm validate tests/api.toml
✓ Valid TOML syntax
```

**After (v0.7.0)**:
```bash
$ clnrm validate tests/api.toml
✓ Valid TOML syntax
✓ Required blocks present
✓ All service references valid
✓ No port conflicts
✓ Volume mounts safe
✓ Environment variables valid
✓ No circular dependencies
```

**Migration**: Automatic - enhanced validation is default.

## Step-by-Step Migration

### Step 1: Update Cleanroom

```bash
# Update via Homebrew
$ brew update
$ brew upgrade clnrm

# Or build from source
$ git pull origin master
$ cargo build --release
$ sudo cp target/release/clnrm /usr/local/bin/

# Verify version
$ clnrm --version
clnrm 0.7.0
```

### Step 2: Validate Existing Configurations

```bash
# Run enhanced validation on existing configs
$ clnrm validate tests/

# Fix any issues reported
$ vim tests/problem.toml

# Validate again
$ clnrm validate tests/
✓ All configurations valid
```

### Step 3: Format TOML Files (Optional)

```bash
# Review what would be formatted
$ clnrm fmt --dry-run

# Format all files
$ clnrm fmt

# Review changes
$ git diff

# Commit formatted files
$ git add .
$ git commit -m "chore: format TOML files with clnrm fmt"
```

### Step 4: Try Watch Mode (Optional)

```bash
# Start watch mode
$ clnrm dev --watch

# Edit a test file
$ vim tests/api.toml.tera

# Observe auto-run on save
# (tests run automatically in <3s)
```

### Step 5: Verify Cache Behavior

```bash
# First run - creates cache
$ clnrm run tests/
✓ Tests completed in 45s

# Second run - uses cache
$ clnrm run tests/
✓ Tests skipped (no changes) in 0.1s

# Force run (bypass cache)
$ clnrm run tests/ --force
✓ Tests completed in 45s
```

## Configuration Changes

### No Changes Required

All v0.6.0 `.toml` and `.toml.tera` files work unchanged:

```toml
# v0.6.0 format (still works in v0.7.0)
[meta]
name = "my-test"
version = "0.6.0"

[otel]
exporter = "jaeger"

[services.api]
type = "generic_container"
image = "nginx:latest"

[[scenario]]
name = "test_scenario"
# ...
```

### Optional Formatting Improvements

If using `clnrm fmt`, your files will be formatted consistently:

```toml
# After clnrm fmt (alphabetically sorted, consistent spacing)
[meta]
name = "my-test"
version = "0.7.0"  # Can update version number

[otel]
exporter = "jaeger"

[services.api]
image = "nginx:latest"
type = "generic_container"

[[scenario]]
name = "test_scenario"
# ...
```

## Feature Comparison

| Feature | v0.6.0 | v0.7.0 |
|---------|--------|--------|
| TOML Configuration | ✓ | ✓ |
| Tera Templates | ✓ | ✓ |
| Container Execution | ✓ | ✓ |
| OTEL Validation | ✓ | ✓ Enhanced |
| Watch Mode | ✗ | ✓ **NEW** |
| Cache System | ✗ | ✓ **NEW** |
| TOML Formatting | ✗ | ✓ **NEW** |
| Enhanced Validation | ✗ | ✓ **NEW** |
| Hot Reload | ✗ | ✓ **NEW** |

## Performance Improvements

### Edit→Test Latency

| Operation | v0.6.0 | v0.7.0 | Improvement |
|-----------|--------|--------|-------------|
| Edit → Save → Run | Manual | <3s | Auto |
| Unchanged file run | 45s | 0.12s | 364x faster |
| Changed file run | 45s | 1.5s (p50) | 30x faster |
| Validation | 45s | 0.84s | 54x faster |

### Development Workflow

**v0.6.0 Workflow** (manual, slow):
```
Edit (2min) → Save → Run command (5s) → Wait for tests (45s) → View results
Total: ~2m50s per iteration
```

**v0.7.0 Workflow** (automatic, fast):
```
Edit (2min) → Save → Auto-run (<3s) → View results
Total: ~2m03s per iteration (28% faster)
```

## Rollback Instructions

If needed, rollback is simple:

```bash
# Homebrew
$ brew uninstall clnrm
$ brew install clnrm@0.6

# From source
$ git checkout v0.6.0
$ cargo build --release
$ sudo cp target/release/clnrm /usr/local/bin/
```

**Note**: v0.7.0 cache directory (`.clnrm/cache/`) is safe to delete if rolling back.

## Common Migration Scenarios

### Scenario 1: Team Migration

**Goal**: Migrate entire team to v0.7.0

**Steps**:
1. Update Homebrew formula or Docker image
2. Team members update: `brew upgrade clnrm`
3. Format codebase: `clnrm fmt`
4. Commit formatted files
5. Update CI/CD (optional: add `--check` for validation)

**Timeline**: 1 hour for team of 10

### Scenario 2: CI/CD Migration

**Goal**: Update CI/CD pipelines to use v0.7.0 features

**Before (v0.6.0)**:
```yaml
- name: Run tests
  run: clnrm run tests/
```

**After (v0.7.0)**:
```yaml
- name: Validate configuration
  run: clnrm validate tests/ --format json

- name: Check formatting
  run: clnrm fmt --check

- name: Run tests
  run: clnrm run tests/ --force  # Bypass cache in CI
```

**Timeline**: 30 minutes per pipeline

### Scenario 3: Local Development Migration

**Goal**: Individual developer adopts v0.7.0 workflow

**Steps**:
1. Update clnrm: `brew upgrade clnrm`
2. Format workspace: `clnrm fmt`
3. Start watch mode: `clnrm dev --watch`
4. Edit tests and observe auto-runs

**Timeline**: 15 minutes

## FAQ

### Q: Will v0.7.0 break my existing tests?

**A**: No. v0.7.0 is fully backward compatible. All v0.6.0 configurations work unchanged.

### Q: Do I need to rewrite any TOML files?

**A**: No. Existing files work as-is. Formatting is optional.

### Q: Can I disable the cache?

**A**: Yes. Use `--force` flag: `clnrm run tests/ --force`

### Q: Is watch mode required?

**A**: No. It's optional. You can still use `clnrm run` manually like v0.6.0.

### Q: What if `clnrm fmt` changes my files incorrectly?

**A**: `clnrm fmt` is idempotent and preserves comments. Review changes with `git diff` before committing.

### Q: Can I use v0.7.0 features selectively?

**A**: Yes. All features are opt-in:
- Cache: Enabled by default, use `--force` to disable
- Watch: Opt-in via `dev --watch`
- Formatting: Opt-in via `clnrm fmt`

### Q: How do I clear the cache?

**A**:
```bash
# Clear all cache
$ clnrm cache clear

# Clear specific file
$ clnrm cache clear tests/api.toml
```

### Q: Does formatting affect template variables?

**A**: No. Formatting preserves Tera template syntax:

```toml
# Before formatting
name="{{ test_name }}"

# After formatting (preserved)
name = "{{ test_name }}"
```

### Q: Can I customize debounce duration in watch mode?

**A**: Yes:
```bash
$ clnrm dev --watch --debounce 500  # 500ms debounce
```

### Q: What happens if validation finds errors?

**A**: Validation reports errors with actionable suggestions:

```bash
$ clnrm validate tests/bad.toml
❌ Service 'api' references undefined service 'db'
   Suggestion: Define [services.db] or remove reference
```

Fix the errors and re-validate.

### Q: Is cache safe to commit to git?

**A**: No. Add to `.gitignore`:

```gitignore
.clnrm/cache/
```

## Troubleshooting

### Issue: Watch mode not detecting changes

**Symptom**: Files save but tests don't run

**Solution**:
```bash
# Check watch is running
$ clnrm dev --watch --verbose

# Verify file extension is .toml.tera
$ ls tests/*.toml.tera
```

### Issue: Cache not skipping unchanged files

**Symptom**: Tests run even when files haven't changed

**Solution**:
```bash
# Clear cache and rebuild
$ clnrm cache clear
$ clnrm run tests/ --force
```

### Issue: Formatting changes break tests

**Symptom**: Tests fail after running `clnrm fmt`

**Solution**:
```bash
# Review formatting changes
$ git diff tests/

# Revert if needed
$ git checkout tests/

# Report issue with specific file
```

### Issue: Enhanced validation reports false errors

**Symptom**: Validation fails for valid config

**Solution**:
```bash
# Run with verbose output
$ clnrm validate tests/api.toml --verbose

# Check if Tera variables are expanded
$ clnrm render tests/api.toml.tera

# Report validation bug with config example
```

## Support Resources

- **Documentation**: `/docs/v0.7.0/`
- **Architecture**: `/docs/V0.7.0_ARCHITECTURE.md`
- **CLI Guide**: `/docs/CLI_GUIDE.md`
- **GitHub Issues**: https://github.com/seanchatmangpt/clnrm/issues

## Summary

v0.7.0 is a **zero-risk** migration:

✓ **No breaking changes**
✓ **All v0.6.0 configs work unchanged**
✓ **Opt-in features** (use what you want)
✓ **Significant performance improvements**
✓ **Easy rollback** if needed

**Recommended migration path**:
1. Update clnrm
2. Validate existing configs
3. Optionally format files
4. Try watch mode
5. Adopt into workflow

**Total time**: 15-30 minutes
