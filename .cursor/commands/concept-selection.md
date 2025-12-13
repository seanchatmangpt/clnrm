# Concept Selection - Multi-Step Workflow

## Purpose

This command guides agents through systematic concept selection methods (Pugh Matrix and AHP) to evaluate and select the best design concepts for clnrm features. Concept selection ensures objective, data-driven decisions when choosing between multiple design alternatives. Experts use systematic methods to avoid bias and select concepts that best meet requirements.

## Workflow Overview

```
Step 1: Define Selection Criteria → Step 2: Generate Concepts → Step 3: Pugh Matrix Evaluation → Step 4: AHP Evaluation → Step 5: Select and Verify
```

## Step-by-Step Instructions

### Step 1: Define Selection Criteria

**Action**: Define criteria for evaluating concepts based on clnrm requirements.

#### 1.1: Identify Criteria from Requirements

**Action**: Extract criteria from clnrm requirements and SLOs.

**Criteria sources**:
- **SLOs**: Performance targets (container startup ≤ 2s, test execution ≤ 60s, OpenTelemetry validation ≤ 5s)
- **Determinism**: 100% reproducible test results
- **Type Safety**: Zero-cost abstractions, type-level guarantees
- **Maintainability**: Code quality, test coverage
- **Integration**: Docker and OpenTelemetry integration

**Example criteria identification**:
```markdown
## Selection Criteria

**From SLOs**:
- C1: Container startup speed (≤ 2s for container pool)
- C2: Test execution memory (≤ 500MB)
- C3: OpenTelemetry validation speed (≤ 5s end-to-end)

**From Determinism Requirements**:
- C4: Reproducible test results (100% deterministic)
- C5: Container state validation (deterministic cleanup)

**From Type Safety Requirements**:
- C6: Compile-time guarantees (type-level encodings)
- C7: Zero-cost abstractions (no runtime overhead)

**From Maintainability**:
- C8: Test coverage (80%+ coverage)
- C9: Code complexity (maintainable patterns)
```

#### 1.2: Prioritize Criteria

**Action**: Determine relative importance of criteria.

**Example prioritization**:
```markdown
## Criteria Prioritization

**Must-Have** (Critical):
- C1: Container startup speed (10/10)
- C4: Reproducible test results (10/10)
- C6: Compile-time guarantees (10/10)

**Important** (High priority):
- C2: Test execution memory (8/10)
- C7: Zero-cost abstractions (8/10)
- C8: Test coverage (8/10)

**Nice-to-Have** (Lower priority):
- C3: OpenTelemetry validation speed (6/10)
- C5: Container state validation (7/10)
- C9: Code complexity (7/10)
```

---

### Step 2: Generate Concepts

**Action**: Generate multiple design concept alternatives for clnrm features.

#### 2.1: Concept Generation Methods

**Action**: Use multiple methods to generate diverse concepts.

**Generation methods**:
- **Brainstorming**: Generate many ideas
- **TRIZ**: Use TRIZ principles for innovative concepts
- **Benchmarking**: Learn from existing solutions
- **Prototyping**: Build quick prototypes to explore

**Example concepts for container pooling**:
```markdown
## Design Concepts

**Concept A**: Pre-warmed Container Pool
- **Description**: Maintain pool of pre-started containers
- **Key Features**: Fast container acquisition, memory-efficient
- **Pros**: Low startup time, handles concurrent tests
- **Cons**: More complex implementation

**Concept B**: On-Demand Container Creation
- **Description**: Create containers when needed
- **Key Features**: Simple implementation, no pool overhead
- **Pros**: Simple implementation, no memory overhead
- **Cons**: High startup time for each test

**Concept C**: Hybrid Approach (Pool + On-Demand)
- **Description**: Pre-warmed pool with on-demand fallback
- **Key Features**: Fast for common cases, flexible for edge cases
- **Pros**: Best of both worlds, balanced performance
- **Cons**: More complex implementation
```

---

### Step 3: Pugh Matrix Evaluation

**Action**: Use Pugh Matrix to compare concepts against baseline.

#### 3.1: Select Baseline

**Action**: Choose baseline concept for comparison.

**Example baseline**:
```markdown
## Baseline Selection

**Baseline**: Current On-Demand Container Creation
- **Description**: Current approach - create containers when needed
- **Rationale**: Known performance, serves as reference point
- **Performance**: 2s container startup, 200MB memory
```

#### 3.2: Create Pugh Matrix

**Action**: Create matrix comparing concepts to baseline.

**Example Pugh Matrix**:
```markdown
## Pugh Matrix

| Criterion | Baseline | Concept A (Pool) | Concept B (On-Demand) | Concept C (Hybrid) |
|-----------|----------|------------------|----------------------|-------------------|
| C1: Speed | 0 | + | 0 | S |
| C2: Memory | 0 | - | + | 0 |
| C4: Determinism | 0 | 0 | 0 | 0 |
| C6: Type Safety | 0 | 0 | 0 | 0 |
| C8: Coverage | 0 | - | 0 | - |
| **Net Score** | 0 | -1 | +1 | +1 |
```

---

### Step 4: AHP Evaluation

**Action**: Use Analytic Hierarchy Process (AHP) for detailed evaluation.

#### 4.1: Create Pairwise Comparison Matrix

**Action**: Compare criteria pairwise to determine weights.

**Example pairwise comparison**:
```markdown
## Pairwise Comparison Matrix (Criteria)

| Criterion | C1 | C2 | C4 | C6 | C8 |
|-----------|----|----|----|----|----|
| C1: Speed | 1 | 3 | 1 | 1 | 3 |
| C2: Memory | 1/3 | 1 | 1/3 | 1/3 | 1 |
| C4: Determinism | 1 | 3 | 1 | 1 | 3 |
| C6: Type Safety | 1 | 3 | 1 | 1 | 3 |
| C8: Coverage | 1/3 | 1 | 1/3 | 1/3 | 1 |

**Weights** (normalized):
- C1: Speed: 0.25
- C2: Memory: 0.10
- C4: Determinism: 0.25
- C6: Type Safety: 0.25
- C8: Coverage: 0.10
```

---

### Step 5: Select and Verify

**Action**: Select best concept(s) and verify selection.

#### 5.1: Compare Methods

**Action**: Compare Pugh Matrix and AHP results.

**Example comparison**:
```markdown
## Method Comparison

**Pugh Matrix Ranking**:
1. Concept C (Hybrid): Net +1
2. Concept B (On-Demand): Net +1
3. Concept A (Pool): Net -1

**AHP Ranking**:
1. Concept C (Hybrid): 0.85
2. Concept B (On-Demand): 0.72
3. Concept A (Pool): 0.65

**Consensus**: Both methods favor Concept C (Hybrid)
**Selected**: Concept C (Hybrid) - highest AHP score, good Pugh score
```

#### 5.2: Verify Selection Criteria Met

**Action**: Verify selected concept meets all criteria.

**Example verification**:
```markdown
## Selection Verification

**Selected**: Concept C (Hybrid - Pool + On-Demand)

**Must-Have Criteria**:
- ✅ C1: Speed - Estimated 0.5s for pool (meets ≤ 2s SLO)
- ✅ C4: Determinism - Pool maintains determinism
- ✅ C6: Type Safety - Type-level encodings supported

**Important Criteria**:
- ✅ C2: Memory - Estimated 300MB (meets ≤ 500MB SLO)
- ✅ C7: Zero-cost - Pool abstractions are zero-cost
- ✅ C8: Coverage - Testable architecture

**Conclusion**: Selected concept meets all must-have criteria ✅
```

#### 5.3: Create Todo List for Concept Implementation

**CRITICAL**: Do NOT write documents or reports. Create todos and execute them.

**Action**: Create 10+ item todo list for implementing selected concept.

**Example todo list**:
```markdown
## Concept Implementation Todos (10+ items)

**Concept C (Hybrid) Implementation**:
- [ ] Design container pool architecture
- [ ] Implement pre-warmed container pool
- [ ] Design on-demand fallback mechanism
- [ ] Implement fallback for edge cases
- [ ] Add type-level encodings for container states
- [ ] Integrate pool with test execution
- [ ] Add error handling and recovery
- [ ] Write comprehensive tests
- [ ] Verify compilation: `cargo make check`
- [ ] Run tests: `cargo make test`
- [ ] Measure performance: `cargo make slo-check`
- [ ] Verify SLOs met: ≤ 2s startup, ≤ 500MB memory
- [ ] Document implementation
```

**Execution**:
1. Create todos using `todo_write` tool (10+ items minimum)
2. Execute todos one by one (implement concept)
3. Mark todos as completed as work is done
4. Verify each step works before moving to next
5. Continue until all todos complete

**Principle**: Execute concept implementation, don't document it. Todos track progress, implementation delivers value.

---

## Integration with Other Commands

- **[Voice of Customer (QFD)](./voice-of-customer-qfd.md)** - Use to identify selection criteria from customer needs
- **[TRIZ Problem Solving](./triz-problem-solving.md)** - Use to generate innovative concepts for evaluation
- **[DMEDI Design Process](./dmedi-design-process.md)** - Use concept selection in Explore phase
- **[Robust Design](./robust-design.md)** - Use to evaluate concepts for robustness
- **[FMEA](./fmea.md)** - Use to evaluate concepts for failure modes

## Expert Insights

**Why this matters**: Systematic concept selection ensures objective, data-driven decisions. Avoids bias and selects concepts that best meet clnrm requirements.

**Key principle**: "Data over opinion" - Use systematic methods (Pugh Matrix, AHP) to make objective decisions, not subjective preferences.

**Remember**: 
- **Multiple concepts**: Generate many concepts before selecting
- **Systematic evaluation**: Use structured methods, not gut feel
- **Multiple methods**: Use both Pugh Matrix and AHP for validation
- **Verify selection**: Ensure selected concept meets all criteria

**DfLSS alignment**: Concept selection supports DfLSS (Design for Lean Six Sigma) by ensuring selected concepts address both efficiency (waste elimination) AND quality (defect prevention) - evaluating concepts against both efficiency and quality criteria. Don't conflate DfLSS with DFSS (Design for Six Sigma) - DFSS only addresses quality, missing critical waste elimination. See [Root Cause Analysis - DfLSS vs DFSS](./root-cause-analysis.md#dflss-vs-dfss-critical-distinction) for why conflating DfLSS with DFSS is a huge error.

---

End Command ---

