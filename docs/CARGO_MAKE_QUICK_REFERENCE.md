# Cargo Make Quick Reference

## Installation

```bash
cargo install cargo-make
```

## Most Common Commands

```bash
cargo make dev              # Format + lint + test (daily workflow)
cargo make quick            # Fast check + test (rapid iteration)
cargo make watch            # Auto-rebuild on file changes
cargo make ci               # Full CI pipeline
cargo make install          # Install clnrm binary
cargo make self-test        # Run framework self-test
```

## Task Categories

### Development
| Command | Description | Time |
|---------|-------------|------|
| `cargo make dev` | fmt + clippy + test | 1min |
| `cargo make quick` | check + test-quick | 30s |
| `cargo make watch` | Auto-rebuild | ∞ |
| `cargo make fix` | Auto-fix fmt + clippy | 40s |

### Building
| Command | Description | Time |
|---------|-------------|------|
| `cargo make build` | Build workspace | 45s |
| `cargo make build-release` | Release build | 2min |
| `cargo make build-core` | Build core only | 30s |
| `cargo make build-otel` | Build with OTEL | 50s |

### Testing
| Command | Description | Time |
|---------|-------------|------|
| `cargo make test` | Library tests | 20s |
| `cargo make test-quick` | Fast tests | 10s |
| `cargo make test-all` | Unit + integration | 50s |
| `cargo make test-verbose` | Tests with output | 20s |
| `cargo make test-proptest` | Property tests | 5min |

### Quality
| Command | Description | Time |
|---------|-------------|------|
| `cargo make clippy` | Strict linting | 30s |
| `cargo make fmt` | Format code | 5s |
| `cargo make fmt-check` | Check formatting | 3s |
| `cargo make check` | Compilation check | 20s |

### CI/CD
| Command | Description | Time |
|---------|-------------|------|
| `cargo make ci` | Full CI pipeline | 2min |
| `cargo make pre-commit` | Pre-commit checks | 45s |
| `cargo make release-check` | Release validation | 3min |
| `cargo make full-check` | Complete validation | 10min |

### Documentation
| Command | Description | Time |
|---------|-------------|------|
| `cargo make doc` | Build docs | 45s |
| `cargo make doc-open` | Build + open docs | 46s |

### Utilities
| Command | Description | Time |
|---------|-------------|------|
| `cargo make audit` | Security audit | 15s |
| `cargo make outdated` | Check updates | 10s |
| `cargo make bloat` | Binary size | 2min |
| `cargo make clean` | Clean artifacts | 2s |

### Installation
| Command | Description | Time |
|---------|-------------|------|
| `cargo make install` | Install binary | 30s |
| `cargo make install-dev` | Install with debug | 45s |
| `cargo make reinstall` | Uninstall + install | 31s |

### Clnrm-Specific
| Command | Description | Time |
|---------|-------------|------|
| `cargo make self-test` | Framework self-test | 30s |
| `cargo make run-examples` | Run examples | 1min |
| `cargo make benchmarks` | Run benchmarks | 2min |

## Common Workflows

### Daily Development Pattern

```bash
# Option 1: Manual iteration
cargo make dev
# Make changes...
cargo make dev

# Option 2: Automated
cargo make watch
# Make changes (auto-rebuilds)

# Option 3: Ultra-fast
cargo make quick
# Make changes...
cargo make quick
```

### Before Git Commit

```bash
cargo make pre-commit   # Quick checks
# or
cargo make ci           # Full validation
```

### Before Pull Request

```bash
cargo make full-check   # Everything
cargo make self-test    # Verify framework
```

### Before Release

```bash
cargo make full-check       # Complete validation
cargo make publish-check    # Verify package
cargo make version-bump     # Check versions
# Update versions manually
cargo make publish          # Publish to crates.io
```

### Troubleshooting

```bash
cargo make clean            # Clean build
cargo make test-verbose     # Debug tests
cargo make clean-all        # Deep clean
```

## Environment Variables

```bash
# Debug logging
RUST_LOG=debug cargo make test-verbose

# OTEL configuration
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318 cargo make build-otel

# Backtrace
RUST_BACKTRACE=1 cargo make test
```

## Shell Aliases (Optional)

Add to `~/.bashrc` or `~/.zshrc`:

```bash
alias cm='cargo make'
alias cmd='cargo make dev'
alias cmq='cargo make quick'
alias cmt='cargo make test'
alias cmc='cargo make ci'
```

## Task Selection Guide

| Scenario | Use This | Why |
|----------|----------|-----|
| Active coding | `watch` | Automatic |
| Quick check | `quick` | Fastest |
| Before commit | `pre-commit` | Essential |
| Before PR | `ci` | Complete |
| Before release | `full-check` | Everything |
| Install binary | `install` | Production |
| Debug binary | `install-dev` | Development |
| Broken build | `clean` + `build` | Fresh start |
| View docs | `doc-open` | Browser |
| Security | `audit` | Vulnerabilities |

## Help Commands

```bash
cargo make --help                    # Show help
cargo make --list-all-steps          # List all tasks
cargo make --print-steps <task>      # Show task details
cargo make                           # Show quick reference
```

## Advanced Usage

### Run Multiple Tasks

```bash
cargo make clean build test install
```

### Custom Environment

```bash
MY_VAR=value cargo make test
```

### Skip Tasks

```bash
cargo make --skip-tasks-pattern test ci
```

## Git Hooks Integration

```bash
# .git/hooks/pre-commit
#!/bin/bash
cargo make pre-commit

# Make executable
chmod +x .git/hooks/pre-commit
```

## Time Estimates

| Duration | Tasks |
|----------|-------|
| < 10s | `fmt`, `fmt-check`, `clean`, `quick` (partial) |
| 10-30s | `test`, `check`, `clippy`, `install` |
| 30s-1min | `build`, `test-quick`, `pre-commit` |
| 1-2min | `dev`, `build-release`, `ci` |
| 2-5min | `benchmarks`, `bloat`, `release-check` |
| 5-10min | `test-proptest`, `full-check` |

## Support

- Full Guide: `/docs/CARGO_MAKE_GUIDE.md`
- Makefile: `/Makefile.toml`
- GitHub: https://github.com/seanchatmangpt/clnrm
- cargo-make docs: https://sagiegurari.github.io/cargo-make/

---

**Print this page for quick reference while developing!**
