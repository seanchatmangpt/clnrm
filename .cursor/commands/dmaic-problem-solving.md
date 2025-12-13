# DMAIC Problem Solving - Multi-Step Workflow

## Purpose

This command guides agents through systematic problem-solving using DMAIC methodology for clnrm issues. DMAIC (Define, Measure, Analyze, Improve, Control) is a data-driven approach to solving problems. Experts use DMAIC to avoid jumping to solutions and ensure fixes address root causes.

## Workflow Overview

```
Step 1: Define → Step 2: Measure → Step 3: Analyze → Step 4: Improve → Step 5: Control
```

## Step-by-Step Instructions

### Step 1: Define

**Action**: Clearly define the problem, scope, and success criteria.

#### 1.1: Define the Problem

**Action**: Write a clear problem statement.

**Example problem statement**:
```markdown
## Problem Statement

**What**: Container startup exceeds SLO (3.2s vs ≤ 2s target)
**Where**: `crates/clnrm-core/src/containers/backend.rs` - `start_container()`
**When**: Starting containers with 100+ test steps
**Impact**: Violates SLO, blocks fast iteration, degrades user experience
**Who**: All users running container-based tests
```

#### 1.2: Define Scope

**Action**: Determine what's in scope and what's out of scope.

**Example scope**:
```markdown
## Scope Definition

**In Scope**:
- Optimize container startup performance
- Maintain determinism (100% reproducible)
- Keep memory usage ≤ 500MB

**Out of Scope**:
- Test execution optimization (address separately)
- CLI argument parsing (not related to container startup)
- Other performance improvements (focus on container startup)

**Boundaries**:
- Focus on container startup only
- Solution must not break determinism
- Solution must meet SLOs
```

#### 1.3: Define Success Criteria

**Action**: Define what success looks like.

**Example success criteria**:
```markdown
## Success Criteria

**Primary**:
- Container startup time ≤ 2s for 100+ steps (meets SLO)
- Measured over 100 consecutive runs

**Secondary**:
- Memory usage ≤ 500MB (maintains SLO)
- Determinism maintained (100% reproducible)
- No regressions in other functionality

**Timeline**:
- Fix implemented within 1 day
- Success verified within 1 week
```

---

### Step 2: Measure

**Action**: Collect data about the problem.

#### 2.1: Collect Baseline Data

**Action**: Measure current state.

**Action**: Run measurements

```bash
# Measure container startup time
time cargo make test --test container_startup
# Output: Average 3.2s (exceeds 2s SLO)

# Measure memory usage
cargo make profile --test container_startup
# Output: Peak memory 300MB (within 500MB SLO)

# Measure determinism
for i in {1..100}; do
    cargo make test --test container_startup > run_$i.txt
done
# Verify all outputs identical (determinism maintained)
```

#### 2.2: Analyze Data Patterns

**Action**: Look for patterns in the data.

**Example analysis**:
```markdown
## Data Patterns

**Startup Time**: 3.2s average (exceeds 2s SLO by 60%)
**Timing**: More time spent in container initialization (60%) vs test execution (40%)
**Memory**: 300MB (within SLO, but could be optimized)
**Patterns**: 
- Container initialization is bottleneck
- Test execution is efficient
- Memory usage acceptable
```

---

### Step 3: Analyze

**Action**: Identify root causes using data.

#### 3.1: Root Cause Analysis

**Action**: Use 5 Whys to find root cause.

**Example 5 Whys**:
```markdown
## Root Cause Analysis (5 Whys)

**Problem**: Container startup takes 3.2s (exceeds 2s SLO)

1. **Why did startup take 3.2s?**
   - Container initialization took 1.9s, test execution took 1.3s

2. **Why did container initialization take 1.9s?**
   - Sequential container startup, no parallelization

3. **Why is container startup sequential?**
   - Backend starts containers one at a time

4. **Why can't backend start containers in parallel?**
   - Backend design doesn't support parallel container startup

5. **Why doesn't backend support parallel container startup?**
   - Backend was designed for correctness, not performance (ROOT CAUSE)

**Root Cause**: Backend design prioritizes correctness over performance, missing optimization opportunities
```

#### 3.2: Verify Root Cause

**Action**: Confirm root cause with data.

**Verification**:
- Does fixing root cause prevent the problem?
- Does data support root cause hypothesis?
- Are there other contributing factors?

---

### Step 4: Improve

**Action**: Implement solution that addresses root cause.

#### 4.1: Generate Solutions

**Action**: Brainstorm solutions that address root cause.

**Example solutions**:
```markdown
## Solution Options

**Option 1**: Optimize sequential container startup
- Pros: Simple, maintains current design
- Cons: Limited improvement potential
- Feasibility: High

**Option 2**: Implement container pool with parallel startup
- Pros: High performance, scalable
- Cons: More complex implementation
- Feasibility: Medium

**Option 3**: Hybrid approach (container pool + selective parallelization)
- Pros: Best performance, maintains determinism
- Cons: Most complex
- Feasibility: Medium

**Selected**: Option 3 - Hybrid approach (addresses root cause, high performance, maintains determinism)
```

#### 4.2: Implement Solution

**Action**: Implement chosen solution.

**Implementation steps**:
1. Make code changes
2. Verify compilation: `cargo make check`
3. Run tests: `cargo make test`
4. Verify fix: Measure performance

**Example implementation**:
```rust
// Before (root cause: sequential container startup)
fn start_containers(config: &TestConfig) -> Result<Vec<Container>, CleanroomError> {
    let mut containers = Vec::new();
    for container_config in config.containers.iter() {
        let container = start_container(container_config)?; // Sequential
        containers.push(container);
    }
    Ok(containers)
}

// After (fix: container pool with parallel startup)
fn start_containers(config: &TestConfig) -> Result<Vec<Container>, CleanroomError> {
    config.containers
        .iter()
        .par_bridge() // Parallel processing
        .map(|container_config| start_container(container_config))
        .collect::<Result<Vec<_>, _>>()
}
```

#### 4.3: Verify Improvement

**Action**: Measure improvement against success criteria.

**Verification**:
- Run performance tests: `cargo make slo-check`
- Check startup time: Should be ≤ 2s
- Check memory usage: Should be ≤ 500MB
- Verify determinism: All runs produce identical output

**Success**: If success criteria met, proceed to Step 5. If not, return to Step 3 (Analyze).

---

### Step 5: Control

**Action**: Prevent problem from returning.

#### 5.1: Add Tests

**Action**: Add tests to prevent regression.

**Example**:
```rust
// Add test that would catch performance regression
#[tokio::test]
async fn test_container_startup_meets_slo() -> Result<(), CleanroomError> {
    let start = Instant::now();
    let containers = start_containers(&large_test_config).await?;
    let duration = start.elapsed();
    
    assert!(duration.as_secs_f64() <= 2.0, "Container startup exceeds 2s SLO");
    assert!(containers.len() > 100);
    Ok(())
}
```

#### 5.2: Create Todo List for Solution Implementation

**CRITICAL**: Do NOT write documents or reports. Create todos and execute them.

**Action**: Create 10+ item todo list for implementing solution and prevention measures.

**Example todo list**:
```markdown
## DMAIC Solution Todos (10+ items)

**Solution Implementation**:
- [ ] Implement container pool with parallel startup
- [ ] Add performance benchmarks
- [ ] Verify compilation: `cargo make check`
- [ ] Run tests: `cargo make test`
- [ ] Measure performance: `cargo make slo-check`
- [ ] Verify SLO met: ≤ 2s container startup

**Prevention Measures**:
- [ ] Add test: `test_container_startup_meets_slo` to catch regression
- [ ] Add SLO check to CI: `cargo make slo-check`
- [ ] Add inline comment: Document why container pooling is used
- [ ] Verify test catches pattern: Test fails if SLO exceeded
```

**Execution**:
1. Create todos using `todo_write` tool (10+ items minimum)
2. Execute todos one by one (implement solution, add prevention)
3. Mark todos as completed as work is done
4. Verify each step works before moving to next
5. Continue until all todos complete

**Principle**: Execute solution and prevention, don't document them. Todos track progress, fixes prevent recurrence.

#### 5.3: Establish Controls

**Action**: Set up controls to prevent problem recurrence.

**Controls**:
- **Code review**: Check for performance regressions
- **Tests**: Run SLO checks in CI
- **Monitoring**: Track performance metrics
- **Standards**: Document pattern to avoid

**Example controls** (implement as todos, not markdown):
```markdown
## Controls Todos (10+ items)

**Code Review Controls**:
- [ ] Add checklist item: Performance meets SLOs
- [ ] Add checklist item: Determinism maintained
- [ ] Update code review process to include SLO checks

**Test Strategy Controls**:
- [ ] Add SLO check to CI pipeline: `cargo make slo-check`
- [ ] Configure alert if SLO exceeded
- [ ] Verify alerts work correctly

**Monitoring Controls**:
- [ ] Set up performance tracking dashboard
- [ ] Configure alerts for SLO violations
- [ ] Review performance trends weekly

**Standards Controls**:
- [ ] Add standard: All performance-critical code must meet SLOs
- [ ] Update team documentation with standard
- [ ] Verify standard is followed in code reviews
```

**CRITICAL**: Implement controls as todos and execute them. Don't just document controls - actually implement them.

---

## Integration with Other Commands

- **[Root Cause Analysis](./root-cause-analysis.md)** - Use 5 Whys in Analyze step
- **[Gemba Walk](./gemba-walk.md)** - Go to source in Measure and Analyze steps
- **[Poka-Yoke Design](./poka-yoke-design.md)** - Use type system in Improve step to prevent recurrence
- **[Andon Signals](./andon-signals.md)** - Use tests as signals in Control step

## Expert Insights

**Why this matters**: Jumping to solutions without analysis leads to fixing symptoms, not causes. DMAIC ensures systematic problem-solving for clnrm issues.

**Key principle**: "Data over assumptions" - Use data to drive decisions, not guesses.

**Remember**: Each step builds on the previous. Don't skip steps. Define clearly, measure accurately, analyze deeply, improve systematically, control consistently.

**DMAIC cycle**: This is iterative. If Improve doesn't work, return to Analyze. If problem returns, strengthen Control.

**DfLSS alignment**: When designing solutions in the Improve step, use DfLSS (Design for Lean Six Sigma) principles - address both efficiency (waste elimination) AND quality (defect prevention) from the start. Don't conflate DfLSS with DFSS (Design for Six Sigma) - DFSS only addresses quality, missing critical waste elimination. Using the wrong methodology is itself a root cause. See [Root Cause Analysis - DfLSS vs DFSS](./root-cause-analysis.md#dflss-vs-dfss-critical-distinction) for detailed explanation.

---

## Command Execution Pattern

**CRITICAL**: DMAIC commands must:
1. **Create 10+ item todo list** - Not documents/reports
2. **Execute todos** - Implement solutions and controls, not document them
3. **Verify fixes** - Test that solutions work
4. **Complete todos** - Mark todos as done as work completes

**Principle**: Execute solutions and controls, don't document them. Todos track progress, solutions fix problems.

---

End Command ---

