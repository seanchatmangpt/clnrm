# clnrm - Hermetic Container Testing Framework

**Core Purpose**: Deterministic, reproducible Docker container lifecycle testing via declarative TOML specifications.

**Tech Stack**: Rust (workspace), Chicago TDD, DfLSS (Design for Lean Six Sigma)
**Philosophy**: Type-safe, zero-runtime-errors, testcontainers integration, observable execution

---

## 🎯 THE VITAL FEW (20% that matters)

### 1. COMMAND RULE
```bash
# ✅ CORRECT: Always use cargo make
cargo make check     # Compilation check
cargo make test      # Run all tests
cargo make lint      # Clippy validation

# ❌ WRONG: Never direct cargo
cargo test
cargo clippy
```

### 2. ERROR HANDLING RULE
**Production code**: `Result<T, CleanroomError>` - NO `unwrap()`/`expect()`
**Test code**: `unwrap()` allowed in `#[test]`, `tests/`, `benches/`

### 3. CHICAGO TDD RULE
```rust
#[test]
fn test_container_lifecycle() {
    // Arrange: Real testcontainer instance
    let container = TestContainer::new().unwrap();

    // Act: Call public API
    container.start().unwrap();

    // Assert: Verify observable state
    assert!(container.is_running());
}
```
Tests verify: Return values, state changes, side effects—NOT implementation

### 4. ANDON SIGNAL RULE
| Signal | Trigger | Action |
|--------|---------|--------|
| **RED** | `error[E...]`, test failures | **STOP** - Fix immediately |
| **YELLOW** | Clippy warnings | Investigate before release |
| **GREEN** | Clean output | Continue |

### 5. CONCURRENT EXECUTION RULE
```javascript
// ✅ CORRECT: Single message with ALL operations
Task("Coder", "Implement Docker integration", "coder")
Task("Tester", "Write container lifecycle tests", "tester")
TodoWrite { todos: [10+ items in ONE call] }
Write "src/docker/mod.rs"
Write "tests/docker_test.rs"
Bash "cargo make check && cargo make test"
```

---

## 🚀 Quick Start

```bash
# Run test suite
clnrm run tests/

# Validate configuration
clnrm validate tests/

# List plugins
clnrm plugins

# Full validation
cargo make test      # All tests
cargo make lint      # Clippy
cargo make pre-commit  # Format + lint + tests
```

---

## 📁 Project Structure

```
clnrm/
├── crates/*/src/        # Source code (per crate)
├── crates/*/tests/      # Integration tests
├── tests/               # Workspace tests (.clnrm.toml specs)
├── scenarios/           # Test scenario definitions
├── docs/                # Documentation
├── benches/             # Benchmarks
├── cleanroom.toml       # Framework configuration
└── Cargo.toml           # Workspace manifest
```

---

## ⚡ Code Standards (Zero Tolerance)

- ✅ **No unwrap/expect in production** → Use `Result<T, CleanroomError>`
- ✅ **80%+ test coverage** → Chicago TDD with AAA pattern
- ✅ **Comprehensive error handling** → All code paths covered
- ✅ **100% type coverage** → No implicit types
- ✅ **Full public API docs** → NumPy-style docstrings
- ✅ **Format with `cargo fmt`** → Automated via hooks
- ✅ **Clippy clean** → No warnings accepted

See [CODE_STANDARDS.md](docs/CODE_STANDARDS.md) for checklist.

---

## 🧪 Self-Testing Principle

This project demonstrates the "eat your own dog food" principle: **the cleanroom framework tests itself** through real Docker container lifecycle tests defined in declarative TOML.

---

## 📋 Definition of Done

**BEFORE marking complete:**
```bash
cargo make check    # Must be clean (RED signal)
cargo make test     # Must be 100% pass (RED signal)
cargo make lint     # Must be clean (YELLOW signal)
```

---

## 🔧 SLOs (Service Level Objectives)

- First build ≤ 15s
- Incremental ≤ 2s
- Container startup ≤ 5s
- Test execution ≤ 30s (full suite)
- 100% reproducible outputs

---

## 🚫 Prohibited Patterns

1. Direct `cargo test` commands → Always use `cargo make test`
2. `unwrap()`/`expect()` in production → Use `Result<T, E>`
3. Skipping tests → Every feature needs test coverage
4. Ignoring Andon signals → Stop and fix when RED
5. Multiple messages for single task → Batch all operations

---

## 📚 Remember

**cargo make is the single source of truth for all commands**

**Stop the line when Andon signals appear**

**Tests verify behavior—code doesn't work if tests don't pass**

**Batch ALL operations in ONE message**

---

## 🔗 Essential Links

- **Standards**: [CODE_STANDARDS.md](docs/CODE_STANDARDS.md)
- **Issue Tracker**: GitHub Issues
- **Contributing**: See CODE_STANDARDS.md
