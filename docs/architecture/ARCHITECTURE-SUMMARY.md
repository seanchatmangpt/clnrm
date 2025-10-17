# Cleanroom Testing Framework - Architecture Summary

**Version:** v0.7.0
**Date:** 2025-10-16
**Status:** Active Development

## Quick Reference

This summary provides a high-level overview of the Cleanroom Testing Framework architecture. For detailed specifications, see [v0.7.0-system-architecture.md](./v0.7.0-system-architecture.md).

## System Purpose

Cleanroom Testing Framework is a hermetic, deterministic integration testing platform that provides:
- **Container-based isolation** for reproducible tests
- **OTEL-first validation** ensuring observability claims are backed by real telemetry
- **Tera template system** for dynamic test configuration
- **Change-aware execution** for 10x faster iteration
- **Self-testing** framework that validates itself using its own capabilities

## Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                    CLI Layer (clnrm)                        │
├─────────────────────────────────────────────────────────────┤
│  Commands: template, dev, run, record, fmt, lint, etc.     │
└───────────────────────────┬─────────────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────────┐
│               Core Library (clnrm-core)                     │
├─────────────────────────────────────────────────────────────┤
│  • Template System    (Tera rendering, variable resolution)│
│  • Configuration      (TOML parsing, schema validation)     │
│  • Execution Engine   (Container orchestration, scenarios)  │
│  • Validation         (OTEL span/trace/graph validators)    │
│  • Reporting          (JSON, JUnit, SHA-256 digests)        │
│  • Support Systems    (Cache, Watch, Telemetry)             │
└─────────────────────────────────────────────────────────────┘
```

## Architecture Principles

### 1. Core Team Standards (MANDATORY)

#### Error Handling
- **NEVER** use `.unwrap()` or `.expect()` in production code
- All functions return `Result<T, CleanroomError>`
- Structured errors with context, source, timestamp

#### Async/Sync Rules
- **NEVER** make trait methods async (breaks dyn compatibility)
- Traits use sync methods with internal `tokio::task::block_in_place`
- Async for I/O, sync for computation

#### Testing Standards
- AAA pattern (Arrange, Act, Assert)
- Descriptive test names: `test_<component>_<scenario>_<expected_result>`
- No fake `Ok(())` - use `unimplemented!()` for incomplete features

### 2. Design Constraints

- **Modular Design**: Files under 500 lines
- **Workspace Isolation**: Experimental features (AI) in separate crates
- **Hermetic Execution**: Each test in fresh container environment
- **Deterministic by Default**: Seeded RNG, frozen clock, stable ordering
- **Observable by Default**: OTEL tracing/metrics/logs built-in

## Key Workflows

### Template Rendering Flow

```
.clnrm.toml.tera → Resolve Variables → Build Tera Context →
  Render Template → Flat TOML → Parse Config
```

**Variable Precedence:** Template vars → ENV → Defaults

### Test Execution Flow

```
Parse TOML → Initialize Backend → Start Services →
  Execute Scenarios → Collect Telemetry → Validate Expectations →
  Generate Reports → Cleanup
```

### Change-Aware Execution

```
Scenario Hash → Check Cache → {Match: Skip, No Match: Execute} →
  Store Results
```

## Module Structure

### Template Module (`template/`)
- **Purpose**: Tera-based template rendering with custom functions
- **Files**: mod.rs (150L), context.rs (200L), functions.rs (250L), determinism.rs (100L)
- **Key Functions**: `env()`, `now_rfc3339()`, `sha256()`, `toml_encode()`
- **Macro Library**: 8 reusable macros for 85% boilerplate reduction

### Configuration Module (`config.rs`)
- **Purpose**: TOML parsing and schema validation
- **Size**: ~500 lines (single file, multiple versions)
- **Formats**: v0.4, v0.6, v0.7 schema support
- **Validation**: Required fields, type checking, reference integrity

### Execution Module (`cleanroom.rs`, `scenario.rs`)
- **Purpose**: Container orchestration and test execution
- **Components**: CleanroomEnvironment, ServicePlugin trait, Backend trait
- **Guarantees**: Fresh environment per test, automatic cleanup, timeout enforcement

### Validation Module (`validation/`)
- **Purpose**: OTEL-first validation of test expectations
- **Validators**: Span, Graph, Hermeticity, Count, Order, Window, Status
- **Output**: ValidationReport with first-failure focus, SHA-256 digest

## Performance Targets

| Operation | Target | Status |
|-----------|--------|--------|
| Template cold run | ≤5s | ✅ Achieved |
| Template hot reload | ≤3s (p95) | ✅ Achieved |
| Container startup | ≤2s | ⚠️ To measure |
| OTEL span collection | ≤500ms | ⚠️ To measure |
| Validation suite | ≤1s | ⚠️ To measure |

**Improvements from v0.6:**
- 30-50% faster suite execution (change-aware)
- 60-80% scenarios skipped on repeat runs
- <3s hot reload latency

## Security & Hermeticity

### Network Isolation
- Isolated Docker network per test
- No host network access
- DNS limited to container network

### File System Isolation
- Fresh volumes per test
- No shared mounts
- Automatic cleanup

### Process Isolation
- Separate PID namespace
- Resource limits (CPU, memory)
- User namespace isolation

## Technology Stack

| Component | Technology | Version |
|-----------|-----------|---------|
| Language | Rust | 2021 Edition |
| Templates | Tera | 1.19 |
| Containers | testcontainers-rs | 0.25 |
| OTEL | opentelemetry | 0.31 |
| Config | TOML | 0.9 |
| CLI | clap | 4.5 |
| Async | tokio | 1.0 |

## Key Architecture Decisions

### ADR-001: Sync Trait Methods
**Decision:** All trait methods are synchronous for dyn compatibility
**Impact:** ✅ Enables plugin system, ❌ Slight performance overhead

### ADR-002: Tera for Templates
**Decision:** Use Tera with custom functions for template rendering
**Impact:** ✅ Jinja2-like syntax, ✅ Pure Rust, ⚠️ Careful error handling needed

### ADR-003: Flat TOML Schema
**Decision:** v0.7 uses flat TOML with top-level sections
**Impact:** ✅ Simpler templates, ✅ Better errors, ❌ Breaking change from v0.6

### ADR-004: Change-Aware Execution
**Decision:** SHA-256 hash of scenario TOML, skip unchanged scenarios
**Impact:** ✅ 30-50% faster, ✅ Deterministic, ⚠️ Cache invalidation needed

### ADR-005: OTEL-First Validation
**Decision:** All validation based on OpenTelemetry spans
**Impact:** ✅ Real observability, ✅ Portable, ❌ Requires OTEL SDK

## File Organization

```
crates/
├── clnrm/                  # CLI binary
├── clnrm-core/             # Core framework library
│   ├── src/
│   │   ├── template/       # Template rendering
│   │   ├── config.rs       # Configuration parsing
│   │   ├── cleanroom.rs    # Execution environment
│   │   ├── scenario.rs     # Scenario orchestration
│   │   ├── validation/     # OTEL validators
│   │   ├── reporting/      # Report generation
│   │   ├── services/       # Service plugins
│   │   ├── backend/        # Container backend
│   │   ├── cache/          # Change detection
│   │   ├── watch/          # File watching
│   │   └── telemetry.rs    # OTEL integration
│   └── tests/              # Integration tests
├── clnrm-shared/           # Shared utilities
└── clnrm-ai/               # Experimental AI features (isolated)
```

## Getting Started

### For Developers

1. Read [v0.7.0-system-architecture.md](./v0.7.0-system-architecture.md) for detailed design
2. Review [CLAUDE.md](/Users/sac/clnrm/CLAUDE.md) for core team standards
3. Check [PRD-v1.md](/Users/sac/clnrm/PRD-v1.md) for requirements and implementation status

### For Contributors

1. Follow core team standards (no unwrap/expect, sync traits, AAA tests)
2. Keep files under 500 lines
3. Add tests for all new functionality
4. Run `cargo clippy -- -D warnings` before committing
5. Ensure `cargo run -- self-test` passes

## Next Steps

1. **Performance Benchmarking**: Measure container startup, OTEL collection, validation times
2. **File Size Audit**: Track module sizes against 500-line budget
3. **Implementation Plan**: Prioritize features from ADRs
4. **Documentation**: Keep architecture docs in sync with code

## Related Documents

- [Detailed Architecture](./v0.7.0-system-architecture.md) - Complete system design
- [PRD](../PRD-v1.md) - Product requirements and implementation status
- [CLAUDE.md](../CLAUDE.md) - Core team standards and development guide
- [CLI Guide](../CLI_GUIDE.md) - Command-line interface documentation
- [TOML Reference](../TOML_REFERENCE.md) - Configuration format specification

---

**Maintained by:** System Architecture Team
**Last Updated:** 2025-10-16
**Questions?** See GitHub Issues or consult detailed architecture document
