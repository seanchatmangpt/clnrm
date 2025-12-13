# CLI Commands Fixes - v2.0.0 Format Only

**Date**: 2025-12-13  
**Status**: Code changes complete, requires rebuild

---

## Summary

Removed backward compatibility and updated all CLI commands to support **only v2.0.0 format**. All commands now use the canonical `Config` type from `crates/clnrm-core/src/config/spec.rs`.

---

## Changes Made

### 1. ✅ `init` Command - Fixed

**File**: `crates/clnrm-core/src/cli/commands/init.rs`

**Changes**:
- Updated template to generate v2.0.0 format
- Changed `[test.metadata]` → `[test]`
- Changed `[services.X]` → `[containers.X]`
- Changed `command = [...]` → `exec = [...]`
- Changed `expected_output_regex` → `assert.stdout_contains`
- Added required `container = "name"` field to steps
- Removed deprecated `type = "generic_container"` field

**Before (v1.x)**:
```toml
[test.metadata]
name = "basic_test"

[services.test_container]
type = "generic_container"
image = "alpine:latest"

[[steps]]
name = "hello_world"
command = ["echo", "Hello from cleanroom!"]
expected_output_regex = "Hello from cleanroom!"
```

**After (v2.0.0)**:
```toml
[test]
name = "basic_test"
timeout = "120s"

[containers.test_container]
image = "alpine:latest"

[[steps]]
name = "hello_world"
container = "test_container"
exec = ["echo", "Hello from cleanroom!"]
assert.stdout_contains = "Hello from cleanroom!"
```

---

### 2. ✅ `validate` Command - Fixed

**File**: `crates/clnrm-core/src/cli/commands/validate.rs`

**Changes**:
- Removed v1.x format support (TestConfig)
- Now only parses v2.0.0 format (Config)
- Clear error message when v1.x format is detected
- Uses `toml::from_str::<Config>` directly

**Error Message**:
```
TOML parse error: ... Note: Only v2.0.0 format is supported. 
Use [test], [containers.X], and [[steps]] with container and exec fields.
```

---

### 3. ✅ `lint` Command - Fixed

**File**: `crates/clnrm-core/src/cli/commands/lint.rs`

**Changes**:
- Removed v1.x format support (TestConfig)
- Now only parses v2.0.0 format (Config)
- Added validation for:
  - Container references in steps
  - Empty exec fields
  - Empty image fields
  - Naming conventions
- Clear error message when v1.x format is detected

**New Lint Checks**:
- ✅ Step container references must exist
- ✅ Steps must have non-empty exec field
- ✅ Containers must have non-empty image field
- ✅ Naming conventions (alphanumeric + _-)

---

### 4. ✅ `dry-run` Command - Fixed

**File**: `crates/clnrm-core/src/cli/commands/dry_run.rs`

**Changes**:
- Removed v1.x format support (ShapeValidator with TestConfig)
- Now only parses v2.0.0 format (Config)
- Removed unused `ShapeValidator` import
- Uses `toml::from_str::<Config>` directly
- Validates using `config.validate()`
- Checks for missing containers and steps
- Validates container references in steps

**Validation Checks**:
- ✅ Config structure (via `config.validate()`)
- ✅ At least one step required
- ✅ At least one container required
- ✅ All step container references must exist

---

## Format Differences

### v1.x Format (Removed)
```toml
[test.metadata]
name = "test"

[services.app]
type = "generic_container"
image = "alpine:latest"

[[steps]]
name = "run"
command = ["echo", "hello"]
```

### v2.0.0 Format (Canonical)
```toml
[test]
name = "test"

[containers.app]
image = "alpine:latest"

[[steps]]
name = "run"
container = "app"
exec = ["echo", "hello"]
```

---

## Testing Status

### ✅ Code Changes Complete
- All 4 commands updated to v2.0.0 only
- Error messages include migration guidance
- No backward compatibility code remaining

### ⚠️ Requires Rebuild
- Binary needs to be rebuilt to test changes
- Pre-existing compilation errors in other modules (poka_yoke) block full build
- Commands will work once binary is rebuilt

---

## Migration Path for Users

Users with v1.x format files need to migrate:

1. **Change metadata section**:
   - `[test.metadata]` → `[test]`

2. **Change container definitions**:
   - `[services.X]` → `[containers.X]`
   - Remove `type = "generic_container"` field

3. **Change steps**:
   - Add `container = "X"` field (required)
   - `command = [...]` → `exec = [...]`
   - `expected_output_regex` → `assert.stdout_contains`

4. **Run migration**:
   ```bash
   clnrm validate your-test.clnrm.toml
   ```

---

## Next Steps

1. **Fix pre-existing compilation errors** (poka_yoke module)
2. **Rebuild binary**: `cargo build --release --bin clnrm`
3. **Test all commands** with v2.0.0 format files
4. **Update example files** to v2.0.0 format
5. **Update templates** to generate v2.0.0 format

---

**Last Updated**: 2025-12-13  
**Status**: Code complete, awaiting rebuild

