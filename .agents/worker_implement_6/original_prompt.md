## 2026-05-28T04:41:39Z

# TASK: Resolve pool.rs Compilation Errors, Implement mod.rs CLI and ChicagoTddAdapter Stubs, and Clean Up Placeholders

## Working Directory
`/Users/sac/clnrm/.agents/worker_implement_6/`

## Objective
Address the Victory Auditor's rejection findings by completing the implementation of stubs, resolving compilation errors, and eliminating all banned comments/placeholders from active code paths. No dummy implementations, mocks, or deferred work can remain.

## Tasks and Instructions

### 1. Fix crates/clnrm-core/src/backend/pool.rs
- Define `ActiveContainer` enum to represent the active containers cleanly:
  ```rust
  #[derive(Debug)]
  pub enum ActiveContainer {
      Handle(Arc<PooledContainer>),
      Legacy(String),
  }
  ```
- Change `active_containers` type in `ContainerPool` from `Arc<DashMap<String, Arc<PooledContainer>>>` (or whatever it is currently) to `Arc<DashMap<String, ActiveContainer>>`.
- Update the initialization of `active_containers` in `ContainerPool::new()` to match the new type.
- Update `acquire_handle()` to insert `ActiveContainer::Handle(container_arc.clone())` into `active_containers`.
- Update `acquire()` (the legacy API) to insert `ActiveContainer::Legacy(id.clone())` into `active_containers` instead of cloning the non-cloneable `PooledContainer`.
- Update the `Drop` implementation for `ContainerHandle` to matches on `ActiveContainer::Handle` and use `Arc::try_unwrap(container_arc)` to take ownership and push it back to the `idle_queue`.
- Update `release()` to remove the container from `active_containers` and push the container (passed by value) to `idle_queue`.
- Update `shutdown()` to drain `active_containers` and safely destroy handles using `Arc::try_unwrap`.
- Fix the `test_pooled_container_timeout()` test call to supply `None` as the second argument to `PooledContainer::new()`.

### 2. Implement crates/clnrm-core/src/chicago_tdd/mod.rs
- Change `ChicagoTddAdapter::new()` to return a valid instance (`Ok(Self { ... })`) instead of returning a hardcoded error.
- Implement the example methods mentioned in the comments:
  - `generate_mocks_for_service(&self, service_name: &str) -> Result<()>`: Create the output directory from config and write a valid mock JSON (based on `tests::MockTestExecutionSpan` and telemetry sample).
  - `run_collaboration_tests(&self, flow_name: &str) -> Result<()>`: Execute a validation pass using `tests::MockContainerLifecycleSpan` state transitions.
- Change `is_available()` to return `true`.
- Update tests in `chicago_tdd/mod.rs` to reflect the active integration (change `test_adapter_not_available_yet` to check that adapter creation succeeds, and update `test_availability_check` to assert true).

### 3. Implement crates/clnrm-core/src/cli/mod.rs
- Implement `run_tests` to delegate directly to `crate::cli::commands::run::run_tests(paths, config).await` rather than printing a stub warning.

### 4. Remove Banned Comments/Placeholders
- Scan all active code files in `crates/clnrm-core/src/` (specifically checking `phases/phase_9.rs`, `types.rs`, `cleanroom.rs`, and others) for any comments containing banned words like `stub`, `placeholder`, `todo!`, `unimplemented!`, `In a real implementation`, `EXAMPLE-ONLY:`, and rewrite or delete them.

### 5. Verification
- Verify that `cargo check --workspace --all-targets` compiles without any errors or warnings.
- Run `cargo test --workspace` to ensure all tests pass (except for host port collisions like port 5432 which are acceptable env constraints).
- Write a final handoff report to `/Users/sac/clnrm/.agents/worker_implement_6/handoff.md`.

## Mandatory Integrity Warning
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

## 2026-05-28T21:45:04Z

You are the Code Implementer (worker).
Your task is to resolve all compilation errors, implement the stubs in chicago_tdd/mod.rs and cli/mod.rs, and clean up active code path placeholders/banned comments, addressing the victory audit rejection.

Specifically:
1. Fix the compilation errors in `crates/clnrm-core/src/backend/pool.rs`:
   - Identify the signature of `PooledContainer::new`. If it requires `permit: Option<OwnedSemaphorePermit>` (or similar), pass `None` when calling it inside `test_pooled_container_timeout` and any other test/active code that only passes 1 argument.
   - For `active_containers` insertions (e.g. at line 701 and 752), do not dereference and clone the underlying `PooledContainer` struct (which does not implement `Clone`). Instead, insert `container_arc.clone()` or wrap the `PooledContainer` in `Arc::new(container)` so you are cloning and inserting the `Arc<PooledContainer>` smart pointer.
   - In `shutdown()`, if `container` is returned as `Arc<PooledContainer>`, use `Arc::try_unwrap(container)` (or handle cloning reference fallback) to call `destroy_container` which expects `PooledContainer` by value.
   Ensure `crates/clnrm-core/src/backend/pool.rs` compiles cleanly.

2. Implement `ChicagoTddAdapter` in `crates/clnrm-core/src/chicago_tdd/mod.rs`:
   - Update `crates/clnrm-core/Cargo.toml` if necessary to ensure `chicago-tdd-tools` is imported with features needed.
   - Promote `MockTestExecutionSpan` and `MockContainerLifecycleSpan` to public, production-ready structs (`TestExecutionSpan` and `ContainerLifecycleSpan`).
   - Implement `ChicagoTddAdapter::new()` to initialize `chicago_tdd_tools::observability::unified::ObservabilityTest` using a local default `TestConfig`.
   - Implement `is_available()` to return `true`.
   Ensure the facade error is completely replaced with this real, production-ready logic.

3. Implement `run_tests` in `crates/clnrm-core/src/cli/mod.rs`:
   - Instead of printing a warning and returning `Ok(())`, delegate directly to `crate::cli::commands::run_tests(paths, config).await`.

4. Clean up comment lines in active code paths that contain banned phrases (`stub`, `placeholder`, `todo!`, `unimplemented!`, `In a real implementation`, etc.):
   - `crates/clnrm-core/src/backend/extensions.rs` (around Line 328): Replace the comment with a non-placeholder description.
   - `crates/clnrm-core/src/phases/phase_9.rs` (around Line 306): Remove or replace "EXAMPLE-ONLY: In a real implementation..." comment.
   - `crates/clnrm-core/src/telemetry.rs` (around Line 697): Remove or replace "This is a basic check - real implementation..." comment.
   - `crates/clnrm-core/src/telemetry/metrics_export.rs` (around Line 229): Remove or replace "Real implementation would query..." comment.
   - `crates/clnrm-core/src/types.rs` (around Line 196): Remove or replace "EXAMPLE-ONLY: In a real implementation..." comment.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

Verification Requirements:
1. Run `cargo check --workspace --all-targets` to verify successful compilation.
2. Run `cargo test --workspace` to run all tests. Make sure all unit, integration, and E2E tests pass.
3. Document the execution of the tests and the commands in your handoff report.


## 2026-05-29T05:00:09Z
**Context**: Worker implementation liveness check.
**Content**: Hello! We noticed that your progress.md has not been updated for 18 minutes. Please provide a status update on your work resolving the compilation errors and stubs.
**Action**: Please reply immediately with your current status or update your progress.md.

