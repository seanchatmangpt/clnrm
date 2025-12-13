# Expert-Level Testing Patterns - Multi-Step Workflow

## Purpose

This command guides agents through implementing expert-level testing patterns that catch 80% of production bugs. It breaks down complex testing scenarios into clear, sequential steps with examples and validation checkpoints.

## Workflow Overview

```
Step 1: Identify Test Type → Step 2: Choose Pattern → Step 3: Implement Test → Step 4: Verify Coverage → Step 5: Validate Quality
```

## Documentation Reference

For complete testing documentation, see:
- **[Chicago TDD Guide](../../docs/testing/chicago-tdd-guide.md)** - Quick start with verified examples and comprehensive testing guide

## Core Principle: 80/20 Rule

**Expert testing focuses on the 20% of test cases that catch 80% of bugs**:
- Error paths (not just happy path)
- Boundary conditions (not just normal values)
- Resource cleanup (not just normal execution)
- Concurrency (not just single-threaded)
- Real dependencies (not just mocks)

## Step-by-Step Pattern Implementation

### Pattern 1: Error Path Testing (Critical - 80% of bugs)

#### Step 1.1: Identify Error Scenarios

**Action**: List all possible error conditions for the function/feature.

**Questions to ask**:
- What inputs cause errors?
- What error variants exist?
- Can errors be recovered from?
- Are errors properly propagated?

**Example**: For `parse_timeout(input: &str) -> Result<Duration, CleanroomError>`
- Empty input → `CleanroomError::EmptyInput`
- Invalid format → `CleanroomError::InvalidFormat`
- Overflow → `CleanroomError::Overflow`
- Edge cases: `"-0"`, `" 60s "`, etc.

#### Step 1.2: Create Test Cases

**Action**: Create test cases for each error scenario.

```rust
use chicago_tdd_tools::prelude::*;

test!(test_parse_timeout_all_error_paths, {
    // Arrange: Test all error variants
    let test_cases = vec![
        ("", CleanroomError::EmptyInput),
        ("abc", CleanroomError::InvalidFormat),
        ("999999999999999999999s", CleanroomError::Overflow),
        ("-0s", CleanroomError::InvalidFormat), // Edge case
        (" 60s ", CleanroomError::InvalidFormat), // Whitespace
    ];
    
    // Act & Assert: Verify each error path
    for (input, expected_error) in test_cases {
        let result = parse_timeout(input);
        assert_err!(&result, format!("Should fail for input: {}", input));
        match result {
            Err(e) => assert_eq!(e, expected_error, "Error variant mismatch"),
            Ok(_) => panic!("Expected error for input: {}", input),
        }
    }
});
```

#### Step 1.3: Test Error Recovery

**Action**: Verify system can recover from errors.

```rust
test!(test_error_recovery, {
    // Arrange: Create parser
    let mut parser = TestConfigParser::new();
    
    // Act: Cause error
    assert_err!(&parser.parse("invalid"), "Should fail for invalid input");
    
    // Assert: Parser should still be usable after error
    assert_ok!(&parser.parse("60s"), "Parser should recover from error");
});
```

#### Step 1.4: Verify Coverage

**Checklist**:
- [ ] All error variants tested
- [ ] Error messages verified
- [ ] Error recovery tested
- [ ] Edge cases covered

**Reference**: See [Chicago TDD Guide](../../docs/testing/chicago-tdd-guide.md) for best practices

---

### Pattern 2: Boundary Condition Testing

#### Step 2.1: Identify Boundaries

**Action**: List all boundary conditions.

**Common boundaries**:
- Empty collections
- Single item
- Maximum size
- Zero values
- Negative values (if applicable)
- Minimum/maximum ranges

#### Step 2.2: Create Boundary Tests

**Action**: Test each boundary condition.

```rust
use chicago_tdd_tools::prelude::*;

test!(test_step_collection_boundaries, {
    // Arrange: Test empty collection
    let empty: Vec<TestStep> = vec![];
    assert_eq!(execute_steps(&empty).unwrap(), TestResults::empty(), "Empty collection should return empty results");
    
    // Arrange: Test single item
    let single = vec![TestStep::new("step1")];
    assert_ok!(&execute_steps(&single), "Single step should work");
    
    // Arrange: Test max capacity (avoid OOM in test)
    // Note: Using reasonable size for tests - adjust based on your needs
    let max_size = vec![TestStep::new("step"); 10000];
    let result = execute_steps(&max_size);
    assert_ok!(&result, "Should handle large collections");
    
    // Arrange: Test zero values
    let zeros = vec![TestStep::new("step"); 100];
    assert_ok!(&execute_steps(&zeros), "Zero values should work");
    
    // Arrange: Test negative values (if applicable)
    let negatives = vec![TestStep::invalid()];
    let result = execute_steps(&negatives);
    match result {
        Ok(_) => panic!("Should fail for invalid steps"),
        Err(e) => assert!(matches!(e, CleanroomError::InvalidStep)),
    }
});
```

#### Step 2.3: Verify Coverage

**Checklist**:
- [ ] Empty collection tested
- [ ] Single item tested
- [ ] Maximum size tested (safely)
- [ ] Zero values tested
- [ ] Negative values tested (if applicable)

---

### Pattern 3: Resource Cleanup Testing

#### Step 3.1: Identify Resources

**Action**: List all resources that need cleanup.

**Common resources**:
- Container handles
- Network connections
- File handles
- Memory allocations
- Locks/mutexes

#### Step 3.2: Test Normal Cleanup

**Action**: Verify resources are cleaned up in normal execution.

```rust
use chicago_tdd_tools::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

struct ContainerResource {
    id: usize,
}

impl Drop for ContainerResource {
    fn drop(&mut self) {
        DROP_COUNT.fetch_add(1, Ordering::SeqCst);
    }
}

test!(test_container_cleanup_normal_path, {
    // Arrange: Reset counter
    DROP_COUNT.store(0, Ordering::SeqCst);
    
    // Act: Create and drop resource
    {
        let resource = ContainerResource { id: 1 };
        // Resource should drop here
    }
    
    // Assert: Verify cleanup
    assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 1, "Resource should be dropped");
});
```

#### Step 3.3: Test Error Path Cleanup

**Action**: Verify resources are cleaned up even when errors occur.

```rust
test!(test_container_cleanup_error_path, {
    // Arrange: Reset counter
    DROP_COUNT.store(0, Ordering::SeqCst);
    
    // Act: Create resource, then error
    let result: Result<(), String> = (|| {
        let resource = ContainerResource { id: 2 };
        return Err("error".to_string()); // Error path
        // Resource should still drop
    })();
    
    // Assert: Verify cleanup happened
    assert_err!(&result, "Should return error");
    assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 1, "Resource should drop even in error path");
});
```

#### Step 3.4: Test Panic Safety

**Action**: Verify resources are cleaned up even on panic.

```rust
test!(test_container_cleanup_panic_safety, {
    // Arrange: Reset counter
    DROP_COUNT.store(0, Ordering::SeqCst);
    
    // Act: Create resource, then panic
    let result = std::panic::catch_unwind(|| {
        let resource = ContainerResource { id: 3 };
        panic!("test panic");
        // Resource should still drop
    });
    
    // Assert: Verify cleanup happened
    assert!(result.is_err(), "Should catch panic");
    assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 1, "Resource should drop even on panic");
});
```

#### Step 3.5: Verify Coverage

**Checklist**:
- [ ] Normal cleanup tested
- [ ] Error path cleanup tested
- [ ] Panic safety tested
- [ ] Double-drop safety verified (if applicable)

---

### Pattern 4: Concurrency Testing

#### Step 4.1: Identify Concurrency Scenarios

**Action**: List concurrent access patterns.

**Common scenarios**:
- Multiple threads accessing shared state
- Race conditions
- Deadlocks
- Send/Sync bounds

#### Step 4.2: Test Concurrent Access

**Action**: Create concurrent test.

```rust
use chicago_tdd_tools::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread;

test!(test_concurrent_container_access, {
    // Arrange: Shared state
    let container_pool = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    
    // Act: Spawn multiple threads
    for _ in 0..10 {
        let pool = Arc::clone(&container_pool);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let mut value = pool.lock().unwrap();
                *value += 1;
            }
        });
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Assert: Verify final state
    let final_value = container_pool.lock().unwrap();
    assert_eq!(*final_value, 1000, "All increments should be applied");
});
```

#### Step 4.3: Test Send/Sync Bounds

**Action**: Verify concurrency safety.

```rust
use std::sync::{Arc, Mutex};

test!(test_container_send_sync_bounds, {
    // Arrange: Type that should be Send + Sync
    let container = Arc::new(Mutex::new(ContainerHandle::new()));
    
    // Assert: Verify Send + Sync bounds
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<Arc<Mutex<ContainerHandle>>>();
    assert_sync::<Arc<Mutex<ContainerHandle>>>();
});
```

#### Step 4.4: Verify Coverage

**Checklist**:
- [ ] Concurrent access tested
- [ ] Race conditions tested
- [ ] Send/Sync bounds verified
- [ ] Deadlock prevention tested (if applicable)

---

## Summary

Expert-level testing focuses on the **80/20 rule**: Test the 20% of cases that cause 80% of bugs:
- Error paths (not just happy path)
- Boundary conditions (not just normal values)
- Resource cleanup (not just normal execution)
- Concurrency (not just single-threaded)
- Real dependencies (not just mocks)

**Remember**: "Never trust the text, only trust test results" - especially for error paths and edge cases.

## Documentation References

- **[Chicago TDD Guide](../../docs/testing/chicago-tdd-guide.md)** - Complete testing guide with patterns
- **[Build System Practices](../rules/build-system-practices.mdc)** - Build commands
- **[Chicago TDD Standards](../rules/chicago-tdd-standards.mdc)** - Testing standards

