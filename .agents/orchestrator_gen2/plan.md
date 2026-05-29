# Execution Plan — Project Orchestrator (Generation 2)

We will direct the resolution of the placeholders and stubs identified in the victory audit rejection by dispatching a Worker subagent, followed by verification using Reviewers and a Forensic Auditor.

## Steps

### Step 1: Dispatch Worker for Code Modifications
We will spawn `teamwork_preview_worker` with the detailed reports and strategies from `explorer_scan_4`, `explorer_scan_5`, and `explorer_scan_6`. The Worker will implement:
- **Phase 9 Scenario Conformance**: Implement real execution of scenario in `crates/clnrm-core/src/phases/phase_9.rs`.
- **Live-Check CLI Execution**: Re-delegate live check in `crates/clnrm-core/src/cli/commands/run/live_check_executor.rs`.
- **Template Rendering**: Completely eliminate the `template_stubs.rs` facade in favor of the production-grade `clnrm-template` library.
- **Service Layer Refactoring**:
  - `service/health.rs`: Implement real container exec and gRPC health checks.
  - `service/backend.rs`: Map gVisor execution to real `runsc` and OCI bundle commands.
  - `service/oci.rs`: Connect container registry pull / layers to `OciImageLoader` & `LayerManager`.
  - `service/registry.rs`: Resolve container network IP dynamically rather than hardcoding localhost.
  - `lib.rs`: Expose the service module `pub mod service;` so it compiles and is tested properly.

### Step 2: Verification and Review
- Spawn two `teamwork_preview_reviewer` instances to run the workspace tests:
  - `cargo test --workspace`
  - Specifically execute the targeted suites: `phases_8_10_chicago_tdd`, `run_live_check_tests`, `template_engine`, `gall_tests`.
- Verify no errors, stubs, or warnings remain.

### Step 3: Forensic Audit
- Spawn a `teamwork_preview_auditor` to check for any residual bypasses or fake implementations.
- Ensure the audit is 100% clean before finalizing.

### Step 4: Final Synthesis & Report
- Synthesize all findings and report to the parent orchestrator via `send_message`.
