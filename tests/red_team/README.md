# Red Team: Fake-Green Detection System

**Mission**: Detect and prevent "fake-green" test attacks where malicious or buggy wrapper scripts report test success without actually executing containers.

## Current Implementation Status

✅ **IMPLEMENTED**:
1. Schema changes (service, run, args, artifacts, wait_for_span)
2. OTEL stdout NDJSON exporter (`Export::StdoutNdjson`)
3. StdoutSpanParser for extracting spans from mixed output
4. All 8 validators (span, graph, counts, window, order, status, hermeticity)
5. First-failing-rule reporting infrastructure

⚠️ **REMAINING WORK**:
1. Wire scenario execution to use `run=` command override
2. Wire `wait_for_span` service readiness polling
3. Create `clnrm self-test` command that emits proper spans
4. Create three attack vector test files
5. Create legitimate baseline test file
6. End-to-end verification

## Test File Summary

- **`clnrm_redteam_catch_verbose.clnrm.toml`** - Main comprehensive red-team test (8 validation layers)
- **`attack_vector_a_no_execution.clnrm.toml`** - Tests detection of no execution (exit 0 without running)
- **`attack_vector_b_missing_edges.clnrm.toml`** - Tests detection of missing parent-child relationships
- **`attack_vector_c_wrong_counts.clnrm.toml`** - Tests detection of insufficient lifecycle events
- **`legitimate_baseline.clnrm.toml`** - Legitimate test that should pass all validations

## Running Tests

```bash
# Should FAIL - attack detected
clnrm run -f tests/red_team/attack_vector_a_no_execution.clnrm.toml

# Should PASS - legitimate execution
clnrm run -f tests/red_team/legitimate_baseline.clnrm.toml
```

See full threat model and implementation details above.
