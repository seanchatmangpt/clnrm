# Onboard New Developer

Comprehensive onboarding guide to get new developers productive with clnrm quickly.

## Welcome to clnrm!

You're joining the **Cleanroom Testing Framework** project - a hermetic integration testing framework for container-based isolation with plugin architecture.

## Day 1: Environment Setup

### 1. Install Prerequisites

**Required**:
```bash
# Rust (1.70+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update

# Docker
# macOS: Install Docker Desktop
brew install --cask docker

# Linux: Install Docker Engine
curl -fsSL https://get.docker.com | sh

# Verify installations
rustc --version
cargo --version
docker --version
```

**Recommended**:
```bash
# Homebrew (macOS)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# cargo-make (task runner)
cargo install cargo-make

# cargo-watch (auto-rebuild)
cargo install cargo-watch

# cargo-tarpaulin (coverage)
cargo install cargo-tarpaulin
```

### 2. Clone Repository

```bash
# Clone the repo
git clone https://github.com/seanchatmangpt/clnrm.git
cd clnrm

# Checkout main branch
git checkout main
git pull origin main
```

### 3. Build Project

```bash
# Build workspace (excludes clnrm-ai by default)
cargo build

# Build with OTEL features
cargo build --features otel

# Build release
cargo build --release --features otel

# Verify build
cargo test --lib
```

### 4. Install Locally

```bash
# Install clnrm CLI via Homebrew
brew install --build-from-source .

# Verify installation
which clnrm
clnrm --version

# Run self-tests
clnrm self-test
```

## Day 2: Project Familiarization

### 5. Understand Project Structure

Read these files in order:

**Essential**:
1. `README.md` - Project overview, features, quick start
2. `CLAUDE.md` - Development guide for Claude Code users
3. `.cursorrules` - Core Team Standards (CRITICAL)
4. `docs/CLI_GUIDE.md` - CLI command reference
5. `docs/TESTING.md` - Testing philosophy and practices

**Architecture**:
6. `Cargo.toml` - Workspace structure (note: `clnrm-ai` excluded)
7. `crates/clnrm-core/src/lib.rs` - Core framework API
8. `crates/clnrm-core/src/cleanroom.rs` - Main abstractions
9. `crates/clnrm-core/src/services/` - Plugin implementations

**Quality**:
10. `scripts/ci-gate.sh` - Quality enforcement
11. `scripts/scan-fakes.sh` - Fake implementation detector
12. `docs/TIER1_ADAPTATION_COMPLETE.md` - Recent kcura adaptations

### 6. Explore the Codebase

```bash
# Workspace crates
ls -la crates/
# clnrm/          - CLI binary
# clnrm-core/     - Core framework library
# clnrm-shared/   - Shared utilities
# clnrm-ai/       - EXPERIMENTAL (excluded from default builds)

# Service plugins
ls -la crates/clnrm-core/src/services/
# generic.rs      - Generic container wrapper
# surrealdb.rs    - SurrealDB plugin
# ollama.rs       - Ollama LLM plugin
# chaos_engine.rs - Chaos engineering

# Tests
ls -la crates/clnrm-core/tests/
# determinism_test.rs            - Hermetic isolation verification
# otel_validation_integration.rs - OTEL telemetry testing
# v1_compliance_comprehensive.rs - Compliance validation
```

### 7. Run Tests

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test '*'

# Specific test
cargo test test_container_creation

# With output
cargo test -- --nocapture

# Determinism tests
cargo test --test determinism_test

# Quality gates
bash scripts/ci-gate.sh

# Fake scanner
bash scripts/scan-fakes.sh
```

## Day 3: Core Concepts

### 8. Understand Hermetic Testing

**Key Principle**: Each test runs in complete isolation with fresh containers.

**Pattern**:
```rust
#[tokio::test]
async fn test_example() -> Result<()> {
    // Each test gets fresh environment
    let env = CleanroomEnvironment::new().await?;

    // Register services
    let plugin = Box::new(GenericContainerPlugin::new("test", "alpine:latest"));
    env.register_service(plugin).await?;

    // Start service
    let handle = env.start_service("test").await?;

    // Execute commands
    let output = env.execute_command(&handle, &["echo", "hello"]).await?;

    // Assert
    assert_eq!(output.stdout.trim(), "hello");

    // Cleanup automatic on drop
    Ok(())
}
```

### 9. Core Team Standards (CRITICAL)

**Read `.cursorrules` carefully**. Key rules:

**Error Handling**:
```rust
// ❌ NEVER
operation().unwrap()
operation().expect("message")

// ✅ ALWAYS
operation().map_err(|e| CleanroomError::internal_error(format!("Failed: {}", e)))?
```

**Async/Sync**:
```rust
// ❌ NEVER - breaks dyn compatibility
trait ServicePlugin {
    async fn start(&self) -> Result<ServiceHandle>;
}

// ✅ ALWAYS - dyn compatible
trait ServicePlugin {
    fn start(&self) -> Result<ServiceHandle>;
}
```

**No Fake Implementations**:
```rust
// ❌ NEVER
fn execute(&self) -> Result<()> { Ok(()) }  // Lying!

// ✅ ALWAYS
fn execute(&self) -> Result<()> { unimplemented!("execute: needs implementation") }
```

### 10. TOML Configuration

Learn TOML test format:

```toml
[test.metadata]
name = "example_test"
description = "Example test description"

[services.my_service]
type = "generic_container"
image = "alpine:latest"

[[steps]]
name = "step_1"
command = ["echo", "hello"]
expected_output_regex = "hello"
service = "my_service"

[assertions]
container_should_have_executed_commands = 1
execution_should_be_hermetic = true
```

Run TOML tests:
```bash
clnrm run tests/example.clnrm.toml
```

## Day 4: First Contribution

### 11. Find a Task

**Good First Issues**:
- Documentation improvements
- Test coverage additions
- Example TOML configurations
- Service plugin enhancements

**Check**:
```bash
# View GitHub issues
gh issue list --label "good first issue"
```

### 12. Development Workflow

```bash
# Create feature branch
git checkout -b feature/my-feature

# Make changes
vim crates/clnrm-core/src/...

# Run tests frequently
cargo test
cargo watch -x test  # Auto-run on changes

# Quality checks
bash scripts/ci-gate.sh
bash scripts/scan-fakes.sh
cargo clippy -- -D warnings
cargo fmt

# Commit (follow conventional commits)
git add .
git commit -m "feat: add new feature description"
```

### 13. Create First PR

```bash
# Push branch
git push origin feature/my-feature

# Create PR
gh pr create --title "feat: My feature" --body "$(cat <<'EOF'
## Summary
Brief description of changes

## Testing
- [x] Unit tests pass
- [x] Integration tests pass
- [x] Quality gates pass
- [x] Manual testing completed

## Checklist
- [x] No unwrap/expect in production
- [x] Proper error handling
- [x] Tests follow AAA pattern
- [x] Documentation updated
EOF
)"
```

## Week 1: Deeper Dive

### 14. Advanced Topics

**Plugin Development**:
- Study `src/services/generic.rs`
- Read `/create-new-service-plugin` command
- Implement simple plugin

**OTEL Integration**:
- Enable OTEL features: `cargo build --features otel`
- Read `src/telemetry/otel.rs`
- Run OTEL validation: `bash scripts/validate-otel.sh`

**Determinism Testing**:
- Study `tests/determinism_test.rs`
- Understand 5-iteration pattern
- Learn output normalization

### 15. Team Workflow

**Daily**:
- Pull latest main: `git pull origin main`
- Run tests before pushing: `cargo test`
- Run quality gates: `bash scripts/ci-gate.sh`

**Before PR**:
- Rebase on main: `git rebase main`
- Run full test suite
- Update documentation
- Self-review changes

**After PR merged**:
- Delete feature branch
- Pull main
- Verify CI passes

## Helpful Commands (Cursor IDE)

In Cursor, type `/` to see these custom commands:

- `/run-quality-gates` - Run all quality checks
- `/run-determinism-tests` - Verify hermetic isolation
- `/validate-otel-integration` - Test OTEL emission
- `/run-framework-self-test` - Run clnrm self-tests
- `/create-new-service-plugin` - Guide for new plugins
- `/review-pr-changes` - PR review checklist
- `/prepare-release` - Release preparation guide

## Resources

**Documentation**:
- Project: `docs/`
- API: `cargo doc --open`
- Examples: `examples/`, `tests/`

**Code Reference**:
- Main API: `crates/clnrm-core/src/lib.rs`
- Plugins: `crates/clnrm-core/src/services/`
- Tests: `crates/clnrm-core/tests/`

**External**:
- Testcontainers: https://docs.rs/testcontainers/latest/testcontainers/
- Tokio: https://tokio.rs/
- TOML: https://toml.io/

## Getting Help

**Stuck?**
1. Check documentation: `docs/`
2. Search issues: `gh issue list`
3. Read existing code
4. Ask in PR comments
5. Reach out to maintainers

## Week 2+: Autonomy

By week 2, you should be able to:
- [ ] Independently develop features
- [ ] Write comprehensive tests
- [ ] Review others' PRs
- [ ] Debug complex issues
- [ ] Contribute to architecture decisions

**Welcome to the team!** 🚀
