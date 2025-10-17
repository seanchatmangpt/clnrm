# London School TDD Mock Contracts - v0.7.0 DX Features

## Overview

This directory contains comprehensive mock interaction contracts for v0.7.0 developer experience features, designed using London School (mockist) TDD methodology.

## Mock Contracts

### 1. MockFileWatcher
**File**: `file_watcher_mock.md`

**Purpose**: Simulate file system watching for live reload functionality.

**Key Interactions**:
- `watch(path)` - Register paths for monitoring
- `set_debounce(duration)` - Configure change detection timing
- `trigger_change(path)` - Simulate file system events
- `on_change(callback)` - Register change handlers

**Use Cases**:
- Verify watch registration sequence (templates before configs)
- Test debounce configuration timing
- Validate change callback invocation
- Simulate file system errors

### 2. MockTemplateRenderer
**File**: `template_renderer_mock.md`

**Purpose**: Define Tera template rendering interactions with context injection.

**Key Interactions**:
- `render(template, context)` - Render template with variables
- `validate_syntax(template)` - Check template validity
- `clear_cache(template)` - Force template re-parsing
- `get_required_variables(template)` - List required context keys

**Use Cases**:
- Verify rendering called with correct context
- Test cache invalidation after template changes
- Validate syntax checking before first render
- Simulate rendering errors and context issues

### 3. MockChangeDetector
**File**: `change_detector_mock.md`

**Purpose**: Track file changes through hash comparison and cache management.

**Key Interactions**:
- `compute_hash(path)` - Calculate content hash
- `has_changed(path)` - Compare current vs stored hash
- `store_hash(path, hash)` - Save hash for future comparison
- `invalidate_cache(path)` - Force hash recomputation

**Use Cases**:
- Verify hash computation before change checks
- Test cache invalidation on file changes
- Validate change detection results used correctly
- Simulate hash storage after successful renders

### 4. MockDryRunValidator
**File**: `dry_run_validator_mock.md`

**Purpose**: Validate rendered TOML without executing tests.

**Key Interactions**:
- `validate_toml(content)` - Comprehensive validation
- `check_required_keys(content)` - Detect missing sections
- `validate_services(content)` - Check service configurations
- `validate_steps(content)` - Verify step definitions

**Use Cases**:
- Verify validation called after rendering
- Test validation failure prevents execution
- Check missing key detection
- Validate warnings don't block execution

### 5. MockDiffEngine
**File**: `diff_engine_mock.md`

**Purpose**: Compare OTEL traces for auto-baselining behavioral verification.

**Key Interactions**:
- `compare_traces(baseline, current)` - Structural comparison
- `format_diff(comparison, format)` - Generate diff reports
- `calculate_similarity(comparison)` - Quantify differences
- `has_significant_changes(comparison, threshold)` - Threshold detection

**Use Cases**:
- Verify trace comparison after test execution
- Test added/removed span detection
- Validate similarity scoring
- Check multiple output format support

## London School TDD Principles

### 1. Interaction-Focused Testing
Tests verify **how objects collaborate**, not their internal state:

```rust
// ❌ WRONG: State-based testing
assert_eq!(watcher.internal_watch_list.len(), 2);

// ✅ CORRECT: Interaction-based testing
assert!(mock_watcher.verify_watched(Path::new("template.toml")));
assert!(mock_watcher.verify_watched(Path::new("config.toml")));
```

### 2. Mock-Driven Design
Mocks define contracts that drive implementation design:

```rust
// Mock defines the contract
trait MockFileWatcher {
    fn watch(&self, path: &Path) -> Result<()>;
    fn set_debounce(&self, duration: Duration) -> Result<()>;
}

// Implementation must satisfy the contract
impl FileWatcher for NotifyWatcher {
    fn watch(&self, path: &Path) -> Result<()> {
        // Real implementation
    }
}
```

### 3. Outside-In Development
Start with high-level behavior, work down to details:

```
1. Test: User triggers live reload
2. Mock: FileWatcher detects change
3. Mock: TemplateRenderer re-renders
4. Mock: ChangeDetector validates hash
5. Mock: DryRunValidator checks TOML
6. Implement: Actual components satisfying contracts
```

### 4. Behavior Verification
Focus on method calls, arguments, and sequences:

```rust
// Verify method called
assert!(mock.verify_render_called());

// Verify arguments
assert!(mock.verify_context_contains("test_name"));

// Verify sequence
assert!(hash_computed_before_change_checked());
```

## Mock Usage Patterns

### Pattern 1: Single Mock Test

```rust
#[tokio::test]
async fn test_watcher_registers_path() -> Result<()> {
    // Arrange
    let mock_watcher = Arc::new(TestFileWatcher::new());
    let orchestrator = LiveReloadOrchestrator::new(mock_watcher.clone());

    // Act
    orchestrator.watch_template("test.toml.tera").await?;

    // Assert
    assert!(mock_watcher.verify_watched(Path::new("test.toml.tera")));

    Ok(())
}
```

### Pattern 2: Multiple Mock Coordination

```rust
#[tokio::test]
async fn test_complete_live_reload_workflow() -> Result<()> {
    // Arrange - All mocks
    let mock_watcher = Arc::new(TestFileWatcher::new());
    let mock_renderer = Arc::new(TestTemplateRenderer::new());
    let mock_detector = Arc::new(TestChangeDetector::new());
    let mock_validator = Arc::new(TestDryRunValidator::new());

    let orchestrator = LiveReloadOrchestrator::new(
        mock_watcher.clone(),
        mock_renderer.clone(),
        mock_detector.clone(),
        mock_validator.clone(),
    );

    // Act - Trigger workflow
    orchestrator.start().await?;
    mock_watcher.trigger_change(Path::new("test.toml.tera"))?;

    // Assert - Verify all collaborator interactions
    assert!(mock_watcher.verify_watched(Path::new("test.toml.tera")));
    assert!(mock_detector.verify_change_detected(Path::new("test.toml.tera")));
    assert!(mock_renderer.verify_cache_cleared("test.toml.tera"));
    assert!(mock_renderer.verify_rendered("test.toml.tera"));
    assert!(mock_validator.verify_validation_called());

    Ok(())
}
```

### Pattern 3: Error Scenario Testing

```rust
#[tokio::test]
async fn test_handles_rendering_error_gracefully() -> Result<()> {
    // Arrange
    let mock_renderer = Arc::new(TestTemplateRenderer::new());
    mock_renderer.configure_error(
        "broken.toml.tera",
        "Syntax error at line 5".to_string(),
    );

    let orchestrator = LiveReloadOrchestrator::new(mock_renderer.clone());

    // Act
    let result = orchestrator.render("broken.toml.tera").await;

    // Assert - Verify error handling
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Syntax error"));

    Ok(())
}
```

### Pattern 4: Interaction Sequence Verification

```rust
#[tokio::test]
async fn test_operations_occur_in_correct_order() -> Result<()> {
    // Arrange
    let mock_renderer = Arc::new(TestTemplateRenderer::new());
    let mock_detector = Arc::new(TestChangeDetector::new());

    // Act
    orchestrator.reload("test.toml.tera").await?;

    // Assert - Verify order
    let render_time = mock_renderer.last_render_call("test.toml.tera")
        .unwrap()
        .timestamp;
    let detect_time = mock_detector.last_change_check("test.toml.tera")
        .unwrap()
        .timestamp;

    assert!(detect_time < render_time, "Detection should occur before rendering");

    Ok(())
}
```

## Mock Configuration

### Configuration Methods

Each mock provides configuration methods for test setup:

```rust
// Configure success scenarios
mock.configure_response("template", "output");
mock.configure_hash("file", "hash_value");
mock.configure_validation_pass("pattern");

// Configure error scenarios
mock.configure_error("template", "Error message");
mock.configure_watch_error(path, "Cannot watch");
mock.configure_validation_error("pattern", error);

// Configure behavioral scenarios
mock.simulate_file_change(path, new_hash);
mock.simulate_no_change(path);
mock.set_default_similarity(0.95);
```

### Verification Methods

Each mock provides verification methods for assertions:

```rust
// Verify method calls
assert!(mock.verify_rendered("template"));
assert!(mock.verify_watched(path));
assert!(mock.verify_validation_called());

// Verify call counts
assert_eq!(mock.render_call_count("template"), 2);
assert_eq!(mock.watch_call_count(), 3);

// Verify arguments
assert!(mock.verify_context_contains("template", "key"));
assert!(mock.verify_debounce_set(Duration::from_millis(100)));

// Get detailed results
let last_call = mock.last_render_call("template");
let report = mock.last_validation_report();
```

## Integration Testing Strategy

### Level 1: Unit Tests (Single Mock)
Test individual components in isolation with one mock:

```rust
test_watcher_registers_paths()
test_renderer_clears_cache()
test_detector_computes_hash()
```

### Level 2: Integration Tests (Multiple Mocks)
Test component interactions with multiple mocks:

```rust
test_live_reload_workflow()
test_auto_baseline_comparison()
test_dry_run_validation()
```

### Level 3: End-to-End Tests (Real Components)
Test complete system with real implementations:

```rust
test_actual_file_watching()
test_real_template_rendering()
test_full_test_execution()
```

## Testing Best Practices

### 1. Arrange-Act-Assert Pattern
Always structure tests with clear phases:

```rust
#[tokio::test]
async fn test_example() -> Result<()> {
    // Arrange: Setup mocks and test data
    let mock = Arc::new(TestMock::new());
    mock.configure_response("input", "output");

    // Act: Perform operation under test
    let result = system.operation("input").await?;

    // Assert: Verify interactions and results
    assert!(mock.verify_called());
    assert_eq!(result, "expected");

    Ok(())
}
```

### 2. Test One Thing
Each test should verify one specific behavior:

```rust
// ✅ GOOD: Focused test
test_watcher_registers_template_path()

// ❌ BAD: Multiple concerns
test_watcher_and_renderer_and_validator()
```

### 3. Descriptive Test Names
Names should explain what is being tested:

```rust
// ✅ GOOD: Clear intent
test_live_reload_invalidates_cache_on_template_change()

// ❌ BAD: Unclear purpose
test_live_reload()
```

### 4. Independent Tests
Tests should not depend on each other:

```rust
// Each test creates fresh mocks
let mock = Arc::new(TestMock::new());

// Or use reset between tests
mock.reset();
```

## Implementation Workflow

### Step 1: Write Failing Test
```rust
#[tokio::test]
async fn test_live_reload_watches_template() -> Result<()> {
    let mock_watcher = Arc::new(TestFileWatcher::new());
    let live_reload = LiveReloadOrchestrator::new(mock_watcher.clone());

    live_reload.start("test.toml.tera").await?;

    assert!(mock_watcher.verify_watched(Path::new("test.toml.tera")));
    Ok(())
}
```

### Step 2: Implement Minimum to Pass
```rust
impl LiveReloadOrchestrator {
    pub async fn start(&self, template: &str) -> Result<()> {
        self.watcher.watch(Path::new(template))?;
        Ok(())
    }
}
```

### Step 3: Refactor
```rust
impl LiveReloadOrchestrator {
    pub async fn start(&self, template: &str) -> Result<()> {
        self.validate_template_path(template)?;
        self.watcher.watch(Path::new(template))?;
        self.initialize_state().await?;
        Ok(())
    }
}
```

## Next Steps

1. **Implement Real Components**: Use mocks as contracts for implementation
2. **Integration Tests**: Combine real components with remaining mocks
3. **End-to-End Tests**: Replace all mocks with real implementations
4. **Performance Tests**: Use mocks to simulate various scenarios
5. **Documentation**: Keep mocks updated as contracts evolve

## Resources

- **London School TDD**: Focus on behavior and interactions
- **Test Doubles**: Mocks, stubs, spies, fakes explained
- **Contract Testing**: Using mocks to define interfaces
- **Rust Testing**: tokio::test, Arc, Mutex patterns

## Contact

For questions about mock contracts or TDD approach:
- Review individual mock documentation files
- See test examples in each mock file
- Check interaction patterns section
