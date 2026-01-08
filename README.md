# Cleanroom Testing Framework (clnrm) - gVisor Edition

High-security, hermetic integration testing without Docker dependencies. **clnrm v2.0** uses gVisor for safer, faster, more deterministic tests.

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-passing-green.svg)](#)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)

## Why gVisor?

**No Docker Daemon Required**
- Run tests anywhere - no Docker socket dependency
- Reduces attack surface and security risks
- Works in restricted environments (CI/CD, serverless)

**Better Isolation**
- gVisor Sentry intercepts all syscalls
- Application kernel runs in userspace
- Superior security compared to Docker's kernel-level containers

**Faster, More Deterministic**
- Container startup: 300-500ms (vs Docker's 1-2s)
- Deterministic execution for reliable test results
- Reduced resource overhead

**Hermetic Testing**
- Complete filesystem isolation
- Network isolation by default
- Resource limit enforcement

## Quick Start

### 1. Install gVisor (5 minutes)

```bash
# Ubuntu/Debian
curl -fsSL https://gvisor.dev/archive.key | sudo gpg --dearmor -o /usr/share/keyrings/gvisor-archive-keyring.gpg
echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/gvisor-archive-keyring.gpg] https://storage.googleapis.com/gvisor/releases release main" | sudo tee /etc/apt/sources.list.d/gvisor.list
sudo apt-get update
sudo apt-get install -y runsc skopeo
```

Verify installation:
```bash
runsc --version
skopeo --version
```

### 2. Run Your First Test

```bash
# Run all tests with gVisor
cargo test --all

# Run specific test
cargo test my_test_name -- --nocapture

# With parallelism
cargo test --all -- --test-threads=8
```

### 3. Configure (Optional)

Create `.clnrm.toml`:
```toml
[backend]
type = "gvisor"

[backend.gvisor]
cache_dir = "/var/cache/clnrm"
startup_timeout = 30

[backend.gvisor.limits]
memory_mb = 512
cpus = 2.0
```

## Documentation

| Document | Purpose | Audience |
|----------|---------|----------|
| [SETUP.md](docs/SETUP.md) | Installation & configuration | Operators |
| [DEVELOPMENT.md](docs/DEVELOPMENT.md) | Developer environment setup | Developers |
| [TESTING.md](docs/TESTING.md) | Running tests with gVisor | QA/Developers |
| [GVISOR_QUICK_START.md](docs/GVISOR_QUICK_START.md) | Quick reference guide | All users |
| [MIGRATION_FROM_DOCKER.md](docs/MIGRATION_FROM_DOCKER.md) | Transitioning from Docker | Existing users |

## Project Structure

- `crates/` - Rust crates
  - `clnrm-core/` - Core framework
  - `clnrm-cli/` - Command-line interface
- `docs/` - Documentation
  - `SETUP.md` - Installation guide
  - `DEVELOPMENT.md` - Developer guide
  - `TESTING.md` - Testing guide
- `tests/` - Test suites
- `examples/` - Example usage
- `cleanroom.toml` - Framework configuration

## Framework Architecture

```
Test Suite
    ↓
CleanroomEnvironment (Backend abstraction)
    ↓
GVisorBackend (runsc runtime)
    ├── OCI Image Manager (pull, cache, extract)
    ├── Container Runtime (create, start, exec)
    └── Service Manager (lifecycle, health checks)
    ↓
Linux Kernel
```

## Code Standards

This project follows strict standards to eliminate Mura (inconsistency):

- **No unwrap/expect in production code** - All errors must be Result types
- **Consistent error handling** - Use `Result<T, CleanroomError>`
- **Minimum 80% test coverage** - Comprehensive test suite required
- **Full documentation** - All public APIs documented
- **gVisor-only approach** - No Docker dependencies

See [CODE_STANDARDS.md](docs/CODE_STANDARDS.md) for comprehensive standards.

## Common Tasks

### Run Tests
```bash
# All tests
cargo test --all

# Specific test
cargo test my_test -- --nocapture

# Integration tests only
cargo test --test '*'

# With debug output
CLNRM_DEBUG=true cargo test
```

### Troubleshoot
```bash
# Check gVisor status
runsc --version

# View gVisor debug logs
journalctl -u runsc

# Run with increased timeout
CLNRM_STARTUP_TIMEOUT=60 cargo test my_test
```

### Development
See [DEVELOPMENT.md](docs/DEVELOPMENT.md) for:
- Setting up your development environment
- Building from source
- Running individual tests
- Debugging test failures

## Frequently Asked Questions

**Q: Can I still use Docker images?**
A: Yes! gVisor uses OCI images, which are compatible with Docker images.

**Q: Is gVisor production-ready?**
A: Yes, Google uses gVisor in production. See https://gvisor.dev for details.

**Q: What about performance?**
A: gVisor is 2-3x faster for container startup. Overall performance depends on workload.

**Q: Do I need root privileges?**
A: Yes, currently gVisor requires root. Rootless mode is coming.

**Q: How do I migrate from Docker?**
A: See [MIGRATION_FROM_DOCKER.md](docs/MIGRATION_FROM_DOCKER.md) for detailed instructions.

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Follow code standards (see [CODE_STANDARDS.md](docs/CODE_STANDARDS.md))
4. Add tests for new functionality
5. Submit a pull request

## Performance Characteristics

| Metric | Performance |
|--------|-------------|
| Container startup (cold) | 2-3s |
| Container startup (warm) | 300-500ms |
| Memory overhead | 50-80MB per container |
| Network latency | 1-2ms |

See [GVISOR_PERFORMANCE_BASELINE.md](docs/GVISOR_PERFORMANCE_BASELINE.md) for detailed benchmarks.

## Security

clnrm prioritizes security through:
- gVisor application kernel isolation
- Reduced attack surface (no Docker daemon)
- Filesystem isolation
- Network isolation
- Resource limits

See [SECURITY.md](SECURITY.md) for detailed security information.

## License

MIT License - See [LICENSE](LICENSE) for details.

## Support

- **Documentation**: See [docs/](docs/)
- **Issues**: https://github.com/seanchatmangpt/clnrm/issues
- **Discussions**: https://github.com/seanchatmangpt/clnrm/discussions

---

**Ready to get started?** Follow the [Quick Start](#quick-start) above or read [SETUP.md](docs/SETUP.md) for detailed installation instructions.
