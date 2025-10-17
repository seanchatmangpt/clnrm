# Quality Remediation Plan
**Based on:** Production Validation Report 2025-10-17
**Target:** v0.7.1 - Production Quality Release

## Quick Reference

**Current DoD Score:** 2/10 ✅✅⚠️⚠️⚠️❌❌❌❌❌

**Blockers:** 5 critical issues preventing production deployment

---

## Phase 1: Critical Blockers (MUST FIX)

### 1.1 ✅ COMPLETED: Fix CLI Non-Exhaustive Pattern Match

**File:** `crates/clnrm-core/src/cli/mod.rs:24`

**Status:** ✅ FIXED - Handler stubs added for all commands

**Verification:**
```bash
cargo build --release  # Should succeed now
```

---

### 1.2 Fix Template Default Implementation

**File:** `crates/clnrm-core/src/template/mod.rs:129`

**Current Code:**
```rust
impl Default for TemplateRenderer {
    fn default() -> Self {
        Self::new().expect("Failed to create default TemplateRenderer")  // ❌ PANIC RISK
    }
}
```

**Required Fix:**
```rust
impl TemplateRenderer {
    /// Creates a new TemplateRenderer with default configuration.
    ///
    /// # Errors
    /// Returns error if handlebars registration fails
    pub fn try_default() -> Result<Self> {
        Self::new()
    }

    /// Creates a TemplateRenderer with basic built-in functions only.
    ///
    /// This is infallible and suitable for Default implementation.
    pub fn minimal() -> Self {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);
        Self { handlebars }
    }
}

impl Default for TemplateRenderer {
    fn default() -> Self {
        Self::minimal()  // ✅ SAFE
    }
}
```

**Impact:** HIGH - Prevents panics in default initialization

**Verification:**
```bash
cargo test -p clnrm-core --lib template
```

---

### 1.3 Fix Clippy Violations in Production Code

#### 1.3.1 Fix len_zero

**File:** `crates/clnrm-core/src/cli/commands/v0_7_0/lint.rs:191`

```rust
// ❌ Current
assert!(result.errors.len() > 0);

// ✅ Fix
assert!(!result.errors.is_empty());
```

#### 1.3.2 Fix field_reassign_with_default

**File:** `crates/clnrm-core/src/watch/watcher.rs:380`

```rust
// ❌ Current
let mut cli_config = CliConfig::default();
cli_config.parallel = true;
cli_config.jobs = 4;

// ✅ Fix
let cli_config = CliConfig {
    parallel: true,
    jobs: 4,
    ..Default::default()
};
```

#### 1.3.3 Fix cloned_ref_to_slice_refs

**File:** `crates/clnrm-core/src/watch/mod.rs:322`

```rust
// ❌ Current
let result = determine_test_paths(&[test_file.clone()])?;

// ✅ Fix
let result = determine_test_paths(std::slice::from_ref(&test_file))?;
```

#### 1.3.4 Fix useless_vec

**File:** `crates/clnrm-core/src/cli/commands/v0_7_0/dev.rs:183`

```rust
// ❌ Current
let paths = vec![PathBuf::from(".")];

// ✅ Fix
let paths = [PathBuf::from(".")];
```

**Impact:** MEDIUM - Code quality and performance

**Verification:**
```bash
cargo clippy --all-targets -- -D warnings
```

---

### 1.4 Replace println! with tracing in Production Code

**Files Affected (20+ files):**
- `crates/clnrm-core/src/cli/commands/v0_7_0/*.rs`
- `crates/clnrm-core/src/cli/commands/*.rs`
- `crates/clnrm-core/src/cli/mod.rs`
- `crates/clnrm-core/src/cleanroom.rs`
- `crates/clnrm-core/src/policy.rs`
- `crates/clnrm-core/src/scenario.rs`

**Pattern to Apply:**

```rust
// ❌ WRONG
println!("Starting test execution");
println!("Error: {}", error);
println!("Warning: deprecated feature");

// ✅ CORRECT
use tracing::{info, error, warn, debug};

info!("Starting test execution");
error!("Execution failed: {}", error);
warn!("Using deprecated feature");
debug!("Internal state: {:?}", state);
```

**Special Case - User-Facing Output:**

For CLI commands that intentionally output to stdout:
```rust
use tracing::info;

// Still use println for intended user output
println!("✅ Tests passed: {}/{}", passed, total);

// But log internally
info!(passed = passed, total = total, "Test execution complete");
```

**Batch Fix Script:**

```bash
#!/bin/bash
# run from repo root

FILES=(
  "crates/clnrm-core/src/cli/commands/v0_7_0/record.rs"
  "crates/clnrm-core/src/cli/commands/v0_7_0/fmt.rs"
  "crates/clnrm-core/src/cli/commands/v0_7_0/diff.rs"
  "crates/clnrm-core/src/cli/commands/v0_7_0/lint.rs"
  "crates/clnrm-core/src/cli/commands/v0_7_0/dry_run.rs"
  "crates/clnrm-core/src/cli/commands/run.rs"
  "crates/clnrm-core/src/cli/commands/init.rs"
  # ... add all files
)

for file in "${FILES[@]}"; do
  echo "Processing: $file"
  # Add tracing import if not present
  if ! grep -q "use tracing::" "$file"; then
    sed -i '' '1i\
use tracing::{info, warn, error, debug};
' "$file"
  fi

  # Manual review required for each println! replacement
  echo "  - Review println! statements manually"
done
```

**Impact:** HIGH - Required for production observability

**Verification:**
```bash
# Should return empty
grep -r "println!" crates/clnrm-core/src/ --exclude-dir=examples | grep -v "// OK:" | grep -v "#\[cfg(test)\]"
```

---

### 1.5 Fix Example Async Trait Violations

**Files Affected:**
- `innovative-dogfood-test.rs` (7 errors)
- `custom-plugin-demo.rs` (7 errors)
- `framework-documentation-validator.rs` (4 errors)
- `distributed-testing-orchestrator.rs` (4 errors)
- `ai-powered-test-optimizer.rs` (9 errors)

**Root Cause:** Examples incorrectly implemented ServicePlugin trait with async methods

**Fix Template:**

```rust
// ❌ WRONG
impl ServicePlugin for MyPlugin {
    fn start(&self) -> Pin<Box<dyn Future<Output = Result<ServiceHandle>> + Send + '_>> {
        Box::pin(async move {
            // async implementation
        })
    }
}

// ✅ CORRECT
impl ServicePlugin for MyPlugin {
    fn start(&self) -> Result<ServiceHandle> {
        // Use block_in_place for async operations
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // async implementation here
                Ok(ServiceHandle {
                    id: Uuid::new_v4().to_string(),
                    service_name: self.name().to_string(),
                    metadata: HashMap::new(),
                })
            })
        })
    }

    fn stop(&self, handle: ServiceHandle) -> Result<()> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // async cleanup
                Ok(())
            })
        })
    }

    fn health_check(&self, handle: &ServiceHandle) -> HealthStatus {
        // Sync implementation
        HealthStatus::Healthy
    }
}
```

**Impact:** MEDIUM - Examples must demonstrate correct patterns

**Verification:**
```bash
cargo build --examples 2>&1 | grep "error\[E0053\]"  # Should be empty
```

---

## Phase 2: High Priority Improvements

### 2.1 Audit unwrap() Usage in Production Code

**Files to Review:**
1. `crates/clnrm-core/src/template/resolver.rs`
2. `crates/clnrm-core/src/template/context.rs`
3. `crates/clnrm-core/src/template/functions.rs`
4. `crates/clnrm-core/src/template/determinism.rs`
5. `crates/clnrm-core/src/cache/file_cache.rs`
6. `crates/clnrm-core/src/cache/memory_cache.rs`
7. `crates/clnrm-core/src/formatting/toml_fmt.rs`
8. `crates/clnrm-core/src/formatting/json.rs`
9. `crates/clnrm-core/src/backend/testcontainer.rs`
10. `crates/clnrm-core/src/services/otel_collector.rs`

**Review Checklist:**

For each `.unwrap()` or `.expect()`:
- [ ] Is this in test code? (Acceptable)
- [ ] Is this in a code example? (Lower priority)
- [ ] Is this in production code? (MUST FIX)
- [ ] Can this fail in normal operation?
- [ ] What is the proper error handling?

**Fix Pattern:**

```rust
// ❌ WRONG
let value = map.get("key").unwrap();

// ✅ CORRECT
let value = map.get("key")
    .ok_or_else(|| CleanroomError::config_error("Missing required key"))?;

// OR for non-critical paths with clear documentation
let value = map.get("key")
    .expect("BUG: Key must exist due to validation at initialization");
```

**Verification:**
```bash
# Find all unwrap/expect in production code
find crates/clnrm-core/src -name "*.rs" -exec grep -Hn "\.unwrap()\|\.expect(" {} \; | grep -v test | grep -v "#\[cfg(test)\]"
```

---

### 2.2 Fix Result Type Alias Usage

**Files:** Multiple examples

**Issue:**
```rust
// ❌ WRONG
async fn main() -> Result<(), CleanroomError>

// ✅ CORRECT
async fn main() -> Result<()>  // Uses type alias from crate::error
```

**Fix:**
1. Ensure `use clnrm_core::error::Result;` at top of file
2. Replace `Result<T, CleanroomError>` with `Result<T>`
3. Only use full form in trait definitions or when disambiguation needed

---

## Phase 3: Medium Priority Polish

### 3.1 Fix Example Compilation Errors

**Strategy:**
1. Fix async trait violations (Phase 1.5)
2. Fix unused imports in examples:
   - `observability-self-test.rs`: Remove `Duration`, `error`, `warn`
   - `surrealdb-ollama-integration.rs`: Remove unused imports
   - Other examples: Run `cargo clippy --fix --examples`

3. Fix clippy::int_plus_one:
   ```rust
   // ❌ WRONG (observability-self-test.rs:58)
   if updated >= initial + 1 {

   // ✅ CORRECT
   if updated > initial {
   ```

**Verification:**
```bash
cargo build --examples
cargo clippy --examples
```

---

### 3.2 Run Full Test Suite

**Prerequisites:**
- Phase 1 complete (compilation succeeds)
- Phase 2 complete (no unwrap in critical paths)

**Commands:**
```bash
# Run all tests
cargo test --all

# Run with verbose output
cargo test --all -- --nocapture

# Run framework self-test
cargo run -- self-test

# Check for false positives
cargo test 2>&1 | grep "test result" | grep "0 failed"
```

**Success Criteria:**
- All tests pass
- No panics from unwrap/expect
- Framework self-test succeeds

---

## Phase 4: CI/CD Integration

### 4.1 Update GitHub Actions

**File:** `.github/workflows/quality.yml` (create if missing)

```yaml
name: Quality Gates

on: [push, pull_request]

jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - name: Check formatting
        run: cargo fmt -- --check

      - name: Clippy (zero warnings)
        run: cargo clippy --all-targets -- -D warnings

      - name: Build release
        run: cargo build --release

      - name: Run tests
        run: cargo test --all

      - name: Check for unwrap/expect in production
        run: |
          ! find crates/*/src -name "*.rs" -exec grep -H "\.unwrap()" {} \; | grep -v test | grep -v "#\[cfg(test)\]"
          ! find crates/*/src -name "*.rs" -exec grep -H "\.expect(" {} \; | grep -v test | grep -v "#\[cfg(test)\]"

      - name: Check for println in production
        run: |
          ! grep -r "println!" crates/clnrm-core/src/ | grep -v "// OK:" | grep -v test

      - name: Framework self-test
        run: cargo run -- self-test
```

---

## Execution Timeline

### Sprint 1 (Days 1-2): Critical Blockers
- ✅ Day 1 AM: CLI pattern match (DONE)
- Day 1 PM: Template Default fix
- Day 2 AM: All clippy fixes
- Day 2 PM: println! → tracing migration

**Milestone:** Clippy passes with -D warnings

### Sprint 2 (Days 3-4): Example Fixes
- Day 3: Fix all async trait violations
- Day 4: Fix remaining example compilation errors

**Milestone:** `cargo build --examples` succeeds

### Sprint 3 (Day 5): Production Code Audit
- Morning: Audit unwrap/expect in 10 production files
- Afternoon: Fix critical unwrap/expect issues

**Milestone:** Zero unwrap/expect in hot paths

### Sprint 4 (Day 6): Testing & Verification
- Morning: Run full test suite, fix failures
- Afternoon: Framework self-test validation

**Milestone:** All tests pass

### Sprint 5 (Day 7): CI/CD & Documentation
- Morning: Set up GitHub Actions quality gates
- Afternoon: Update documentation

**Milestone:** CI enforces Definition of Done

---

## Success Metrics

### Definition of Done Checklist

- [x] `cargo build --release` succeeds (CLI fix done)
- [ ] `cargo clippy -- -D warnings` zero issues
- [ ] No unwrap/expect in production code
- [ ] All traits dyn compatible
- [ ] Proper Result<T> usage
- [ ] Tests follow AAA pattern
- [ ] No false positives
- [ ] No println! in production
- [ ] Framework self-test passes
- [ ] CI enforces all rules

**Target:** 10/10 ✅

---

## Risk Mitigation

### High-Risk Changes

1. **println! → tracing migration**
   - Risk: Breaking user-facing output
   - Mitigation: Keep println! for intentional CLI output, add tracing in parallel
   - Verification: Manual CLI testing of all commands

2. **Template Default removal**
   - Risk: Breaking existing code using Default::default()
   - Mitigation: Provide minimal() alternative, deprecation warning
   - Verification: Grep for TemplateRenderer::default() usage

3. **Example rewrites**
   - Risk: Examples no longer demonstrate intended patterns
   - Mitigation: Review each example's purpose before changing
   - Verification: Run examples, verify output matches intent

---

## Tracking

### GitHub Issues to Create

1. `[CRITICAL] Replace .expect() in TemplateRenderer::default()`
2. `[CRITICAL] Fix 4 clippy violations in production code`
3. `[CRITICAL] Replace println! with tracing in 20+ production files`
4. `[HIGH] Fix async trait violations in 5 examples`
5. `[HIGH] Audit unwrap/expect usage in 10 production files`
6. `[MEDIUM] Fix remaining example compilation errors`
7. `[MEDIUM] Add quality gates to CI/CD`

### Labels
- `quality` - Definition of Done compliance
- `production-blocker` - Prevents v0.7.1 release
- `technical-debt` - Non-blocking but important

---

## Notes

- Keep `examples/` lower priority than `src/`
- Test code can use unwrap/expect
- Document any intentional println! with `// OK: user-facing output`
- Run `cargo clippy --fix` carefully (review changes)

**Next Action:** Start with 1.2 (Template Default fix) since 1.1 is complete.
