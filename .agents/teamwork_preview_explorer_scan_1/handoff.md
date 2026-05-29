# Handoff Report — Explorer Scan 1

This handoff report documents all placeholders, TODOs, stubs, and unimplemented markers identified in the core library under `/Users/sac/clnrm/crates/clnrm-core/src/`.

---

## 1. Observation

A total of 24 distinct placeholder, stub, or unimplemented items were observed on disk using `view_file` to bypass local tool-side output rewrites. Here are the exact file paths, line numbers, verbatim text, and context:

### [1] `crates/clnrm-core/src/synthesis/coverage.rs` (Line 313)
* **Line Number:** 313
* **Verbatim Text:** `id: unimplemented!("ORACLE-GAP Refusal: Content hashing is not yet implemented"),`
* **Context:** Inside the test helper function `create_test_receipt` within `mod tests`. It eagerly panics when the helper is evaluated, crashing tests.

### [2] `crates/clnrm-core/src/receipts/mod.rs` (Line 59)
* **Line Number:** 59
* **Verbatim Text:** `//! # let receipt: TestReceipt = unimplemented!("ORACLE-GAP Refusal: Path not fully mapped");`
* **Context:** Inside a hidden doc-test example for `ReceiptStore` integration.

### [3] `crates/clnrm-core/src/chaos/orchestrator.rs` (Line 150-165)
* **Line Number:** 151-152, 155, 161
* **Verbatim Text:** 
  * Line 151: `// Disk fill is not yet implemented in ChaosEnginePlugin`
  * Line 152: `// For now, map to memory exhaustion as a placeholder`
  * Line 155: `"disk_fill experiment not yet implemented, using memory_stress as fallback"`
  * Line 161: `Ok(ChaosScenario::MemoryExhaustion {`
* **Context:** Translates chaos experiment `disk_fill` requests into `MemoryExhaustion` because disk fill is not implemented.

### [4] `crates/clnrm-core/src/chicago_tdd/mod.rs` (Lines 41-60)
* **Line Number:** 41-60
* **Verbatim Text:** 
  * Line 41: `/// **NOTE**: This is a placeholder for future integration when chicago-tdd-tools`
  * Line 45: `_placeholder: (),`
  * Line 49: `/// Create a new adapter (placeholder implementation)`
  * Line 55: `Err(CleanroomError::internal_error("Chicago-TDD-Tools integration is available in v1.4.0. ..."))`
  * Line 63: `pub fn is_available() -> bool { false }`
* **Context:** The `ChicagoTddAdapter` is completely stubbed out awaiting dependency availability.

### [5] `crates/clnrm-core/src/cli/commands/run/executor.rs` (Lines 321-325)
* **Line Number:** 322-324
* **Verbatim Text:** 
  * Line 322: `// For now, record as miss since we need to refactor run_single_test`
  * Line 323: `// to actually use the pool. This is a placeholder for metrics.`
  * Line 324: `metrics_clone.record_miss();`
* **Context:** Container pooling check-out/check-in metrics are hardcoded to record a miss during test runs.

### [6] `crates/clnrm-core/src/cli/commands/services_noun_verb.rs` (Lines 17-61)
* **Line Number:** 17-61
* **Verbatim Text:** 
  * Line 18: `// In production, this would call CleanroomEnvironment::new().await.services()`
  * Line 19: `// For now, we provide a demonstration implementation`
  * Line 23: `"No services currently running. Run 'clnrm run <test_file>' to start services."`
  * Lines 30-36, 40-50, 54-61: Hardcoded stub struct configurations for logs, start service, and stop service commands.
* **Context:** Noun-verb service command functions contain demonstration stubs returning hardcoded mock strings.

### [7] `crates/clnrm-core/src/cli/mod.rs` (Lines 24-28)
* **Line Number:** 24-28
* **Verbatim Text:** 
  * Line 24: `// For now, this is a stub. In the future, this should call the actual`
  * Line 28: `println!("⚠️  Watch-triggered test execution is not yet implemented");`
* **Context:** In `pub async fn run_tests`, the watch mode trigger implementation is missing.

### [8] `crates/clnrm-core/src/environment/compiler.rs` (Line 557)
* **Line Number:** 557
* **Verbatim Text:** `digest: format!("sha256:placeholder-{}", service_id), // Populated at runtime`
* **Context:** Sets up dummy placeholders for registry layer digests in OCI `ImageDigest` structs during compilation.

### [9] `crates/clnrm-core/src/service/backend.rs` (Lines 242-248)
* **Line Number:** 242-248
* **Verbatim Text:** 
  * Line 242: `// TODO: Implement OCI bundle creation and runsc execution`
  * Line 243: `// Return a placeholder result`
  * Line 244: `warn!("gVisor backend is not fully implemented yet - returning placeholder result");`
  * Line 248: `stdout: "gVisor backend placeholder".to_string(),`
* **Context:** gVisor sandbox container startup / runsc command executor is stubbed out.

### [10] `crates/clnrm-core/src/service/oci.rs` (Lines 62, 69, 109, 115)
* **Line Number:** 62, 69, 109, 115
* **Verbatim Text:** 
  * Line 62: `// TODO: Implement actual OCI image pulling`
  * Line 69: `warn!("OCI image pulling not yet implemented - creating placeholder");`
  * Line 109: `// TODO: Implement actual bundle creation`
  * Line 115: `warn!("OCI bundle creation not yet implemented - creating placeholder");`
* **Context:** Core OCI loader methods for retrieving layer manifests and building container runtimes are stubs.

### [11] `crates/clnrm-core/src/service/registry.rs` (Lines 226-228)
* **Line Number:** 226-228
* **Verbatim Text:** 
  * Line 226: `// In a real gVisor deployment we would query the actual container IP.`
  * Line 227: `// Assume localhost mapping.`
  * Line 228: `let container_ip = "127.0.0.1";`
* **Context:** Service registry health checking relies on assuming localhost instead of querying the OCI network.

### [12] `crates/clnrm-core/src/telemetry.rs` (Lines 808-813)
* **Line Number:** 808-813
* **Verbatim Text:** 
  * Line 810: `// Note: This is a simplified example - in practice you'd need a proper logger provider`
  * Line 811: `// Use the default registry without the logs layer`
* **Context:** `add_otel_logs_layer` is stubbed out and delegates to standard stderr initialization.

### [13] `crates/clnrm-core/src/telemetry/exporters.rs` (Lines 182-186, 199-203)
* **Line Number:** 182-186, 199-203
* **Verbatim Text:** 
  * Line 182: `// Jaeger is not currently integrated`
  * Line 185: `"Jaeger exporter not yet implemented. Use OTLP exporter to send to Jaeger collector instead."`
  * Line 199: `// Zipkin is not currently integrated`
  * Line 202: `"Zipkin exporter not yet implemented. Use OTLP exporter to send to Zipkin collector instead."`
* **Context:** Core exporters return errors rather than registering endpoints.

### [14] `crates/clnrm-core/src/telemetry/generated/mod.rs` (Lines 14, 59, 108, 127)
* **Line Number:** 14, 59, 108, 127
* **Verbatim Text:** 
  * Line 14: `// Placeholder spans module - will be replaced by weaver generation`
  * Line 59: `// Placeholder metrics module - will be replaced by weaver generation`
  * Line 108: `// Placeholder mocks module - will be replaced by weaver generation`
  * Line 127: `// Placeholder events module - will be replaced by weaver generation`
* **Context:** Telemetry schema structs are dummy models awaiting generation.

### [15] `crates/clnrm-core/src/telemetry/live_check/orchestrator.rs` (Lines 855-856)
* **Line Number:** 855-856
* **Verbatim Text:** 
  * Line 855: `// For full integration, caller would run tests here`
  * Line 856: `// Stop and return report`
* **Context:** Live check execution starts and immediately stops Weaver validation without running tests.

### [16] `crates/clnrm-core/src/telemetry/live_check/stop_coordinator.rs` (Lines 383-385)
* **Line Number:** 383-385
* **Verbatim Text:** 
  * Line 383: `// OTEL SDK flush happens when the guard is dropped`
  * Line 384: `// Ensure a small delay to allow in-flight exports`
* **Context:** Delay flush workaround during coordinator OTLP stops.

### [17] `crates/clnrm-core/src/telemetry/metrics_export.rs` (Lines 227-230)
* **Line Number:** 227-230
* **Verbatim Text:** 
  * Line 227: `// This would require accessing meter provider's internal state`
  * Line 228: `// Return 1.0 as placeholder`
  * Line 229: `// Real implementation would query the metric values`
* **Context:** Metric export success rate calculation is hardcoded.

### [18] `crates/clnrm-core/src/telemetry/semantic_conventions/gvisor.rs` (Lines 432-435)
* **Line Number:** 432-435
* **Verbatim Text:** 
  * Line 432: `// Note: Gauge observation requires callback registration`
  * Line 433: `// Implementation placeholder`
  * Line 434: `let _ = (sandbox_id, bytes, gauge);`
* **Context:** Memory usage metric observation is mocked out.

### [19] `crates/clnrm-core/src/telemetry/testing.rs` (Lines 57-82)
* **Line Number:** 57-61, 64-68, 71-75, 78-82
* **Verbatim Text:** 
  * Line 58: `// Return empty vector - real implementation would convert`
  * Line 59: `// from OpenTelemetry SDK SpanData to our SpanData`
  * Line 60: `Vec::new()`
* **Context:** In-memory trace verification search helpers return empty lists.

### [20] `crates/clnrm-core/src/timing/validator.rs` (Lines 40-41, 207)
* **Line Number:** 40-41, 207
* **Verbatim Text:** 
  * Line 40: `/// Structure for handling timing validations.`
  * Line 41: `/// The actual format will be defined by the μ-kernel team.`
  * Line 207: `// Actual logic depends on μ-kernel spec`
* **Context:** Kernel timing validations and receipt structures are unaligned placeholders.

### [21] `crates/clnrm-core/src/types.rs` (Line 196)
* **Line Number:** 196
* **Verbatim Text:** `// We use runtime checks here instead of static assertions`
* **Context:** Delays check inside `validate_priority_rules` uses runtime panics.

### [22] `crates/clnrm-core/src/validation/otel/validator.rs` (Lines 109, 127-129, 133-136)
* **Line Number:** 109, 127-128, 135
* **Verbatim Text:** 
  * Line 109: `// Implement basic validation without OTel SDK integration`
  * Line 127: `// Simulate finding the attribute (in real implementation,`
  * Line 128: `// this would query the span data from OTel SDK)`
  * Line 135: `// Simulate a reasonable duration for testing`
* **Context:** Default span validation pipeline mimics results rather than evaluating traces.

### [23] `crates/clnrm-core/src/cli/commands/run/live_check_executor.rs` (Lines 16, 64-83)
* **Line Number:** 16, 64-83
* **Verbatim Text:** 
  * Line 16: `/// # Status: REFUSAL - Awaiting CLI Integration (v1.3.1)`
  * Line 64: `Err(CleanroomError::configuration_error("Live-check CLI integration is not yet complete..."))`
* **Context:** Explicit CLI live check run executor stub.

### [24] `crates/clnrm-core/src/template_stubs.rs` (Lines 45, 59-65)
* **Line Number:** 45, 59-65
* **Verbatim Text:** 
  * Line 45: `pub fn merge_user_vars(&mut self, _vars: HashMap<String, Value>) {}`
  * Line 59: `pub fn render_template_file(_path: &Path, _vars: HashMap<String, Value>) -> std::result::Result<String, TemplateError> { Ok(String::new()) }`
  * Line 63: `pub fn is_template(_content: &str) -> bool { false }`
* **Context:** Entire file is a redundant template engine stub superseded by the `clnrm-template` dependency.

### [25] `crates/clnrm-core/tests/gall_test_suites/cli_management.rs` (Test Failure Regression)
* **Line Number:** 35-39
* **Verbatim Text:** 
  * Line 37: `let err = result.unwrap_err();`
  * Line 38: `assert!(err.to_string().contains("CLI-GALL-1 Refusal"));`
* **Context:** The test `gall_gap_test_cli_management_commands` panics because it expects `Commands::run` to return an error containing `"CLI-GALL-1 Refusal"`, but it returned `Ok(())` because a script (`fix_cli.py`) implemented full command dispatch routing.

---

## 2. Logic Chain

1. **Grepping Search:** Ripgrep matched markers like "TODO", "unimplemented!", "placeholder", "stub", "mock", and phrases across `crates/clnrm-core/src/`.
2. **De-Obfuscation Step:** RIPGrep output demonstrated rewritten tokens (e.g. `preliminary` -> `EXAMPLE-ONLY: placeholder`, `TODO` -> `ORACLE-GAP Refusal`). We executed `view_file` to read the verbatim strings directly from disk.
3. **Trace Refusal/Stub Contexts:** Analyzed call chains for each match (e.g., verifying if stubs are dead code, if they cause test panics, or if they mock out validation rules).
4. **Test Run and Verification:** Executed `cargo test --manifest-path crates/clnrm-core/Cargo.toml` and identified a test regression in `gall_test_suites::cli_management::gall_gap_test_cli_management_commands` caused by command runner implementation in `cli/types.rs` which previously returned a CLI-GALL-1 Refusal error.
5. **Resolution Strategy:** Recommended production-ready, non-stub patterns for all 24 markers, plus a regression fix for the command dispatch.

---

## 3. Caveats

* Scanned files are strictly limited to `/Users/sac/clnrm/crates/clnrm-core/src/`. Workspace files in other crates or scripts were not scanned (except for tracing the test failure regression in `tests/gall_test_suites/cli_management.rs`).
* We assumed that dependencies like `clnrm-template` are intended to completely replace local `template_stubs.rs`.

---

## 4. Conclusion

The 24 identified items fall into 4 key actions for the implementer, plus 1 test regression fix:
1. **Remove redundant stubs:** Delete `template_stubs.rs` and its occurrences.
2. **Fix test/doc failures:** Resolve eager `unimplemented!` panics in `synthesis/coverage.rs` and `receipts/mod.rs`.
3. **Integrate real modules:** Replace simulated validation pipelines (`otel/validator.rs`), metric providers (`metrics_export.rs`, `gvisor.rs`), and watch logic (`cli/mod.rs`).
4. **Complete Backend features:** Finalize OCI pulling/bundle setup (`oci.rs`, `backend.rs`), gVisor networking IP queries (`registry.rs`), and Weaver integration (`live_check_executor.rs`).
5. **Resolve CLI command run regression:** Fix `gall_gap_test_cli_management_commands` using one of two options:
   * *Option A (Keep Refusal):* If `clnrm-core` is meant to refuse orphaned commands explicitly because CLI commands were moved to `clnrm-cli`, revert `Commands::run` in `crates/clnrm-core/src/cli/types.rs` to return `Err(CleanroomError::internal_error("CLI-GALL-1 Refusal: Command dispatch not yet implemented for new modular structure"))`.
   * *Option B (Update Test):* If `Commands::run` is intended to be fully implemented in core, update `crates/clnrm-core/tests/gall_test_suites/cli_management.rs` to expect `Ok(())` or assert the implemented dispatch side-effects instead of expecting a refusal error.

---

## 5. Verification Method

To independently verify the scan and current test compliance:
1. Run `cargo test --manifest-path crates/clnrm-core/Cargo.toml` to view current core unit/doc tests execution and observe the regression failure.
2. Inspect the file listings in this report against `/Users/sac/clnrm/crates/clnrm-core/src/` to verify their verbatim code segments.
