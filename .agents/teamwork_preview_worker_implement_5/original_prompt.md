## 2026-05-28T21:20:40-07:00
You are the Code Implementer (worker).
Your task is to implement the real, production-ready logic for all stubs, facades, and placeholders in the clnrm codebase, addressing the victory audit rejection.

Specifically, read the analysis and implementation strategies in:
1. /Users/sac/clnrm/.agents/teamwork_preview_explorer_scan_4/handoff.md
2. /Users/sac/clnrm/.agents/teamwork_preview_explorer_scan_5/handoff.md
3. /Users/sac/clnrm/.agents/teamwork_preview_explorer_scan_6/handoff.md

You must modify the codebase to resolve:
1. crates/clnrm-core/src/phases/phase_9.rs: Implement BackendConformanceHarness::check_scenario to resolve the scenario ID, run it on the backend, calculate hashes, parse OpenTelemetry spans, and build the real BackendExecutionResult.
2. crates/clnrm-core/src/cli/commands/run/live_check_executor.rs: Delegate to the actual Weaver live-check orchestration using CliConfig and run_tests_with_shard. Remove `#[ignore]` from the live-check tests so they are executed and pass.
3. crates/clnrm-core/src/template_stubs.rs: Completely delete this facade, clean up its module registration in crates/clnrm-core/src/lib.rs, error.rs, and update crates/clnrm-core/tests/gall_test_suites/template_engine.rs to use the actual `clnrm_template` engine. Also clean up any other reference (e.g. in oracle_gaps.rs).
4. crates/clnrm-core/src/service/health.rs: Implement check_exec and check_grpc using the designs (including runsc exec and tonic-health or pure HTTP/2 via reqwest). Update the check signature in health.rs and registry.rs to pass the container_id.
5. crates/clnrm-core/src/service/backend.rs: Implement the gVisor backend execution by creating OCI bundle and invoking runsc.
6. crates/clnrm-core/src/service/oci.rs: Implement actual OCI image pulling and bundle creation.
7. crates/clnrm-core/src/service/registry.rs: Retrieve the actual container IP and host ports dynamically instead of hardcoding localhost.
8. Register the service module: Ensure `pub mod service;` is present in crates/clnrm-core/src/lib.rs so it compiles and is fully verified.
9. Ensure there are no other "TODO", "unimplemented!", "placeholder", "stub", or deferred work in the active codebase.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

Verification Requirements:
1. Run `cargo check --workspace --all-targets` to verify successful compilation.
2. Run `cargo test --workspace` to run all tests. Make sure all unit, integration, and E2E tests pass.
3. Document the execution of the tests and the commands in your handoff report.
