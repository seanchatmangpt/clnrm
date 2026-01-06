# gVisor Integration Documentation

**Complete Network and Filesystem Isolation Without Docker**

This directory contains comprehensive documentation for replacing Docker/testcontainers with gVisor (runsc) for complete isolation without daemon dependencies.

## Quick Navigation

### 🚀 Getting Started
1. **[Quick Start Guide](GVISOR_QUICK_START.md)** - Install and run your first container
2. **[Architecture Overview](GVISOR_ISOLATION_DESIGN.md)** - Complete technical design
3. **[Implementation Checklist](GVISOR_IMPLEMENTATION_CHECKLIST.md)** - Step-by-step tasks

### 📊 Decision Making
- **[Executive Summary](GVISOR_EXECUTIVE_SUMMARY.md)** - Business case and ROI
- **[Docker vs gVisor Comparison](GVISOR_VS_DOCKER.md)** - Detailed feature comparison
- **[Architecture Diagrams](GVISOR_ARCHITECTURE_DIAGRAMS.md)** - Visual architecture reference

### 🔧 Implementation
- **[Implementation Checklist](GVISOR_IMPLEMENTATION_CHECKLIST.md)** - Phase-by-phase tasks
- **[Isolation Design](GVISOR_ISOLATION_DESIGN.md)** - Network & filesystem isolation details

## Document Index

### Core Design Documents

| Document | Purpose | Audience |
|----------|---------|----------|
| [GVISOR_ISOLATION_DESIGN.md](GVISOR_ISOLATION_DESIGN.md) | Complete technical architecture | Engineers |
| [GVISOR_QUICK_START.md](GVISOR_QUICK_START.md) | Getting started guide | All |
| [GVISOR_VS_DOCKER.md](GVISOR_VS_DOCKER.md) | Feature comparison | Decision makers |
| [GVISOR_ARCHITECTURE_DIAGRAMS.md](GVISOR_ARCHITECTURE_DIAGRAMS.md) | Visual architecture | All |
| [GVISOR_IMPLEMENTATION_CHECKLIST.md](GVISOR_IMPLEMENTATION_CHECKLIST.md) | Implementation tasks | Engineers |
| [GVISOR_EXECUTIVE_SUMMARY.md](GVISOR_EXECUTIVE_SUMMARY.md) | Business case | Leadership |

### Additional Resources

| Document | Purpose |
|----------|---------|
| GVISOR_IMPLEMENTATION_ROADMAP.md | Phase-by-phase plan |
| GVISOR_MIGRATION_PLAN.md | Migration strategy |
| GVISOR_SERVICE_MANAGEMENT.md | Service orchestration |
| GVISOR_DOCKER_ELIMINATION_VALIDATION.md | Validation framework |

## Key Features Designed

### 1. Network Isolation ✅
- **Network Namespaces**: Isolated network per container
- **veth Pairs**: Virtual ethernet for host-container communication
- **IP Allocation**: Thread-safe IP allocator (10.88.0.0/16)
- **Port Mapping**: iptables-based DNAT/MASQUERADE
- **DNS**: Custom resolv.conf per container

### 2. Filesystem Isolation ✅
- **OCI Images**: Pull with skopeo, unpack with umoci
- **Rootfs**: Proper layer extraction and whiteout handling
- **Mounts**: /proc, /dev, /sys, /tmp with correct options
- **Volumes**: Bind mounts with MS_SLAVE propagation
- **Permissions**: Automated permission setup

### 3. Resource Management ✅
- **Memory Limits**: cgroup memory controller
- **CPU Limits**: cgroup CPU controller
- **PID Limits**: Prevent fork bombs
- **Cleanup**: Automated orphan cleanup

### 4. Error Handling ✅
- **Recovery Strategies**: Comprehensive error recovery
- **Retry Logic**: Exponential backoff
- **Cleanup**: Automatic resource cleanup on failure

## Implementation Timeline

```
Week 1: Foundation
  - GvisorBackend structure
  - OCI image handling
  - Basic execution

Week 2: Network Isolation
  - Network namespaces
  - IP allocation
  - Port mapping

Week 3: Filesystem Isolation
  - Mount management
  - Volume handling
  - Permissions

Week 4: Resource Management
  - Cleanup manager
  - Resource limits
  - Error recovery

Week 5: Testing & Validation
  - Unit tests
  - Integration tests
  - Performance benchmarks
```

## Performance Targets

| Metric | Docker | gVisor Target | Improvement |
|--------|--------|---------------|-------------|
| Cold start | 2.5s | 1.8s | 28% faster |
| Warm start | 0.8s | 0.4s | 50% faster |
| Memory/container | 50-100MB | 30-60MB | 40% less |
| Total overhead (10 containers) | 650-1150MB | 300-600MB | 50-55% reduction |

## Security Improvements

```
Docker Security:
App → Container → Kernel
(2 barriers)

gVisor Security:
App → Container → Sentry → Kernel
(3 barriers)

+ No daemon = No daemon attack surface
+ Syscall filtering = Reduced kernel exposure
+ User-space kernel = Extra isolation layer
```

## Dependencies

### System Tools
```bash
# Required
runsc     # gVisor runtime
skopeo    # OCI image operations
umoci     # OCI image unpacking
ip        # Network configuration
iptables  # Port mapping

# Verification
runsc --version
skopeo --version
umoci --version
```

### Rust Crates
```toml
serde = "1.0"
serde_json = "1.0"
uuid = "1.0"
thiserror = "1.0"
walkdir = "2.3"
sha2 = "0.10"
```

## File Structure

```
crates/clnrm-core/src/backend/gvisor/
├── mod.rs              # GvisorBackend main interface
├── network.rs          # Network isolation
├── filesystem.rs       # Filesystem isolation
├── oci.rs              # OCI image handling
├── mount.rs            # Mount management
├── dns.rs              # DNS configuration
├── permissions.rs      # Permission management
├── cleanup.rs          # Resource cleanup
├── errors.rs           # Error types
└── metrics.rs          # Performance monitoring
```

## Usage Example

```rust
use clnrm_core::backend::gvisor::GvisorBackend;

// Create gVisor backend
let backend = GvisorBackend::new("alpine:latest")?
    .with_platform(GvisorPlatform::Ptrace)
    .with_memory_limit(512)
    .with_network(NetworkConfig {
        subnet: "10.88.0.0/16".to_string(),
        enable_network: true,
        dns_servers: vec!["8.8.8.8".to_string()],
    });

// Run command
let cmd = Cmd::new("echo").arg("Hello from gVisor!");
let result = backend.run_cmd(cmd)?;

assert_eq!(result.exit_code, 0);
assert_eq!(result.stdout.trim(), "Hello from gVisor!");
```

## Key Design Decisions

### 1. No Docker Daemon
**Decision**: Direct runsc invocation
**Rationale**: Eliminates daemon attack surface, reduces complexity
**Trade-off**: More manual network/filesystem setup required

### 2. Manual Network Setup
**Decision**: Direct ip/iptables commands
**Rationale**: Full control, no daemon dependency
**Trade-off**: More code to maintain

### 3. OCI Standard
**Decision**: Use skopeo/umoci for OCI images
**Rationale**: Compatible with Docker registries, no Docker dependency
**Trade-off**: Additional dependencies (but lightweight)

### 4. Linux-Only
**Decision**: Linux-only support initially
**Rationale**: gVisor is Linux-specific, matches production environment
**Trade-off**: No Windows/Mac support (acceptable for CI/CD)

## Testing Strategy

### Unit Tests
- OCI config generation
- IP allocation/release
- Mount configuration
- Permission calculations

### Integration Tests
- Container execution end-to-end
- Network isolation verification
- Filesystem isolation verification
- Resource limits enforcement
- Cleanup procedures

### Performance Tests
- Container startup time
- Memory usage
- Network throughput
- Disk I/O performance

## Common Tasks

### View Architecture
```bash
# View full architecture
cat docs/GVISOR_ARCHITECTURE_DIAGRAMS.md

# View network isolation
grep -A 50 "Network Isolation" docs/GVISOR_ARCHITECTURE_DIAGRAMS.md
```

### Start Implementation
```bash
# Install dependencies
./scripts/install-gvisor-deps.sh

# Create module structure
mkdir -p crates/clnrm-core/src/backend/gvisor

# Begin Phase 1
# Follow docs/GVISOR_IMPLEMENTATION_CHECKLIST.md
```

### Run Tests
```bash
# Unit tests
cargo test --package clnrm-core gvisor

# Integration tests
cargo test --test gvisor_integration

# Performance tests
cargo bench --bench gvisor_performance
```

## Troubleshooting

### Issue: runsc not found
**Solution**: Install gVisor
```bash
curl -fsSL https://gvisor.dev/archive.key | sudo apt-key add -
sudo add-apt-repository "deb https://storage.googleapis.com/gvisor/releases release main"
sudo apt-get update && sudo apt-get install -y runsc
```

### Issue: Permission denied (network setup)
**Solution**: Run with sudo or configure user namespaces
```bash
# Option 1: Run with sudo
sudo cargo test

# Option 2: Configure user namespaces
sysctl -w kernel.unprivileged_userns_clone=1
```

### Issue: Image pull timeout
**Solution**: Check network, increase timeout
```bash
# Test connectivity
skopeo inspect docker://alpine:latest

# Increase timeout in code
.with_pull_timeout(Duration::from_secs(300))
```

## Contributing

When adding new features:
1. Update relevant design documents
2. Add tests (unit + integration)
3. Update this README
4. Add performance benchmarks
5. Document error scenarios

## Next Steps

1. ✅ Read [Executive Summary](GVISOR_EXECUTIVE_SUMMARY.md)
2. ✅ Review [Architecture Diagrams](GVISOR_ARCHITECTURE_DIAGRAMS.md)
3. ✅ Follow [Implementation Checklist](GVISOR_IMPLEMENTATION_CHECKLIST.md)
4. ✅ Start with [Quick Start](GVISOR_QUICK_START.md)

## Questions?

- **Technical**: Review [Isolation Design](GVISOR_ISOLATION_DESIGN.md)
- **Comparison**: Read [Docker vs gVisor](GVISOR_VS_DOCKER.md)
- **Implementation**: See [Implementation Checklist](GVISOR_IMPLEMENTATION_CHECKLIST.md)

---

**Document Version**: 1.0
**Last Updated**: 2026-01-05
**Status**: Design Complete, Ready for Implementation
