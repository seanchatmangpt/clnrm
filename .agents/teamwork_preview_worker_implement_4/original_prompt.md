## 2026-05-28T20:53:03-07:00
You are the Worker agent "worker_implement_4" for the Cleanroom placeholder resolution project.
Your working directory is /Users/sac/clnrm/.agents/teamwork_preview_worker_implement_4.
Your mission:
1. Read the Explorer scan reports:
   - `/Users/sac/clnrm/.agents/teamwork_preview_explorer_scan_4/handoff.md` (Strategy for phase_9.rs, live_check_executor.rs, template_stubs.rs)
   - `/Users/sac/clnrm/.agents/teamwork_preview_explorer_scan_5/handoff.md` (Strategy for service/health.rs)
   - `/Users/sac/clnrm/.agents/teamwork_preview_explorer_scan_6/handoff.md` (Strategy for service/backend.rs, service/oci.rs, etc.)
2. Fully implement the genuine, production-grade logic for all these files to eliminate the stubs, facades, and bypass comments as required by the Victory Audit. Do not leave any placeholders, stubs, or bypass comments.
3. Specifically:
   - In `crates/clnrm-core/src/phases/phase_9.rs`, replace the dummy `BackendExecutionResult` block with actual GvisorBackend container running, exit code extraction, stdout/stderr Sha256 hashing, and OTel span parsing.
   - In `crates/clnrm-core/src/cli/commands/run/live_check_executor.rs`, delegate CLI commands to `run_tests_with_shard` instead of returning configuration errors.
   - Delete `crates/clnrm-core/src/template_stubs.rs` entirely. Remove it from `lib.rs` and `error.rs`, and update tests (`template_engine.rs`) to use the real `clnrm_template` engine.
   - Register `pub mod service;` in `crates/clnrm-core/src/lib.rs`.
   - In `crates/clnrm-core/src/service/health.rs`, update `check_exec` to run real runsc commands (with host process fallback) and `check_grpc` to perform genuine health checks via cleartext HTTP/2 reqwest queries using standard protobuf framing.
   - Connect `service/backend.rs` and `service/oci.rs` to use the real OCI image client and runsc backend executors.
   - In `service/registry.rs`, pass container IDs and use real container network IP checking.
4. Run `cargo test --workspace` and `cargo test --workspace --lib --tests` to verify everything compiles and passes cleanly without failures.
5. Write your implementation report to handoff.md in your working directory.
6. Use send_message to notify the orchestrator (conversation ID: 20e5a9e8-d38a-4a86-b3b1-77bdba233792) when you are done.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT
hardcode test results, create dummy/facade implementations, or
circumvent the intended task. A Forensic Auditor will independently
verify your work. Integrity violations WILL be detected and your
work WILL be rejected.
