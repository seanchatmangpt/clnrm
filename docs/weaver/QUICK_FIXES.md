# Quick Fixes for Weaver Validation

**TL;DR:** Three critical fixes needed to get Weaver validation working. Total time: ~4 hours.

---

## The Problem in One Sentence

OTEL sends telemetry to the wrong endpoint and drops the connection before export completes, so Weaver never receives any spans to validate.

---

## Three Critical Fixes

### Fix #1: Weaver-First Initialization ⏱️ 90 minutes

**Problem:** OTEL initializes first with port 4317, then Weaver starts on port 54321. OTEL never switches.

**Fix:** In `/Users/sac/clnrm/crates/clnrm-core/src/cli/commands/run/mod.rs` line 304-385, **swap the order**:

```rust
// OLD ORDER (wrong):
// 1. init_otel() → sends to localhost:4317
// 2. WeaverController::start() → listens on :54321
// Result: Telemetry goes to wrong port

// NEW ORDER (correct):
// 1. WeaverController::start() → listens on :54321
// 2. Get controller.get_otlp_port() → returns 54321
// 3. init_otel() with endpoint localhost:54321
// Result: Telemetry goes to Weaver ✅
```

**Code change:** See section 8, Fix #2 in BLIND_SPOTS_ANALYSIS.md

---

### Fix #2: OTEL Guard Flush ⏱️ 30 minutes

**Problem:** OtelGuard drops immediately after tests, killing in-flight async exports.

**Fix:** In same file, after line 495, **add explicit flush**:

```rust
// After tests complete, BEFORE _otel_guard drops:
if let Some(ref guard) = _otel_guard {
    info!("🔄 Flushing telemetry...");
    let _ = guard.tracer_provider.force_flush();
    tokio::time::sleep(Duration::from_secs(2)).await;
    info!("✅ Telemetry flushed");
}
```

**Why 2 seconds?** Batch exporter is async, needs time to send HTTP/gRPC request.

---

### Fix #3: Weaver Lifecycle Completion ⏱️ 60 minutes

**Problem:** Weaver starts but never stops. No validation report generated.

**Fix:** In same file, after OTEL flush, **add validation report**:

```rust
if let Some(mut controller) = weaver_controller {
    tokio::time::sleep(Duration::from_secs(2)).await; // Let Weaver process
    controller.stop_live_check()?;
    let report = controller.get_validation_report()?;

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║          Weaver Validation Report                       ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!("Status:       {:?}", report.status);
    println!("Violations:   {}", report.violations);
    println!("Coverage:     {:.1}%", report.registry_coverage * 100.0);

    if report.violations > 0 {
        return Err(CleanroomError::validation_error(format!(
            "Weaver validation failed: {} violations", report.violations
        )));
    }
}
```

**Code change:** See section 8, Fix #3 in BLIND_SPOTS_ANALYSIS.md

---

## Testing the Fixes

```bash
# 1. Apply all three fixes
# (Edit /Users/sac/clnrm/crates/clnrm-core/src/cli/commands/run/mod.rs)

# 2. Rebuild
cargo build --release --features otel

# 3. Install
brew uninstall clnrm
brew install --build-from-source .

# 4. Run with validation
clnrm run --otel-exporter otlp-grpc --validate tests/

# 5. Expected output:
# 🔍 Starting Weaver live-check validation
# ✅ Weaver listening on port 54321
# 🔧 Initializing OTEL with endpoint: localhost:54321
# ... test execution ...
# 🔄 Flushing telemetry...
# ✅ Telemetry flushed
# 🛑 Stopping Weaver live-check
# 📊 Retrieving Weaver validation report
#
# ╔══════════════════════════════════════════════════════════╗
# ║          Weaver Validation Report                       ║
# ╚══════════════════════════════════════════════════════════╝
# Status:       Success
# Violations:   0
# Coverage:     100.0%
#
# ✅ Weaver validation passed
```

---

## What This Achieves

**Before fixes:**
- OTEL emits spans ✅
- Weaver listens ✅
- Spans reach Weaver ❌
- Validation report ❌
- **Result:** 0% functional

**After fixes:**
- OTEL emits spans ✅
- Weaver listens ✅
- Spans reach Weaver ✅
- Validation report ✅
- **Result:** 100% functional for test_execution spans

---

## What's Still Missing (Post-MVP)

These are **NOT blockers** for basic validation:

1. **Container lifecycle spans** - Container start/exec/stop not instrumented
2. **Hardcoded image names** - "alpine:latest" hardcoded instead of actual image
3. **Schema coverage** - Only 2/13 schemas implemented (test_execution + metrics)
4. **CLI command coverage** - Only 2/23 commands instrumented

**Priority:** Get basic validation working first (Fixes #1-3), then add coverage.

---

## Estimated Timeline

- **Fix #1 (Weaver-first):** 90 minutes (reorder + test)
- **Fix #2 (OTEL flush):** 30 minutes (add 5 lines + test)
- **Fix #3 (Report):** 60 minutes (add report display + test)
- **Buffer:** 60 minutes (debugging, iteration)

**Total:** ~4 hours to working validation

---

## Success Criteria

**Definition of Done:**
```bash
clnrm run --otel-exporter otlp-grpc --validate tests/basic-test.clnrm.toml
```

**Expected:**
1. ✅ Weaver starts on dynamic port
2. ✅ OTEL sends to that port
3. ✅ Test executes and emits span with ALL attributes
4. ✅ Weaver receives span
5. ✅ Weaver validates against schema
6. ✅ Report shows: Status=Success, Violations=0, Coverage=100%
7. ✅ Command exits with code 0

**If ANY step fails, validation is broken.**

---

## Next Steps After Phase 1

Once basic validation works:

1. **Instrument container backend** (Fix #4) - 4 hours
2. **Fix image name propagation** (Fix #6) - 2 hours
3. **Implement remaining schemas** - 2-3 days
4. **Instrument all CLI commands** - 2-3 days

**Total to production-ready:** 1 week

---

## Reference

Full analysis: `/Users/sac/clnrm/docs/weaver/BLIND_SPOTS_ANALYSIS.md` (1042 lines)

All code changes: Section 8 of BLIND_SPOTS_ANALYSIS.md

Questions: Check BLIND_SPOTS_ANALYSIS.md sections 4 (execution trace) and 7 (summary)
