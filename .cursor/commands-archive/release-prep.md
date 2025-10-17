# Release Preparation

Prepare clnrm for release with comprehensive validation, versioning, and publish dry-run.

## Release Process Overview

1. **Pre-Release Validation**
2. **Version Bump**
3. **Changelog Update**
4. **Crate Validation**
5. **Publish Dry-Run**
6. **Tag and Release**

## What This Does

### 1. Pre-Release Validation
```bash
cargo make release-validation
```

Runs:
- Complete CI pipeline (fmt, clippy, test, build)
- Production readiness validation
- Performance benchmarking
- Security audit
- Documentation build
- Publish dry-run check

### 2. Version Bump

Update version in all `Cargo.toml` files:
- `crates/clnrm-shared/Cargo.toml`
- `crates/clnrm-core/Cargo.toml`
- `crates/clnrm/Cargo.toml`

Update dependency versions in workspace members.

### 3. Changelog Update

Update `CHANGELOG.md` following [Keep a Changelog](https://keepachangelog.com/) format:

```markdown
## [X.Y.Z] - YYYY-MM-DD

### Added
- New features

### Changed
- Changes in existing functionality

### Deprecated
- Soon-to-be removed features

### Removed
- Removed features

### Fixed
- Bug fixes

### Security
- Security fixes
```

### 4. Crate Validation

```bash
# Validate each crate
cargo make validate-crate
./scripts/validate-crate.sh crates/clnrm-core clnrm-core
./scripts/validate-crate.sh crates/clnrm clnrm
```

### 5. Publish Dry-Run

```bash
cargo make publish-check
```

Validates:
- clnrm-shared (published first)
- clnrm-core (depends on clnrm-shared)
- clnrm (depends on clnrm-core)

### 6. Create Git Tag

```bash
git tag -a vX.Y.Z -m "Release version X.Y.Z"
git push origin vX.Y.Z
```

## Release Checklist

- [ ] All tests passing (`cargo make test-all`)
- [ ] Production validation passed (`cargo make production-readiness-full`)
- [ ] No `.unwrap()` or `.expect()` in production code
- [ ] Clippy clean with zero warnings
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version bumped in all Cargo.toml files
- [ ] Dependency versions updated
- [ ] Publish dry-run successful
- [ ] Git tag created
- [ ] Release notes prepared

## Commands to Execute

```bash
# Step 1: Validate everything
cargo make release-validation

# Step 2: Update versions (manual)
# Edit crates/*/Cargo.toml files

# Step 3: Dry-run publish
cargo make publish-check

# Step 4: Update changelog
# Edit CHANGELOG.md

# Step 5: Commit changes
git add .
git commit -m "chore: prepare release vX.Y.Z"

# Step 6: Tag release
git tag -a vX.Y.Z -m "Release version X.Y.Z"

# Step 7: Push
git push origin master
git push origin vX.Y.Z
```

## Publishing Order (CRITICAL)

Crates MUST be published in dependency order:

1. **clnrm-shared** (no dependencies)
   ```bash
   cd crates/clnrm-shared
   cargo publish
   # Wait 30 seconds for crates.io to index
   ```

2. **clnrm-core** (depends on clnrm-shared)
   ```bash
   cd crates/clnrm-core
   cargo publish
   # Wait 30 seconds for crates.io to index
   ```

3. **clnrm** (depends on clnrm-core)
   ```bash
   cd crates/clnrm
   cargo publish
   ```

## Rollback Procedure

If release fails:

```bash
# Delete tag
git tag -d vX.Y.Z
git push origin :refs/tags/vX.Y.Z

# Revert commit
git revert HEAD
```

## Post-Release

1. Announce release on GitHub
2. Update documentation site
3. Update Homebrew formula (if applicable)
4. Notify users

## Time Estimate

- Validation: ~10-15 minutes
- Version updates: ~5 minutes
- Publish: ~5 minutes per crate
- Total: ~30-45 minutes

## Required Tools

- `cargo-audit` - Security auditing
- `cargo-outdated` - Dependency checking
- `cargo-make` - Task automation

Install with:
```bash
cargo make setup-dev-tools
```
