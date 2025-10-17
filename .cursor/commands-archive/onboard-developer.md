# Onboard Developer - Complete Onboarding Guide

Comprehensive onboarding process to get a new developer productive with clnrm quickly.

## Overview

This guide walks through complete clnrm setup from zero to running tests, following the "eat your own dogfood" principle.

## Prerequisites Check

Before starting, verify you have:

- [ ] macOS, Linux, or WSL2 (Windows)
- [ ] Homebrew installed (macOS/Linux)
- [ ] Docker or Podman running
- [ ] Git configured
- [ ] GitHub account with SSH keys

## Step 1: Install Rust Toolchain (10 minutes)

```bash
# Install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Configure current shell
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version

# Install additional components
rustup component add rustfmt clippy
```

**Expected output**: `cargo 1.70.0+`

## Step 2: Install Development Tools (5 minutes)

```bash
# macOS/Linux with Homebrew
brew install git
brew install docker  # or Docker Desktop

# Verify Docker is running
docker ps

# Install cargo tools
cargo install cargo-make
cargo install cargo-deny
cargo install cargo-audit
cargo install cargo-llvm-cov
```

## Step 3: Clone clnrm Repository (2 minutes)

```bash
# Clone repository
git clone git@github.com:seanchatmangpt/clnrm.git
cd clnrm

# Or via HTTPS
git clone https://github.com/seanchatmangpt/clnrm.git
cd clnrm

# Verify workspace structure
ls -la crates/
```

**Expected**: See `clnrm-core`, `clnrm-shared`, `clnrm` directories

## Step 4: Build clnrm (5-10 minutes)

```bash
# Build all crates with all features
cargo build --release --all-features

# Verify build succeeded
ls -lh target/release/clnrm
```

**Expected**: Binary ~10-20MB

## Step 5: Install clnrm via Homebrew (5 minutes)

**Critical**: This is the dogfooding step - we test using the production installation method.

```bash
# Build from source
cargo build --release --features otel

# Install via Homebrew (if formula exists)
brew install clnrm

# OR install manually
sudo cp target/release/clnrm /usr/local/bin/

# Verify installation
which clnrm
clnrm --version
```

**Expected**: `clnrm --version` shows current version

## Step 6: Run Self-Tests (3 minutes)

```bash
# Basic self-test
clnrm self-test

# With OTEL validation
clnrm self-test --suite otel --otel-exporter stdout

# Initialize test project
mkdir test-clnrm
cd test-clnrm
clnrm init
```

**Expected**: All self-tests pass ✅

## Step 7: Run Full Test Suite (5 minutes)

```bash
cd /path/to/clnrm

# Run all tests
cargo test --all-features

# Run with output
cargo test --all-features -- --nocapture

# Run specific test
cargo test --lib test_name
```

**Expected**: All tests pass

## Step 8: Development Environment Setup (10 minutes)

### Configure IDE (Choose one)

**VS Code**:
```bash
# Install recommended extensions
code --install-extension rust-lang.rust-analyzer
code --install-extension tamasfe.even-better-toml
code --install-extension serayuzgur.crates

# Open workspace
code /path/to/clnrm
```

**RustRover/IntelliJ IDEA**:
- Install Rust plugin
- Open clnrm directory as project
- Wait for indexing

### Configure Git Hooks

```bash
# Install pre-commit hook
cp scripts/pre-commit.sh .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit

# Test hook
git add .
git commit -m "test"  # Should run checks
```

## Step 9: Read Core Documentation (15 minutes)

**Must Read** (in order):

1. `README.md` - Project overview
2. `CLAUDE.md` - Development standards
3. `docs/CLI_GUIDE.md` - CLI usage
4. `docs/TOML_REFERENCE.md` - Test configuration
5. `.cursorrules` - Core team standards (CRITICAL)

**Key Takeaways**:
- ✅ NO `.unwrap()` or `.expect()` in production
- ✅ NO async trait methods (breaks `dyn`)
- ✅ ALL tests follow AAA pattern
- ✅ Dogfood principle: test with Homebrew installation

## Step 10: Understand Architecture (20 minutes)

### Workspace Structure

```
clnrm/
├── crates/
│   ├── clnrm/              # CLI binary
│   ├── clnrm-core/         # Core library
│   ├── clnrm-shared/       # Shared utilities
│   └── clnrm-ai/           # EXPERIMENTAL AI features (isolated)
├── tests/                  # Integration tests
├── examples/               # Example test projects
├── scripts/                # Automation scripts
└── docs/                   # Documentation
```

### Key Concepts

1. **Hermetic Testing** - Each test runs in complete isolation
2. **Container-Based** - Uses testcontainers-rs for Docker
3. **Plugin Architecture** - Extensible service plugins
4. **TOML Configuration** - Declarative test definitions
5. **Dogfooding** - Framework tests itself

### Core Abstractions

```rust
// Main entry point
CleanroomEnvironment::new().await?

// Service plugins
ServicePlugin trait - start(), stop(), health_check()

// Container backend
TestcontainerBackend - Docker abstraction

// Configuration
TestConfig - From .clnrm.toml files
```

## Step 11: Make Your First Contribution (30 minutes)

### Find a Good First Issue

```bash
# Check GitHub issues
gh issue list --label "good first issue"

# Or browse
open https://github.com/seanchatmangpt/clnrm/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22
```

### Create Feature Branch

```bash
# Create branch
git checkout -b feature/your-feature-name

# Or for bug fix
git checkout -b fix/issue-123
```

### Development Workflow

```bash
# 1. Make changes
# Edit files...

# 2. Run quick check
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test --lib

# 3. Run full validation
cargo test --all-features
clnrm self-test

# 4. Commit
git add .
git commit -m "feat: Description of change"

# 5. Push and create PR
git push -u origin feature/your-feature-name
gh pr create
```

## Step 12: Complete Onboarding Checklist

- [ ] Rust toolchain installed and working
- [ ] Development tools installed (cargo-make, etc.)
- [ ] clnrm repository cloned
- [ ] All tests passing locally
- [ ] Can build release binary
- [ ] Homebrew installation successful
- [ ] Self-tests passing
- [ ] IDE configured and working
- [ ] Git hooks installed
- [ ] Read core documentation
- [ ] Understand architecture
- [ ] Made first contribution (or ready to)
- [ ] Can run tests via production installation (`clnrm`)
- [ ] Understand dogfooding principle
- [ ] Know core team standards (no unwrap, no async traits, AAA tests)

## Common Setup Issues

### Issue: Docker not running

```bash
# Start Docker
open -a Docker  # macOS

# Or start Docker daemon
sudo systemctl start docker  # Linux
```

### Issue: Cargo build fails

```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

### Issue: Tests fail

```bash
# Ensure Docker is running
docker ps

# Run with output to see errors
cargo test -- --nocapture

# Run specific failing test
cargo test test_name -- --nocapture
```

### Issue: clnrm command not found

```bash
# Check PATH
echo $PATH | grep .cargo/bin

# Add to PATH if missing (add to ~/.bashrc or ~/.zshrc)
export PATH="$HOME/.cargo/bin:$PATH"

# Or install to system location
sudo cp target/release/clnrm /usr/local/bin/
```

## Next Steps

After onboarding:

1. **Join team communication** - Slack/Discord/etc.
2. **Review recent PRs** - See coding patterns
3. **Pick first issue** - Start contributing
4. **Ask questions** - Team is here to help
5. **Read kgold analysis** - See `/Users/sac/clnrm/docs/KGOLD_REPOSITORY_ANALYSIS.md`

## Resources

- **Documentation**: `docs/` directory
- **Examples**: `examples/` directory
- **Tests**: `tests/` and `crates/*/tests/`
- **Core Standards**: `.cursorrules`
- **CLI Guide**: `docs/CLI_GUIDE.md`
- **TOML Reference**: `docs/TOML_REFERENCE.md`

## Help and Support

**Stuck? Try these:**

1. Check documentation in `docs/`
2. Search GitHub issues for similar problems
3. Ask in team chat
4. Create discussion on GitHub
5. Run `/fix-test-failures` if tests failing

## Estimated Total Time

- ⏱️ **Initial setup**: ~30-40 minutes
- ⏱️ **Reading docs**: ~30 minutes
- ⏱️ **First contribution**: ~30-60 minutes
- ⏱️ **Total to productive**: ~2 hours

**Welcome to the clnrm team! 🎉**
