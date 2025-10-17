# clnrm Command Help

Quick reference for all available commands.

## Essential Commands (Type `/` to use)

| Command | Purpose | Time |
|---------|---------|------|
| `/dev` | Quick dev iteration (fmt + lint + test) | 30s |
| `/test` | Run all tests | 1-2m |
| `/validate` | Production validation | 5-10m |
| `/fix` | Auto-fix formatting & clippy | 10-30s |
| `/release` | Release preparation | 10-15m |
| `/help` | Show this help | - |

## Cargo-Make Tasks

View all available tasks:
```bash
cargo make --list-all-steps
```

### Quick Reference

**Build:**
```bash
cargo make build              # Debug build
cargo make build-release      # Release build
cargo make clean              # Clean artifacts
```

**Test:**
```bash
cargo make test               # Unit tests
cargo make test-all           # All tests
cargo make test-cleanroom     # Cleanroom tests
cargo make test-proptest      # Property tests
```

**Quality:**
```bash
cargo make fmt                # Format code
cargo make clippy             # Lint code
cargo make check              # Quick compilation check
cargo make audit              # Security audit
```

**Validation:**
```bash
cargo make validate           # Production validation
cargo make validate-crate     # Crate validation
cargo make verify-cleanroom   # Cleanroom verification
cargo make production-ready   # Complete suite
```

**Development:**
```bash
cargo make dev                # Quick iteration
cargo make quick              # check + test-quick
cargo make watch              # Watch for changes
cargo make pre-commit         # Pre-commit checks
```

**Documentation:**
```bash
cargo make doc                # Build docs
cargo make doc-open           # Build & open docs
```

**Benchmarks:**
```bash
cargo make benchmarks         # Run benchmarks
cargo make cleanroom-slo-check # SLO validation
```

## Core Team Standards

All commands enforce:
- ❌ NO `.unwrap()` in production code
- ❌ NO `.expect()` in production code
- ✅ `Result<T, CleanroomError>` error handling
- ✅ Sync trait methods (dyn compatible)
- ✅ Zero clippy warnings
- ✅ AAA test pattern

## Quick Workflows

**Daily Development:**
```bash
/dev                 # Quick iteration
```

**Before Commit:**
```bash
cargo make pre-commit  # fmt + clippy + test
```

**Before PR:**
```bash
/test                # All tests
/validate            # Production validation
```

**Before Release:**
```bash
/release             # Release validation
```

## Need More Help?

- **Makefile.toml** - View all task definitions
- **docs/GGEN_ADAPTATION.md** - Implementation details
- **.cursorrules** - Core team standards
- **cargo make --list-all-steps** - List all tasks
