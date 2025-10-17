# Production Validation

Comprehensive production readiness validation.

## Command
```bash
cargo make validate
```

## Alias for
```bash
cargo make validate-production-readiness
```

## What It Does
- ✅ **Prerequisites** (Docker, Cargo)
- ✅ **Core team standards** (NO .unwrap()/.expect())
- ✅ **Test suite** (unit + integration)
- ✅ **Linting** (zero warnings)
- ✅ **Release build**
- ✅ **Performance SLOs** (build < 120s, CLI < 2s)

## Complete Suite
```bash
cargo make production-ready
```
Includes: fmt-check, clippy, test-all, cleanroom-validate, build-release, validate-crate, validate-production-readiness

## Use When
- Before production deployment
- Before creating release
- Weekly validation runs

## Time: ~5-10 minutes
