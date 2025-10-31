# clnrm Self-Test Suite: The ONLY Valid Proof

**Philosophy:** "We should be able to use clnrm itself to prove all clnrm commands work using TOML only"

---

## 🎯 Purpose

This test suite is the **ONLY valid proof** that clnrm works correctly:

- ✅ Uses clnrm to test clnrm ("eat our own dog food")
- ✅ Tests execute actual commands with real arguments
- ✅ Weaver validates telemetry behavior
- ❌ NO `--help` validation (proves nothing)
- ❌ NO README claims (can be outdated)
- ❌ NO unit tests only (can pass with broken features)

---

## 📊 Test Coverage

### File: `self-test-all-commands.clnrm.toml`

Tests ALL clnrm commands by actually executing them:

**Basic CLI Commands (6 tests):**
- `clnrm --version`
- `clnrm --help`
- `clnrm init`
- `clnrm validate`
- `clnrm plugins`
- `clnrm health`

**Core Functionality (2 tests):**
- `clnrm run`
- `clnrm self-test`

**Service Management (1 test):**
- `clnrm services status`

**Reporting (1 test):**
- `clnrm report`

**v0.7.0 Commands (10 tests):**
- `clnrm dev`
- `clnrm dry-run`
- `clnrm fmt`
- `clnrm lint`
- `clnrm diff`
- `clnrm record`
- `clnrm pull`
- `clnrm repro`
- `clnrm red-green`
- `clnrm render`

**OTEL Commands (4 tests):**
- `clnrm graph`
- `clnrm spans`
- `clnrm collector status`
- `clnrm analyze`

**Total: 24 command tests** (all commands covered)

---

## 🚀 Running the Self-Test Suite

```bash
# Run the comprehensive self-test suite
clnrm run tests/self-test-all-commands.clnrm.toml

# With Weaver validation (RECOMMENDED)
clnrm run tests/self-test-all-commands.clnrm.toml --validate

# With verbose output
clnrm run tests/self-test-all-commands.clnrm.toml -vvv
```

---

## ✅ What Success Looks Like

### Passing Test Suite

```
🚀 Executing test: clnrm_self_test_all_commands
📝 Description: Comprehensive self-test proving ALL clnrm commands work

✅ test_version_command - PASS
✅ test_help_command - PASS
✅ test_init_command - PASS
✅ test_validate_command - PASS
✅ test_plugins_command - PASS
✅ test_health_command - PASS
✅ test_self_test_command - PASS

⚠️  test_run_command_basic - EXPECTED FAILURE (plugin mismatch)
⚠️  test_dev_command - EXPECTED FAILURE (v0.7.0 unimplemented)
⚠️  test_dry_run_command - EXPECTED FAILURE (v0.7.0 unimplemented)
...

Test Results: 7 passed, 17 expected failures
✅ Weaver received 127 telemetry samples
📊 Registry coverage: 73.2%
✅ Weaver validation passed
```

### Failing Test Suite (Current Reality)

```
🚀 Executing test: clnrm_self_test_all_commands

❌ test_run_command_basic - FAIL
   Error: ValidationError: Unknown service plugin: alpine
   Root Cause: Plugin name mismatch

❌ test_dev_command - FAIL
   Error: unimplemented!()

Test Results: 7 passed, 17 failed
```

---

## 🚨 CRITICAL: False Positive Prevention

### ❌ What This Suite Does NOT Do

1. **Does NOT check `--help` text**
   - Help text can exist for non-functional commands
   - Proves nothing about actual behavior

2. **Does NOT trust unit tests**
   - Unit tests can pass with broken features
   - Only integration tests prove end-to-end workflows

3. **Does NOT rely on README claims**
   - Documentation can be outdated
   - Only actual execution proves functionality

4. **Does NOT use mocked/stubbed services**
   - Real containers, real commands, real behavior
   - If it doesn't work in production, test fails

### ✅ What This Suite DOES

1. **Executes every command with real arguments**
   - Not just `--help`
   - Actual command with actual inputs

2. **Verifies actual behavior**
   - Checks exit codes
   - Validates output patterns
   - Confirms expected effects

3. **Tests in production environment**
   - Uses real containers
   - Real Docker/Podman
   - Real telemetry emission

4. **Weaver validates telemetry**
   - Proves commands emit correct spans
   - Validates against schema
   - Detects fake-green scenarios

---

## 📊 Expected Results (Current v1.1.0)

### Commands That Should Pass

| Command | Status | Evidence |
|---------|--------|----------|
| `clnrm --version` | ✅ PASS | Returns version number |
| `clnrm --help` | ✅ PASS | Returns usage info |
| `clnrm init` | ✅ PASS | Creates project files |
| `clnrm validate` | ✅ PASS | Validates TOML syntax |
| `clnrm plugins` | ✅ PASS | Lists plugins |
| `clnrm health` | ✅ PASS | System health check |
| `clnrm self-test` | ✅ PASS | Framework self-tests |

### Commands That Should Fail (Known Issues)

| Command | Status | Reason |
|---------|--------|--------|
| `clnrm run` | ❌ FAIL | Plugin name mismatch |
| `clnrm dev` | ❌ FAIL | v0.7.0 unimplemented |
| `clnrm dry-run` | ❌ FAIL | v0.7.0 unimplemented |
| `clnrm fmt` | ❌ FAIL | v0.7.0 unimplemented |
| `clnrm lint` | ❌ FAIL | v0.7.0 unimplemented |
| All other v0.7.0 | ❌ FAIL | v0.7.0 unimplemented |

### Commands That Require Setup

| Command | Status | Requirement |
|---------|--------|-------------|
| `clnrm collector status` | 🚧 PARTIAL | Needs OTEL collector installed |
| `clnrm analyze` | 🚧 PARTIAL | Needs collector + trace files |
| `clnrm graph` | 🚧 PARTIAL | Needs trace files |
| `clnrm spans` | 🚧 PARTIAL | Needs trace files |

---

## 🔧 Fixing the Test Suite

### Priority 1: Fix Plugin Name Mismatch

**Current Problem:**
```toml
# Generated by clnrm init
[services.alpine]
image = "alpine:latest"
```

**Error:**
```
ValidationError: Unknown service plugin: alpine
```

**Fix Option A: Update clnrm init**
```toml
# Should generate:
[services.my_alpine]
type = "generic_container"
image = "alpine:latest"
```

**Fix Option B: Update Plugin Registry**
```rust
// Accept image-based lookups
if let Some(plugin) = registry.get(&service_type) {
    return plugin;
}

// Fallback: treat as generic container with image
if service_config.image.is_some() {
    return GenericContainerPlugin::new(&service_name, &service_config.image);
}
```

### Priority 2: Implement v0.7.0 Commands

**Options:**
1. Implement all v0.7.0 commands (large effort)
2. Remove help text for unimplemented commands (honest about limitations)
3. Mark as experimental in help text (set expectations)

**Recommendation:** Option 2 (remove help text for unimplemented)
- Prevents false positives
- Honest about current state
- Can add back when implemented

---

## 🎯 Success Criteria

This test suite PASSES when:

1. ✅ ALL basic commands execute successfully
2. ✅ `clnrm run` works end-to-end (after plugin fix)
3. ✅ Weaver validation passes (telemetry conforms to schema)
4. ✅ Zero false positives (no `--help` only validation)
5. ✅ Tests run in production environment (Homebrew installation)

---

## 📚 Related Documentation

- **Evaluation Report:** `docs/EVALUATION_REPORT.md` - Detailed analysis of what works vs claims
- **CLAUDE.md:** Anti-false-positive methodology and validation standards
- **README.md:** Honest feature matrix (updated based on this test suite)

---

## 🏆 The Gold Standard

**This test suite IS the gold standard for clnrm validation:**

- If this passes → clnrm works ✅
- If this fails → clnrm is broken ❌
- No other validation method is acceptable (all can produce false positives)

**Command to run the gold standard test:**
```bash
clnrm run tests/self-test-all-commands.clnrm.toml --validate
```

If this command succeeds with zero failures, clnrm is production-ready.

---

**Created:** 2025-10-31
**Purpose:** The ONLY valid proof that clnrm works
**Methodology:** Actual execution, no false positives
**Status:** 7/24 commands passing (29% working, 71% broken/unimplemented)
