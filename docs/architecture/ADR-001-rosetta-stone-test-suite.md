# ADR-001: Rosetta Stone Test Suite for v1.0.1 Release Validation

## Status

**PROPOSED** - Awaiting implementation validation

**Date**: 2025-01-17

## Context

The clnrm v1.0.1 release requires **100% confidence** before shipping. Traditional testing approaches relying on exit codes and text output are insufficient for hermetic, deterministic validation. We need a comprehensive test suite that:

1. **Proves correctness via telemetry** (OTEL spans), not exit codes
2. **Ensures hermetic isolation** across all test runs
3. **Validates determinism** using the kcura 5-iteration pattern
4. **Prevents false positives** by requiring concrete execution evidence
5. **Validates the complete installation flow** via Homebrew

### Problem Statement

How do we create a test suite that gives absolute confidence to ship v1.0.1 while:
- Avoiding fake green scenarios (empty `Ok(())` implementations)
- Ensuring reproducible results across environments
- Detecting state leakage and container pollution
- Validating real-world installation paths (Homebrew)
- Providing clear pass/fail criteria

### Existing Approaches and Limitations

**Exit Code Testing**:
- ❌ Can be faked with `Ok(())` stubs
- ❌ Doesn't prove actual execution happened
- ❌ No structural validation of behavior

**Log-Based Validation**:
- ❌ Non-deterministic (timestamps, UUIDs)
- ❌ Difficult to parse reliably
- ❌ No graph structure validation

**Manual Testing**:
- ❌ Not reproducible
- ❌ Human error prone
- ❌ Doesn't scale

## Decision

We will implement the **Rosetta Stone Test Suite** for v1.0.1 release validation, based on three canonical TOML files that define the complete Homebrew installation validation pattern.

### Architecture

#### 1. Rosetta Stone Foundation

Three canonical TOML files define the specification:

1. **`tests/homebrew_selftest/clnrm_brew_install_selftest_verbose.clnrm.toml`** (174 lines)
   - Hyper-verbose specification with line-by-line commentary
   - Defines determinism, OTEL, hermeticity patterns
   - Stdout OTEL exporter for zero-infrastructure validation

2. **`examples/integration-tests/homebrew-install-selftest.clnrm.toml`** (163 lines)
   - Structured validation with comprehensive expectations
   - Span graph topology validation
   - Step-by-step installation flow

3. **`tests/readme_examples/install_homebrew.clnrm.toml`** (46 lines)
   - README example validation
   - Simplified installation verification

#### 2. Test Suite Structure

**Location**: `/tests/v1.0.1_release/`

**Files**:
- `rosetta_stone_suite.clnrm.toml` - Master orchestration
- `scenario_01_minimal.clnrm.toml` - Minimal validation
- `scenario_02_full_surface.clnrm.toml` - All self-test suites
- `scenario_03_otlp.clnrm.toml` - OTLP collector integration
- `scenario_04_hermetic.clnrm.toml` - Isolation verification
- `scenario_05_determinism.clnrm.toml` - 5-iteration validation
- `README.md` - Suite documentation

#### 3. Rust Test Runner

**Location**: `/crates/clnrm-core/tests/v1_release_confidence.rs`

**Key Components**:
```rust
// Determinism validation (kcura pattern)
const ITERATIONS: usize = 5;
async fn validate_determinism(test_file: &str) -> Result<DeterminismResult>;

// Span normalization (remove non-deterministic fields)
fn normalize_spans(result: &TestResult) -> Vec<NormalizedSpan>;

// Hash calculation (SHA-256)
fn calculate_hash(spans: &[NormalizedSpan]) -> String;

// Container cleanup (hermetic isolation)
async fn cleanup_test_containers() -> Result<()>;
```

**Test Functions**:
- `test_scenario_01_minimal_is_deterministic()` - 5-iteration validation
- `test_scenario_02_full_surface_validation()` - Comprehensive validation
- `test_scenario_04_hermetic_isolation()` - State leakage detection
- `test_full_release_confidence_suite()` - Complete suite with report

### Key Design Principles

#### Principle 1: OTEL-First Validation

**Proof comes from spans, not exit codes**:
- Root span `clnrm.run` with `result="pass"` proves test passed
- Step spans with lifecycle events prove actual execution
- Graph topology proves correct sequencing
- Span counts detect duplication/spoofing

**Why**: Exit codes can be faked. Spans prove behavior.

#### Principle 2: Hermetic Isolation

**Each test runs in a fresh container**:
- No state leakage between iterations
- Complete container cleanup after each run
- Unique container IDs verified
- No shared networks or volumes

**Validation**:
- Forbidden span attributes: `cache.hit`, `previous_run`, `persistent_state`
- Container ID uniqueness check
- Cleanup verification between runs

**Why**: Hermetic tests are reproducible and don't hide bugs via state.

#### Principle 3: Determinism (kcura 5-Iteration Pattern)

**5 iterations must produce identical results**:
```rust
for i in 0..5 {
    let result = run_test().await?;
    let normalized = normalize_spans(&result);
    let hash = calculate_hash(&normalized);
    hashes.push(hash);
}
assert!(hashes.windows(2).all(|w| w[0] == w[1]));
```

**Normalization Strategy**:
- Exclude: timestamps, container IDs, span IDs, trace IDs, UUIDs, durations
- Preserve: span names, kinds, parent-child edges, events, status, deterministic attributes
- Sort: attributes alphabetically for consistent ordering

**Hash Algorithm**: SHA-256 of JSON-serialized normalized spans

**Why**: Deterministic tests are trustworthy and debuggable.

#### Principle 4: Weight-Based Confidence

**Each scenario has a weight representing criticality**:
- Scenario 01 (Minimal): 1.0
- Scenario 02 (Full Surface): 1.5
- Scenario 03 (OTLP): 1.0
- Scenario 04 (Hermetic): 2.0
- Scenario 05 (Determinism): 2.5
- **Total**: 8.0

**Confidence Calculation**:
```
Confidence = (Σ passed_scenario_weights / total_weight) × 100%
```

**Release Threshold**: 100% (8.0/8.0 - all scenarios must pass)

**Why**: Prioritizes critical validations while allowing future extension.

### Validation Criteria

#### Span Validations (ALL scenarios)
- ✅ Root span `clnrm.run` exists with `result="pass"`
- ✅ Step span `clnrm.step:hello_world` with lifecycle events
- ✅ Parent-child edges: `["clnrm.run", "clnrm.step:hello_world"]`
- ✅ Acyclic graph topology
- ✅ Span counts within expected ranges
- ✅ Zero error spans
- ✅ All spans with OK status

#### Hermeticity Validations (Scenario 04)
- ✅ Unique container IDs across 3 runs
- ✅ No `cache.hit` attributes
- ✅ No `previous_run` attributes
- ✅ Complete container cleanup
- ✅ No shared networks

#### Determinism Validations (Scenario 05)
- ✅ 5 iterations executed
- ✅ 5 normalized span graphs generated
- ✅ 5 SHA-256 hashes calculated
- ✅ All 5 hashes identical
- ✅ Tolerance: 0ms (perfect determinism)

### Test Execution Flow

```bash
# Step 1: Build with OTEL features
cargo build --release --features otel

# Step 2: Install via Homebrew (dogfooding)
brew install --build-from-source .

# Step 3: Run TOML-based suite (optional)
clnrm run tests/v1.0.1_release/rosetta_stone_suite.clnrm.toml

# Step 4: Run Rust test suite (recommended)
cargo test --test v1_release_confidence

# Step 5: Generate release confidence report
cargo test --test v1_release_confidence test_full_release_confidence_suite -- --nocapture
```

### Expected Output

```
╔═══════════════════════════════════════════════════════════════╗
║         v1.0.1 RELEASE CONFIDENCE VALIDATION SUITE            ║
╚═══════════════════════════════════════════════════════════════╝

1️⃣  Scenario 01: Minimal Validation + Determinism
   ✅ Deterministic: 5/5 hashes match

2️⃣  Scenario 02: Full Surface Validation
   ✅ Passed (15 spans)

...

╔═══════════════════════════════════════════════════════════════╗
║                    RELEASE CONFIDENCE REPORT                  ║
╚═══════════════════════════════════════════════════════════════╝

Version: v1.0.1
Scenarios Tested: 5
Scenarios Passed: 5/5
Confidence: 100.0%

✅ SHIP v1.0.1 - 100% CONFIDENCE
```

## Consequences

### Positive

1. **100% Confidence for Release**
   - Comprehensive validation across all framework capabilities
   - No false positives via OTEL-first validation
   - Deterministic results proven via 5-iteration pattern

2. **Hermetic Isolation Guaranteed**
   - Each test runs in fresh container
   - State leakage automatically detected
   - Container cleanup verified

3. **Reproducible Across Environments**
   - Deterministic configuration (`seed=42`, `freeze_clock`)
   - Normalized span hashing
   - Container-based isolation

4. **Real-World Validation**
   - Homebrew installation flow tested
   - Production installation path validated
   - Dogfooding principle enforced

5. **Clear Pass/Fail Criteria**
   - Weight-based confidence calculation
   - Explicit release threshold (100%)
   - Automated report generation

6. **Future Extension Path**
   - New scenarios can be added with weights
   - Suite structure supports growth
   - Patterns reusable for future releases

### Negative

1. **Execution Time**
   - 5 scenarios × determinism iterations = longer CI time
   - Mitigated by: parallel scenario execution (future), caching

2. **Homebrew Dependency**
   - Requires Homebrew tap to be available
   - Mitigated by: mock Homebrew mode (future), offline testing

3. **Container Resource Requirements**
   - Each scenario needs fresh container
   - Mitigated by: resource limits, cleanup policies

4. **Complexity**
   - More complex than simple exit code testing
   - Mitigated by: comprehensive documentation, clear patterns

### Risks and Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Homebrew tap unavailable | High | Low | Mock mode, offline cache |
| Container cleanup fails | Medium | Low | Retry logic, force cleanup |
| Non-determinism in code | High | Low | Strict normalization, frozen clock |
| Long execution time | Medium | High | Parallel execution, scenario caching |
| False negatives | High | Low | Multiple validation layers |

## Alternatives Considered

### Alternative 1: Exit Code Testing Only
**Rejected**: Can be faked with empty `Ok(())` implementations. No structural validation.

### Alternative 2: Log Parsing
**Rejected**: Non-deterministic timestamps, difficult to parse reliably, no graph structure.

### Alternative 3: Manual Testing
**Rejected**: Not reproducible, human error prone, doesn't scale.

### Alternative 4: Single-Iteration Testing
**Rejected**: Doesn't prove determinism. Kcura 5-iteration pattern is industry standard.

### Alternative 5: 100% Unit Test Coverage
**Rejected**: Doesn't validate integration or real-world installation paths.

## Related Documents

- **[V1.0.1_ROSETTA_STONE_VALIDATION.md](../V1.0.1_ROSETTA_STONE_VALIDATION.md)**: Complete validation report
- **[tests/v1.0.1_release/README.md](../../tests/v1.0.1_release/README.md)**: Suite documentation
- **[Rosetta Stone Files](../../tests/homebrew_selftest/)**: Source TOML specifications

## Implementation Checklist

- [x] Create `/tests/v1.0.1_release/` directory
- [x] Implement `rosetta_stone_suite.clnrm.toml`
- [x] Implement `scenario_01_minimal.clnrm.toml`
- [x] Implement `scenario_02_full_surface.clnrm.toml`
- [x] Implement `scenario_03_otlp.clnrm.toml`
- [x] Implement `scenario_04_hermetic.clnrm.toml`
- [x] Implement `scenario_05_determinism.clnrm.toml`
- [x] Create Rust test runner `v1_release_confidence.rs`
- [x] Implement span normalization utilities
- [x] Implement hash calculation utilities
- [x] Create validation report template
- [x] Create suite README documentation
- [ ] Execute test suite and validate results
- [ ] Update validation report with actual results
- [ ] Generate final release confidence report

## Success Metrics

- [ ] All 5 scenarios pass
- [ ] Determinism test shows 5/5 identical hashes
- [ ] Zero hermetic isolation violations
- [ ] Total confidence: 100%
- [ ] Release recommendation: "✅ SHIP v1.0.1 - 100% CONFIDENCE"

## Approval

**Author**: System Architect (Claude Code)
**Reviewers**: Core Team (pending)
**Status**: Awaiting validation execution
**Decision Date**: 2025-01-17

---

**Next Steps**:
1. Execute test suite: `cargo test --test v1_release_confidence`
2. Review results and update validation report
3. Generate final release confidence report
4. Approve/block v1.0.1 release based on 100% confidence threshold
