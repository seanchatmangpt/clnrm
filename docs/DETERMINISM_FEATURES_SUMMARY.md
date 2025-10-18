# Determinism Features Summary

**Quick Reference**: Determinism features in clnrm for reproducible testing

---

## Overview

Determinism features enable **fully reproducible test execution** by controlling:
1. **Time** - Freeze clock to specific timestamp
2. **Randomness** - Seed random number generators

---

## Status: ✅ Partial - Core Complete, Integration Needed

| Component | Status |
|-----------|--------|
| TOML Parsing | ✅ Complete |
| DeterminismEngine | ✅ Complete |
| Chaos Tests Config | ✅ Complete |
| Template Integration | ❌ Not Connected |

**Full Report**: See `DETERMINISM_VALIDATION_REPORT.md`

---

## Usage in TOML Files

### Basic Configuration

```toml
[meta]
name = "my_test"
version = "1.0.0"

[determinism]
seed = 42                            # Fixed random seed
freeze_clock = "2025-01-01T00:00:00Z"  # Frozen timestamp (RFC3339)
```

### Chaos Testing (Actual Example from Rosetta Stone)

```toml
# Container Failures Chaos Test
[meta]
name = "chaos_container_failures"
version = "1.0.1"

[determinism]
seed = 666  # Chaos seed for reproducibility
freeze_clock = "2025-01-01T00:00:00Z"

[service.chaos_victim]
type = "generic_container"
image = "alpine:latest"

[[scenario]]
name = "container_killed_mid_step"
service = "chaos_victim"
run = "sh -c 'sleep 2 && echo hello'"
```

**Result**: Every test run produces identical:
- Timestamps
- Random failures
- Trace spans
- Test artifacts

---

## Features

### 1. Frozen Clock ⏰

**Purpose**: Make `now_rfc3339()` template function return the same time on every call

**Configuration**:
```toml
[determinism]
freeze_clock = "2025-06-15T12:00:00Z"
```

**Current Status**: ✅ Engine works, ❌ Not integrated with templates

**What Works**:
- `DeterminismEngine.get_timestamp()` returns frozen time
- Validates RFC3339 format at parse time
- Multiple timezone formats supported

**What's Missing**:
- Template `now_rfc3339()` doesn't use the engine
- Manual integration required

### 2. Seeded Random Generation 🎲

**Purpose**: Make fake data generation deterministic

**Configuration**:
```toml
[determinism]
seed = 42
```

**Current Status**: ✅ Engine works, ❌ Not integrated with template functions

**What Works**:
- `DeterminismEngine.next_u64()` - deterministic random u64
- `DeterminismEngine.next_u32()` - deterministic random u32
- `DeterminismEngine.fill_bytes()` - deterministic byte arrays
- Same seed = identical random sequence

**What's Missing**:
- Fake data functions (`fake_name()`, `fake_email()`, etc.) don't use engine seed
- Manual integration required

---

## API

### DeterminismConfig (TOML)

```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeterminismConfig {
    pub seed: Option<u64>,
    pub freeze_clock: Option<String>,
}
```

**Methods**:
- `is_deterministic()` - Returns true if seed or freeze_clock set

### DeterminismEngine

```rust
use clnrm_core::determinism::{DeterminismEngine, DeterminismConfig};

// Create from config
let config = DeterminismConfig {
    seed: Some(42),
    freeze_clock: Some("2025-01-01T00:00:00Z".to_string()),
};

let engine = DeterminismEngine::new(config)?;

// Get frozen timestamp
let timestamp = engine.get_timestamp_rfc3339();
// Always returns "2025-01-01T00:00:00+00:00"

// Get deterministic random values
let random_val = engine.next_u64()?;
// Same seed = same value
```

**Methods**:
- `get_timestamp()` - Returns `DateTime<Utc>` (frozen or current)
- `get_timestamp_rfc3339()` - Returns RFC3339 string
- `next_u64()` - Returns deterministic u64
- `next_u32()` - Returns deterministic u32
- `fill_bytes(&mut [u8])` - Fills buffer with deterministic bytes
- `is_deterministic()` - Check if any features enabled
- `has_seed()` - Check if seed configured
- `has_frozen_clock()` - Check if clock frozen

---

## Test Coverage ✅

**Test File**: `crates/clnrm-core/tests/determinism_validation.rs`

```
running 9 tests
test test_determinism_config_parsing_from_toml ... ok
test test_determinism_validation_report ... ok
test test_determinism_engine_with_seed ... ok
test test_determinism_engine_with_freeze_clock ... ok
test test_template_now_rfc3339_freeze_integration ... ok
test test_chaos_concurrent_chaos_toml_has_determinism ... ok
test test_determinism_engine_with_both_features ... ok
test test_chaos_container_failures_toml_has_determinism ... ok
test test_template_rendering_with_freeze_clock ... ok

test result: ok. 9 passed; 0 failed; 0 ignored
```

**Run Tests**:
```bash
cargo test --test determinism_validation
```

---

## Real-World Examples

### Example 1: Reproducible Chaos Testing

**File**: `tests/chaos/container_failures.clnrm.toml`

```toml
[determinism]
seed = 666
freeze_clock = "2025-01-01T00:00:00Z"

[[scenario]]
name = "container_killed_mid_step"
chaos_injection = {
    type = "kill_container",
    delay_ms = 500,
    signal = "SIGKILL"
}
```

**Benefits**:
- Same chaos failures every run
- Debuggable edge cases
- Reproducible for CI/CD
- Trace spans with fixed timestamps

### Example 2: Concurrent Chaos with Different Seeds

**File**: `tests/chaos/concurrent_chaos.clnrm.toml`

```toml
[determinism]
seed = 670  # Different seed = different failure pattern
freeze_clock = "2025-01-01T00:00:00Z"

[[scenario]]
name = "concurrent_container_failures"
parallel_services = ["concurrent_01", "concurrent_02", "concurrent_03"]
```

---

## Integration Roadmap 🚧

### Current State (v1.0.1)

```
✅ DeterminismConfig - parses from TOML
✅ DeterminismEngine - provides frozen time + seeded RNG
✅ Chaos tests - configured with [determinism]
❌ Template integration - not connected
```

### Target State (v1.1.0 - Planned)

```
✅ DeterminismConfig - parses from TOML
✅ DeterminismEngine - provides frozen time + seeded RNG
✅ Chaos tests - configured with [determinism]
✅ Template integration - now_rfc3339() uses frozen time
✅ Fake data - all fake_*() functions use seeded RNG
✅ CLI support - --seed and --freeze-clock flags
```

### Required Work

**Priority 1**: Template Integration (1-2 days)
1. Pass `DeterminismEngine` to `TemplateRenderer`
2. Connect `now_rfc3339()` to engine
3. Connect all `fake_*()` functions to engine seed
4. Add integration tests

**Priority 2**: Documentation (4 hours)
1. Update TOML reference guide
2. Add cookbook examples
3. Document reproducible testing workflow

**Priority 3**: CLI Flags (2 hours)
1. `clnrm run --seed 42 tests/`
2. `clnrm run --freeze-clock "2025-01-01T00:00:00Z" tests/`
3. `clnrm run --deterministic tests/` (default seed + frozen clock)

---

## See Also

- **Full Validation Report**: `DETERMINISM_VALIDATION_REPORT.md`
- **Chaos Testing Guide**: `tests/chaos/CHAOS_ROSETTA_STONE_README.md`
- **Source Code**:
  - `crates/clnrm-core/src/determinism/mod.rs`
  - `crates/clnrm-core/src/config/types.rs`
  - `crates/clnrm-core/src/template/functions.rs`
- **Test Suite**: `crates/clnrm-core/tests/determinism_validation.rs`

---

**Last Updated**: 2025-01-17
**Version**: v1.0.1 (partial implementation)
**Next Milestone**: v1.1.0 (full integration)
