# Tera v0.6.0 Architecture - Deliverables Summary

## Overview

This document summarizes all architectural deliverables for the Tera template system integration into clnrm v0.6.0.

**Created**: 2025-10-16
**Status**: Design Phase Complete
**Next Phase**: Implementation (Week 1 starts upon approval)

---

## Delivered Documents

### 1. Core Architecture Document

**File**: `/Users/sac/clnrm/docs/architecture/tera-v0.6.0-architecture.md`
**Size**: ~35,000 words
**Sections**: 15 major sections

**Contents**:
- Executive summary and system overview
- Module structure and responsibilities
- Detailed data flow pipeline
- Tera integration points (variables, loops, conditionals, includes, macros)
- Configuration schema extensions (7 new config blocks)
- Custom Tera functions (env, now_rfc3339, sha256, toml_encode)
- Template context model
- Validation architecture (OrderValidator, StatusValidator)
- Reporting system (JSON, JUnit, Digest)
- Backward compatibility strategy
- Error handling strategy
- Performance considerations
- Testing strategy (unit, integration, E2E, red-team)
- Implementation roadmap
- Security considerations

**Key Design Decisions**:
1. "Render First, Parse Second" architecture
2. Automatic template detection via syntax markers
3. Backward compatibility with non-template files
4. Fail-fast error handling
5. Custom Tera functions for common operations

---

### 2. User Guide

**File**: `/Users/sac/clnrm/docs/TERA_TEMPLATE_GUIDE.md`
**Size**: ~12,000 words
**Sections**: 12 major sections

**Contents**:
- Quick start guide
- Template basics (detection, rendering, sections)
- Variable substitution patterns
- Loops and matrix expansion
- Conditionals (if/elif/else)
- Includes and reusability
- Macros for reusable blocks
- Custom functions reference
- Determinism configuration
- Best practices
- Common patterns (multi-env, feature flags, CI/CD, service mesh)
- Troubleshooting guide

**Target Audience**: End users writing `.clnrm.toml` templates

---

### 3. Example Templates

**Location**: `/Users/sac/clnrm/examples/templates/`
**Count**: 7 templates + README

**Files**:
1. `simple-variables.clnrm.toml` - Basic variable substitution
2. `matrix-expansion.clnrm.toml` - Loop-based scenario generation
3. `multi-environment.clnrm.toml` - Environment-specific configs
4. `service-mesh.clnrm.toml` - Complex multi-service orchestration
5. `ci-integration.clnrm.toml` - CI/CD pipeline integration
6. `macros-and-includes.clnrm.toml` - Reusable template blocks
7. `advanced-validators.clnrm.toml` - New v0.6.0 validators
8. `README.md` - Example documentation

**Purpose**:
- Demonstrate all Tera features
- Provide copy-paste starting points
- Validate architecture through real-world use cases

---

### 4. Implementation Roadmap

**File**: `/Users/sac/clnrm/docs/architecture/tera-implementation-roadmap.md`
**Size**: ~8,000 words
**Timeline**: 4 weeks

**Contents**:
- Week-by-week breakdown
- Day-by-day tasks
- Deliverables per phase
- Testing strategies per phase
- Risk mitigation
- Success criteria
- Post-release tasks

**Phases**:
1. Week 1: Core Tera integration & schema extensions
2. Week 2: Validators & reporting
3. Week 3: Documentation & examples (partially complete)
4. Week 4: Release preparation

---

## Architecture Highlights

### Module Structure

```
crates/clnrm-core/src/
├── template/              # NEW - Tera integration
│   ├── mod.rs
│   ├── context.rs
│   ├── functions.rs
│   ├── determinism.rs
│   └── loader.rs
├── config/                # MODIFIED - Add rendering
├── validation/            # EXTENDED - New validators
│   ├── order_validator.rs      # NEW
│   └── status_validator.rs     # NEW
└── reporting/             # NEW - Report generation
    ├── json.rs
    ├── junit.rs
    └── digest.rs
```

### Data Flow

```
.clnrm.toml (template)
    ↓
Detect Tera syntax ({{ or {%)
    ↓
Render with Tera (if template)
    ↓
Parse TOML (serde_toml)
    ↓
Execute scenarios
    ↓
Validate spans
    ↓
Generate reports
    ↓
Exit
```

### Key Features

1. **Template Detection**: Automatic (no config needed)
2. **Backward Compatibility**: Non-templates work unchanged
3. **Custom Functions**: `env()`, `now_rfc3339()`, `sha256()`, `toml_encode()`
4. **Matrix Expansion**: `{% for %}` loops for scenario generation
5. **Conditionals**: `{% if %}` for environment-specific config
6. **New Validators**: Order (temporal) and Status (OK/ERROR/UNSET)
7. **Reporting**: JSON, JUnit, SHA-256 digest
8. **Determinism**: Clock freezing and seeding for reproducibility

---

## Configuration Extensions

### New Top-Level Sections

```toml
[determinism]
seed = 42
freeze_clock = "2025-01-01T00:00:00Z"

[report]
json = "results/report.json"
junit = "results/junit.xml"
digest = "results/test.digest"

[limits]
cpu_millicores = 2000
memory_mb = 4096
```

### New Expectation Blocks

```toml
[expect.order]
must_precede = [
    ["container.start", "container.exec"],
]

[expect.status]
all = "OK"

[expect.status.by_name]
"container.*" = "OK"
"error_*" = "ERROR"
```

### Template Sections (Not Rendered)

```toml
[template.vars]
service = "clnrm"
version = "0.6.0"

[template.matrix]
exporters = ["stdout", "otlp", "jaeger"]

[template.otel]
endpoint = "http://localhost:4318"
```

---

## Testing Strategy

### Test Pyramid

```
       ┌─────────────┐
       │ E2E (10)    │  Full pipeline
       └─────────────┘
            ▲
       ┌────┴─────┐
       │Integration│   Template → TOML
       │  (20)     │
       └───────────┘
            ▲
       ┌────┴─────┐
       │Unit (50)  │   Functions, validators
       └───────────┘
```

**Total**: 150+ tests

### Coverage Goals

- Unit tests: >90% coverage
- Integration tests: All major paths
- E2E tests: All example templates
- Red-team tests: No false positives

---

## Implementation Effort

### Estimated Time

- **Week 1**: Core Tera + Schema (40 hours)
- **Week 2**: Validators + Reporting (40 hours)
- **Week 3**: Docs + Examples (40 hours) - *Partially complete*
- **Week 4**: Testing + Release (40 hours)

**Total**: 160 hours (4 weeks @ 40 hours/week)

### Team Size

- 1-2 developers
- 1 reviewer (for code review)

### Complexity

**Medium**: Leveraging Tera library (proven, mature) reduces custom code by ~2000 lines compared to custom interpolation.

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Tera API changes | Low | Medium | Pin version to 1.19 |
| Backward compat issues | Medium | High | Extensive testing |
| Performance regression | Low | Medium | Benchmarking |
| Template injection | Low | High | Security audit |

**Overall Risk**: Low

---

## Success Metrics

### Functional

- [ ] All existing tests pass (no regression)
- [ ] Template rendering works
- [ ] New validators work
- [ ] Reports generate correctly

### Quality

- [ ] Zero clippy warnings (`-D warnings`)
- [ ] Test coverage >80%
- [ ] No `.unwrap()` in production
- [ ] All errors meaningful

### Performance

- [ ] Template rendering <1ms (typical files)
- [ ] No memory leaks
- [ ] CI tests <5 minutes

### Documentation

- [ ] All features documented
- [ ] Examples comprehensive
- [ ] Migration guide clear

---

## Next Steps

### Immediate (Upon Approval)

1. **Review & Approve**: Stakeholder review of architecture
2. **Kickoff Meeting**: Align on timeline and responsibilities
3. **Start Week 1**: Begin core Tera integration

### Week 1 Deliverables

- Core `template/` module implemented
- Custom Tera functions working
- Config schema extensions added
- Unit tests passing (20+)

### Week 2 Deliverables

- OrderValidator and StatusValidator implemented
- Reporting system (JSON, JUnit, Digest) working
- Integration tests passing (30+)

### Week 3 Deliverables

- Documentation complete (README, TOML_REFERENCE, migration guide)
- All example templates tested
- E2E tests passing (10+)

### Week 4 Deliverables

- Code review complete
- Performance optimizations applied
- Security audit passed
- v0.6.0 released to crates.io

---

## Open Questions

### For Stakeholder Review

1. **Template Syntax**: Is Jinja2-like syntax (Tera) acceptable, or prefer alternative (Handlebars, Liquid)?
   - **Recommendation**: Tera (Jinja2-like) for familiarity

2. **Resource Limits**: Should template size limit (1 MB) be configurable?
   - **Recommendation**: Hard limit for v0.6.0, make configurable in v0.7.0

3. **Template Inheritance**: Should we support Tera's `{% extends %}` in v0.6.0?
   - **Recommendation**: Defer to v0.7.0 (not in examples, low demand)

4. **Additional Reporters**: HTML reporter requested?
   - **Recommendation**: JSON + JUnit + Digest sufficient for v0.6.0

5. **CLI Commands**: Add `clnrm template render <file>` command?
   - **Recommendation**: Yes, useful for debugging templates

---

## Dependencies

### Rust Crates

```toml
[dependencies]
tera = "1.19"           # Template engine
sha2 = "0.10"           # SHA-256 hashing
serde_json = "1.0"      # JSON reporting (existing)
# ... existing dependencies
```

### External Tools

- Docker or Podman (for testing)
- Rust 1.70+ (existing requirement)

---

## Backward Compatibility

### v0.5.0 Files

All existing `.clnrm.toml` files work unchanged in v0.6.0:

```toml
# This works in both v0.5.0 and v0.6.0
[meta]
name = "my_test"

[otel]
exporter = "stdout"

[[scenario]]
name = "test_1"
run = "clnrm run"
```

### Migration Path

**Option 1**: Keep existing files (no changes needed)

**Option 2**: Convert to templates for DRY benefits:

```toml
# v0.6.0 template version
[template.vars]
exporter = "stdout"

[meta]
name = "my_test"

[otel]
exporter = "{{ vars.exporter }}"

[[scenario]]
name = "test_1"
run = "clnrm run --otel-exporter {{ vars.exporter }}"
```

**Migration Guide**: `docs/MIGRATION_v0.5_to_v0.6.md` (to be written Week 3)

---

## Documentation Status

| Document | Status | Location |
|----------|--------|----------|
| Architecture | Complete | `docs/architecture/tera-v0.6.0-architecture.md` |
| User Guide | Complete | `docs/TERA_TEMPLATE_GUIDE.md` |
| Examples | Complete | `examples/templates/` (7 files) |
| Roadmap | Complete | `docs/architecture/tera-implementation-roadmap.md` |
| Migration Guide | Pending | `docs/MIGRATION_v0.5_to_v0.6.md` (Week 3) |
| README Update | Pending | Update root `README.md` (Week 3) |
| TOML Reference Update | Pending | Update `docs/TOML_REFERENCE.md` (Week 3) |

---

## Approval Sign-Off

### Stakeholder Review

- [ ] Architecture approved
- [ ] Timeline approved
- [ ] Resource allocation approved
- [ ] Open questions resolved

### Signatures

- **Architect**: _____________________________ Date: __________
- **Tech Lead**: _____________________________ Date: __________
- **Product Owner**: _________________________ Date: __________

---

## Appendix: File Locations

All deliverables are located in the clnrm repository:

```
/Users/sac/clnrm/
├── docs/
│   ├── architecture/
│   │   ├── tera-v0.6.0-architecture.md        # COMPLETE
│   │   ├── tera-implementation-roadmap.md     # COMPLETE
│   │   └── DELIVERABLES_SUMMARY.md            # THIS FILE
│   └── TERA_TEMPLATE_GUIDE.md                 # COMPLETE
└── examples/
    └── templates/
        ├── simple-variables.clnrm.toml        # COMPLETE
        ├── matrix-expansion.clnrm.toml        # COMPLETE
        ├── multi-environment.clnrm.toml       # COMPLETE
        ├── service-mesh.clnrm.toml            # COMPLETE
        ├── ci-integration.clnrm.toml          # COMPLETE
        ├── macros-and-includes.clnrm.toml     # COMPLETE
        ├── advanced-validators.clnrm.toml     # COMPLETE
        └── README.md                          # COMPLETE
```

---

## Contact

For questions about this architecture:

- **GitHub Issues**: https://github.com/seanchatmangpt/clnrm/issues
- **Architecture Lead**: [To be filled]
- **Implementation Lead**: [To be filled]

---

**Status**: Design Phase Complete - Ready for Implementation
