# Weaver Live-Check Feature Matrix for clnrm

**Quick Reference** | **Last Updated:** 2025-10-30

---

## 1. Feature Status Legend

| Symbol | Meaning | Action Needed |
|--------|---------|---------------|
| ✅ | **USED** - Currently integrated and working | None - validated |
| ⚠️ | **UNTESTED** - Available but not validated | Add tests |
| 🟢 | **AVAILABLE** - Ready to use | Integrate if needed |
| 🔴 | **CRITICAL** - High priority for 80/20 | Validate ASAP |
| 🟡 | **IMPORTANT** - Medium priority | Test when time permits |

---

## 2. Input Sources & Formats

| Feature | Status | Priority | CLI Usage | clnrm Status |
|---------|--------|----------|-----------|--------------|
| **OTLP gRPC Listener** | ✅ | 🔴 HIGH | `--input-source otlp` (default) | WeaverController integrated |
| **OTLP HTTP Listener** | 🟢 | 🟡 MEDIUM | `--otlp-http-port 4318` | Not configured |
| **JSON File** | 🟢 | 🟡 MEDIUM | `--input-source path/to/file.json` | Not used |
| **JSON stdin** | 🟢 | 🟢 LOW | `--input-source stdin --input-format json` | Not used |
| **Text File** | 🟢 | 🟢 LOW | `--input-source path/to/file.txt --input-format text` | Not used |
| **Text stdin** | 🟢 | 🟢 LOW | `--input-source stdin --input-format text` | Not used |

**80/20 Coverage:** OTLP gRPC = 80% of use cases

---

## 3. Sample Types Validation

| Sample Type | Struct | Status | Priority | Test Needed |
|-------------|--------|--------|----------|-------------|
| **Attribute** | `SampleAttribute` | ✅ | 🔴 HIGH | Validated via OTLP |
| **Metric** | `SampleMetric` | ✅ | 🔴 HIGH | Validated via OTLP |
| **NumberDataPoint** | `SampleNumberDataPoint` | ✅ | 🔴 HIGH | Gauge/Counter validated |
| **HistogramDataPoint** | `SampleHistogramDataPoint` | ⚠️ | 🟡 MEDIUM | Add histogram test |
| **ExponentialHistogramDataPoint** | `SampleExponentialHistogramDataPoint` | ⚠️ | 🟡 MEDIUM | Add exp-histogram test |
| **Exemplar** | `SampleExemplar` | ⚠️ | 🟢 LOW | Add exemplar test |
| **Span** | `SampleSpan` | ⚠️ | 🟡 MEDIUM | Add span validation test |
| **SpanEvent** | `SampleSpanEvent` | ⚠️ | 🟢 LOW | Add event test |
| **SpanLink** | `SampleSpanLink` | ⚠️ | 🟢 LOW | Add link test |
| **Resource** | `SampleResource` | ⚠️ | 🟡 MEDIUM | Add resource test |

**80/20 Coverage:** Attribute + Metric + NumberDataPoint = 80% of telemetry

---

## 4. Built-in Advisors

| Advisor | Purpose | Advice Types | Status | Always Active |
|---------|---------|--------------|--------|---------------|
| **DeprecatedAdvisor** | Detects deprecated attrs/metrics | `deprecated` | ✅ | Yes |
| **StabilityAdvisor** | Checks stability levels | `not_stable` | ✅ | Yes |
| **TypeAdvisor** | Validates types/instruments/units | `type_mismatch`, `unexpected_instrument`, `unit_mismatch`, `required_attribute_not_present`, `recommended_attribute_not_present` | ✅ | Yes |
| **EnumAdvisor** | Validates enum variants | `undefined_enum_variant` | ✅ | Yes |
| **RegoAdvisor** | Custom policy engine | User-defined | ✅ | Yes (default OTel policies) |

**All built-in advisors are active by default in clnrm.**

---

## 5. Advice Levels

| Level | Severity | Exit Code | Impact | clnrm Behavior |
|-------|----------|-----------|--------|----------------|
| **Violation** | Error | Non-zero | Blocks release | ✅ Test fails |
| **Improvement** | Warning | Zero | Suggests changes | ✅ Logged |
| **Information** | Info | Zero | FYI | ✅ Logged |

**Weaver exits non-zero if ANY violations are detected.**

---

## 6. Output Formats

| Format | Purpose | Usage | Status | clnrm Integration |
|--------|---------|-------|--------|-------------------|
| **JSON** | Machine-readable reports | `--format json` | ✅ | WeaverController parses this |
| **ANSI** | Human-readable console | `--format ansi` (default) | 🟢 | Not configured |
| **Custom Jinja** | User templates | `--templates path/to/templates/` | 🟢 | Not used |

---

## 7. OTLP Configuration

| Feature | CLI Flag | Default | Status | WeaverConfig Field |
|---------|----------|---------|--------|--------------------|
| **gRPC Port** | `--otlp-grpc-port` | 4317 | ✅ | `otlp_port` |
| **gRPC Address** | `--otlp-grpc-address` | 127.0.0.1 | ✅ | Hardcoded |
| **Admin Port** | `--admin-port` | 8080 | ✅ | `admin_port` |
| **Inactivity Timeout** | `--inactivity-timeout` | 60s | ✅ | `inactivity_timeout` |
| **Streaming** | `--stream` / `--no-stream` | false | ⚠️ | `stream` (not enabled) |
| **Output Dir** | `--output` | stdout | ✅ | `output_dir` |

---

## 8. Statistics & Coverage

| Metric | JSON Field | Status | WeaverController |
|--------|------------|--------|------------------|
| **Registry Coverage** | `registry_coverage` | ✅ | Parsed |
| **Violation Count** | `advice_level_counts.violation` | ✅ | Parsed |
| **Improvement Count** | `advice_level_counts.improvement` | ✅ | Parsed |
| **Information Count** | `advice_level_counts.information` | ✅ | Parsed |
| **Entity Counts** | `total_entities_by_type` | ✅ | Parsed |
| **Seen Registry Attributes** | `seen_registry_attributes` | ✅ | Available |
| **Seen Non-Registry Attributes** | `seen_non_registry_attributes` | ✅ | Available |
| **Seen Registry Metrics** | `seen_registry_metrics` | ✅ | Available |
| **Seen Non-Registry Metrics** | `seen_non_registry_metrics` | ✅ | Available |

**All statistics are tracked and available in validation reports.**

---

## 9. Shutdown Control

| Method | Signal/Endpoint | Platform | Status | WeaverController |
|--------|-----------------|----------|--------|------------------|
| **SIGHUP** | Unix signal | Unix | ✅ | Used for graceful shutdown |
| **SIGINT** | Ctrl+C | All | ✅ | Supported |
| **HTTP /stop** | Admin endpoint | All | ✅ | Available but not used |
| **Inactivity Timeout** | Auto-stop | All | ✅ | Configured to 30s |

---

## 10. Custom Rego Policies

| Feature | CLI Flag | Status | Example |
|---------|----------|--------|---------|
| **Custom Policy Dir** | `--advice-policies path/` | ⚠️ | See `INTEGRATION_EXAMPLES.rs` example 7 |
| **Custom JQ Preprocessor** | `--advice-preprocessor path.jq` | ⚠️ | Not tested |
| **Default OTel Policies** | Embedded | ✅ | Active by default |

**Custom policies allow domain-specific validation rules for clnrm.**

---

## 11. 80/20 Priority Matrix

### Phase 1: Core Validation (80% Value) ✅ COMPLETE

| Feature | Value | Effort | Status |
|---------|-------|--------|--------|
| OTLP Listener | 40% | Low | ✅ Done |
| Built-in Advisors | 30% | Low | ✅ Done |
| JSON Reports | 10% | Low | ✅ Done |

**Total: 80% value, 20% effort - COMPLETE**

### Phase 2: Live Testing (15% Value) ⚠️ PENDING

| Feature | Value | Effort | Status |
|---------|-------|--------|--------|
| Span Validation | 5% | Medium | ⚠️ Need test |
| Resource Validation | 5% | Low | ⚠️ Need test |
| Histogram Metrics | 5% | Medium | ⚠️ Need test |

**Total: 15% value, 30% effort - PENDING**

### Phase 3: Advanced Features (5% Value) 🟢 OPTIONAL

| Feature | Value | Effort | Status |
|---------|-------|--------|--------|
| Custom Rego | 2% | High | 🟢 Future |
| Streaming Output | 2% | Low | 🟢 Future |
| Custom Templates | 1% | Medium | 🟢 Future |

**Total: 5% value, 50% effort - OPTIONAL**

---

## 12. Implementation Checklist

### Immediate (v1.2.0 Completion)

- [ ] **Test OTLP end-to-end** - Verify WeaverController works with real telemetry
- [ ] **Test span validation** - Emit spans and verify Weaver validates them
- [ ] **Test resource validation** - Emit resources and verify validation
- [ ] **Test histogram metrics** - Emit histograms and verify validation
- [ ] **Document validation results** - Update WEAVER_V1_2_0_VALIDATION_SUMMARY.md

### Short-Term (v1.3.0)

- [ ] **Enable streaming output** - Set `stream: true` in WeaverConfig
- [ ] **Test exponential histograms** - Add exp-histogram samples
- [ ] **Test exemplars** - Add exemplar validation
- [ ] **Add ANSI output** - Human-readable console reports

### Long-Term (v2.0.0)

- [ ] **Custom Rego policies** - clnrm-specific validation rules
- [ ] **Custom Jinja templates** - CI/CD reporting
- [ ] **File/stdin ingesters** - Offline validation
- [ ] **HTTP OTLP support** - Alternative to gRPC

---

## 13. Test Coverage Matrix

| Feature | Unit Test | Integration Test | End-to-End Test | Status |
|---------|-----------|------------------|-----------------|--------|
| **OTLP Listener** | N/A | ⚠️ Needed | ⚠️ Needed | Infrastructure ready |
| **Attribute Validation** | N/A | ⚠️ Needed | ⚠️ Needed | Should work |
| **Metric Validation** | N/A | ⚠️ Needed | ⚠️ Needed | Should work |
| **Span Validation** | N/A | ⚠️ Needed | ⚠️ Needed | Untested |
| **Resource Validation** | N/A | ⚠️ Needed | ⚠️ Needed | Untested |
| **Histogram** | N/A | ⚠️ Needed | ⚠️ Needed | Untested |
| **Statistics** | N/A | ⚠️ Needed | ⚠️ Needed | Should work |

**Test File Location:** `crates/clnrm-core/tests/telemetry/weaver_live_check_tests.rs`

---

## 14. Key Constraints & Limitations

| Constraint | Impact | Workaround |
|------------|--------|------------|
| **Weaver must be installed** | Can't run without Weaver binary | Docker container or CI install |
| **Registry must exist** | No validation without schemas | Provide registry/ directory |
| **OTLP port conflicts** | Multiple tests can't run simultaneously | Use different ports or sequential tests |
| **JSON parsing errors** | Malformed output breaks WeaverController | Robust error handling implemented |

---

## 15. Quick Command Reference

### Start Weaver (Manual)

```bash
weaver registry live-check \
  --registry registry/ \
  --otlp-grpc-port 4317 \
  --admin-port 8080 \
  --format json \
  --output validation_output/ \
  --inactivity-timeout 30
```

### Stop Weaver (Manual)

```bash
# Unix
kill -HUP <weaver_pid>

# Or via admin endpoint
curl -X POST http://localhost:8080/stop
```

### Check Validation Results

```bash
# Parse JSON report
cat validation_output/live_check_report.json | jq '.statistics'
```

### Via WeaverController (Rust)

```rust
let config = WeaverConfig { /* ... */ };
let mut weaver = WeaverController::new(config)?;
weaver.start()?;
// Run tests
let report = weaver.stop_and_get_report()?;
```

---

## 16. References

- **Feature Analysis:** `WEAVER_LIVE_CHECK_FEATURE_ANALYSIS.md`
- **Integration Examples:** `INTEGRATION_EXAMPLES.rs`
- **Source Code:** `vendors/weaver/crates/weaver_live_check/`
- **WeaverController:** `crates/clnrm-core/src/telemetry/weaver_controller.rs`
- **Official Docs:** `vendors/weaver/crates/weaver_live_check/README.md`

---

**End of Feature Matrix**
