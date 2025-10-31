# Schema Review Guide

## Purpose

This guide ensures every schema change is reviewed for correctness, completeness, and alignment with clnrm's zero-false-positive validation goals.

## Review Checklist

### For Each Schema:

#### 1. Purpose Clear
- [ ] What behavior does this schema prove?
- [ ] Can this behavior be faked by stub implementations?
- [ ] Is telemetry the ONLY way to prove this behavior?
- [ ] Does the schema `brief` clearly state the proof it provides?

**Example:**
```yaml
# ✅ GOOD - Clear proof statement
brief: Represents a complete test execution in an isolated container
note: 'container.id MUST exist (proves container ran)'

# ❌ BAD - Vague purpose
brief: Test data
note: 'Information about tests'
```

#### 2. Required Attributes
- [ ] Are all critical attributes marked `required`?
- [ ] Can the proof fail without each required attribute?
- [ ] Would optional attributes allow false positives?
- [ ] Is every required attribute always available?

**Critical Attributes Checklist:**

For `span.clnrm.test_execution`:
- [ ] `container.id` - REQUIRED (proves container ran)
- [ ] `test.isolated` - REQUIRED (proves hermetic isolation)
- [ ] `test.result` - REQUIRED (proves execution completed)
- [ ] `test.duration_ms` - REQUIRED (proves actual execution time)
- [ ] `test.cleanup_performed` - REQUIRED (proves cleanup happened)

For `span.clnrm.container_lifecycle`:
- [ ] `container.id` - REQUIRED (primary key)
- [ ] `container.created_at` - REQUIRED (proves creation)
- [ ] `container.destroyed_at` - REQUIRED (proves cleanup)
- [ ] `container.state` - REQUIRED (tracks lifecycle)
- [ ] `cleanup.success` - REQUIRED (verifies cleanup)

For `span.clnrm.plugin_execution`:
- [ ] `plugin.name` - REQUIRED (identifies plugin)
- [ ] `plugin.state` - REQUIRED (tracks lifecycle)
- [ ] `container.id` - REQUIRED (links to container)
- [ ] `plugin.health_check.performed` - REQUIRED (proves health checking)
- [ ] `plugin.health_check.passed` - REQUIRED (verifies health)

#### 3. Types Correct
- [ ] String for identifiers and names?
- [ ] Int for counts and codes?
- [ ] Double for durations and measurements?
- [ ] Boolean for flags and results?
- [ ] Enum for states and results?
- [ ] String[] for lists?

**Type Selection Guide:**

```yaml
# ✅ GOOD - Appropriate types
container.id: string          # Unique identifier
test.duration_ms: double      # Measurement with precision
test.isolated: boolean        # True/false flag
test.result: enum             # Limited valid values
container.ports: string[]     # List of mappings

# ❌ BAD - Wrong types
container.id: int             # IDs aren't numbers
test.duration_ms: int         # Loses precision
test.result: string           # Allows arbitrary values
```

#### 4. Enums Defined
- [ ] All possible values listed in `members`?
- [ ] `allow_custom_values: false` set?
- [ ] Each member has clear `brief`?
- [ ] No overlapping meanings?

**Enum Checklist:**

```yaml
# ✅ GOOD - Complete enum
test.result:
  type:
    allow_custom_values: false
    members:
      - id: pass
        value: pass
        brief: Test passed all assertions
      - id: fail
        value: fail
        brief: Test failed one or more assertions
      - id: error
        value: error
        brief: Test encountered an execution error

# ❌ BAD - Allows custom values
test.result:
  type: string  # Any value allowed!
```

#### 5. Stability
- [ ] `stable` for production-ready schemas?
- [ ] `experimental` for new features?
- [ ] `deprecated` for schemas being removed?
- [ ] Stability matches actual usage?

**Stability Guide:**

- **stable**: Production-ready, no breaking changes allowed
- **experimental**: Testing, may change or be removed
- **deprecated**: Being removed, migration guide required

#### 6. Documentation
- [ ] `brief` is clear and concise?
- [ ] `note` explains validation strategy?
- [ ] Examples show realistic values?
- [ ] Edge cases documented?

### For Required Attributes:

#### Does This Attribute PROVE a Behavior?

Ask these questions:

1. **Can tests pass without this attribute?**
   - If YES → attribute must be REQUIRED
   - If NO → consider making it optional

2. **Does this catch false positives?**
   - If YES → attribute must be REQUIRED
   - If NO → why is it in the schema?

3. **Is it always available?**
   - If YES → safe to make REQUIRED
   - If NO → document when it's missing

**Examples:**

```yaml
# ✅ GOOD - Proves behavior
container.id:
  requirement_level: required
  note: 'CANNOT exist without real container - proves container ran'

# ❌ BAD - Optional allows false positives
container.id:
  requirement_level: recommended
  # Tests could pass without containers!
```

### For Optional Attributes:

#### Why Is This Optional?

Every optional attribute needs justification:

1. **Not always available** (e.g., `error.message` only on errors)
   ```yaml
   error.message:
     requirement_level:
       conditionally_required: Only when test.result is 'error'
   ```

2. **Performance overhead** (e.g., detailed metrics)
   ```yaml
   plugin.config:
     requirement_level: recommended
     note: 'Optional - may contain sensitive data'
   ```

3. **Nice-to-have data** (e.g., human-readable names)
   ```yaml
   container.name:
     requirement_level: recommended
     note: 'Human-readable name, ID is sufficient for tracking'
   ```

**Questions to Ask:**

- [ ] Should this be required instead?
- [ ] What happens if it's missing?
- [ ] Does this create a false positive risk?

### Red Flags

These indicate potential false positive risks:

#### ❌ Critical Attributes Marked Optional

```yaml
# RED FLAG!
container.id:
  requirement_level: recommended  # Should be REQUIRED!
```

**Fix:** Change to `required` with clear note about why it's critical.

#### ❌ Arbitrary String Types (Should Be Enum)

```yaml
# RED FLAG!
test.result:
  type: string  # Allows any value!
```

**Fix:** Define enum with `allow_custom_values: false`.

#### ❌ Missing container.id in Container Spans

```yaml
# RED FLAG!
span.clnrm.plugin_execution:
  attributes:
    plugin.name: ...
    # Where's container.id?
```

**Fix:** Add `container.id` as required attribute.

#### ❌ Missing test.isolated in Test Spans

```yaml
# RED FLAG!
span.clnrm.test_execution:
  attributes:
    test.name: ...
    # Where's test.isolated?
```

**Fix:** Add `test.isolated` as required boolean.

#### ❌ Missing Error Attributes in Error Spans

```yaml
# RED FLAG!
span.clnrm.test_execution:
  attributes:
    # No error.type or error.message!
```

**Fix:** Add conditionally required error attributes.

### Green Flags

These indicate good schema design:

#### ✅ All Critical Behaviors Have Required Attributes

```yaml
span.clnrm.test_execution:
  attributes:
    - id: container.id
      requirement_level: required
    - id: test.isolated
      requirement_level: required
    - id: test.result
      requirement_level: required
```

#### ✅ Enums for State/Result/Type Fields

```yaml
test.result:
  type:
    allow_custom_values: false
    members:
      - id: pass
      - id: fail
      - id: error
```

#### ✅ Clear Documentation

```yaml
container.id:
  brief: Unique identifier of the container where test ran
  requirement_level: required
  note: 'CRITICAL PROOF: This attribute CANNOT exist without a real container.'
```

#### ✅ Type-Safe (No Arbitrary Values)

```yaml
test.duration_ms: double       # Precise measurements
test.isolated: boolean         # True/false only
test.result: enum              # Limited valid values
container.ports: string[]      # Typed array
```

## Review Process

### Before Submitting Schema Changes:

1. **Self-Review:**
   - [ ] Run this checklist on your changes
   - [ ] Check for red flags
   - [ ] Verify green flags present
   - [ ] Document any trade-offs

2. **Validation:**
   - [ ] Run `weaver registry check -r registry/`
   - [ ] Run schema completeness checker
   - [ ] Run false positive detector
   - [ ] Check for breaking changes

3. **Documentation:**
   - [ ] Update CHANGELOG with schema changes
   - [ ] Add migration guide if breaking
   - [ ] Update examples if needed

### During Review:

Reviewers should verify:

1. **Completeness:**
   - All critical behaviors have schemas
   - All required attributes present
   - No false positive risks

2. **Correctness:**
   - Types appropriate
   - Enums complete
   - Documentation clear

3. **Compatibility:**
   - No breaking changes without migration
   - Stability levels correct
   - Versioning followed

## Schema Evolution Rules

### Safe Changes (Non-Breaking):

✅ **Adding New Schemas**
- New span/metric/event types
- New optional attributes
- New enum members (if `allow_custom_values: true`)

✅ **Adding Documentation**
- Improving `brief` and `note`
- Adding examples
- Clarifying usage

✅ **Relaxing Requirements**
- `required` → `recommended`
- `recommended` → `optional`
- (But consider if this allows false positives!)

### Breaking Changes (Require Migration):

❌ **Removing Schemas**
- Provide replacement schema
- Document migration path
- Deprecate first, remove later

❌ **Removing Required Attributes**
- Major version bump required
- Migration guide mandatory
- Consider backwards compatibility

❌ **Changing Types**
- Never change attribute types
- Create new attribute instead
- Deprecate old attribute

❌ **Removing Enum Members**
- Document which code breaks
- Provide alternative values
- Consider deprecation period

❌ **Making Attributes Required**
- Existing code may not set them
- Add as `recommended` first
- Require in next major version

## Common Mistakes

### Mistake 1: Optional Critical Attributes

```yaml
# ❌ WRONG
container.id:
  requirement_level: recommended

# ✅ RIGHT
container.id:
  requirement_level: required
  note: 'Proves container actually ran'
```

### Mistake 2: String Instead of Enum

```yaml
# ❌ WRONG
test.result:
  type: string

# ✅ RIGHT
test.result:
  type:
    allow_custom_values: false
    members:
      - id: pass
      - id: fail
      - id: error
```

### Mistake 3: Missing Validation Notes

```yaml
# ❌ WRONG
test.duration_ms:
  type: double
  requirement_level: required

# ✅ RIGHT
test.duration_ms:
  type: double
  requirement_level: required
  note: 'Must be > 0, proving actual execution occurred'
```

### Mistake 4: Vague Descriptions

```yaml
# ❌ WRONG
brief: Test information

# ✅ RIGHT
brief: Represents a complete test execution in an isolated container
note: 'This span PROVES containers ran via required container.id attribute'
```

## Summary

**Golden Rules:**

1. **Every schema must PROVE a behavior** (not just record data)
2. **Critical attributes must be REQUIRED** (not optional or recommended)
3. **States and results must be ENUMS** (not arbitrary strings)
4. **Documentation must explain VALIDATION** (not just description)
5. **Changes must preserve COMPATIBILITY** (or provide migration)

**Before Merging:**

- [ ] All checklists completed
- [ ] All validation tools pass
- [ ] No critical issues
- [ ] Documentation updated
- [ ] Tests updated

**Remember:** Schemas are the contract. If schemas are wrong, everything built on them is wrong.
