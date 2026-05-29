## 2026-05-29T03:35:27Z
You are the Worker agent "worker_implement_3" for the Cleanroom placeholder resolution project.
Your working directory is /Users/sac/clnrm/.agents/teamwork_preview_worker_implement_3.
Your mission:
1. Fix the compilation errors in the library unit tests (`cargo test --workspace --lib --tests`). Specifically, read the handoff report of Reviewer 2 at `/Users/sac/clnrm/.agents/teamwork_preview_reviewer_rev_2/handoff.md` which lists the compilation errors:
   - `crates/clnrm-core/src/cli/commands/run/container_executor.rs:772:32`: `StepAssertion` not found in `crate::config`.
   - `crates/clnrm-core/src/telemetry/semantic_conventions.rs:268:20`: use of unresolved module `clnrm_core`.
   - `crates/clnrm-core/src/backend/oci/config_parser.rs:286:37`: method `to_runtime_config` takes 3 arguments but 2 supplied.
   - `crates/clnrm-core/src/capabilities/scenario.rs:409:26`: `missing field allowed_effects` in `BackendCapability` initializer.
   - `crates/clnrm-core/src/cli/commands/run/live_check_executor.rs:92:9`: `missing field containers` in `TestConfig` initializer.
   - `crates/clnrm-core/src/policy.rs:658:36`: `no field level on type SecurityPolicy` (should be `security_level`).
   - `crates/clnrm-core/src/policy.rs:678:37`: `no field max_cpu_percent on type ResourcePolicy` (should be `max_cpu_usage_percent` or similar).
2. Fix any other test compilation errors that arise when running `cargo test --workspace --lib --tests`.
3. Verify that `cargo test --workspace --lib --tests` and `cargo test --workspace` both compile and pass cleanly.
4. Write your implementation report to handoff.md in your working directory.
5. Use send_message to notify the orchestrator (conversation ID: 20e5a9e8-d38a-4a86-b3b1-77bdba233792) when you are done.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT
hardcode test results, create dummy/facade implementations, or
circumvent the intended task. A Forensic Auditor will independently
verify your work. Integrity violations WILL be detected and your
work WILL be rejected.
