# gVisor Implementation - Quick Reference Guide

**Quick Links:**
- [Full Roadmap](./GVISOR_IMPLEMENTATION_ROADMAP.md)
- [Validation Checklist](./GVISOR_DOCKER_ELIMINATION_VALIDATION.md)
- [Service Management](./GVISOR_SERVICE_MANAGEMENT.md)

---

## Quick Start for Developers

### Setup gVisor

```bash
# Install gVisor
sudo apt-get update
sudo apt-get install gvisor-runsc

# Verify installation
runsc --version
```

### Enable gVisor Backend

```toml
# .clnrm.toml
[backend]
type = "gvisor"
platform = "ptrace"  # or "kvm" for better performance
```

### Run Tests with gVisor

```bash
# Set environment variable
export CLNRM_BACKEND=gvisor

# Run tests
cargo test --features gvisor

# Run specific test
cargo test --test integration_test --features gvisor
```

---

## Phase Implementation Checklist

### ✅ Phase 1: Foundation (Week 1)
```bash
# Files to create:
crates/clnrm-core/src/backend/gvisor/mod.rs
crates/clnrm-core/src/backend/gvisor/runtime.rs
crates/clnrm-core/src/backend/gvisor/image.rs
crates/clnrm-core/src/backend/gvisor/config.rs

# Dependencies to add:
oci-distribution = "0.11"
oci-spec = "0.6"
```

**Validation:**
```bash
# Test basic container execution
cargo test backend::gvisor::tests::test_basic_container

# Verify feature flags
cargo build --features gvisor
cargo build --features testcontainers
```

---

### ✅ Phase 2: Core Runtime (Week 2)
```bash
# Files to create:
crates/clnrm-core/src/backend/gvisor/execution.rs
crates/clnrm-core/src/backend/gvisor/network.rs
crates/clnrm-core/src/backend/gvisor/filesystem.rs
crates/clnrm-core/src/backend/gvisor/ports.rs
```

**Validation:**
```bash
# Test network isolation
cargo test backend::gvisor::network::test_isolation

# Test port allocation
cargo test backend::gvisor::ports::test_allocation

# Benchmark performance
cargo bench --bench container_startup
```

---

### ✅ Phase 3: Services (Week 3)
```bash
# Files to create:
crates/clnrm-core/src/services/gvisor/surrealdb.rs
crates/clnrm-core/src/services/gvisor/generic.rs
crates/clnrm-core/src/services/registry.rs
crates/clnrm-core/src/services/health.rs
```

**Validation:**
```bash
# Test SurrealDB service
cargo test services::gvisor::surrealdb::test_lifecycle

# Test service registry
cargo test services::registry::test_discovery
```

---

### ✅ Phase 4: Migration (Week 4)
```bash
# Migration tools:
scripts/migrate_tests.rs
scripts/migrate_toml_configs.py

# Commands:
clnrm migrate tests/ --verbose
clnrm migrate examples/ --dry-run
```

**Validation:**
```bash
# Verify all tests pass
CLNRM_BACKEND=gvisor cargo test --all

# Check for Docker references
./scripts/validate_docker_elimination.sh
```

---

### ✅ Phase 5: Integration (Week 5)
```bash
# Files to create:
crates/clnrm-core/src/backend/gvisor/telemetry.rs
crates/clnrm-core/tests/telemetry/gvisor_otlp_test.rs
tests/weaver/gvisor_weaver_test.rs
benches/gvisor_performance.rs
```

**Validation:**
```bash
# Test OTLP export
cargo test telemetry::gvisor_otlp_test

# Run benchmarks
cargo bench --bench gvisor_performance

# Weaver integration
cargo test --test weaver_gvisor_integration
```

---

### ✅ Phase 6: Validation (Week 6)
```bash
# Validation scripts:
scripts/validate_docker_elimination.sh
scripts/validate_gvisor_tests.sh

# Documentation:
book/src/backends/gvisor.md
docs/GVISOR_MIGRATION_GUIDE.md
docs/GVISOR_ARCHITECTURE.md
docs/GVISOR_TROUBLESHOOTING.md
```

**Validation:**
```bash
# Full validation suite
./scripts/validate_docker_elimination.sh
./scripts/validate_gvisor_tests.sh

# Build documentation
mdbook build book/

# Run full test suite
cargo test --all --features gvisor
```

---

## File Structure

```
crates/clnrm-core/src/backend/gvisor/
├── mod.rs                 # Main module, GvisorBackend struct
├── config.rs              # Configuration types
├── runtime.rs             # runsc wrapper (create, start, exec, delete)
├── image.rs               # OCI image management and caching
├── execution.rs           # Container execution engine
├── network.rs             # Network isolation and port mapping
├── filesystem.rs          # Volume mounts and filesystem isolation
├── ports.rs               # Port allocation system
└── telemetry.rs           # OpenTelemetry integration

crates/clnrm-core/src/services/gvisor/
├── mod.rs                 # Service module
├── surrealdb.rs           # SurrealDB service plugin
├── generic.rs             # Generic service plugin (any image)
├── postgres.rs            # PostgreSQL service plugin
└── redis.rs               # Redis service plugin

crates/clnrm-core/src/services/
├── registry.rs            # Service registry and discovery
├── health.rs              # Health check system
└── factory.rs             # Service plugin factory

scripts/
├── migrate_tests.rs       # Test migration tool
├── migrate_toml_configs.py # TOML config migration
├── validate_docker_elimination.sh
└── validate_gvisor_tests.sh

docs/
├── GVISOR_IMPLEMENTATION_ROADMAP.md  # This document
├── GVISOR_MIGRATION_GUIDE.md         # User migration guide
├── GVISOR_ARCHITECTURE.md            # Technical architecture
├── GVISOR_TROUBLESHOOTING.md         # Common issues
└── GVISOR_QUICK_REFERENCE.md         # Quick reference

book/src/backends/
└── gvisor.md              # User-facing documentation
```

---

## Common Commands

### Development

```bash
# Build with gVisor
cargo build --features gvisor

# Run tests
CLNRM_BACKEND=gvisor cargo test

# Run benchmarks
cargo bench --features gvisor

# Check for Docker references
grep -rn "docker\s" --include="*.rs" . | grep -v docs
```

### Migration

```bash
# Migrate single test
./scripts/migrate_tests.rs tests/example_test.rs

# Migrate all tests
find tests -name "*.rs" -exec ./scripts/migrate_tests.rs {} \;

# Migrate TOML configs
python3 scripts/migrate_toml_configs.py

# Validate migration
./scripts/validate_docker_elimination.sh
```

### Validation

```bash
# Docker elimination check
./scripts/validate_docker_elimination.sh

# Full test suite
./scripts/validate_gvisor_tests.sh

# Performance benchmarks
cargo bench --features gvisor --bench '*'

# CI simulation
act -j test-gvisor
```

---

## Performance Targets

| Metric | Baseline (Docker) | Target (gVisor) | Command |
|--------|------------------|-----------------|---------|
| Cold start | 3-5s | < 3s | `cargo bench container_startup_cold` |
| Warm start | 1-2s | < 500ms | `cargo bench container_startup_warm` |
| Memory overhead | 150-200MB | < 100MB | `cargo bench memory_usage` |
| Network latency | 0.5-1ms | < 2ms | `cargo bench network_latency` |

---

## Troubleshooting

### Common Issues

**Issue**: `runsc not found`
```bash
# Solution: Install gVisor
sudo apt-get install gvisor-runsc
# Or download directly
wget https://storage.googleapis.com/gvisor/releases/release/latest/runsc
sudo install runsc /usr/local/bin/
```

**Issue**: Permission denied
```bash
# Solution: Add user to required groups
sudo usermod -aG docker $USER
# Or run with sudo (not recommended)
```

**Issue**: Image pull fails
```bash
# Solution: Check network and credentials
docker login  # If pulling from private registry
# Or use local cache
export CLNRM_IMAGE_CACHE=/path/to/cache
```

**Issue**: Tests fail with gVisor
```bash
# Solution: Check backend configuration
export CLNRM_BACKEND=gvisor
export RUST_LOG=debug
cargo test -- --nocapture
```

---

## Key Decision Points

### When to Use Each Platform

| Platform | Use When | Performance | Compatibility |
|----------|----------|-------------|---------------|
| ptrace | Default, maximum compatibility | Medium | Excellent |
| kvm | Best performance, KVM available | High | Good (requires KVM) |
| systrap | New platform, good balance | Medium-High | Good |

### Feature Flag Strategy

```toml
# Development: Both backends available
[features]
default = ["gvisor", "testcontainers"]

# Production: gVisor only
[features]
default = ["gvisor"]

# CI: Test both
[features]
default = ["gvisor"]
testcontainers = []
```

---

## Integration Points

### With Existing Code

1. **Backend Trait**: Implement `Backend` for `GvisorBackend`
2. **Service Plugins**: Implement `ServicePlugin` for gVisor services
3. **Telemetry**: Use existing `opentelemetry` infrastructure
4. **Configuration**: Extend `.clnrm.toml` format

### With External Tools

1. **CI/CD**: GitHub Actions, GitLab CI
2. **Monitoring**: Prometheus, Grafana
3. **Telemetry**: Jaeger, Zipkin, OTLP collectors
4. **Registries**: Docker Hub, GHCR, ECR

---

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_gvisor_container_creation() {
    let backend = GvisorBackend::new(GvisorConfig::default()).unwrap();
    let cmd = Cmd::new("echo").arg("test");
    let result = backend.run_cmd(cmd).unwrap();
    assert!(result.success());
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_service_lifecycle() {
    let backend = Arc::new(GvisorBackend::new(GvisorConfig::default()).unwrap());
    let service = GvisorSurrealDbPlugin::new(backend);
    let handle = service.start().unwrap();
    assert_eq!(service.health_check(&handle), HealthStatus::Healthy);
    service.stop(handle).unwrap();
}
```

### Property Tests
```rust
proptest! {
    #[test]
    fn test_port_allocation_no_conflicts(ports in prop::collection::vec(1024u16..65535, 1..100)) {
        let allocator = PortAllocator::new(1024..65535);
        let mut allocated = Vec::new();

        for _ in ports {
            if let Ok(port) = allocator.allocate() {
                assert!(!allocated.contains(&port));
                allocated.push(port);
            }
        }
    }
}
```

---

## Release Checklist

- [ ] All tests pass with gVisor backend
- [ ] Zero Docker references in code
- [ ] Documentation complete
- [ ] Performance benchmarks pass
- [ ] Migration guide published
- [ ] CI/CD updated
- [ ] Security audit complete
- [ ] User feedback incorporated
- [ ] Changelog updated
- [ ] Release notes prepared

---

## Support and Resources

### Documentation
- [gVisor Documentation](https://gvisor.dev/docs/)
- [OCI Image Spec](https://github.com/opencontainers/image-spec)
- [OCI Runtime Spec](https://github.com/opencontainers/runtime-spec)

### Community
- GitHub Issues: Report bugs and request features
- Discussions: Ask questions and share feedback
- Slack/Discord: Real-time support

### Monitoring
- Performance Dashboard: Track metrics
- Error Tracking: Monitor failures
- User Analytics: Usage patterns

---

**Last Updated**: 2026-01-05
**Version**: 1.0
**Maintainer**: Platform Team
