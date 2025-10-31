# Weaver Live-Check Documentation Index

**Generated:** 2025-10-30
**Purpose:** Central index for all Weaver integration documentation

---

## Quick Navigation

| Document | Purpose | Use When |
|----------|---------|----------|
| **[ANALYSIS_SUMMARY.md](./ANALYSIS_SUMMARY.md)** | Executive summary | Starting point, overview |
| **[FEATURE_MATRIX.md](./FEATURE_MATRIX.md)** | Quick reference | Looking up specific features |
| **[WEAVER_LIVE_CHECK_FEATURE_ANALYSIS.md](./WEAVER_LIVE_CHECK_FEATURE_ANALYSIS.md)** | Comprehensive analysis | Deep dive, research |
| **[INTEGRATION_EXAMPLES.rs](./INTEGRATION_EXAMPLES.rs)** | Code examples | Implementing new features |

---

## Document Descriptions

### 1. ANALYSIS_SUMMARY.md
**Purpose:** Executive summary of Weaver analysis
**Audience:** Project leads, architects
**Contents:**
- Mission status
- Key findings
- Deliverables
- 80/20 analysis
- Next steps

**When to read:** Start here for high-level overview

### 2. FEATURE_MATRIX.md
**Purpose:** Quick reference guide for all features
**Audience:** Developers, testers
**Contents:**
- Feature status table
- Input sources matrix
- Sample types matrix
- Built-in advisors
- 80/20 priority matrix
- Quick command reference

**When to read:** Looking up feature status or CLI commands

### 3. WEAVER_LIVE_CHECK_FEATURE_ANALYSIS.md
**Purpose:** Comprehensive feature analysis
**Audience:** Architects, senior developers
**Contents:**
- Complete architecture overview
- All 10 feature categories
- Currently used vs untested features
- Gap analysis
- Integration architecture
- Testing recommendations
- Key insights

**When to read:** Deep dive into Weaver capabilities

### 4. INTEGRATION_EXAMPLES.rs
**Purpose:** Executable code examples
**Audience:** Developers implementing features
**Contents:**
- 15 complete Rust examples
- OTLP integration
- All sample types
- Custom Rego policies
- Statistics tracking
- End-to-end validation

**When to read:** Implementing new Weaver features

---

## Quick Start

### For New Developers

1. **Read:** [ANALYSIS_SUMMARY.md](./ANALYSIS_SUMMARY.md) - Get overview
2. **Browse:** [FEATURE_MATRIX.md](./FEATURE_MATRIX.md) - Understand what's available
3. **Study:** [INTEGRATION_EXAMPLES.rs](./INTEGRATION_EXAMPLES.rs) - See code examples

### For Integration Work

1. **Check:** [FEATURE_MATRIX.md](./FEATURE_MATRIX.md) - Verify feature status
2. **Reference:** [INTEGRATION_EXAMPLES.rs](./INTEGRATION_EXAMPLES.rs) - Copy example code
3. **Test:** Run example, adapt for your use case

### For Architecture Decisions

1. **Read:** [WEAVER_LIVE_CHECK_FEATURE_ANALYSIS.md](./WEAVER_LIVE_CHECK_FEATURE_ANALYSIS.md) - Complete analysis
2. **Review:** [FEATURE_MATRIX.md](./FEATURE_MATRIX.md#9-8020-priority-matrix) - 80/20 priorities
3. **Plan:** [ANALYSIS_SUMMARY.md](./ANALYSIS_SUMMARY.md#next-steps) - Implementation phases

---

## Feature Status Overview

### ✅ Complete (v1.2.0)

**Infrastructure:**
- OTLP gRPC listener
- WeaverController integration
- JSON report parsing
- All built-in advisors active
- Violation detection

**Sample Types:**
- Attributes
- Metrics
- NumberDataPoints

### ⚠️ Pending (v1.2.0 Testing)

**High Priority:**
- Span validation tests
- Resource validation tests
- Histogram metric tests
- End-to-end OTLP tests

### 🟢 Future (v1.3.0+)

**Medium Priority:**
- Streaming output
- Exponential histograms
- Exemplar validation

**Low Priority:**
- Custom Rego policies
- File/stdin ingesters
- Custom Jinja templates

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    clnrm Tests                          │
│  - Testcontainers                                       │
│  - OTLP Telemetry Emission                              │
└──────────────────┬──────────────────────────────────────┘
                   │ OTLP gRPC (port 4317)
                   ▼
┌─────────────────────────────────────────────────────────┐
│              Weaver Live-Check Process                  │
│  ┌───────────────────────────────────────────────────┐  │
│  │ Input: OTLP Listener                              │  │
│  └────────────────┬──────────────────────────────────┘  │
│                   ▼                                      │
│  ┌───────────────────────────────────────────────────┐  │
│  │ LiveChecker                                       │  │
│  │  - Registry (schemas)                             │  │
│  │  - Advisors (built-in + Rego)                     │  │
│  └────────────────┬──────────────────────────────────┘  │
│                   ▼                                      │
│  ┌───────────────────────────────────────────────────┐  │
│  │ Validation                                        │  │
│  │  - Type checking                                  │  │
│  │  - Deprecation checks                             │  │
│  │  - Stability checks                               │  │
│  │  - Enum validation                                │  │
│  │  - Rego policies                                  │  │
│  └────────────────┬──────────────────────────────────┘  │
│                   ▼                                      │
│  ┌───────────────────────────────────────────────────┐  │
│  │ Statistics & Coverage                             │  │
│  │  - Registry coverage                              │  │
│  │  - Advice counts                                  │  │
│  │  - Entity counts                                  │  │
│  └────────────────┬──────────────────────────────────┘  │
│                   ▼                                      │
│  ┌───────────────────────────────────────────────────┐  │
│  │ Output: JSON Report                               │  │
│  └───────────────────────────────────────────────────┘  │
└──────────────────┬──────────────────────────────────────┘
                   │ SIGHUP (graceful shutdown)
                   ▼
┌─────────────────────────────────────────────────────────┐
│              WeaverController                           │
│  - Parses JSON report                                   │
│  - Extracts ValidationReport                            │
│  - Checks for violations                                │
└──────────────────┬──────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│              Test Result                                │
│  - Pass if violations == 0                              │
│  - Fail if violations > 0                               │
└─────────────────────────────────────────────────────────┘
```

---

## Key Concepts

### Schema-First Validation

**Traditional Testing:**
```
Test passes ✅ → Assume feature works → False positive ❌
```

**Weaver Validation:**
```
Schema defines behavior → Runtime telemetry matches → True positive ✅
```

### Live Validation

Weaver validates telemetry **as it's emitted**, not after the fact:
- Real-time feedback
- Immediate violation detection
- No post-processing required

### Registry Coverage

Weaver tracks what percentage of the registry is used:
```json
{
  "registry_coverage": 0.72,  // 72% of registry used
  "seen_registry_attributes": {
    "http.method": 10,
    "http.status_code": 10
  },
  "seen_non_registry_attributes": {
    "custom.attribute": 5
  }
}
```

---

## Testing Strategy

### 80/20 Approach

**Phase 1: Core (80% value, 20% effort) ✅ DONE**
- OTLP listener
- Built-in advisors
- JSON reports

**Phase 2: Validation (15% value, 30% effort) ⚠️ PENDING**
- Span tests
- Resource tests
- Histogram tests

**Phase 3: Advanced (5% value, 50% effort) 🟢 FUTURE**
- Custom policies
- Streaming
- Custom templates

### Test Priorities

| Priority | Feature | Effort | Impact |
|----------|---------|--------|--------|
| 🔴 HIGH | OTLP end-to-end | Low | Critical |
| 🔴 HIGH | Span validation | Medium | Critical |
| 🟡 MEDIUM | Histogram metrics | Medium | Important |
| 🟢 LOW | Custom Rego | High | Nice-to-have |

---

## Quick Commands

### Start Weaver (via WeaverController)

```rust
use crate::telemetry::weaver_controller::{WeaverConfig, WeaverController};

let config = WeaverConfig {
    registry_path: PathBuf::from("registry/"),
    otlp_port: 4317,
    admin_port: 8080,
    output_dir: PathBuf::from("validation_output"),
    stream: false,
    inactivity_timeout: 30,
};

let mut weaver = WeaverController::new(config)?;
weaver.start()?;

// Run tests
run_tests()?;

// Get results
let report = weaver.stop_and_get_report()?;
assert_eq!(report.violations, 0);
```

### Start Weaver (manual CLI)

```bash
weaver registry live-check \
  --registry registry/ \
  --otlp-grpc-port 4317 \
  --admin-port 8080 \
  --format json \
  --output validation_output/ \
  --inactivity-timeout 30
```

### Stop Weaver (manual)

```bash
# Unix
kill -HUP <weaver_pid>

# Or via admin endpoint
curl -X POST http://localhost:8080/stop
```

---

## Common Issues

### Issue 1: Weaver not installed

**Error:**
```
Error: Command 'weaver' not found
```

**Solution:**
```bash
# Install Weaver
cargo install weaver

# Or use Docker
docker run -p 4317:4317 ghcr.io/open-telemetry/weaver:latest
```

### Issue 2: Registry not found

**Error:**
```
Error: Registry path 'registry/' does not exist
```

**Solution:**
```bash
# Verify registry exists
ls -la registry/

# Or provide correct path
let config = WeaverConfig {
    registry_path: PathBuf::from("/absolute/path/to/registry/"),
    ...
};
```

### Issue 3: Port already in use

**Error:**
```
Error: Address already in use (port 4317)
```

**Solution:**
```rust
// Use different port
let config = WeaverConfig {
    otlp_port: 4318,  // Changed from 4317
    ...
};
```

---

## Additional Resources

### External Documentation

- **Weaver Official Docs:** [Weaver GitHub](https://github.com/open-telemetry/weaver)
- **OpenTelemetry Semantic Conventions:** [OTel Semconv](https://opentelemetry.io/docs/specs/semconv/)
- **Rego Policy Language:** [Open Policy Agent](https://www.openpolicyagent.org/docs/latest/policy-language/)

### clnrm Documentation

- **Main README:** `../../README.md`
- **Weaver v1.2.0 Summary:** `../WEAVER_V1_2_0_VALIDATION_SUMMARY.md`
- **WeaverController Source:** `../../crates/clnrm-core/src/telemetry/weaver_controller.rs`

### Source Code

- **Weaver Source:** `../../vendors/weaver/crates/weaver_live_check/src/`
- **clnrm Registry:** `../../registry/`

---

## Contributing

### Adding New Features

1. **Check availability:** [FEATURE_MATRIX.md](./FEATURE_MATRIX.md)
2. **Review example:** [INTEGRATION_EXAMPLES.rs](./INTEGRATION_EXAMPLES.rs)
3. **Implement feature**
4. **Add tests**
5. **Update documentation**

### Reporting Issues

1. **Check known issues:** This README
2. **Review feature status:** [FEATURE_MATRIX.md](./FEATURE_MATRIX.md)
3. **File issue:** GitHub with example code

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| v1.2.0 | 2025-10-30 | Initial Weaver infrastructure complete |
| v1.2.0 (pending) | TBD | Live validation testing |
| v1.3.0 (planned) | TBD | Advanced features (streaming, histograms) |
| v2.0.0 (planned) | TBD | Custom policies, templates |

---

## Contact

For questions or issues with Weaver integration:

- **Project:** clnrm (Cleanroom Testing Framework)
- **Repository:** https://github.com/seanchatmangpt/clnrm
- **Issues:** GitHub Issues

---

**End of Index**
