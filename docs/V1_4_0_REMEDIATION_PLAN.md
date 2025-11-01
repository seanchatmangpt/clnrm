# v1.4.0 Remediation Plan

**Status:** REQUIRED before production release
**Priority:** P0 (Blocks Release)
**Owner:** Development Team
**Target:** Complete before v1.4.0 release

---

## Quick Action Summary

**Fix Order (by priority):**

1. **P0: Fix Compilation Errors** (2-3 hours)
   - 5 files won't compile, blocking all validation

2. **P0: Fix Breaking API Changes** (2-4 hours)
   - Restore backward compatibility or document migration

3. **P1: Fix Clippy Errors** (30 minutes)
   - Dead code in tests blocking CI/CD

4. **P1: Fix Build Warnings** (1 hour)
   - Unused imports and dead code in production

5. **P2: Fix Unwrap/Expect** (4-6 hours)
   - 50+ violations across 30 files

---

## P0: Compilation Errors (BLOCKS EVERYTHING)

### Issue 1: ServicePlugin Trait Lifetime Mismatch

**Files Affected:**
- `crates/clnrm-core/examples/framework-self-testing/innovative-dogfood-test.rs:209`
- `crates/clnrm-core/examples/framework-self-testing/innovative-dogfood-test.rs:229`
- `examples/framework-self-testing/plugin_system_test.rs:167`
- `examples/framework-self-testing/plugin_system_test.rs:175`

**Error:**
```
error[E0195]: lifetime parameters or bounds on method `start` do not match the trait declaration
```

**Root Cause:**
The `ServicePlugin` trait definition was changed to add lifetimes, but example implementations weren't updated.

**Fix Option A (Recommended):** Revert trait to original signature
```rust
// In src/cleanroom.rs or wherever ServicePlugin is defined
pub trait ServicePlugin {
    fn start(&self) -> Result<ServiceHandle>;  // No lifetime
    fn stop(&self, handle: ServiceHandle) -> Result<()>;
}
```

**Fix Option B:** Update all implementations to match new signature
```rust
// In each example file
impl ServicePlugin for TestPlugin {
    fn start<'a>(&'a self) -> Result<ServiceHandle> {  // Add lifetime
        // ...
    }

    fn stop<'a>(&'a self, handle: ServiceHandle) -> Result<()> {
        // ...
    }
}
```

**Recommendation:** Use Option A unless lifetimes are absolutely required for v1.4.0 features.

---

### Issue 2: execute_in_container API Breaking Change

**Files Affected:**
- `crates/clnrm-core/examples/framework-self-testing/innovative-dogfood-test.rs:52`
- `crates/clnrm-core/examples/framework-self-testing/innovative-dogfood-test.rs:136`
- `crates/clnrm-core/examples/framework-self-testing/innovative-dogfood-test.rs:157`

**Error:**
```
error[E0061]: this method takes 4 arguments but 2 arguments were supplied
```

**Root Cause:**
Method signature changed from 2 to 4 parameters without updating callsites.

**Old Signature:**
```rust
async fn execute_in_container(
    &self,
    container_name: &str,
    command: &[String],
) -> Result<String>
```

**New Signature:**
```rust
async fn execute_in_container(
    &self,
    container_name: &str,
    command: &[String],
    working_dir: Option<&str>,          // NEW
    env_vars: Option<&HashMap<String, String>>, // NEW
) -> Result<String>
```

**Fix Option A (Recommended):** Make new parameters default to None
```rust
// Update all callsites to include None for new params
.execute_in_container(
    container_name,
    &["echo".to_string(), "test".to_string()],
    None,  // working_dir
    None,  // env_vars
)
```

**Fix Option B:** Provide default method with old signature
```rust
// Add deprecated shim
#[deprecated(since = "1.4.0", note = "Use execute_in_container_with_env instead")]
async fn execute_in_container(
    &self,
    container_name: &str,
    command: &[String],
) -> Result<String> {
    self.execute_in_container_with_env(container_name, command, None, None).await
}

async fn execute_in_container_with_env(
    &self,
    container_name: &str,
    command: &[String],
    working_dir: Option<&str>,
    env_vars: Option<&HashMap<String, String>>,
) -> Result<String> {
    // New implementation
}
```

**Recommendation:** Use Option B for backward compatibility, or Option A and bump to v2.0.0.

---

### Issue 3: CliConfig Structure Breaking Changes

**Files Affected:**
- `crates/clnrm-core/tests/integration_concurrency_limiting.rs:27-34`

**Errors:**
```
error[E0308]: mismatched types - verbose: expected `u8`, found `bool`
error[E0308]: mismatched types - format: expected `OutputFormat`, found `String`
error[E0560]: struct has no field named `otel_exporter`, `custom_registry`, `live_check`, etc.
```

**Root Cause:**
Major refactoring of `CliConfig` structure without updating test code.

**Old Structure:**
```rust
pub struct CliConfig {
    pub verbose: bool,
    pub format: String,
    pub otel_exporter: Option<String>,
    pub custom_registry: Option<String>,
    pub live_check: bool,
    pub json_output: Option<String>,
    pub list_spans: bool,
    pub filter: Option<String>,
}
```

**New Structure:**
```rust
pub struct CliConfig {
    pub verbose: u8,  // Changed type
    pub format: OutputFormat,  // Changed type
    pub parallel: bool,  // New field
    pub watch: bool,  // New field
    pub force: bool,  // New field
    // Removed fields: otel_exporter, custom_registry, live_check, json_output, list_spans, filter
}
```

**Fix Option A (Recommended):** Update test to use new structure
```rust
// In integration_concurrency_limiting.rs
let config = CliConfig {
    verbose: 0,  // Change from false to 0
    format: OutputFormat::Human,  // Change from String to enum
    parallel: false,
    watch: false,
    force: false,
    digest: None,
    validate: false,
    // ... other fields
};
```

**Fix Option B:** Restore old fields with deprecation warnings
```rust
pub struct CliConfig {
    pub verbose: u8,
    pub format: OutputFormat,
    pub parallel: bool,
    pub watch: bool,
    pub force: bool,

    // Deprecated fields
    #[deprecated(since = "1.4.0", note = "No longer used")]
    pub otel_exporter: Option<String>,
    #[deprecated(since = "1.4.0", note = "No longer used")]
    pub custom_registry: Option<String>,
    // ... etc
}
```

**Recommendation:** Use Option A (update tests) since this is test-only code, not public API.

---

### Issue 4: Async Lifetime Issues

**Files Affected:**
- `crates/clnrm-core/tests/integration_async_plugins.rs:201`
- `crates/clnrm-core/tests/integration_async_plugins.rs:207`
- `crates/clnrm-core/tests/integration_async_plugins.rs:213`
- `crates/clnrm-core/tests/integration_async_plugins.rs:252`
- `crates/clnrm-core/tests/integration_concurrency_limiting.rs:664`

**Error:**
```
error[E0716]: temporary value dropped while borrowed
```

**Root Cause:**
Temporary array is created inline but borrowed beyond its lifetime in async context.

**Broken Code:**
```rust
let (result1, result2, result3) = tokio::join!(
    env.execute_in_container(
        "alpine",
        &["echo".to_string(), "test1".to_string()],  // Temporary dropped
    ),
    // ...
);
```

**Fix:**
```rust
// Create owned values before async block
let cmd1 = vec!["echo".to_string(), "test1".to_string()];
let cmd2 = vec!["echo".to_string(), "test2".to_string()];
let cmd3 = vec!["echo".to_string(), "test3".to_string()];

let (result1, result2, result3) = tokio::join!(
    env.execute_in_container("alpine", &cmd1, None, None),
    env.execute_in_container("alpine", &cmd2, None, None),
    env.execute_in_container("alpine", &cmd3, None, None),
);
```

**Files to Update:**
1. `tests/integration_async_plugins.rs` - Lines 198-217, 248-257
2. `tests/integration_concurrency_limiting.rs` - Line 660-667

---

## P1: Clippy Errors (BLOCKS CI/CD)

### Issue: Dead Code in Tests

**File:** `crates/clap-noun-verb/tests/unit.rs:51-52`

**Error:**
```
error: fields `name` and `about` are never read
```

**Fix:**
```rust
#[allow(dead_code)]  // Test fixture, fields used for construction
struct TestVerb {
    name: String,
    about: String,
}
```

**Alternative:** If fields are truly unused, either use them or remove them:
```rust
// Option 1: Use the fields
#[test]
fn test_verb_fields() {
    let verb = TestVerb {
        name: "test".to_string(),
        about: "Test description".to_string(),
    };
    assert_eq!(verb.name, "test");
    assert_eq!(verb.about, "Test description");
}

// Option 2: Remove unused fields
struct TestVerb {
    // Only keep fields that are actually used
}
```

---

## P1: Build Warnings (BLOCKS CLEAN BUILD)

### Issue 1: Unused Import - AtomicUsize

**File:** `crates/clnrm-core/src/backend/pool.rs:17`

**Warning:**
```
warning: unused import: `AtomicUsize`
```

**Fix:**
```rust
// Line 17 - Remove AtomicUsize
use std::sync::atomic::{AtomicU64, Ordering};  // Removed: AtomicUsize
```

---

### Issue 2: Unused Method - record_hit

**File:** `crates/clnrm-core/src/cli/commands/run/executor.rs:162`

**Warning:**
```
warning: method `record_hit` is never used
```

**Context:**
```rust
impl PoolMetrics {
    fn record_hit(&self) {  // NEVER CALLED
        self.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    fn record_miss(&self) {  // IS THIS CALLED?
        self.misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}
```

**Fix Option A (Recommended):** Use the method
```rust
// Find where pool metrics should be recorded and add:
pool_metrics.record_hit();  // On cache hit
pool_metrics.record_miss(); // On cache miss
```

**Fix Option B:** Mark as dead code if intentionally unused
```rust
#[allow(dead_code)]  // Reserved for future pool metrics tracking
fn record_hit(&self) {
    self.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}
```

**Fix Option C:** Remove if truly not needed
```rust
// Delete the method entirely if no plans to use it
```

---

## P2: Unwrap/Expect Violations (50+ instances)

### Strategy

**Goal:** Eliminate all `.unwrap()` and `.expect()` from production code paths.

**Approach:**
1. **Tests (15 instances):** Mark with `#[allow(clippy::unwrap_used)]` - acceptable in tests
2. **Impossible Errors (10 instances):** Replace with descriptive comments and `unreachable!()`
3. **Propagate Errors (25 instances):** Replace with `?` operator and proper error handling

### High-Priority Files (Production Code)

#### File: `src/backend/pool.rs`

**Line 305:**
```rust
// BEFORE
let container = container.expect("Container should exist");

// AFTER
let container = container.ok_or_else(|| {
    CleanroomError::internal_error("Container allocation failed: empty option")
})?;
```

**Lines 585, 600, 604, 608, 612:**
These are in `#[cfg(test)]` module - mark the test function:
```rust
#[tokio::test]
#[allow(clippy::unwrap_used, clippy::expect_used)]  // Test code
async fn test_pool_acquire_release_cycle() {
    // ... existing test code with expect() is fine
}
```

---

#### File: `src/determinism/ports.rs`

**Line 187:**
```rust
// BEFORE
.expect("Port allocator lock poisoned during clone")

// AFTER
.unwrap_or_else(|_| {
    // Lock poisoning means another thread panicked while holding lock
    // This is unrecoverable - create new allocator
    PortAllocator::new().expect("Failed to create new port allocator")
})
```

Or better:
```rust
.map_err(|e| CleanroomError::internal_error(
    format!("Port allocator lock poisoned: {}", e)
))?
```

---

#### File: `src/cli/commands/run/executor.rs`

**Line 257:**
```rust
// BEFORE
.expect("Semaphore closed unexpectedly");

// AFTER
.map_err(|_| CleanroomError::internal_error(
    "Concurrency semaphore closed unexpectedly. This indicates a critical runtime error."
))?
```

---

#### File: `src/cli/commands/run/live_check_executor.rs`

**Lines 125, 161:**
```rust
// BEFORE
let rt = tokio::runtime::Runtime::new().unwrap();

// AFTER
let rt = tokio::runtime::Runtime::new()
    .map_err(|e| CleanroomError::internal_error(
        format!("Failed to create tokio runtime: {}", e)
    ))?;
```

**Line 163:**
```rust
// BEFORE
config.weaver.as_mut().unwrap().enabled = false;

// AFTER
if let Some(weaver) = config.weaver.as_mut() {
    weaver.enabled = false;
}
```

---

#### File: `src/telemetry/live_check/stop_coordinator.rs`

**Lines 161, 173, 185, 205:**
```rust
// BEFORE
signal(SignalKind::interrupt()).expect("Failed to install SIGINT handler");

// AFTER
signal(SignalKind::interrupt()).map_err(|e| {
    CleanroomError::internal_error(format!("Failed to install SIGINT handler: {}", e))
})?;
```

---

#### File: `src/telemetry/live_check/port_allocator.rs`

**Line 389:**
```rust
// BEFORE
Self::new().expect("Failed to create default PortAllocator")

// AFTER
Self::new().unwrap_or_else(|e| {
    // This is Default impl - panicking is acceptable here
    // as it indicates system-level failure
    panic!("Failed to create default PortAllocator: {}", e)
})
```

Or change return type:
```rust
// Change Default impl to return Result
impl PortAllocator {
    pub fn default_or_error() -> Result<Self> {
        Self::new()
    }
}
```

---

### Automated Fix Script

Create `scripts/fix_unwrap.sh`:

```bash
#!/bin/bash
set -e

echo "Fixing unwrap/expect violations..."

# Fix specific patterns
find crates/clnrm-core/src -name "*.rs" -exec sed -i '' \
  's/\.expect("Semaphore closed unexpectedly")/\.map_err(|_| CleanroomError::internal_error("Semaphore closed"))?/g' {} \;

# Add test annotations
find crates/clnrm-core/src -name "*.rs" -exec sed -i '' \
  '/^#\[tokio::test\]/a\
#[allow(clippy::unwrap_used, clippy::expect_used)]
' {} \;

echo "Done. Review changes and test."
```

---

## Verification Checklist

After applying fixes:

```bash
# 1. Compilation
cargo build --release --features otel 2>&1 | tee build.log
grep -i "warning" build.log && echo "FAILED: Warnings found" || echo "PASSED: No warnings"

# 2. Clippy
cargo clippy --all-targets --all-features -- -D warnings
echo "Exit code: $?"

# 3. Tests
cargo test --all-features
echo "Exit code: $?"

# 4. Examples
cargo build --examples
echo "Exit code: $?"

# 5. Unwrap check
grep -r "\.unwrap()\|\.expect(" crates/clnrm-core/src --include="*.rs" | grep -v "^Binary\|#\[cfg(test)\]" > unwrap_violations.txt
wc -l unwrap_violations.txt

# 6. Weaver validation
weaver registry check -r registry/
weaver registry live-check --registry registry/
```

Expected results:
- ✅ Build: 0 warnings
- ✅ Clippy: exit code 0
- ✅ Tests: All pass
- ✅ Examples: Compile successfully
- ✅ Unwrap violations: 0 in production code
- ✅ Weaver: Both checks pass

---

## Timeline Estimate

| Task | Time | Priority |
|------|------|----------|
| Fix ServicePlugin trait | 30 min | P0 |
| Fix execute_in_container API | 1 hour | P0 |
| Fix CliConfig structure | 30 min | P0 |
| Fix async lifetime issues | 1 hour | P0 |
| Fix clippy dead code | 15 min | P1 |
| Fix build warnings | 30 min | P1 |
| Fix unwrap/expect (production) | 4 hours | P2 |
| Verification & testing | 2 hours | P1 |
| **TOTAL** | **9-10 hours** | |

**Recommended Approach:**
1. Day 1 Morning (4 hours): Fix all P0 compilation errors
2. Day 1 Afternoon (2 hours): Fix P1 warnings + verification
3. Day 2 (4 hours): Fix P2 unwrap/expect violations
4. Day 2 End: Re-run production certification

---

## Success Criteria

Before re-attempting production certification:

- [ ] `cargo build --release --features otel` - **0 warnings**
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` - **PASS**
- [ ] `cargo test --all-features` - **100% pass**
- [ ] `cargo build --examples` - **ALL compile**
- [ ] Unwrap/expect in production code - **0 instances**
- [ ] `weaver registry check -r registry/` - **PASS**
- [ ] `weaver registry live-check --registry registry/` - **PASS**
- [ ] Backward compatibility verified - **v1.3.0 code runs**

---

**Next Steps:**
1. Assign tasks to developers
2. Create feature branch: `fix/v1.4.0-compilation-errors`
3. Apply fixes in priority order
4. Run verification checklist after each priority level
5. Request re-certification after all fixes applied

**Contact:** Agent 15 (Production Validator) for certification re-run
