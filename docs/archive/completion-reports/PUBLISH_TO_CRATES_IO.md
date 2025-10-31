# Publishing clnrm v1.0.0 to crates.io

**Status**: ✅ Ready to Publish
**Date**: 2025-10-17

---

## Pre-Flight Checklist ✅

- [x] All code committed to git
- [x] v1.0.0 tag created
- [x] Build passes: `cargo build --release`
- [x] 749 tests passing: `cargo test --lib`
- [x] Documentation complete
- [x] CHANGELOG.md updated
- [x] Version numbers consistent across workspace

---

## Publishing Steps

### 1. Dry Run (Verify Package Contents)

```bash
# Check clnrm-shared
cd /Users/sac/clnrm/crates/clnrm-shared
cargo package --list
cargo publish --dry-run

# Check clnrm-core
cd /Users/sac/clnrm/crates/clnrm-core
cargo package --list
cargo publish --dry-run

# Check clnrm (binary)
cd /Users/sac/clnrm/crates/clnrm
cargo package --list
cargo publish --dry-run
```

### 2. Publish to crates.io (IN ORDER)

**IMPORTANT**: Publish in dependency order!

```bash
# Step 1: Publish clnrm-shared (no dependencies)
cd /Users/sac/clnrm/crates/clnrm-shared
cargo publish

# Wait ~30 seconds for crates.io to index

# Step 2: Publish clnrm-core (depends on clnrm-shared)
cd /Users/sac/clnrm/crates/clnrm-core
cargo publish

# Wait ~30 seconds for crates.io to index

# Step 3: Publish clnrm binary (depends on clnrm-core)
cd /Users/sac/clnrm/crates/clnrm
cargo publish
```

### 3. Verify Publication

```bash
# Check each crate appeared on crates.io
open https://crates.io/crates/clnrm-shared
open https://crates.io/crates/clnrm-core
open https://crates.io/crates/clnrm

# Verify version shows as 1.0.0
# Verify documentation rendered correctly
# Verify download badges appear
```

### 4. Push Git Tag to GitHub

```bash
# Push commits
git push origin master

# Push tag
git push origin v1.0.0

# Verify tag appears on GitHub
open https://github.com/seanchatmangpt/clnrm/releases
```

### 5. Create GitHub Release

```bash
# Use GitHub CLI (if available)
gh release create v1.0.0 \
  --title "v1.0.0 - Production-Ready Framework" \
  --notes-file CHANGELOG.md

# Or manually at:
# https://github.com/seanchatmangpt/clnrm/releases/new
```

---

## Post-Publication Steps

### 1. Test Installation

```bash
# Test installing from crates.io
cargo install clnrm

# Verify version
clnrm --version
# Should output: clnrm 1.0.0

# Test basic functionality
clnrm init
clnrm --help
```

### 2. Update Homebrew Formula (If Applicable)

```bash
# Update homebrew/Formula/clnrm.rb with:
# - New version number
# - New source tarball SHA256
# - New bottle SHA256s (after bottles are built)

# Test Homebrew installation
brew install clnrm
clnrm --version
```

### 3. Announce Release

**GitHub Release Notes** (auto-generated from tag):
```markdown
# clnrm v1.0.0 - Production-Ready Hermetic Testing Framework

[Full CHANGELOG](https://github.com/seanchatmangpt/clnrm/blob/master/CHANGELOG.md)

## 🎉 Major Features

- **Red-Team OTLP Validation**: 7-layer fake-green detection system
- **Environment Variable Resolution**: Template → ENV → defaults precedence
- **SDK Resource Validation**: Authentic telemetry verification
- **Comprehensive Documentation**: 25+ guides, 500KB+ content
- **Production Quality**: 749 tests, zero unwrap/expect violations

## 📦 Installation

```bash
# Via Cargo
cargo install clnrm

# Via Homebrew (macOS/Linux)
brew install clnrm

# From source
git clone https://github.com/seanchatmangpt/clnrm.git
cd clnrm
cargo install --path crates/clnrm
```

## 🚀 Quick Start

```bash
# Initialize new project
clnrm init

# Run tests
clnrm run tests/

# Self-test framework
clnrm self-test
```

## 📊 Performance

- Hot reload: <2.5s average
- Change detection: 10x faster iteration
- Parallel execution: 4-8x speedup
- Template rendering: <50ms

See full release notes in [CHANGELOG.md](CHANGELOG.md)
```

**Community Announcements** (if applicable):
- [ ] Post to Rust Users forum
- [ ] Post to Reddit r/rust
- [ ] Tweet announcement
- [ ] Update project website/docs
- [ ] Notify existing users

### 4. Monitor Initial Adoption

```bash
# Check download statistics
open https://crates.io/crates/clnrm/stats

# Monitor GitHub issues for bug reports
# Monitor crates.io for feedback

# Watch for first issues/PRs from community
```

---

## Rollback Plan (If Needed)

If critical issues are discovered after publication:

```bash
# Yank broken version (keeps it in registry but warns users)
cargo yank --version 1.0.0 clnrm
cargo yank --version 1.0.0 clnrm-core
cargo yank --version 1.0.0 clnrm-shared

# Fix issues locally
# Bump to v1.0.1
# Publish patch release
```

Note: Yanking doesn't delete the version, it just marks it as broken.

---

## Expected Timeline

| Step | Duration | Notes |
|------|----------|-------|
| Dry run | 5 min | Verify package contents |
| Publish clnrm-shared | 2 min | First crate, no deps |
| Index wait | 30 sec | Wait for crates.io |
| Publish clnrm-core | 2 min | Depends on shared |
| Index wait | 30 sec | Wait for crates.io |
| Publish clnrm | 2 min | Main binary |
| Verify | 5 min | Check all 3 crates |
| Push tags | 2 min | Git push |
| GitHub release | 10 min | Release notes |
| **TOTAL** | **~30 min** | End-to-end |

---

## Success Criteria

- [ ] All 3 crates visible on crates.io
- [ ] Version shows as 1.0.0
- [ ] Documentation renders correctly
- [ ] Download badges appear
- [ ] `cargo install clnrm` works
- [ ] `clnrm --version` shows 1.0.0
- [ ] GitHub release created
- [ ] Git tag pushed
- [ ] No critical issues reported in first 24 hours

---

## Contact Information

**Maintainer**: Sean Chatman <seanchatmangpt@gmail.com>
**Repository**: https://github.com/seanchatmangpt/clnrm
**Crates.io**: https://crates.io/crates/clnrm
**License**: MIT

---

## Notes

### Why This Order?

1. **clnrm-shared** first - No dependencies, pure utilities
2. **clnrm-core** second - Depends on clnrm-shared
3. **clnrm** last - Depends on both clnrm-shared and clnrm-core

### About clnrm-ai

The `clnrm-ai` crate (v0.5.0) is intentionally excluded from this release:
- Marked as experimental
- Excluded from `default-members` in workspace
- Will be published separately when stable

### Version Consistency

All published crates use v1.0.0:
- `clnrm` = 1.0.0
- `clnrm-core` = 1.0.0
- `clnrm-shared` = 1.0.0

Path dependencies automatically resolve to crates.io versions after publication.

---

**Last Updated**: 2025-10-17
**Status**: ✅ Ready to Execute
