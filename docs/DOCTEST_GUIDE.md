# clnrm Doctest Guide

This guide explains how to write and run Rust doctests in the clnrm codebase.

## What Are Doctests?

Doctests are code examples embedded in Rust documentation comments that are automatically compiled and executed as tests. They ensure documentation stays in sync with the code.

## Running Doctests

```bash
# Run all doctests
cargo test --doc

# Run doctests for a specific crate
cargo test --doc -p clnrm-core

# Run doctests for a specific module
cargo test --doc config

# Verbose output
cargo test --doc -- --nocapture
```

## Writing Doctests

### Basic Pattern

```rust
/// Parse a configuration file.
///
/// # Examples
///
/// ```
/// use clnrm_core::config::Config;
///
/// let toml = r#"
/// [test]
/// name = "example"
///
/// [containers.alpine]
/// image = "alpine:latest"
/// "#;
///
/// let config: Config = toml::from_str(toml).unwrap();
/// assert_eq!(config.test.name, "example");
/// ```
pub fn parse(input: &str) -> Result<Config> {
    // implementation
}
```

### Key Elements

1. **`# Examples`** - Section header (required for documentation)
2. **Triple backticks** - Code fence with `rust` language (optional but recommended)
3. **Use statements** - Import required types
4. **Assertions** - Verify expected behavior

## Doctest Patterns for clnrm

### Pattern 1: Config Parsing

```rust
/// Parse test configuration from TOML.
///
/// # Examples
///
/// ```
/// use clnrm_core::config::spec::Config;
///
/// let toml = r#"
/// [test]
/// name = "my_test"
/// timeout = "60s"
///
/// [containers.postgres]
/// image = "postgres:15"
/// env = { POSTGRES_PASSWORD = "test" }
///
/// [[steps]]
/// name = "verify"
/// container = "postgres"
/// exec = ["pg_isready", "-U", "postgres"]
/// "#;
///
/// let config: Config = toml::from_str(toml).unwrap();
/// assert_eq!(config.test.name, "my_test");
/// assert!(config.containers.contains_key("postgres"));
/// assert_eq!(config.steps.len(), 1);
/// ```
```

### Pattern 2: Error Handling

```rust
/// Create a configuration error.
///
/// # Examples
///
/// ```
/// use clnrm_core::error::{CleanroomError, ErrorKind};
///
/// let error = CleanroomError::new(
///     ErrorKind::ConfigurationError,
///     "Invalid container reference"
/// );
///
/// assert_eq!(error.kind, ErrorKind::ConfigurationError);
/// assert!(error.message.contains("Invalid"));
/// ```
```

### Pattern 3: Fallible Operations (with Result)

```rust
/// Parse a shell command string.
///
/// # Examples
///
/// ```
/// use clnrm_core::config::types::parse_shell_command;
///
/// let cmd = parse_shell_command("echo hello world").unwrap();
/// assert_eq!(cmd, vec!["echo", "hello", "world"]);
///
/// // Handles quoted strings
/// let quoted = parse_shell_command(r#"echo "hello world""#).unwrap();
/// assert_eq!(quoted, vec!["echo", "hello world"]);
/// ```
///
/// # Errors
///
/// ```
/// use clnrm_core::config::types::parse_shell_command;
///
/// // Unbalanced quotes fail
/// let result = parse_shell_command(r#"echo "hello"#);
/// assert!(result.is_err());
/// ```
```

### Pattern 4: no_run for Docker-Dependent Code

Use `no_run` for code that requires Docker but should still compile:

```rust
/// Start a container.
///
/// # Examples
///
/// ```no_run
/// use clnrm_core::backend::TestcontainerBackend;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let backend = TestcontainerBackend::new("alpine:latest")?;
///     let container = backend.start().await?;
///     println!("Container started: {}", container.id);
///     Ok(())
/// }
/// ```
```

### Pattern 5: ignore for Long-Running Tests

Use `ignore` for tests that take too long for CI:

```rust
/// Run a full integration test.
///
/// # Examples
///
/// ```ignore
/// // This test requires Docker and takes 30+ seconds
/// use clnrm_core::CleanroomEnvironment;
///
/// let env = CleanroomEnvironment::new();
/// env.run_full_suite().await;
/// ```
```

### Pattern 6: Hidden Setup Lines

Use `#` to hide setup code that's not relevant to the example:

```rust
/// Get container health status.
///
/// # Examples
///
/// ```
/// # use clnrm_core::cleanroom::{HealthStatus, ServiceHandle};
/// # use std::collections::HashMap;
/// # let handle = ServiceHandle {
/// #     id: "test-123".to_string(),
/// #     service_name: "test".to_string(),
/// #     metadata: HashMap::new(),
/// # };
/// // Check health status
/// let status = HealthStatus::Healthy;
/// assert_eq!(status, HealthStatus::Healthy);
/// ```
```

### Pattern 7: Compile-Only Tests

Use `compile_fail` to verify code that should NOT compile:

```rust
/// Container references must be valid.
///
/// ```compile_fail
/// use clnrm_core::config::spec::Config;
///
/// let toml = r#"
/// [test]
/// name = "invalid"
///
/// [[steps]]
/// name = "bad_step"
/// container = "nonexistent"  // Error: container not defined
/// exec = ["echo", "hello"]
/// "#;
///
/// let config: Config = toml::from_str(toml).unwrap();
/// // This should fail at parse time
/// ```
```

## Common Issues and Solutions

### Issue 1: Missing Use Statements

```rust
// Wrong - will fail with "cannot find type"
/// ```
/// let config = Config::new();
/// ```

// Correct - include use statement
/// ```
/// use clnrm_core::config::Config;
/// let config = Config::default();
/// ```
```

### Issue 2: Async Code

```rust
// Wrong - async code needs runtime
/// ```
/// async fn example() {
///     let result = do_something().await;
/// }
/// ```

// Correct - use tokio::main or block_on
/// ```
/// use tokio::runtime::Runtime;
///
/// let rt = Runtime::new().unwrap();
/// rt.block_on(async {
///     // async code here
/// });
/// ```

// Or use #[tokio::main]
/// ```no_run
/// #[tokio::main]
/// async fn main() {
///     // async code here
/// }
/// ```
```

### Issue 3: Error Propagation

```rust
// Wrong - ? operator needs Result return type
/// ```
/// let config = Config::from_file("test.toml")?;
/// ```

// Correct - wrap in function or use Ok pattern
/// ```
/// use clnrm_core::config::Config;
/// fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let config = Config::from_file("test.toml")?;
///     Ok(())
/// }
/// ```

// Or use hidden main
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # Ok(())
/// # }
/// ```
```

### Issue 4: Feature-Gated Code

```rust
/// ```
/// #[cfg(feature = "otel")]
/// use clnrm_core::telemetry::init_otel;
///
/// #[cfg(feature = "otel")]
/// fn example() {
///     init_otel();
/// }
/// ```
```

## Doctest Best Practices

### 1. Keep Examples Simple

Focus on the API being documented:

```rust
// Good - focused example
/// ```
/// use clnrm_core::error::CleanroomError;
/// let err = CleanroomError::timeout("Operation timed out");
/// assert_eq!(err.kind, clnrm_core::error::ErrorKind::Timeout);
/// ```

// Bad - too much unrelated code
/// ```
/// use std::collections::HashMap;
/// use std::sync::Arc;
/// use tokio::sync::RwLock;
/// // ... 20 lines of setup ...
/// ```
```

### 2. Use Realistic Examples

Show actual use cases:

```rust
// Good - realistic TOML example
/// ```
/// let toml = r#"
/// [test]
/// name = "api_integration"
///
/// [containers.postgres]
/// image = "postgres:15"
/// "#;
/// ```

// Bad - unrealistic minimal example
/// ```
/// let toml = "name = 'x'";
/// ```
```

### 3. Document Error Cases

Show what can go wrong:

```rust
/// # Errors
///
/// Returns an error if:
/// - The file doesn't exist
/// - The TOML is malformed
/// - Container references are invalid
///
/// ```
/// use clnrm_core::config::Config;
///
/// // Malformed TOML fails
/// let result = Config::from_str("[invalid");
/// assert!(result.is_err());
/// ```
```

### 4. Test Edge Cases

```rust
/// # Examples
///
/// ```
/// use clnrm_core::config::types::parse_shell_command;
///
/// // Empty strings fail
/// assert!(parse_shell_command("").is_err());
///
/// // Whitespace-only fails
/// assert!(parse_shell_command("   ").is_err());
///
/// // Normal commands work
/// assert!(parse_shell_command("echo hello").is_ok());
/// ```
```

## CI Integration

Add doctest validation to your CI workflow:

```yaml
# .github/workflows/ci.yml
jobs:
  doctests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Run doctests
        run: cargo test --doc --all-features
```

## Module-Level Documentation

Add module-level doctests in `mod.rs` or at the top of files:

```rust
//! # Configuration Module
//!
//! This module handles TOML configuration parsing.
//!
//! ## Quick Start
//!
//! ```
//! use clnrm_core::config::spec::Config;
//!
//! let config: Config = toml::from_str(r#"
//! [test]
//! name = "quickstart"
//!
//! [containers.alpine]
//! image = "alpine:latest"
//! "#).unwrap();
//! ```
//!
//! ## Advanced Usage
//!
//! See [`Config`] for all configuration options.
```

## Doctest Coverage

Target coverage for clnrm v2.0.0:

| Module | Target | Description |
|--------|--------|-------------|
| `config::spec` | 10+ examples | Config parsing |
| `config::types` | 5+ examples | Helper types |
| `backend` | 5+ examples | Backend trait |
| `cleanroom` | 3+ examples | Core environment |
| `error` | 5+ examples | Error handling |

---

**Last Updated:** 2025-12-03
**Version:** 2.0.0
