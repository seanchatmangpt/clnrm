# v1.0.1 Release Validation Suite

## Overview

This directory contains the **Rosetta Stone Test Suite** for v1.0.1 release validation. It provides comprehensive, hermetic, deterministic testing to give **100% confidence** before shipping.

## Philosophy

### Hermetic Isolation
Every test runs in a **fresh container** with:
- No state leakage between runs
- Complete container cleanup
- Network isolation (except Homebrew fetch)
- No host filesystem pollution

### Determinism (kcura 5-Iteration Pattern)
All tests must produce **bit-identical results** across 5 iterations:
```rust
const ITERATIONS: usize = 5;
for i in 0..ITERATIONS {
    let result = run_test().await?;
    let hash = calculate_hash(&normalize(&result));
    hashes.push(hash);
}
assert!(hashes.windows(2).all(|w| w[0] == w[1]));
```

### OTEL-First Validation
**Proof comes from spans, not exit codes**:
- Root span `clnrm.run` with `result="pass"`
- Step spans prove actual execution
- Graph topology validates sequencing
- Span counts detect duplication/spoofing

## Test Suite Files

### Master Orchestration
- **`rosetta_stone_suite.clnrm.toml`**: Master suite orchestrating all 5 scenarios

### Test Scenarios
1. **`scenario_01_minimal.clnrm.toml`**: Minimal validation (smoke test)
2. **`scenario_02_full_surface.clnrm.toml`**: All self-test suites
3. **`scenario_03_otlp.clnrm.toml`**: OTLP collector integration
4. **`scenario_04_hermetic.clnrm.toml`**: Isolation verification
5. **`scenario_05_determinism.clnrm.toml`**: 5-iteration hash validation

## Running the Suite

### Quick Start (TOML-Based)

```bash
# Run the complete suite
clnrm run tests/v1.0.1_release/rosetta_stone_suite.clnrm.toml

# Run individual scenarios
clnrm run tests/v1.0.1_release/scenario_01_minimal.clnrm.toml
clnrm run tests/v1.0.1_release/scenario_02_full_surface.clnrm.toml
clnrm run tests/v1.0.1_release/scenario_03_otlp.clnrm.toml
clnrm run tests/v1.0.1_release/scenario_04_hermetic.clnrm.toml
clnrm run tests/v1.0.1_release/scenario_05_determinism.clnrm.toml
```

### Rust Test Runner (Recommended)

```bash
# Run determinism validation (5 iterations)
cargo test --test v1_release_confidence test_scenario_01_minimal_is_deterministic

# Run full surface validation
cargo test --test v1_release_confidence test_scenario_02_full_surface_validation

# Run hermetic isolation verification
cargo test --test v1_release_confidence test_scenario_04_hermetic_isolation

# Run complete release confidence suite
cargo test --test v1_release_confidence test_full_release_confidence_suite

# Generate release confidence report
cargo test --test v1_release_confidence -- --nocapture > release_confidence.txt
```

## Scenario Details

### Scenario 01: Minimal Validation
**Purpose**: Smoke test for basic Homebrew installation

**What it tests**:
- Homebrew tap and install
- Basic self-test suite
- Stdout OTEL exporter
- Root and step span validation

**Expected spans**:
- `clnrm.run` (root)
- `clnrm.step:hello_world` (execution proof)

**Timeout**: 10 minutes
**Weight**: 1.0

### Scenario 02: Full Surface Validation
**Purpose**: Comprehensive validation of all framework capabilities

**What it tests**:
- All self-test suites (basic, container, service, otel)
- Complete span coverage
- Plugin registry validation
- Service lifecycle validation

**Expected spans**: 10+ spans across all suites

**Timeout**: 20 minutes
**Weight**: 1.5

### Scenario 03: OTLP Collector Integration
**Purpose**: End-to-end telemetry pipeline validation

**What it tests**:
- OTLP HTTP exporter
- Jaeger collector integration
- Span export verification
- Network communication with collector

**Infrastructure**: Jaeger all-in-one container

**Expected spans**: Includes `clnrm.otel.export`

**Timeout**: 15 minutes
**Weight**: 1.0

### Scenario 04: Hermetic Isolation Verification
**Purpose**: Detect state leakage across sequential runs

**What it tests**:
- 3 sequential runs in fresh containers
- Unique container IDs per run
- No persistent state
- No cache hits
- No shared networks

**Validation**:
- Each run completely independent
- No `cache.hit` or `previous_run` attributes
- Container cleanup verification

**Timeout**: 15 minutes
**Weight**: 2.0

### Scenario 05: Determinism Validation
**Purpose**: Verify bit-identical results across iterations

**What it tests**:
- 5 iterations of identical test
- Normalized span graph hashing
- Hash comparison across iterations
- Deterministic configuration enforcement

**Normalization**: Excludes timestamps, container IDs, span IDs, trace IDs, UUIDs

**Expected**: All 5 hashes identical

**Timeout**: 20 minutes
**Weight**: 2.5

## Validation Criteria

### Span Validations
Every scenario must have:
- Root span `clnrm.run` with `result="pass"`
- Step span(s) with lifecycle events
- Acyclic graph topology
- Correct parent-child edges
- Expected span counts
- Zero error spans
- All spans with OK status

### Hermeticity Validations
Every scenario must have:
- Fresh container per run
- No state leakage detected
- No forbidden attributes (cache hits, previous runs)
- Unique container IDs
- Complete cleanup verification

### Determinism Validations
Scenario 05 must have:
- 5 iterations executed
- 5 normalized span graphs generated
- 5 SHA-256 hashes calculated
- All 5 hashes identical
- Tolerance: 0ms (perfect determinism)

## Pass Criteria for v1.0.1 Release

**ALL must be true**:
- [ ] All 5 scenarios pass
- [ ] Determinism test shows 5/5 identical hashes
- [ ] Zero hermetic isolation violations
- [ ] All span validations pass
- [ ] Span graphs are acyclic
- [ ] `errors_total == 0` in all runs
- [ ] No container leakage detected
- [ ] Homebrew installation succeeds
- [ ] Self-test passes with OTEL validation
- [ ] Report shows 100% confidence

## Confidence Calculation

**Total weight**: 8.0 (sum of all scenario weights)

**Formula**:
```
Confidence = (Σ passed_scenario_weights / total_weight) × 100%
```

**Weight breakdown**:
- Scenario 01: 1.0
- Scenario 02: 1.5
- Scenario 03: 1.0
- Scenario 04: 2.0
- Scenario 05: 2.5

**Release threshold**: 100% (8.0/8.0)

## Release Decision Matrix

| Confidence | Recommendation | Action |
|-----------|----------------|--------|
| 100% | ✅ SHIP v1.0.1 - 100% CONFIDENCE | Release approved |
| 80-99% | ⚠️ REVIEW v1.0.1 - HIGH CONFIDENCE WITH WARNINGS | Review failures |
| < 80% | ❌ BLOCK v1.0.1 - FAILURES DETECTED | Block release |

## Expected Output

### Console Output (Rust Test)
```
╔═══════════════════════════════════════════════════════════════╗
║         v1.0.1 RELEASE CONFIDENCE VALIDATION SUITE            ║
╚═══════════════════════════════════════════════════════════════╝

1️⃣  Scenario 01: Minimal Validation + Determinism
   ✅ Deterministic: 5/5 hashes match

2️⃣  Scenario 02: Full Surface Validation
   ✅ Passed (15 spans)

3️⃣  Scenario 03: OTLP Collector Integration
   ✅ Passed (8 spans)

4️⃣  Scenario 04: Hermetic Isolation Verification
   ✅ Passed (3/3 runs isolated)

5️⃣  Scenario 05: Determinism Validation
   ✅ Passed (5/5 hashes identical)

╔═══════════════════════════════════════════════════════════════╗
║                    RELEASE CONFIDENCE REPORT                  ║
╚═══════════════════════════════════════════════════════════════╝

Version: v1.0.1
Scenarios Tested: 5
Scenarios Passed: 5/5
Confidence: 100.0%

✅ SHIP v1.0.1 - 100% CONFIDENCE

📄 Report saved to: target/v1.0.1_release_confidence.json
```

### JSON Report
**Location**: `target/v1.0.1_release_confidence.json`

```json
{
  "version": "1.0.1",
  "scenarios": [
    {
      "name": "01_minimal_validation",
      "passed": true,
      "span_count": 5,
      "errors": 0,
      "hermetic": true
    }
  ],
  "determinism_results": [
    {
      "scenario": "scenario_01_minimal.clnrm.toml",
      "iterations": 5,
      "hashes": ["abc123...", "abc123...", "abc123...", "abc123...", "abc123..."],
      "is_deterministic": true,
      "confidence": 1.0
    }
  ],
  "total_confidence": 100.0,
  "recommendation": "✅ SHIP v1.0.1 - 100% CONFIDENCE"
}
```

## Troubleshooting

### Scenario Fails
1. Check span validation output
2. Verify Homebrew tap is accessible
3. Ensure Docker/Podman is running
4. Check container resource limits

### Determinism Test Fails
1. Verify `freeze_clock` configuration
2. Check for non-deterministic code paths
3. Ensure `seed = 42` is set
4. Review excluded fields list

### Hermetic Test Fails
1. Check for container reuse
2. Verify cleanup between runs
3. Look for cache hit attributes
4. Check network isolation

### OTLP Test Fails
1. Verify Jaeger container starts
2. Check endpoint configuration
3. Ensure port 4318 is available
4. Review collector logs

## Related Documentation

- **[V1.0.1_ROSETTA_STONE_VALIDATION.md](../../docs/V1.0.1_ROSETTA_STONE_VALIDATION.md)**: Complete validation report
- **[Rosetta Stone Files](../homebrew_selftest/)**: Source TOML specifications
- **[Rust Test Runner](../../crates/clnrm-core/tests/v1_release_confidence.rs)**: Implementation details

## Contributing

To add a new scenario:
1. Create `scenario_XX_name.clnrm.toml` in this directory
2. Add scenario to `rosetta_stone_suite.clnrm.toml`
3. Add test function to `v1_release_confidence.rs`
4. Update this README with scenario details
5. Update weight calculation in confidence report

## License

Part of the clnrm project. See repository LICENSE.
