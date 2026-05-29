# Migration & Stabilization Plan: Post-GGEN Cleanup

## 1. Immediate Compilation Fixes (Blocking)
- [ ] **Fix Global Variable Error:**
    - File: `crates/clnrm-core/src/backend/engine.rs`
    - Action: Wrap the `let engine` initialization inside a `lazy_static!` block or a `fn` to avoid global variable declaration errors.
- [ ] **Resolve Unclosed Delimiter:**
    - File: `crates/clnrm-core/src/backend/oci/mod.rs`
    - Action: Locate the unclosed brace causing the syntax error and restore the structural integrity of the module.

## 2. Structural & Trait Consistency
- [ ] **Trait Derivation Audit:**
    - Systematically ensure that `Debug`, `Default`, `Eq`, and `Hash` are correctly implemented for:
        - `LayerManager`, `ConfigParser`, `RegistryClient`, `ImageCache`, `LocalImageStore`.
    - Use `#[derive(Debug)]` as a standard if the types allow.
- [ ] **Private Method Access:**
    - File: `crates/clnrm-core/src/testing/mod.rs`
    - Action: Fix access to `enhanced_error.context.len()` by using `Option::map_or(0, |c| c.len())` or similar idiomatic access.

## 3. Logic & Borrow Checker Refactor
- [ ] **Port Allocator Stability:**
    - The current `PortAllocator` is a placeholder. Complete the logic to ensure `allocate` and `release` correctly manage the internal `HashSet` without mutable borrow conflicts.
    - Resolve the `MutexGuard` issue by calling `release` on the `PortAllocator` instance itself, not the guard.
- [ ] **Log Buffer Cleanup:**
    - File: `crates/clnrm-core/src/service/logs.rs`
    - Action: Refactor the `drain` logic to avoid holding a mutable borrow while calculating `len()`. Use `logs.len()` first, then drain the specific range.

## 4. Final Validation
- [ ] **Dependency Audit:** Verify `Cargo.toml` is clean and all members are valid.
- [ ] **Build Check:** Run `cargo check --workspace` to ensure no remaining errors.
- [ ] **Test Suite:** Execute unit tests to ensure that the removal of `ggen` didn't break core `gVisor` logic.
