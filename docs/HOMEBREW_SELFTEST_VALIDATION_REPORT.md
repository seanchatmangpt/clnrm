# Homebrew Self-Test Validation Report

**Date**: 2025-10-17
**Test Subject**: Complete 8-layer OTEL validation for Homebrew-installed clnrm
**Test Spec**: `/Users/sac/clnrm/tests/homebrew_selftest/clnrm_brew_install_selftest_verbose.clnrm.toml`
**Overall Status**: ⛔ **BLOCKED**

---

## Executive Summary

The Homebrew self-test validation **CANNOT BE EXECUTED** due to two critical blockers:

1. **BLOCKER 1**: The `clnrm run` command does not execute scenarios
2. **BLOCKER 2**: All existing scenario-based TOML files are missing the required `type` field

While the TOML spec itself is **syntactically valid** and the 8 validation layers are **correctly configured**, the execution engine cannot run the test.

---

## Validation Execution Results

### Step 1: TOML Spec Validation ✅

**Initial Issues Found**:
- Line 95: `attrs.any` used array instead of inline table
- Line 46: Missing required `type` field in service definition

**Fixes Applied**:
```diff
- attrs.any=["component=plugin.registry"]
+ attrs.any={ component="plugin.registry" }

  [service.brew]
+ type="generic_container"
  plugin="generic_container"
```

**Result**: TOML parses successfully after fixes.

### Step 2: Configuration Parsing ✅

**Test Command**:
```bash
./target/release/clnrm run tests/homebrew_selftest/clnrm_brew_install_selftest_verbose.clnrm.toml
```

**Result**: Configuration loads successfully:
- 1 service defined (`brew` using `homebrew/brew:latest`)
- 1 scenario defined (`brew_install_and_selftest_stdout`)
- 8 validation layers configured:
  1. SpanValidator (`[[expect.span]]` blocks)
  2. GraphValidator (`[expect.graph]`)
  3. CountValidator (`[expect.counts]`)
  4. WindowValidator (`[expect.windows]`)
  5. OrderValidator (`[expect.order]`)
  6. StatusValidator (`[expect.status]`)
  7. HermeticityValidator (`[expect.hermeticity]`)
  8. DeterminismEngine (`[expect.determinism]`)

**Output**:
```
🚀 Executing test: clnrm_brew_install_selftest_verbose
📝 Description: Brew-install clnrm in-container and validate via OTEL (stdout) spans
🎉 Test 'clnrm_brew_install_selftest_verbose' completed successfully!
✅ clnrm_brew_install_selftest_verbose.clnrm.toml - PASS (0ms)
```

### Step 3: Scenario Execution ⛔ **BLOCKED**

**Critical Finding**: The test reports "PASS (0ms)" but **NO ACTUAL EXECUTION OCCURRED**.

**Root Cause Analysis**:

1. **Examined `run_single_test` function** (`crates/clnrm-core/src/cli/commands/run/mod.rs`):
   ```rust
   // Execute test steps
   for (i, step) in test_config.steps.iter().enumerate() {
       // ... execution logic
   }
   ```
   - Only iterates over `test_config.steps`
   - **No code path for `test_config.scenario`**

2. **Grepped for scenario execution logic**:
   ```bash
   grep -r "scenario" crates/clnrm-core/src/cli/commands/run/
   # Result: No matches found
   ```

3. **Verified scenario struct exists** in `crates/clnrm-core/src/config/types.rs`:
   ```rust
   pub struct TestConfig {
       pub scenario: Vec<ScenarioConfig>,  // ✅ Defined
       // ...
   }

   pub struct ScenarioConfig {
       pub name: String,
       pub service: Option<String>,
       pub run: Option<String>,  // v1.0 command execution
       // ...
   }
   ```

**Conclusion**:
- ✅ Scenario config structs exist
- ✅ TOML parsing works
- ⛔ **Execution logic NOT IMPLEMENTED**

The framework has the **data structures** for v1.0 scenarios but the **execution engine** hasn't been updated to use them.

---

## Impact Analysis

### Affected Test Files

**All scenario-based tests are broken**:
```bash
$ find tests/ examples/ -name "*.clnrm.toml" -exec grep -l "^\[\[scenario\]\]" {} \;
```

**Count**: 20+ files affected, including:
- `tests/self-test/clnrm-v0.6.0-self-validation.clnrm.toml`
- `tests/integration/prd_otel_workflow.clnrm.toml`
- `tests/red_team/*.clnrm.toml` (5 files)
- `tests/fake_green_detection/*.clnrm.toml` (2 files)
- `tests/exit_checks/generated_otel.clnrm.toml`
- `examples/templates/*.clnrm.toml` (8 files)
- `examples/case-studies/redteam-otlp-env.clnrm.toml`

**Secondary Issue**: All these files also lack the `type` field in service definitions.

### What Works

✅ **TOML Parsing**: Config loads successfully
✅ **Validation Layer Configuration**: All 8 layers properly defined
✅ **Service Registration**: Service configs parse correctly
✅ **OTEL Binary**: Built successfully with `--features otel`
✅ **Docker Availability**: Container runtime available

### What Doesn't Work

⛔ **Scenario Execution**: Not implemented
⛔ **Validation Layer Testing**: Can't run without scenario execution
⛔ **Report Generation**: No execution = no reports
⛔ **Determinism Testing**: Can't test what doesn't run
⛔ **Failure Detection**: Can't validate error handling

---

## Validation Layer Status

Since scenarios don't execute, we **CANNOT VERIFY** any validation layers:

| Layer | Config Status | Execution Status | Verdict |
|-------|---------------|------------------|---------|
| 1. SpanValidator | ✅ Configured | ⛔ Not testable | **BLOCKED** |
| 2. GraphValidator | ✅ Configured | ⛔ Not testable | **BLOCKED** |
| 3. CountValidator | ✅ Configured | ⛔ Not testable | **BLOCKED** |
| 4. WindowValidator | ✅ Configured | ⛔ Not testable | **BLOCKED** |
| 5. OrderValidator | ✅ Configured | ⛔ Not testable | **BLOCKED** |
| 6. StatusValidator | ✅ Configured | ⛔ Not testable | **BLOCKED** |
| 7. HermeticityValidator | ✅ Configured | ⛔ Not testable | **BLOCKED** |
| 8. DeterminismEngine | ✅ Configured | ⛔ Not testable | **BLOCKED** |

---

## Required Implementation

### Option 1: Implement Scenario Execution

**Location**: `crates/clnrm-core/src/cli/commands/run/mod.rs`

**Required Changes**:
```rust
pub async fn run_single_test(path: &PathBuf, _config: &CliConfig) -> Result<()> {
    // ... existing setup ...

    // Add scenario execution logic
    if !test_config.scenario.is_empty() {
        for scenario in &test_config.scenario {
            info!("📋 Scenario: {}", scenario.name);

            // Get service for this scenario
            let service_name = scenario.service.as_ref()
                .ok_or_else(|| CleanroomError::validation_error("Scenario missing service"))?;

            let service_handle = service_handles.get(service_name)
                .ok_or_else(|| CleanroomError::validation_error(
                    format!("Service '{}' not found", service_name)
                ))?;

            // Execute scenario.run command if provided
            if let Some(run_cmd) = &scenario.run {
                let output = environment.execute_in_service(service_handle, run_cmd).await?;

                // Collect artifacts if configured
                if let Some(artifacts) = &scenario.artifacts {
                    // Parse OTEL spans from stdout if artifacts.collect includes "spans:default"
                    if artifacts.collect.iter().any(|a| a.starts_with("spans:")) {
                        let spans = parse_otel_spans_from_output(&output)?;

                        // Run OTEL validation if configured
                        if let Some(otel_config) = &test_config.otel {
                            validate_otel_spans(&spans, otel_config)?;
                        }
                    }
                }
            }

            // Execute scenario.steps if provided (backward compatibility)
            for step in &scenario.steps {
                // ... execute step logic (reuse existing step execution) ...
            }
        }
    } else {
        // Fall back to legacy steps-based execution
        for step in &test_config.steps {
            // ... existing step execution ...
        }
    }

    // ... rest of function ...
}
```

**Additional Requirements**:
1. Add `parse_otel_spans_from_output()` function to extract JSON spans from stdout
2. Add `validate_otel_spans()` function to run all configured validation layers
3. Add artifact collection logic for `artifacts.collect` field
4. Add support for `scenario.run` shell command execution
5. Update `execute_in_service()` to capture stdout/stderr separately

**Estimated Effort**: 8-12 hours of focused development

### Option 2: Convert to Steps-Based Format

**Alternative**: Convert the Homebrew test to use `steps` instead of `scenario`:

```toml
[[steps]]
name = "brew_install_and_selftest"
service = "brew"
command = ["bash", "-lc", "brew update && brew tap seanchatmangpt/clnrm && brew install clnrm && clnrm self-test --otel-exporter stdout"]
capture_stdout = true

[steps.0.artifacts]
collect = ["spans:default"]
```

**Issue**: This still requires artifact collection and OTEL validation support, which may not be implemented for steps either.

---

## Recommendations

### Immediate Actions

1. **Implement scenario execution** in `clnrm run` command (Option 1)
2. **Add mass migration tool** to add `type` field to all service definitions:
   ```bash
   clnrm migrate --add-service-types tests/ examples/
   ```
3. **Add integration test** for scenario execution before claiming v1.0 support

### Testing Strategy

Once scenario execution is implemented:

1. **Unit tests** for scenario execution logic
2. **Integration test** using simple alpine container + echo command
3. **Full Homebrew validation** using the fixed TOML spec
4. **Regression tests** for all 20+ scenario-based files

### Documentation Updates

1. Update `docs/TOML_REFERENCE.md` with scenario execution semantics
2. Add `docs/SCENARIOS_VS_STEPS.md` explaining the difference
3. Update `README.md` to clarify v1.0 scenario support status
4. Add migration guide for v0.4.x → v1.0 format

---

## Files Modified

### Fixed TOML Spec

**File**: `/Users/sac/clnrm/tests/homebrew_selftest/clnrm_brew_install_selftest_verbose.clnrm.toml`

**Changes**:
1. Added `type="generic_container"` to service definition
2. Fixed `attrs.any` syntax from array to inline table

**Status**: ✅ **Syntactically valid, semantically complete, ready for execution**

### Artifacts Generated

**None** - Cannot generate artifacts without execution:
- ❌ `brew-selftest.report.json` - Not generated
- ❌ `brew-selftest.junit.xml` - Not generated
- ❌ `brew-selftest.trace.sha256` - Not generated

---

## Final Verdict

**Status**: ⛔ **BLOCKED - Cannot Execute**

### What We Verified

✅ TOML spec is **syntactically valid**
✅ All 8 validation layers are **correctly configured**
✅ Service definitions **parse successfully**
✅ OTEL-enabled binary **builds successfully**
✅ Docker **is available and working**

### What We Cannot Verify

⛔ Validation layer **execution**
⛔ Report **generation**
⛔ Determinism **enforcement**
⛔ Failure **detection**
⛔ End-to-end **workflow**

### Unblocking Path

**Implement scenario execution in `clnrm run` command**, then re-run this validation.

**Estimated Time to Unblock**: 1-2 days for a senior Rust developer

---

## Appendix: Test Execution Log

### Attempt 1: Initial Run
```bash
$ ./target/release/clnrm run tests/homebrew_selftest/clnrm_brew_install_selftest_verbose.clnrm.toml
✅ clnrm_brew_install_selftest_verbose.clnrm.toml - PASS (0ms)
Test Results: 1 passed, 0 failed
```
**Issue**: False positive - no actual execution

### Attempt 2: Force Run (Bypass Cache)
```bash
$ ./target/release/clnrm run tests/homebrew_selftest/clnrm_brew_install_selftest_verbose.clnrm.toml --force
🔨 Force mode enabled - bypassing cache
✅ clnrm_brew_install_selftest_verbose.clnrm.toml - PASS (0ms)
Test Results: 1 passed, 0 failed
```
**Issue**: Same result - scenarios not executing

### Code Analysis
```bash
$ grep -r "scenario" crates/clnrm-core/src/cli/commands/run/
# No matches found
```
**Conclusion**: Execution logic not implemented

---

## Appendix: Affected Files Requiring `type` Field

All files using `[service.name]` syntax need `type` field added:

```bash
tests/self-test/clnrm-v0.6.0-self-validation.clnrm.toml
tests/integration/prd_otel_workflow.clnrm.toml
tests/red_team/attack_c_empty_otel.clnrm.toml
tests/red_team/legitimate_self_test.clnrm.toml
tests/red_team/attack_a_echo.clnrm.toml
tests/red_team/attack_b_logs.clnrm.toml
tests/red_team/clnrm_redteam_catch_verbose.clnrm.toml
tests/fake_green_detection/clnrm_otel_full_surface.clnrm.toml
tests/fake_green_detection/fake_green_case_study.clnrm.toml
tests/exit_checks/generated_otel.clnrm.toml
tests/homebrew_selftest/clnrm_brew_install_selftest_verbose.clnrm.toml
examples/case-studies/redteam-otlp-env.clnrm.toml
examples/templates/advanced-validators.clnrm.toml
examples/templates/matrix-expansion.clnrm.toml
examples/templates/ci-integration.clnrm.toml
examples/templates/simple-variables.clnrm.toml
examples/templates/macros-and-includes.clnrm.toml
examples/templates/multi-environment.clnrm.toml
examples/templates/service-mesh.clnrm.toml
examples/templates/env_resolution_demo.clnrm.toml
```

**Total**: 20 files requiring migration

---

**Report Generated**: 2025-10-17T07:47:00Z
**Author**: QA Validation Agent
**Framework Version**: clnrm v0.4.1 (git: 84ce86d)
