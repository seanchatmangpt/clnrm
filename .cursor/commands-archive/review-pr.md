# Review PR - Code Review Checklist

Comprehensive code review process for clnrm pull requests following core team standards.

## What This Does

Systematic code review ensuring production quality:
1. **Automated checks** - CI/CD status
2. **Code quality** - Standards compliance
3. **Testing** - Coverage and correctness
4. **Documentation** - Completeness
5. **Security** - Vulnerability review
6. **Performance** - Benchmark impact

## Review Checklist

### 1. Automated Checks ✅

- [ ] **CI passes** - All GitHub Actions workflows green
- [ ] **Tests pass** - 100% test success rate
- [ ] **No clippy warnings** - Zero warnings with `-D warnings`
- [ ] **Code formatted** - `cargo fmt --check` passes
- [ ] **Build succeeds** - `cargo build --release --features otel`

### 2. Core Team Standards Compliance 🎯

#### Error Handling (MANDATORY)

- [ ] **No `.unwrap()` or `.expect()`** in production code paths
  ```rust
  // ❌ WRONG
  let result = operation().unwrap();

  // ✅ CORRECT
  let result = operation().map_err(|e| {
      CleanroomError::internal_error(format!("Operation failed: {}", e))
  })?;
  ```

- [ ] **All functions return `Result<T, CleanroomError>`**
- [ ] **Meaningful error messages** with context

#### Async/Sync Rules (CRITICAL)

- [ ] **No async trait methods** - Maintains `dyn` compatibility
  ```rust
  // ❌ WRONG - breaks dyn ServicePlugin
  pub trait ServicePlugin {
      async fn start(&self) -> Result<ServiceHandle>;
  }

  // ✅ CORRECT - dyn compatible
  pub trait ServicePlugin {
      fn start(&self) -> Result<ServiceHandle>;
  }
  ```

- [ ] **Use `tokio::task::block_in_place` for async in sync contexts**

#### Testing Standards

- [ ] **All tests follow AAA pattern** (Arrange, Act, Assert)
  ```rust
  #[tokio::test]
  async fn test_container_creation_succeeds() -> Result<()> {
      // Arrange
      let environment = TestEnvironments::unit_test().await?;

      // Act
      let container = environment.create_container("alpine:latest").await?;

      // Assert
      assert!(container.is_running());
      Ok(())
  }
  ```

- [ ] **Descriptive test names** explaining what is being tested
- [ ] **Test isolation** - No shared state between tests

#### No False Positives (CRITICAL)

- [ ] **No fake `Ok(())` stubs** without real implementation
  ```rust
  // ❌ WRONG - lying about success
  pub fn execute_test(&self) -> Result<()> {
      println!("Test executed");
      Ok(())  // Did nothing!
  }

  // ✅ CORRECT - honest about incompleteness
  pub fn execute_test(&self) -> Result<()> {
      unimplemented!("execute_test: needs container execution")
  }
  ```

### 3. Code Quality 📊

- [ ] **Follows Rust idioms** - Idiomatic Rust patterns
- [ ] **No `println!` in production** - Use `tracing` macros
- [ ] **Proper visibility** - Only public what needs to be
- [ ] **No dead code** - Remove unused functions
- [ ] **Consistent naming** - Follow clnrm conventions
- [ ] **DRY principle** - No code duplication
- [ ] **Small functions** - Single responsibility

### 4. Testing 🧪

- [ ] **Unit tests added** for new functionality
- [ ] **Integration tests** if touching core systems
- [ ] **Edge cases covered** - Null, empty, boundary values
- [ ] **Error paths tested** - Failure scenarios
- [ ] **No flaky tests** - Run tests 10 times to verify
  ```bash
  for i in {1..10}; do cargo test new_test || exit 1; done
  ```

### 5. Documentation 📝

- [ ] **All public items documented** - Doc comments with examples
  ```rust
  /// Creates a new service plugin
  ///
  /// # Examples
  ///
  /// ```
  /// use clnrm_core::services::MyPlugin;
  ///
  /// let plugin = MyPlugin::new("test", "alpine:latest");
  /// ```
  pub fn new(name: &str, image: &str) -> Self { }
  ```

- [ ] **README updated** if public API changes
- [ ] **CHANGELOG updated** with changes
- [ ] **Inline comments** for complex logic
- [ ] **Examples work** - Doc tests pass

### 6. Security 🔒

- [ ] **No hardcoded credentials** or secrets
- [ ] **Input validation** - Sanitize external input
- [ ] **No SQL injection** vectors (if applicable)
- [ ] **Dependencies vetted** - No suspicious crates
- [ ] **Security audit clean** - `cargo audit` passes
- [ ] **License compliance** - `cargo deny check` passes

### 7. Performance ⚡

- [ ] **No unnecessary allocations** in hot paths
- [ ] **Appropriate data structures** - Vec vs HashMap, etc.
- [ ] **No excessive cloning** - Use references where possible
- [ ] **Benchmarks run** if performance-critical
  ```bash
  cargo bench
  ```

### 8. Configuration & Dependencies 🔧

- [ ] **New dependencies justified** - Explained in PR description
- [ ] **Features used minimally** - No bloat
- [ ] **Version pinning** appropriate
- [ ] **No wildcard versions** - Specific version numbers
- [ ] **Cargo.toml clean** - Alphabetized, organized

### 9. Git Hygiene 🌿

- [ ] **Commit messages clear** - Explain why, not just what
- [ ] **Atomic commits** - One logical change per commit
- [ ] **No merge commits** (if rebasing policy)
- [ ] **Branch up to date** with master/main
- [ ] **No unrelated changes** - Focused PR scope

### 10. Clnrm-Specific Checks 🧹

- [ ] **Dogfooding validated** - Tested with Homebrew installation
  ```bash
  brew install clnrm
  clnrm self-test
  ```

- [ ] **Not tested with `cargo run`** for final validation
- [ ] **TOML configs valid** if touching test definitions
- [ ] **Plugin registration** if new service plugin
- [ ] **Backend compatibility** - Works with TestcontainerBackend

## Review Process

### 1. Initial Review (5-10 minutes)

```bash
# Checkout PR branch
gh pr checkout <PR-number>

# Run quick validation
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --lib
```

### 2. Deep Review (15-30 minutes)

- Read all changed files in GitHub UI
- Check for patterns listed above
- Look for edge cases not tested
- Verify error handling is comprehensive

### 3. Functional Testing (10-15 minutes)

```bash
# Build and test
cargo build --release --all-features
cargo test --all-features

# If new feature, test manually
clnrm <new-command> <args>

# Self-test
clnrm self-test
```

### 4. Provide Feedback

#### Types of Comments

**🚨 Critical (Must Fix)**
```
🚨 This uses `.unwrap()` which can panic. Please use proper error handling:

\`\`\`rust
operation().map_err(|e| CleanroomError::internal_error(format!("Failed: {}", e)))?
\`\`\`
```

**⚠️ Important (Should Fix)**
```
⚠️ This test doesn't follow the AAA pattern. Consider restructuring:

\`\`\`rust
#[test]
fn test_name() {
    // Arrange
    let input = setup();

    // Act
    let result = function(input);

    // Assert
    assert_eq!(result, expected);
}
\`\`\`
```

**💡 Suggestion (Nice to Have)**
```
💡 Consider using a HashMap here instead of Vec for O(1) lookups.
```

**✅ Praise (Acknowledge Good Work)**
```
✅ Excellent error handling here! Clear messages with proper context.
```

### 5. Approval Decision

**Approve if:**
- ✅ All critical checks pass
- ✅ All important checks pass or have acceptable plan
- ✅ Tests comprehensive
- ✅ Documentation complete
- ✅ No security concerns

**Request changes if:**
- ❌ Any critical standard violated
- ❌ Tests missing or inadequate
- ❌ Security issues present
- ❌ Breaking changes undocumented

**Comment (don't approve) if:**
- 💭 Need clarification
- 💭 Want to see alternative approach
- 💭 Waiting for CI results

## Common Issues to Watch For

### 1. The Unwrap Trap
```rust
// ❌ Found in many PRs
let value = map.get("key").unwrap();

// ✅ Should be
let value = map.get("key")
    .ok_or_else(|| CleanroomError::config_error("Missing key"))?;
```

### 2. Async Trait Methods
```rust
// ❌ Breaks dyn compatibility
trait MyTrait {
    async fn do_thing(&self) -> Result<()>;
}

// ✅ Keeps dyn compatibility
trait MyTrait {
    fn do_thing(&self) -> Result<()> {
        tokio::task::block_in_place(|| {
            // async work
        })
    }
}
```

### 3. Missing Error Context
```rust
// ❌ Generic error
Err(anyhow::anyhow!("Failed"))

// ✅ Specific error with context
Err(CleanroomError::internal_error(
    format!("Failed to start container '{}': {}", name, error)
))
```

### 4. Poor Test Names
```rust
// ❌ Unclear
#[test]
fn test_1() { }

// ✅ Descriptive
#[test]
fn test_container_creation_with_invalid_image_fails() { }
```

### 5. Fake Implementations
```rust
// ❌ Pretends to work
pub fn validate(&self) -> Result<()> {
    println!("Validated");
    Ok(())  // Fake!
}

// ✅ Honest about status
pub fn validate(&self) -> Result<()> {
    unimplemented!("Validation logic not yet implemented")
}
```

## When to Escalate

Escalate to maintainers if:
- Major architecture changes proposed
- Breaking API changes
- New external dependencies
- Security concerns
- Licensing issues

## After Approval

- [ ] **Merge when CI green**
- [ ] **Delete feature branch**
- [ ] **Close linked issues**
- [ ] **Update project board** if applicable

## When to Use

- Reviewing pull requests
- Self-reviewing before requesting review
- Teaching new contributors
- Quality assurance process
