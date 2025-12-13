# Eliminate Mura (Unevenness) - Multi-Step Workflow

## Purpose

This command guides agents to eliminate unevenness (Mura) in clnrm code quality, patterns, and style. Mura refers to variability or inconsistency. Experts maintain consistent quality and patterns across the codebase.

## Workflow Overview

```
Step 1: Identify Mura → Step 2: Measure Variability → Step 3: Standardize → Step 4: Apply Consistently → Step 5: Control
```

## Step-by-Step Instructions

### Step 1: Identify Mura (Unevenness)

**Action**: Find inconsistencies in clnrm code quality, patterns, and style.

**Types of Mura to identify**:
1. **Code style inconsistency** - Different formatting, naming conventions
2. **Pattern inconsistency** - Same problem solved differently
3. **Quality inconsistency** - Different quality levels
4. **Complexity inconsistency** - Different complexity levels for similar problems
5. **Documentation inconsistency** - Different documentation levels

**Example Mura inventory**:
```markdown
## Mura Inventory

### Code Style Inconsistency
- [ ] `crates/clnrm-core/src/config/parser.rs` uses `snake_case` for functions
- [ ] `crates/clnrm-core/src/scenarios/executor.rs` uses inconsistent formatting

### Pattern Inconsistency
- [ ] `crates/clnrm-core/src/containers/backend.rs` uses `Result<T, CleanroomError>` for errors
- [ ] `crates/clnrm-core/src/scenarios/executor.rs` uses `Result<T, CleanroomError>` for errors
- [ ] Different error handling patterns for similar operations

### Quality Inconsistency
- [ ] `crates/clnrm-core/src/containers/` has 90% test coverage
- [ ] `crates/clnrm-core/src/scenarios/` has 20% test coverage
- [ ] Uneven test coverage
```

---

### Step 2: Measure Variability

**Action**: Quantify the inconsistency.

**Metrics to measure**:
- **Style consistency** - How many style violations?
- **Pattern consistency** - How many different patterns for same problem?
- **Quality consistency** - What's the quality variance?

**Action**: Measure variability

```bash
# Measure style consistency
cargo make fmt --check
# Count violations

# Measure test coverage consistency
cargo make test -- --test-threads 1 --nocapture 2>&1 | grep "test result"
# Compare coverage across modules
```

---

### Step 3: Standardize

**Action**: Establish consistent standards.

#### 3.1: Define Standards

**Action**: Define what the standard should be.

**Example standards**:
```markdown
## Standards Definition

### Style Standards
- **Naming**: Use `snake_case` for functions (Rust convention)
- **Formatting**: Use `cargo make fmt` (enforced)
- **Imports**: Alphabetical, grouped by std/external/local

### Pattern Standards
- **Error handling**: Use `Result<T, E>` with project error types
- **Validation**: Use type-level validation (Poka-yoke) when possible
- **Testing**: Minimum 80% test coverage for all modules

### Quality Standards
- **Test coverage**: Minimum 80% for all modules
- **Error handling**: All fallible operations return `Result`
- **Documentation**: All public APIs documented
```

---

### Step 4: Apply Consistently

**Action**: Apply standards across codebase.

#### 4.1: Apply Style Standards

**Action**: Standardize code style.

**Steps**:
1. Run formatter: `cargo make fmt`
2. Fix naming violations
3. Fix import ordering
4. Verify: `cargo make check`

#### 4.2: Apply Pattern Standards

**Action**: Standardize patterns.

**Steps**:
1. Identify inconsistent patterns
2. Refactor to match standard pattern
3. Verify functionality: `cargo make test`
4. Verify no regressions

#### 4.3: Apply Quality Standards

**Action**: Bring all code to minimum quality level.

**Steps**:
1. Identify low-quality code
2. Add tests to reach coverage threshold
3. Add error handling where missing
4. Add documentation where missing
5. Verify: `cargo make test`

---

### Step 5: Control (Prevent Inconsistency)

**Action**: Establish controls to prevent Mura from returning.

#### 5.1: Automated Checks

**Action**: Use automated tools to enforce standards.

**Automated checks**:
- **Formatting**: `cargo make fmt` in CI
- **Linting**: `cargo make lint` in CI
- **Tests**: `cargo make test` in CI
- **Coverage**: Coverage checks in CI

#### 5.2: Code Review Checklist

**Action**: Add standards to code review checklist.

**Checklist items**:
- [ ] Code follows style standards
- [ ] Code uses standard patterns
- [ ] Code meets quality standards
- [ ] Code has required documentation

#### 5.3: Create Todo List for Controls

**CRITICAL**: Do NOT write documents or reports. Create todos and execute controls.

**Action**: Create 10+ item todo list for implementing controls.

**Example control todo list**:
```markdown
## Mura Control Todos (10+ items)

**Automated Checks**:
- [ ] Add CI check: `cargo make fmt --check`
- [ ] Add CI check: `cargo make lint`
- [ ] Add CI check: Coverage threshold enforcement
- [ ] Verify automated checks work correctly

**Code Review Controls**:
- [ ] Add checklist item: Code follows style standards
- [ ] Add checklist item: Code uses standard patterns
- [ ] Update code review process to include checklist

**Standards Controls**:
- [ ] Add standard: Use `snake_case` for functions
- [ ] Add standard: Use `Result<T, E>` for error handling
- [ ] Update team documentation with standards
- [ ] Verify standards are followed in code reviews
```

**Execution**:
1. Create todos using `todo_write` tool (10+ items minimum)
2. Execute todos one by one (implement controls)
3. Mark todos as completed as controls are implemented
4. Verify each control works before moving to next
5. Continue until all controls implemented

**Principle**: Implement controls to prevent inconsistency, don't just document them. Todos track progress, controls prevent recurrence.

---

## Integration with Other Commands

- **[Kaizen Improvement](./kaizen-improvement.md)** - Use Kaizen to standardize incrementally
- **[Eliminate Muda](./eliminate-muda.md)** - Remove waste while standardizing
- **[Gemba Walk](./gemba-walk.md)** - Go to source to verify standards applied
- **[Andon Signals](./andon-signals.md)** - Use linting as signals of inconsistency

## Expert Insights

**Why this matters**: Inconsistency increases cognitive load and maintenance cost. Consistent code is easier to understand and maintain.

**Key principle**: "Consistency is more important than perfection" - It's better to have consistent, good code than perfect code in some places and poor code in others.

**Remember**: Standardization is continuous. New code should follow standards. Existing code should be gradually standardized. Use Kaizen to standardize incrementally.

**DfLSS alignment**: Eliminating Mura (inconsistency) supports DfLSS (Design for Lean Six Sigma) by ensuring both efficiency (consistent patterns reduce waste) AND quality (consistent quality standards prevent defects). Don't conflate DfLSS with DFSS (Design for Six Sigma) - DFSS only addresses quality, missing critical waste elimination. See [Root Cause Analysis - DfLSS vs DFSS](./root-cause-analysis.md#dflss-vs-dfss-critical-distinction) for why conflating DfLSS with DFSS is a huge error.

---

## Command Execution Pattern

**CRITICAL**: Eliminate Mura commands must:
1. **Create 10+ item todo list** - Not documents/reports
2. **Execute todos** - Standardize code, not document it
3. **Verify fixes** - Test that standardization works
4. **Complete todos** - Mark todos as done as standardization completes

**Principle**: Standardize code, don't document it. Todos track progress, standardization improves consistency.

---

End Command ---

