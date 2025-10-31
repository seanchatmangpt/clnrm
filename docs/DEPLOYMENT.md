# clnrm Deployment Guide

**Version:** v1.2.1+
**Last Updated:** 2025-10-31
**Maintainer:** DevOps Team

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Release Process](#release-process)
3. [Deployment Methods](#deployment-methods)
4. [CI/CD Pipeline](#cicd-pipeline)
5. [Homebrew Tap](#homebrew-tap)
6. [Rollback Procedures](#rollback-procedures)
7. [Monitoring](#monitoring)

---

## Quick Start

### Prerequisites

- Rust 1.70+ (`rustc --version`)
- Docker (`docker --version`)
- Weaver CLI (`weaver --version`)
- Git (`git --version`)

### Local Build

```bash
# Clone repository
git clone https://github.com/user/clnrm.git
cd clnrm

# Build with OTEL features
cargo build --release --features otel

# Install locally
cargo install --path crates/clnrm --features otel

# Verify installation
clnrm --version
```

---

## Release Process

### 1. Prepare Release

```bash
# Update version in Cargo.toml files
vim Cargo.toml
vim crates/clnrm/Cargo.toml
vim crates/clnrm-core/Cargo.toml

# Update CHANGELOG.md
vim CHANGELOG.md

# Commit changes
git add .
git commit -m "chore: bump version to v1.2.1"
git push origin master
```

### 2. Create Git Tag

```bash
# Create annotated tag
git tag -a v1.2.1 -m "Release v1.2.1: Critical bug fixes"

# Push tag (triggers release workflow)
git push origin v1.2.1
```

### 3. Automated Release

GitHub Actions will automatically:
1. Create GitHub release
2. Build binaries for all platforms
3. Publish to crates.io
4. Update Homebrew tap

### 4. Manual Verification

```bash
# Wait ~10 minutes for release to propagate

# Test Homebrew installation
brew update
brew install clnrm
clnrm --version  # Should show v1.2.1

# Test cargo installation
cargo install clnrm --features otel
clnrm --version
```

---

## Deployment Methods

### Method 1: Homebrew (Recommended for macOS/Linux)

**Installation:**
```bash
brew tap user/clnrm
brew install clnrm
```

**Update:**
```bash
brew update
brew upgrade clnrm
```

**Uninstall:**
```bash
brew uninstall clnrm
```

**Installation Layout:**
```
$(brew --prefix)/
├── bin/clnrm                         # Binary
└── share/clnrm/registry/             # Weaver schemas
    ├── registry_manifest.yaml
    ├── cli/
    ├── core/
    ├── metrics/
    └── events/
```

### Method 2: Cargo (Cross-platform)

**Installation:**
```bash
cargo install clnrm --features otel
```

**Update:**
```bash
cargo install clnrm --features otel --force
```

**Uninstall:**
```bash
cargo uninstall clnrm
```

**Registry Path Setup:**
```bash
# After cargo install, set registry path
export CLNRM_REGISTRY_PATH=/path/to/clnrm/registry

# Or clone registry separately
git clone https://github.com/user/clnrm.git /opt/clnrm
export CLNRM_REGISTRY_PATH=/opt/clnrm/registry

# Add to ~/.bashrc or ~/.zshrc
echo 'export CLNRM_REGISTRY_PATH=/opt/clnrm/registry' >> ~/.bashrc
```

### Method 3: Binary Download (No dependencies)

**Download:**
```bash
# Linux x86_64
curl -LO https://github.com/user/clnrm/releases/download/v1.2.1/clnrm-linux-x86_64.tar.gz
tar xzf clnrm-linux-x86_64.tar.gz
sudo mv clnrm /usr/local/bin/

# macOS ARM64 (M1/M2)
curl -LO https://github.com/user/clnrm/releases/download/v1.2.1/clnrm-macos-aarch64.tar.gz
tar xzf clnrm-macos-aarch64.tar.gz
sudo mv clnrm /usr/local/bin/

# Download registry
curl -LO https://github.com/user/clnrm/archive/v1.2.1.tar.gz
tar xzf v1.2.1.tar.gz
sudo mkdir -p /usr/local/share/clnrm
sudo mv clnrm-1.2.1/registry /usr/local/share/clnrm/

# Verify
clnrm --version
```

### Method 4: Docker (Containerized)

**Dockerfile:**
```dockerfile
FROM rust:1.70 as builder
WORKDIR /build
COPY . .
RUN cargo build --release --features otel

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/clnrm /usr/local/bin/
COPY --from=builder /build/registry /usr/local/share/clnrm/registry
ENV CLNRM_REGISTRY_PATH=/usr/local/share/clnrm/registry
ENTRYPOINT ["clnrm"]
CMD ["--help"]
```

**Build & Run:**
```bash
docker build -t clnrm:v1.2.1 .
docker run --rm clnrm:v1.2.1 --version
docker run --rm -v $(pwd):/workspace clnrm:v1.2.1 run /workspace/tests/
```

---

## CI/CD Pipeline

### GitHub Actions Workflows

**1. CI Workflow (`.github/workflows/ci.yml`)**
- **Trigger:** Push to main/master, PRs
- **Jobs:**
  - Test suite (Ubuntu, macOS)
  - Security audit
  - Code coverage
- **Duration:** ~8 minutes

**2. Release Workflow (`.github/workflows/release.yml`)**
- **Trigger:** Git tags (`v*.*.*`)
- **Jobs:**
  - Create GitHub release
  - Build binaries (Linux/macOS x86_64/ARM64)
  - Publish to crates.io
  - Update Homebrew tap
- **Duration:** ~15 minutes

**3. Weaver Validation (`.github/workflows/weaver-validation.yml`)**
- **Trigger:** Changes to `registry/` or `src/`
- **Jobs:**
  - Schema validation
  - Live-check integration
- **Duration:** ~5 minutes

### Required Secrets

Configure in GitHub Settings → Secrets:

```yaml
CARGO_REGISTRY_TOKEN: # crates.io API token
TAP_GITHUB_TOKEN:     # GitHub PAT with repo access for homebrew-clnrm
```

**Create crates.io token:**
1. Visit https://crates.io/settings/tokens
2. Create new token: "GitHub Actions - clnrm"
3. Add to GitHub secrets as `CARGO_REGISTRY_TOKEN`

**Create GitHub PAT for Homebrew:**
1. Visit https://github.com/settings/tokens
2. Generate new token (classic)
3. Scopes: `repo` (full control)
4. Add to GitHub secrets as `TAP_GITHUB_TOKEN`

---

## Homebrew Tap

### Setup Homebrew Tap Repository

```bash
# Create homebrew-clnrm repository
mkdir homebrew-clnrm
cd homebrew-clnrm

# Initialize
git init
mkdir Formula

# Copy formula
cp /path/to/clnrm/homebrew/Formula/clnrm.rb Formula/

# Commit and push
git add .
git commit -m "Initial commit: clnrm formula"
git remote add origin https://github.com/user/homebrew-clnrm.git
git push -u origin main
```

### Update Formula Manually

```bash
cd homebrew-clnrm

# Get version and SHA256
VERSION=1.2.1
curl -LO "https://github.com/user/clnrm/archive/v${VERSION}.tar.gz"
SHA256=$(shasum -a 256 v${VERSION}.tar.gz | awk '{print $1}')

# Update formula
sed -i '' "s|url \".*\"|url \"https://github.com/user/clnrm/archive/v${VERSION}.tar.gz\"|" Formula/clnrm.rb
sed -i '' "s|sha256 \".*\"|sha256 \"${SHA256}\"|" Formula/clnrm.rb
sed -i '' "s|version \".*\"|version \"${VERSION}\"|" Formula/clnrm.rb

# Test formula
brew install --build-from-source ./Formula/clnrm.rb
brew test clnrm

# Commit and push
git add Formula/clnrm.rb
git commit -m "Update clnrm to v${VERSION}"
git push
```

---

## Rollback Procedures

### Rollback Homebrew Installation

```bash
# Uninstall current version
brew uninstall clnrm

# Install specific version
brew install clnrm@1.2.0

# Or download old binary
curl -LO https://github.com/user/clnrm/releases/download/v1.2.0/clnrm-macos-aarch64.tar.gz
tar xzf clnrm-macos-aarch64.tar.gz
sudo mv clnrm /usr/local/bin/
```

### Rollback Cargo Installation

```bash
# Uninstall current
cargo uninstall clnrm

# Install specific version
cargo install clnrm --version 1.2.0 --features otel
```

### Rollback Git Tag

```bash
# Delete remote tag
git push --delete origin v1.2.1

# Delete local tag
git tag -d v1.2.1

# Delete GitHub release (manual via web UI)
# https://github.com/user/clnrm/releases
```

### Emergency Rollback Checklist

- [ ] Identify issue and affected version
- [ ] Notify users via GitHub release notes
- [ ] Rollback Homebrew formula to previous version
- [ ] Yank broken version from crates.io if critical
- [ ] Delete problematic GitHub release
- [ ] Hotfix and release patched version

---

## Monitoring

### Health Checks

```bash
# Verify clnrm installation
clnrm --version

# Run self-test
clnrm self-test

# Check registry installation
ls -la /usr/local/share/clnrm/registry/

# Validate Weaver integration
clnrm run tests/ --validate
```

### CI/CD Monitoring

**GitHub Actions Status:**
- https://github.com/user/clnrm/actions

**Key Metrics:**
- CI success rate: Target >95%
- Average CI duration: Target <10 minutes
- Release build success: Target 100%
- Weaver validation pass rate: Target 100%

**Alerts:**
- CI failure → Slack/Email notification
- Release failure → Page on-call engineer
- Weaver validation failure → Block merge

### User Metrics

**Track via GitHub:**
- Release download count
- Stars/forks/watchers
- Issue open/close rate
- PR merge frequency

**Track via crates.io:**
- Total downloads
- Recent downloads (last 90 days)
- Version distribution

---

## Best Practices

### Security

- ✅ Never commit secrets or API tokens
- ✅ Use GitHub secrets for sensitive data
- ✅ Run `cargo audit` before release
- ✅ Enable Dependabot for security updates
- ✅ Review all dependencies regularly

### Immutable Deployments

- ✅ Git tags are immutable (never delete/recreate)
- ✅ Crates.io versions are immutable (cannot yank and re-upload)
- ✅ Homebrew formula versions are version-controlled

### Testing Before Release

```bash
# Complete pre-release checklist
cargo test --all-features                    # All tests pass
cargo clippy -- -D warnings                  # Zero clippy warnings
weaver registry check -r registry/           # Schema validation
./tests/e2e/v1_2_1_validation.sh            # E2E tests pass
cargo build --release --features otel        # Binary builds
./target/release/clnrm self-test            # Self-test passes
```

---

## Troubleshooting

### Issue: Homebrew installation fails

**Symptoms:**
```
Error: clnrm: undefined method `share' for nil:NilClass
```

**Solution:**
```bash
# Update Homebrew
brew update

# Clear cache
rm -rf ~/Library/Caches/Homebrew/clnrm--*

# Reinstall
brew uninstall clnrm
brew install clnrm
```

### Issue: Registry not found

**Symptoms:**
```
Error: Registry path not found: registry
```

**Solution:**
```bash
# For Homebrew installation
export CLNRM_REGISTRY_PATH="$(brew --prefix)/share/clnrm/registry"

# For cargo installation
export CLNRM_REGISTRY_PATH="/path/to/clnrm/source/registry"

# Verify
ls -la $CLNRM_REGISTRY_PATH
```

### Issue: Weaver validation fails

**Symptoms:**
```
Error: Weaver exited prematurely with status: exit status: 1
```

**Solution:**
```bash
# Check Weaver installation
weaver --version

# Verify registry path
echo $CLNRM_REGISTRY_PATH
ls -la $CLNRM_REGISTRY_PATH

# Check registry validity
weaver registry check -r $CLNRM_REGISTRY_PATH

# Run with debug logging
RUST_LOG=debug clnrm run tests/ --validate
```

---

## Support

**Documentation:** https://github.com/user/clnrm/tree/master/docs
**Issues:** https://github.com/user/clnrm/issues
**Discussions:** https://github.com/user/clnrm/discussions

**Maintainers:**
- DevOps: @devops-team
- Security: @security-team
- Release Manager: @release-manager

---

**Last Updated:** 2025-10-31
**Next Review:** 2025-11-30
