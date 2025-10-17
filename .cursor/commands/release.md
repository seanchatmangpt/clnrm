# Release Preparation

Validate codebase is ready for release.

## Command
```bash
cargo make release-validation
```

## What It Does
- Complete CI pipeline
- Production readiness validation
- Performance benchmarking
- Publish dry-run check

## Release Checklist
Run these commands in order:

```bash
# 1. Full validation
cargo make production-ready

# 2. Update version in Cargo.toml files
# (manual step)

# 3. Dry-run publish
cargo make publish-check

# 4. Create git tag
git tag -a vX.Y.Z -m "Release vX.Y.Z"

# 5. Actual publish (if dry-run succeeded)
cargo make publish
```

## Use When
- Before creating release tag
- Before publishing to crates.io
- Weekly release preparation

## Time: ~10-15 minutes
