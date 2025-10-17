# Run Determinism Tests

Verify that clnrm's hermetic isolation guarantees produce deterministic, repeatable test results.

## What Determinism Tests Do

Execute each test **5 times** (kcura standard) and verify:
- Identical output across all runs
- Container execution produces same results
- Service lifecycle is repeatable
- TOML parsing is consistent
- Metrics collection is accurate
- Log output is deterministic (excluding timestamps)

## Why This Matters

Hermetic testing is only valuable if results are **deterministic**. Non-deterministic tests indicate:
- Container state leakage between runs
- Race conditions in service management
- Timestamp/randomness pollution in output
- Broken isolation guarantees

## Running the Tests

```bash
# Run all determinism tests
cargo test --test determinism_test

# Run specific determinism test
cargo test --test determinism_test test_container_execution_is_deterministic

# Run with output
cargo test --test determinism_test -- --nocapture

# Run in release mode for performance
cargo test --release --test determinism_test
```

## Test Categories

The suite includes **10 test functions** covering:

1. **Container Execution** (3 tests)
   - Basic container command execution
   - Multi-command sequences
   - Environment variable handling

2. **Service Lifecycle** (2 tests)
   - Service start/stop cycles
   - Service state transitions

3. **TOML Parsing** (2 tests)
   - Configuration file parsing
   - Complex nested structures

4. **Metrics Collection** (1 test)
   - Performance metric tracking

5. **Backend Operations** (1 test)
   - Backend command execution

6. **Log Output** (1 test)
   - Structured logging consistency

## Pattern Used

Each test follows the **5-iteration verification pattern**:

```rust
const ITERATIONS: usize = 5;
let mut hashes = Vec::new();

for iteration in 0..ITERATIONS {
    let result = run_hermetic_test().await?;
    let normalized = normalize_output(&result);
    let hash = calculate_hash(&normalized);
    hashes.push(hash);
}

// All iterations must produce identical hashes
assert!(hashes.windows(2).all(|w| w[0] == w[1]),
    "Hermetic test results must be deterministic");
```

## Output Normalization

Tests normalize output to exclude:
- Timestamps (ISO8601, Unix, relative)
- Container IDs (dynamic Docker identifiers)
- UUIDs and random identifiers
- Process IDs

This ensures we verify **semantic determinism**, not byte-for-byte identical output.

## Expected Results

✅ **PASS**: All 10 tests complete with 5 identical hashes each
❌ **FAIL**: Any hash mismatch indicates broken hermetic isolation

## Troubleshooting

If tests fail:
1. Check for timestamp/UUID pollution in output
2. Verify container cleanup between runs
3. Ensure no global state mutation
4. Review service lifecycle for race conditions
5. Check for non-deterministic randomness

## Documentation

See `/docs/TESTING.md` and `/docs/DETERMINISM_TESTING_IMPLEMENTATION.md` for complete details.
