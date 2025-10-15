# Implementation Roadmap: Fixing False Positives

## Phase 1: Core Test Execution Infrastructure (Week 1-2)

### Priority 1A: Container Command Execution
**Objective:** Implement the missing `execute_in_container` method

**Tasks:**
1. **Add ExecutionResult structure**
   ```rust
   #[derive(Debug, Clone)]
   pub struct ExecutionResult {
       pub exit_code: i32,
       pub stdout: String,
       pub stderr: String,
       pub duration: Duration,
       pub command: Vec<String>,
   }
   ```

2. **Implement execute_in_container method**
   ```rust
   impl CleanroomEnvironment {
       pub async fn execute_in_container(
           &self,
           container_name: &str,
           command: &[String]
       ) -> Result<ExecutionResult> {
           // Check if container exists in registry
           // Execute command using testcontainers backend
           // Capture output and timing
           // Return structured result
       }
   }
   ```

3. **Add regex validation helper**
   ```rust
   impl ExecutionResult {
       pub fn matches_regex(&self, pattern: &str) -> Result<bool> {
           Ok(regex::Regex::new(pattern)?.is_match(&self.stdout))
       }

       pub fn does_not_match_regex(&self, pattern: &str) -> Result<bool> {
           Ok(!regex::Regex::new(pattern)?.is_match(&self.stdout))
       }
   }
   ```

### Priority 1B: TOML Test Execution Pipeline
**Objective:** Connect TOML parsing to actual test execution

**Tasks:**
1. **Implement test execution from TOML config**
   ```rust
   impl CleanroomEnvironment {
       pub async fn execute_test_config(&self, config: TestConfig) -> Result<TestExecutionResult> {
           // For each scenario in config
           // For each step in scenario
           // Execute command in appropriate container
           // Validate expected output regex
           // Track execution results
       }
   }
   ```

2. **Add assertion validation**
   ```rust
   pub async fn validate_assertions(
       &self,
       results: &[StepExecutionResult],
       assertions: &HashMap<String, Value>
   ) -> Result<Vec<AssertionResult>> {
       // Parse assertion values from TOML
       // Validate each assertion against results
       // Return pass/fail with details
   }
   ```

3. **Update CLI run_single_test function**
   ```rust
   async fn run_single_test(path: &PathBuf, config: &CliConfig) -> Result<TestExecutionResult> {
       let test_config = load_config_from_file(path)?;
       let env = CleanroomEnvironment::new().await?;
       let result = env.execute_test_config(test_config).await?;
       Ok(result)
   }
   ```

## Phase 2: CLI Integration Fixes (Week 3)

### Priority 2A: File Extension Consistency
**Tasks:**
1. **Update CLI to accept both extensions**
   ```rust
   const ACCEPTED_EXTENSIONS: &[&str] = &[".toml", ".clnrm.toml"];

   if ACCEPTED_EXTENSIONS.iter().any(|ext| path_str.ends_with(ext)) {
       test_files.push(path.clone());
   }
   ```

2. **Update example files to use consistent extension**
   - Rename example files to `.clnrm.toml` or
   - Update CLI to prefer `.toml` extension

### Priority 2B: Framework Self-Tests CLI Integration
**Tasks:**
1. **Debug CLI self-test failures**
   - Run individual test functions to identify failures
   - Fix error handling in CLI integration
   - Ensure proper async execution context

2. **Fix result reporting**
   ```rust
   // Ensure run_framework_tests() returns proper FrameworkTestResults
   // Fix CLI display and error handling
   ```

## Phase 3: Example Validation and Fixes (Week 4)

### Priority 3A: Fix Broken Examples
**Tasks:**
1. **Fix import errors**
   ```rust
   // Change from:
   use clnrm_core::{cleanroom_test, CleanroomEnvironment, Result};
   // To:
   use clnrm_core::{CleanroomEnvironment};
   use clnrm_core::error::Result;
   ```

2. **Remove references to non-existent methods**
   ```rust
   // Remove calls to execute_in_container until implemented
   // Use direct backend calls or mock implementations for now
   ```

3. **Add missing dependencies**
   ```toml
   # Add to Cargo.toml if needed
   [dependencies]
   futures = "0.3"
   ```

### Priority 3B: Create Working Examples
**Tasks:**
1. **Create simple working TOML test**
   ```toml
   [test.metadata]
   name = "working_example"
   description = "A working TOML test example"

   [services.test_container]
   type = "generic_container"
   plugin = "alpine"
   image = "alpine:latest"

   [[steps]]
   name = "echo_test"
   command = ["echo", "Hello from TOML test"]
   expected_output_regex = "Hello from TOML test"
   ```

2. **Create CLI execution verification script**
   ```bash
   #!/bin/bash
   # Verify end-to-end TOML execution
   clnrm run examples/working_example.clnrm.toml
   ```

## Phase 4: Advanced Features (Week 5+)

### Priority 4A: Interactive Mode
**Tasks:**
1. **Implement interactive debugging**
   ```rust
   pub async fn run_interactive_mode(&self, test_path: &Path) -> Result<()> {
       // Parse config
       // Show each step before execution
       // Wait for user input (continue/skip/retry/quit)
       // Execute with user control
   }
   ```

### Priority 4B: Watch Mode
**Tasks:**
1. **Implement file watching**
   ```rust
   pub async fn run_watch_mode(&self, paths: &[PathBuf]) -> Result<()> {
       // Set up file watcher
       // Run tests on file changes
       // Debounce rapid changes
   }
   ```

### Priority 4C: Report Generation
**Tasks:**
1. **Implement HTML report generation**
   ```rust
   pub async fn generate_html_report(&self, results: &TestResults) -> Result<String> {
       // Generate structured HTML
       // Include timing, pass/fail status
       // Add styling and interactivity
   }
   ```

## Testing and Validation Strategy

### Unit Tests for New Functionality
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_in_container() {
        let env = CleanroomEnvironment::new().await.unwrap();
        let result = env.execute_in_container(
            "test_container",
            &["echo", "test"].iter().map(|s| s.to_string()).collect::<Vec<_>>()
        ).await.unwrap();

        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("test"));
    }

    #[tokio::test]
    async fn test_regex_validation() {
        let result = ExecutionResult {
            exit_code: 0,
            stdout: "SUCCESS: Pattern matched".to_string(),
            stderr: "".to_string(),
            duration: Duration::from_secs(1),
            command: vec!["echo".to_string(), "test".to_string()],
        };

        assert!(result.matches_regex("SUCCESS.*matched").unwrap());
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_end_to_end_toml_execution() {
    let env = CleanroomEnvironment::new().await.unwrap();
    let config = TestConfig {
        name: "integration_test".to_string(),
        scenarios: vec![/* ... */],
        // ...
    };

    let result = env.execute_test_config(config).await.unwrap();
    assert!(result.passed);
    assert_eq!(result.steps_executed, 3);
}
```

### CLI Integration Tests
```bash
#!/bin/bash
# Test CLI end-to-end
set -e

# Test TOML validation
clnrm validate examples/simple_test.clnrm.toml

# Test TOML execution
clnrm run examples/simple_test.clnrm.toml

# Test self-tests
clnrm self-test

# Test report generation
clnrm report --format junit > test_results.xml
```

## Success Metrics

### Technical Success Criteria:
- [ ] All tests pass (`cargo test`)
- [ ] No compilation warnings (`cargo clippy`)
- [ ] No `unwrap()` or `expect()` in production code
- [ ] All traits remain `dyn` compatible
- [ ] Proper async/sync patterns throughout

### Functional Success Criteria:
- [ ] `clnrm run test.toml` executes tests end-to-end
- [ ] `clnrm self-test` passes all framework tests
- [ ] `clnrm validate test.toml` works with both extensions
- [ ] All examples compile and run successfully
- [ ] Regex validation works in container output

### User Experience Success Criteria:
- [ ] CLI provides helpful error messages
- [ ] Examples are copy-paste ready
- [ ] Documentation matches implementation
- [ ] Interactive mode provides useful debugging
- [ ] Reports are informative and well-formatted

## Risk Mitigation

### Dependencies:
- Ensure all required crates are properly versioned in Cargo.toml
- Test with different Docker/containerd versions
- Validate on multiple platforms (Linux, macOS)

### Error Handling:
- Comprehensive error context at every level
- User-friendly error messages in CLI
- Proper cleanup on failures
- Graceful degradation for optional features

### Performance:
- Container reuse actually provides 10-50x improvement
- Parallel execution scales properly
- Memory usage stays reasonable
- Startup time is acceptable

## Rollout Plan

### Week 1: Core Implementation
- Implement container command execution
- Add regex validation
- Basic TOML test execution

### Week 2: CLI Integration
- Fix file extension handling
- Connect CLI to test execution
- Fix framework self-tests

### Week 3: Example Fixes
- Fix all broken examples
- Create working demonstration files
- Update documentation

### Week 4: Advanced Features
- Interactive mode
- Watch mode
- Report generation

### Week 5: Testing and Polish
- Comprehensive testing
- Performance optimization
- Documentation updates
- Final validation

This roadmap provides a structured approach to fixing the identified false positives and implementation gaps while maintaining the core team's high standards for code quality and reliability.
