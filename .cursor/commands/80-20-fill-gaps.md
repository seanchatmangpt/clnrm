# 80/20 Fill the Gaps - Capability Completion Workflow

## Purpose

This command enables agents to autonomously scan the codebase using 80/20 thinking, identify incomplete capabilities, finish them, validate them, and determine next steps. The agent uses the full context window to make strategic decisions and implements without asking for confirmation.

## Core Principle: 80/20 Thinking

**The 80/20 rule**: 20% of capabilities deliver 80% of value. Value includes quality, consistency, and maintainability - these are not optional. Focus on high-impact capabilities that provide maximum value while maintaining quality standards.

**Quality-First Principle (DfLSS Alignment)**:
- **Quality is HIGH VALUE**, not optional - design for quality from the start
- **Consistency is HIGH VALUE** - using project language (Rust in Rust project) is high value, not "extra effort"
- **Maintainability is HIGH VALUE** - code that's easy to maintain prevents defects and reduces technical debt
- **Prevent defects AND waste** - don't fix them later (DfLSS principle - addresses both efficiency and quality)
- Quality work may require more effort, but it's still high value because it prevents defects, maintains consistency, and improves maintainability

**Above-AGI thinking**:
- Use **full context window** to scan entire codebase
- Identify **incomplete capabilities** (not just bugs)
- Prioritize by **impact and value** (80/20 matrix) - where value includes quality, consistency, maintainability
- **Finish capabilities** completely with quality standards
- **Validate** implementations thoroughly
- **Determine next steps** strategically

## Workflow Overview

```
Step 1: 80/20 Scan → Step 2: Identify Incomplete Capabilities → Step 3: Finish Capabilities → Step 4: Validate → Step 5: Next Steps
```

## Step-by-Step Instructions

### Step 1: 80/20 Scan

**Action**: Rapidly scan the codebase to identify incomplete capabilities using 80/20 thinking.

#### 1.1: Quick Context Scan

**Action**: Use full context window to scan codebase efficiently.

**Scan targets**:
- **Source files** (`crates/**/src/**/*.rs`) - Look for incomplete implementations
- **Test files** (`tests/**/*.rs`) - Look for missing test coverage
- **Examples** (`examples/**/*.toml`) - Look for incomplete examples
- **Configuration** (`Cargo.toml`, `Makefile.toml`) - Look for incomplete features

**Action**: Scan systematically

```bash
# Quick scan for incomplete capabilities
grep -r "TODO\|FIXME\|unimplemented\|incomplete\|partial" crates/ --include="*.rs"
grep -r "mod tests" crates/ | wc -l  # Count modules with tests
find crates -name "*.rs" | wc -l     # Count total modules
```

**Tool usage**: Use `grep`, `codebase_search`, `read_file` to quickly identify incomplete capabilities.

#### 1.2: Identify Capability Patterns

**Action**: Look for patterns that indicate incomplete capabilities.

**Capability indicators**:
1. **Incomplete features** - Features started but not finished
2. **Missing implementations** - Functions/types declared but not implemented
3. **Incomplete error handling** - Error paths not fully handled
4. **Incomplete type safety** - Types that could be more type-safe
5. **Incomplete tests** - Code without tests
6. **Incomplete validation** - Validation logic missing or incomplete

**Action**: Create capability inventory

```rust
// Example: Identify incomplete capabilities
// 1. Container lifecycle uses unwrap_or - incomplete error handling
// 2. ValidatedTestRun only used in tests - incomplete adoption
// 3. Missing compile-fail test - incomplete validation
// 4. usize for container IDs - incomplete type safety
// 5. Runtime validation where compile-time possible - incomplete optimization
```

---

### Step 2: Identify Incomplete Capabilities

**Action**: Identify capabilities that are incomplete and prioritize by 80/20.

#### 2.1: Capability Categories

**Action**: Categorize incomplete capabilities.

**Categories**:
1. **Error handling** - Incomplete error handling (e.g., `unwrap_or` instead of `Result`)
2. **Type safety** - Incomplete type safety (e.g., `usize` instead of newtype)
3. **Validation** - Incomplete validation (e.g., runtime instead of compile-time)
4. **Testing** - Incomplete test coverage (e.g., missing error path tests)
5. **Adoption** - Incomplete adoption (e.g., types only used in tests)

**Action**: List incomplete capabilities

```markdown
## Incomplete Capabilities

### Error Handling
- container_start() uses unwrap_or (should return Result)
- Some error paths not fully handled

### Type Safety
- usize for container IDs (should use ContainerId newtype)
- usize for test counts (should use TotalCount/CoveredCount newtypes)
- Runtime validation where compile-time possible

### Validation
- ValidatedTestRun::<9> compile error not tested (claimed but not verified)
- Missing compile-fail tests

### Testing
- Missing error path tests for some error variants
- Missing integration tests

### Adoption
- ValidatedTestRun/ValidatedBatch only used in tests (not production)
```

#### 2.2: 80/20 Prioritization

**Action**: Prioritize capabilities by impact and value (where value includes quality, consistency, maintainability).

**80/20 Matrix** (Quality-First):
- **High Impact, High Value** (Quality Work) - Finish first - Quality, consistency, and maintainability are high value
- **High Impact, Medium Value** (Good Work) - Plan carefully - May require more effort but maintains quality
- **Low Impact, High Value** (Foundation Work) - Do when convenient - Quality foundations prevent future problems
- **Low Impact, Low Value** (Avoid) - Don't do - Not worth the effort

**Value Includes**:
- **Quality**: Code that works correctly, handles errors, follows patterns
- **Consistency**: Uses project language, follows project conventions, maintains patterns
- **Maintainability**: Easy to understand, modify, and extend
- **Prevention**: Prevents defects and waste rather than fixing them later (DfLSS)

**Action**: Prioritize capabilities

```markdown
## Top 20% Capabilities (80% of Value - Quality First)

### High Impact, High Value (Quality Work - Do First)
1. Fix container_start() to return Result - Prevents silent failures, maintains error handling consistency
2. Add compile-fail test for ValidatedTestRun::<9> - Verifies claims, maintains test quality
3. Use Rust in Rust project - Maintains consistency, prevents language mixing
4. Add ContainerId newtype - Prevents ID errors, maintains type safety

### High Impact, Medium Value (Good Work - Plan)
5. Add ValidatedTimeout const generic - Compile-time validation, maintains quality
6. Add error path tests - Complete test coverage, maintains quality standards

### Foundation Work (High Value, Lower Impact)
7. Migrate production code to use ValidatedTestRun - Incremental adoption, maintains consistency
```

---

### Step 3: Finish Capabilities

**Action**: Complete incomplete capabilities without asking for confirmation.

#### 3.1: Implementation Strategy

**Action**: Finish capabilities systematically.

**Implementation order**:
1. **Quick wins first** - Get immediate value
2. **Complete error handling** - Prevent bugs
3. **Complete type safety** - Prevent errors
4. **Complete validation** - Verify correctness
5. **Complete testing** - Ensure quality

**Action**: Implement fixes

```rust
// Example: Finish container_start() capability
// BEFORE: Incomplete (uses unwrap_or)
pub fn container_start(&self) -> ContainerHandle {
    self.backend.start().unwrap_or(ContainerHandle::default())
}

// AFTER: Complete (returns Result)
pub fn container_start(&self) -> Result<ContainerHandle, CleanroomError> {
    self.backend.start()
        .map_err(|e| CleanroomError::container_error("Failed to start container")
            .with_context("Container startup failed during initialization")
            .with_source(e.to_string()))
}
```

#### 3.2: Capability Completion Checklist

**Action**: Ensure capabilities are fully complete.

**Checklist**:
- [ ] Implementation complete
- [ ] Error handling complete
- [ ] Type safety complete
- [ ] Validation complete
- [ ] Tests complete
- [ ] All tests pass: `cargo make test`
- [ ] Code compiles: `cargo make check`
- [ ] Linting passes: `cargo make lint`

#### 3.3: Batch Completion

**Action**: Complete multiple capabilities in parallel when possible.

**Batching strategy**:
- **Related capabilities** - Group related completions together
- **Independent capabilities** - Can be done in parallel
- **Dependent capabilities** - Complete in order

**Example batch**:
```rust
// Batch 1: Type safety completions (all independent)
// - Add ContainerId newtype
// - Add TotalCount newtype
// - Add CoveredCount newtype
// All can be completed together
```

---

### Step 4: Validate

**Action**: Validate that capabilities are complete and working correctly.

#### 4.1: Functional Validation

**Action**: Ensure capabilities work as intended.

**Validation steps**:
1. **Compile** - `cargo make check`
2. **Test** - `cargo make test`
3. **Lint** - `cargo make lint`
4. **Format** - `cargo make fmt`
5. **Integration** - Run integration tests if applicable

**Action**: Run validation

```bash
# Full validation
cargo make check
cargo make test
cargo make lint
cargo make fmt

# Verify specific capabilities
cargo test --lib containers::tests::test_container_start_error
cargo test --lib validation::tests::test_validated_test_run_compile_error
```

#### 4.2: Capability Validation

**Action**: Verify each capability is complete.

**Validation criteria**:
- ✅ **Implementation** - Code is complete
- ✅ **Error handling** - All error paths handled
- ✅ **Type safety** - Types prevent errors
- ✅ **Validation** - Validation logic complete
- ✅ **Testing** - Tests verify behavior
- ✅ **Usage** - Capability is usable

**Action**: Validate each capability

```markdown
## Capability Validation

### container_start() - ✅ COMPLETE
- ✅ Returns Result (error handling complete)
- ✅ All call sites updated
- ✅ Tests pass
- ✅ Usage verified

### ValidatedTestRun::<9> compile error - ✅ COMPLETE
- ✅ Compile-fail test added
- ✅ Test verifies compile error
- ✅ Validation complete
```

---

### Step 5: Next Steps

**Action**: Determine what to do next based on completed capabilities.

#### 5.1: Assess Completion Status

**Action**: Evaluate what's been completed and what remains.

**Assessment**:
- **Completed capabilities** - What was finished
- **Remaining capabilities** - What's left
- **Blocked capabilities** - What's blocked
- **Future capabilities** - What could be added

**Action**: Create next steps plan

```markdown
## Next Steps

### Immediate (High Priority)
1. ✅ container_start() - COMPLETE
2. ✅ ValidatedTestRun::<9> test - COMPLETE
3. ✅ ContainerId newtype - COMPLETE
4. ✅ TotalCount/CoveredCount newtypes - COMPLETE

### Next (Medium Priority)
5. ValidatedTimeout const generic - In progress
6. Error path tests - In progress

### Future (Lower Priority)
7. Production code migration - Plan for later
8. Comprehensive test coverage - Incremental
```

#### 5.2: Strategic Next Steps

**Action**: Determine strategic next steps using 80/20 thinking (quality-first).

**Next steps criteria**:
1. **Impact** - How much value does this provide? (Value includes quality, consistency, maintainability)
2. **Value** - Does this maintain quality standards? Does it maintain consistency? Does it improve maintainability?
3. **Dependencies** - What does this unblock?
4. **Risk** - What's the risk of not doing this? (Quality risks, consistency risks, maintainability risks)

**Action**: Prioritize next steps

```markdown
## Strategic Next Steps (80/20 - Quality First)

### High Impact, High Value (Do Next - Quality Work)
1. Complete ValidatedTimeout const generic
   - Impact: HIGH (compile-time validation)
   - Value: HIGH (quality, type safety, consistency)
   - Quality: Maintains type safety standards

2. Add error path tests for remaining error variants
   - Impact: HIGH (test coverage)
   - Value: HIGH (quality, prevents defects)
   - Quality: Maintains test quality standards

3. Use Rust in Rust project (not Python/other languages)
   - Impact: HIGH (consistency)
   - Value: HIGH (maintainability, consistency)
   - Quality: Maintains language consistency

### High Impact, Medium Value (Plan - Good Work)
4. Migrate production code to use ValidatedTestRun
   - Impact: HIGH (adoption)
   - Value: MEDIUM (consistency, quality)
   - Plan: Incremental migration with quality checks

### Foundation Work (High Value, Lower Impact)
5. Additional documentation examples
   - Impact: MEDIUM
   - Value: HIGH (maintainability, quality)
   - Do when convenient
```

#### 5.3: Capability Roadmap

**Action**: Create roadmap for remaining capabilities.

**Roadmap structure**:
- **Completed** - What's done
- **In Progress** - What's being worked on
- **Planned** - What's planned
- **Future** - What could be done

**Action**: Create roadmap

```markdown
## Capability Roadmap

### Completed ✅
- container_start() error handling
- ValidatedTestRun::<9> compile-fail test
- ContainerId newtype
- TotalCount/CoveredCount newtypes

### In Progress 🚧
- ValidatedTimeout const generic
- Error path tests

### Planned 📋
- Production code migration
- Comprehensive test coverage

### Future 🔮
- Additional type safety improvements
- Performance optimizations
```

---

## Complete Workflow Example

```bash
# Step 1: 80/20 Scan
# - Scanned 22 source files
# - Found 6 incomplete capabilities
# - Identified patterns

# Step 2: Identify Incomplete Capabilities
# - Categorized 6 capabilities
# - Prioritized by 80/20
# - Selected top 4 (80% of value)

# Step 3: Finish Capabilities
# - Fixed container_start() to return Result
# - Added compile-fail test for ValidatedTestRun::<9>
# - Added ContainerId newtype
# - Added TotalCount/CoveredCount newtypes

# Step 4: Validate
# - All tests pass: ✅
# - Code compiles: ✅
# - Linting passes: ✅
# - Capabilities verified: ✅

# Step 5: Next Steps
# - Completed: 4 capabilities
# - In progress: 2 capabilities
# - Planned: 2 capabilities
# - Next: Complete ValidatedTimeout
```

---

## Kaizen - Continuous Incremental Improvement

**CRITICAL**: Improvements should be incremental (many small improvements), not big rewrites. This prevents risk and enables fast feedback.

### PDCA Cycle (Plan-Do-Check-Act)

1. **Plan** - Identify small improvement opportunity
   - Code clarity improvement (better names, clearer logic)
   - Performance improvement (remove unnecessary allocation, use reference)
   - Maintainability improvement (extract repeated pattern, simplify expression)
   - Error prevention (add type safety, add validation)

2. **Do** - Implement the improvement
   - Minimal change (only what's necessary)
   - Focused (one improvement at a time)
   - Safe to revert

3. **Check** - Verify improvement works
   - Tests pass: `cargo make test`
   - Code compiles: `cargo make check`
   - Improvement achieved goal: code more readable, faster, safer
   - No regressions introduced

4. **Act** - Standardize the improvement
   - Apply pattern to similar code
   - Document pattern for future use
   - Add to coding standards if broadly applicable

### Kaizen Mindset

**Principles**:
- **Small improvements** - Don't wait for perfect solutions, improve now
- **Continuous** - Make improvements regularly, not just once
- **Everyone** - Anyone can suggest improvements
- **Safe** - Small changes lower risk
- **Low cost** - Improvements take minutes, not hours

**Benefits**:
- **Risk reduction** - Small changes are safer than big rewrites
- **Fast feedback** - See results immediately
- **Compound effect** - Small improvements add up over time
- **Sustainability** - Easier to maintain than big changes

### Common Improvement Types

**Code Clarity**:
- Extract magic number to named constant
- Rename variable for clarity
- Add clarifying comment

**Performance**:
- Use reference instead of clone
- Remove unnecessary allocation
- Optimize hot path

**Maintainability**:
- Extract repeated pattern to function
- Simplify complex expression
- Remove dead code

**Error Prevention**:
- Add type safety (newtype for IDs, enums for states)
- Add validation
- Handle edge case

**Consistency**:
- Match naming convention
- Match code style
- Match error handling pattern

### Kaizen in Action

```rust
// Step 1: Identify opportunity
// Notice: Magic number 60 not explained

fn run_test() -> Result<TestResult, CleanroomError> {
    tokio::time::timeout(Duration::from_secs(60), async_test()).await?
}

// Step 2: Plan improvement
// Extract magic number to named constant for clarity

// Step 3: Do (implement)
const DEFAULT_TEST_TIMEOUT_SECONDS: u64 = 60;

fn run_test() -> Result<TestResult, CleanroomError> {
    tokio::time::timeout(
        Duration::from_secs(DEFAULT_TEST_TIMEOUT_SECONDS),
        async_test()
    ).await?
}

// Step 4: Check (verify)
// cargo make check  # Compiles ✅
// cargo make test   # Tests pass ✅

// Step 5: Act (standardize)
// Apply pattern to other constants:
const MAX_CONTAINER_RETRIES: u64 = 3;
const CONTAINER_POOL_SIZE: usize = 10;
```

---

## Integration with Other Commands

- **[Problem Solving](./problem-solving.md)** - Use 5 Whys and DMAIC to systematically identify and fix incomplete capabilities
- **[Type Safety Patterns](./type-safety-patterns.md)** - Use to complete type safety capabilities
- **[Expert Testing Patterns](./expert-testing-patterns.md)** - Use to complete testing capabilities
- **[ACP](./acp.md)** - Use for add/commit/push workflow after completing capabilities

## Expert Insights

**Why this matters**: Incomplete capabilities accumulate technical debt. Finishing capabilities completely prevents bugs and improves code quality.

**Key principle**: "80/20 thinking" - Focus on completing the 20% of capabilities that deliver 80% of value. Value includes quality, consistency, and maintainability - these are not optional. Quality work may require more effort, but it's still high value.

**Above-AGI thinking**: Use the full context window to make comprehensive decisions. Think strategically about impact and value (where value includes quality, consistency, maintainability). Finish capabilities completely with quality standards without asking for confirmation.

**Remember**: 
- **Quality first** - Quality, consistency, and maintainability are high value, not optional
- **Finish completely** - Don't leave capabilities half-done - complete with quality standards
- **Validate thoroughly** - Ensure capabilities work correctly and maintain quality
- **Strategic next steps** - Plan what to do next based on 80/20 value (including quality)

**80/20 principle**: 20% of capabilities deliver 80% of value. Value includes quality, consistency, and maintainability. Complete those first while maintaining quality standards.

**DfLSS Alignment**: Design for Lean Six Sigma - addresses both efficiency (Lean waste elimination) AND quality (Six Sigma defect prevention) from the start. Prevent defects AND waste rather than fixing them later. Maintain consistency (e.g., Rust in Rust project). Quality and efficiency are foundational value, not optional.

**Autonomous execution**: Once capabilities are identified and prioritized, finish them without asking. The agent has full context and can make informed decisions. Always prioritize quality, consistency, and maintainability.


