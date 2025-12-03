# Changelog

All notable changes to the clnrm project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0] - 2025-12-03

### Breaking Changes

- **Config format renamed**: `[services.X]` → `[containers.X]`
- **Step format changed**: `service = "X"` → `container = "X"`
- **Command format changed**: `command = "..."` → `exec = [...]`
- **Metadata section**: `[test.metadata]` → `[test]`
- Removed deprecated `type = "generic_container"` field
- Removed deprecated `plugin = "..."` field

### Added
- **Comprehensive doctests** for config and error modules (20+ examples)
- **New documentation** for v2.0.0:
  - `docs/V2_0_0_ARCHITECTURE.md` - C4 diagrams and architecture overview
  - `docs/V2_0_0_MIGRATION_GUIDE.md` - v1.x to v2.0.0 migration
  - `docs/V2_0_0_CONFIG_REFERENCE.md` - Complete TOML reference
  - `docs/DOCTEST_GUIDE.md` - Doctest patterns for clnrm
- **Documentation archive**: Historical docs moved to `docs/archive/`

### Fixed
- **CRITICAL: Environment variables now work in container steps**
  - Prior versions created NEW containers per step, losing env vars
  - v2.0.0 uses `docker exec` semantics - commands run in RUNNING containers
  - Env vars persist across all steps in the same container

### Changed
- README.md updated with v2.0.0 canonical format
- SECURITY.md TODO fixed with proper contact information

### Migration

See `docs/V2_0_0_MIGRATION_GUIDE.md` for step-by-step migration instructions.

---

## [1.7.0] - 2025-12-03

### Added
- **Canonical Config Format** with proper `docker exec` semantics
  - New `[test]`, `[containers.X]`, `[[steps]]` config structure
  - Environment variables now work correctly (critical fix)
  - Commands execute via `docker exec` into running containers (not new containers)
  - Parse-time validation for all container and step references
- **`run_test_with_fallback()` function** for 80/20 backward compatibility
  - Tries new Config format first (docker exec semantics)
  - Falls back to legacy TestConfig for existing tests
  - Seamless migration path for existing test files
- **Sample test file** `examples/advanced-features/env-vars-test.clnrm.toml`
  - Validates environment variables work in docker exec
  - Demonstrates new canonical config format

### Fixed
- **CRITICAL: Environment variables now work in container steps**
  - Prior to v1.7.0, `execute_in_service()` created NEW containers, so env vars were lost
  - Now uses `docker exec` semantics - commands run in the SAME container where env vars are set
  - Validated with Docker: 4/4 steps passed in env-vars-test
- **Container keepalive** - containers now stay running for step execution
  - Added `command = ["sh", "-c", "while true; do sleep 1; done"]` pattern

### Changed
- Test execution path now prefers new Config format over legacy TestConfig
- Error messages improved for config parsing failures
- Clearer separation between new and legacy config formats

### Documentation
- Added `examples/advanced-features/env-vars-test.clnrm.toml` as reference
- Plan file documents v2 architecture design decisions

### Testing
- ✅ Docker validation: 4/4 steps passed in env-vars test
- ✅ Environment variables verified: MY_VAR, ANOTHER_VAR, DB_HOST
- ✅ Backward compatibility with legacy TestConfig format

## [1.6.0] - 2025-11-15

### Added
- **Docker-integration feature flag** for environment-dependent tests
  - Separates Docker-requiring tests from core unit tests
  - CI/CD compatible with optional Docker dependencies
  - Feature flag: `docker-integration` in Cargo.toml
  - Tests are compile-time gated; no runtime failures
- **New CI/CD workflow** (`.github/workflows/unit-tests.yml`)
  - Fast unit tests on all PRs (no Docker required)
  - Separate integration test workflow with Docker
  - Both Ubuntu and macOS validation

### Fixed
- **TOML Configuration Standardization** (all 131 test configs)
  - Fixed metadata sections: `[meta]` and `[test]` → `[test.metadata]` (42 files)
  - Removed redundant `plugin` fields from services (169 instances across 84 files)
  - Standardized timeout format: `timeout_seconds` and `timeout_ms` → `timeout = "XXs"` (15 files)
  - Fixed command format: string → array syntax (6 files)
  - **Pre-audit compliance:** 32% (42/131 files)
  - **Post-audit compliance:** 99.2% (130/131 files)
- **Unused import warning** in determinism_validation.rs

### Changed
- Environment-dependent tests now properly feature-gated
- CI/CD pipeline split into unit tests (fast, no Docker) and integration tests (full suite)
- All TOML test configs follow core team standards

### Documentation
- Added `docs/TOML_AUDIT_2025_11_15.md` with comprehensive audit report
- Updated `CLAUDE.md` with "Environment-Dependent Test Strategy" section
- Documented docker-integration feature usage and CI/CD strategy

### Performance
- No runtime performance impact
- CI/CD performance: Faster PR feedback (unit tests run without Docker)

### Testing
- ✅ All 203 unit tests passing
- ✅ All 131 TOML configs validated
- ✅ 99.2% compliance with core team standards
- ✅ Docker-independent tests verified

## [1.4.1] - 2025-11-01

### Added
- **Parallel container pre-warming** (80% faster initialization: 20-50s → 2-5s)
- **Lock-free idle queue** using SegQueue (50% faster acquire/release: 0.1-0.5ms → 0.05-0.2ms)
- **Health check lock optimization** (eliminated 100-500ms blocking on health checks)
- **VALIDATED status** added to all performance claims with benchmark references

### Fixed
- **CRITICAL: Removed all 28 unwrap/expect calls from production code**
  - `pool.rs:426` - Logic error now returns `Error` instead of panic
  - `orchestrator.rs` - State machine errors properly handled (7 fixes)
  - `cache.rs` - RwLock replaced with DashMap (19 fixes, lock poisoning eliminated)
  - `ports.rs` - Port lock failure properly handled
- **Documentation: Removed non-existent `--enable-pooling` CLI flag** (28 occurrences across docs)
  - Corrected to `CLNRM_ENABLE_POOLING=1` environment variable (the ONLY method)
- **Documentation: Fixed migration guide link** (MIGRATING_TO_V1_4_0.md → MIGRATION_V1_3_TO_V1_4.md)

### Changed
- **Configs wrapped in `Arc<T>`** for 15-25% fewer allocations
- **Documentation updated to v1.4.1** with clear implementation status
- **Performance claims marked as VALIDATED** with benchmark file references

### Performance
- **Initialization**: 20-50s → 2-5s (parallel pre-warming)
- **Acquire/release**: 0.1-0.5ms → 0.05-0.2ms (lock-free queue)
- **Overall improvement**: 12-13x faster than v1.3.0
- **Pool hit rate**: 92-95% (exceeds 90% target)
- **Throughput**: 500-1000 tests/s (10x improvement over v1.3.0)

### Documentation
- All `--enable-pooling` flag references replaced with `CLNRM_ENABLE_POOLING=1`
- Performance claims marked with `✅ VALIDATED` status
- Added implementation status sections to architecture docs
- Migration guide link fixed across all documentation

## [1.4.0] - 2025-10-30

### Added
- **Container pooling** for 10x performance improvement
  - Pre-warmed containers eliminate 80% of startup overhead
  - Lock-free atomic metrics with DashMap
  - Background health checks for non-blocking lifecycle management
- **Async ServicePlugin trait** for better concurrency
- **Semaphore-based concurrency control** with fair queuing
- **Atomic metrics** for zero-contention performance tracking

### Changed
- Container acquisition: 2-5s → 0.1-0.5ms (pool hit)
- Maximum concurrency: 50-100 → 500-1000 concurrent tests
- Throughput: 50 tests/s → 500-1000 tests/s

### Documentation
- Added container pooling guide
- Added performance tuning guide
- Added concurrency architecture deep-dive
- Added container pool implementation guide

## [1.3.0] - 2025-10-30

### Added
- **Weaver live-check infrastructure** (Phases 1-2)
  - WeaverController for OTLP collection
  - Schema registry validation
  - Live telemetry validation
- **Production hardening** with TDD validation
- **Performance benchmarks** and crate publishing workflow

### Changed
- Release process updated for crates.io publishing
- API stability guarantees for clap-noun-verb v1.0.0

## [1.2.1] - 2025-10-29

### Fixed
- Production hardening with comprehensive TDD validation
- Error handling improvements

## [1.2.0] - 2025-10-28

### Added
- **Weaver as core validation** - OTel schema validation as source of truth
- Comprehensive OpenTelemetry support
- Docker testcontainers backend

### Changed
- Validation hierarchy: Weaver > Compilation > Tests
- Framework now self-validates using Weaver

## [1.1.0] - 2025-10-20

### Added
- TOML-based test configuration
- Service plugin architecture
- CLI commands: init, run, validate, health, self-test, plugins

### Changed
- Improved error handling
- Better documentation

## [1.0.0] - 2025-10-01

### Added
- Initial release
- Docker container isolation
- Basic CLI interface
- Test execution framework

---

## Version Numbering

- **Major (x.0.0)**: Breaking API changes
- **Minor (0.x.0)**: New features, backward compatible
- **Patch (0.0.x)**: Bug fixes, performance improvements

## Links

- [Repository](https://github.com/seanchatmangpt/clnrm)
- [Documentation](https://github.com/seanchatmangpt/clnrm/tree/master/docs)
- [Releases](https://github.com/seanchatmangpt/clnrm/releases)
