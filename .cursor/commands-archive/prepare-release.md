# Prepare Release

Complete checklist for preparing a new clnrm release following best practices.

## Pre-Release Checklist

### 1. Version Bump

Update version in all relevant files:

```bash
# Update Cargo.toml files
vim Cargo.toml  # Workspace version
vim crates/clnrm/Cargo.toml
vim crates/clnrm-core/Cargo.toml
vim crates/clnrm-shared/Cargo.toml
# Note: clnrm-ai is excluded from workspace

# Verify versions are consistent
grep -r "version =" Cargo.toml crates/*/Cargo.toml | grep -v dependencies
```

**Version Strategy**:
- Patch (0.4.x): Bug fixes, minor improvements
- Minor (0.x.0): New features, non-breaking changes
- Major (x.0.0): Breaking changes, major rewrites

### 2. Run Complete Test Suite

```bash
# Build with all features
cargo build --release --features otel
cargo build --release --all-features

# Unit tests
cargo test --lib --all-features

# Integration tests
cargo test --test '*' --all-features

# Determinism tests
cargo test --test determinism_test

# OTEL validation
docker-compose -f tests/integration/docker-compose.otel-test.yml up -d
cargo test --features otel --test otel_validation_integration -- --ignored
docker-compose -f tests/integration/docker-compose.otel-test.yml down -v

# Quality gates
bash scripts/ci-gate.sh

# Fake scanner
bash scripts/scan-fakes.sh

# Clippy strict
cargo clippy --all-features -- -D warnings
```

### 3. Homebrew Installation Test

Test the actual installation process:

```bash
# Uninstall current version
brew uninstall clnrm

# Install from source
brew install --build-from-source .

# Verify installation
which clnrm
clnrm --version

# Run self-tests with production binary
clnrm self-test
clnrm self-test --suite otel --otel-exporter stdout

# Test key commands
clnrm init
clnrm plugins
clnrm --help
```

### 4. Update Documentation

**CHANGELOG.md**:
```markdown
## [X.Y.Z] - YYYY-MM-DD

### Added
- New features and capabilities

### Changed
- Modifications to existing features

### Fixed
- Bug fixes and corrections

### Security
- Security improvements

### Breaking Changes
- Any breaking API changes
```

**README.md**:
- [ ] Update version badge
- [ ] Update installation instructions
- [ ] Update feature list (if changed)
- [ ] Update examples (if API changed)
- [ ] Verify all links work

**docs/**:
- [ ] Update CLI_GUIDE.md with new commands
- [ ] Update TOML_REFERENCE.md with new config options
- [ ] Update TESTING.md if testing approach changed
- [ ] Update ARCHITECTURE.md if structure changed

### 5. Git Hygiene

```bash
# Ensure working tree is clean
git status

# Create release branch
git checkout -b release/vX.Y.Z

# Commit version bump
git add Cargo.toml crates/*/Cargo.toml Cargo.lock
git commit -m "chore: bump version to X.Y.Z"

# Commit documentation updates
git add CHANGELOG.md README.md docs/
git commit -m "docs: update documentation for vX.Y.Z release"
```

### 6. Build and Tag

```bash
# Build release binaries
cargo build --release --features otel

# Run final validation
cargo test --all-features
bash scripts/ci-gate.sh

# Create annotated tag
git tag -a vX.Y.Z -m "Release vX.Y.Z - Brief description"

# Verify tag
git tag -l -n9 vX.Y.Z
```

### 7. CI Validation

Push to branch and verify CI passes:

```bash
# Push release branch
git push origin release/vX.Y.Z

# Wait for CI to complete
# Check GitHub Actions: https://github.com/seanchatmangpt/clnrm/actions

# Ensure all jobs pass:
# - fast-tests (fake-scanner, ci-gate)
# - integration-tests (determinism, OTEL validation)
# - build (all feature combinations)
```

### 8. Create Release PR

```bash
# Create PR from release branch to main
gh pr create --title "Release vX.Y.Z" --body "$(cat <<'EOF'
## Release vX.Y.Z

### Changes
[Paste from CHANGELOG.md]

### Testing
- [x] All unit tests pass
- [x] All integration tests pass
- [x] Determinism tests pass
- [x] OTEL validation passes
- [x] Quality gates pass
- [x] Homebrew installation tested
- [x] Self-tests pass with production binary

### Checklist
- [x] Version bumped in all Cargo.toml files
- [x] CHANGELOG.md updated
- [x] Documentation updated
- [x] CI passes
- [x] Breaking changes documented (if any)

### Review Focus
- API changes (if any)
- Breaking changes (if any)
- Documentation completeness
EOF
)"
```

### 9. Final Validation After Merge

```bash
# After PR merged to main
git checkout main
git pull origin main

# Tag the release
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin vX.Y.Z

# Build final release artifacts
cargo build --release --features otel

# Test Homebrew installation from main
brew uninstall clnrm
brew install --build-from-source .
clnrm self-test
```

### 10. GitHub Release

Create GitHub release via web UI or CLI:

```bash
gh release create vX.Y.Z \
  --title "clnrm vX.Y.Z" \
  --notes "$(cat CHANGELOG.md | sed -n '/## \[X.Y.Z\]/,/## \[/p' | head -n -1)" \
  target/release/clnrm
```

**Release notes should include**:
- Summary of changes
- Installation instructions
- Breaking changes (if any)
- Migration guide (if needed)
- Notable bug fixes

## Post-Release

### 11. Verify Release

```bash
# Test installation from GitHub release
brew install seanchatmangpt/tap/clnrm

# Verify version
clnrm --version

# Run self-tests
clnrm self-test
```

### 12. Announce Release

Update:
- [ ] Project README badges
- [ ] GitHub Discussions (if applicable)
- [ ] Team Slack/Discord
- [ ] Social media (if applicable)

### 13. Monitor

After release:
- [ ] Watch for GitHub issues
- [ ] Monitor CI on main branch
- [ ] Check for installation issues
- [ ] Address urgent bugs with patch release

## Emergency Patch Release

If critical bug found post-release:

```bash
# Create hotfix branch from tag
git checkout -b hotfix/vX.Y.Z+1 vX.Y.Z

# Fix the bug
# ... make changes ...

# Test thoroughly
cargo test --all-features
bash scripts/ci-gate.sh

# Bump patch version
# Update CHANGELOG.md with hotfix

# Commit and tag
git commit -m "fix: critical bug description"
git tag -a vX.Y.Z+1 -m "Hotfix vX.Y.Z+1"

# Push and release
git push origin hotfix/vX.Y.Z+1
git push origin vX.Y.Z+1
```

## Release Cadence

**Recommended schedule**:
- **Patch releases**: As needed for critical bugs
- **Minor releases**: Every 2-4 weeks for features
- **Major releases**: When breaking changes necessary

## Quality Standards

Release is only created when:
- ✅ All CI checks pass
- ✅ Zero clippy warnings
- ✅ Fake scanner clean
- ✅ Determinism tests pass
- ✅ OTEL validation passes
- ✅ Homebrew installation tested
- ✅ Self-tests pass
- ✅ Documentation complete
- ✅ CHANGELOG updated

**Never release with**:
- ❌ Failing tests
- ❌ Compiler warnings
- ❌ Known critical bugs
- ❌ Incomplete documentation
- ❌ Fake implementations
