# Changelog

All notable changes to the clnrm project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
