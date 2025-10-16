# Snapshot Testing Workflow & Best Practices

## Overview

This document outlines the comprehensive snapshot testing infrastructure for the CLNRM project, covering Rust code, CLI output, data structures, and React UI components.

## Directory Structure

```
tests/snapshots/
├── rust/                         # Rust code snapshots
│   ├── snapshot_infrastructure.rs
│   ├── scenario_snapshots.rs
│   └── __snapshots__/           # Generated snapshots
├── cli/                          # CLI output snapshots
│   ├── cli_output_snapshots.rs
│   └── __snapshots__/
├── data/                         # Data structure snapshots
│   ├── data_structure_snapshots.rs
│   └── __snapshots__/
└── ui/                           # UI component snapshots
    └── __snapshots__/
```

## Snapshot Testing Infrastructure

### 1. Rust Snapshot Testing (using insta)

**Setup:**
```bash
cargo install cargo-insta
cargo add insta --dev
```

**Usage:**
```rust
use insta::assert_snapshot;

#[test]
fn test_output() {
    let result = generate_output();
    assert_snapshot!(result);
}
```

**Review Snapshots:**
```bash
# Review all pending snapshots
cargo insta review

# Accept all snapshots
cargo insta accept

# Reject all snapshots
cargo insta reject
```

### 2. CLI Output Snapshots

**Features:**
- Normalized timestamps and paths
- Dynamic value replacement
- Exit code tracking
- Stdout/stderr separation

**Best Practices:**
- Always normalize timestamps: `[TIMESTAMP]`
- Replace durations: `[DURATION]ms`
- Replace absolute paths: `[PATH]`
- Separate success/error cases

### 3. Data Structure Snapshots

**Formats Supported:**
- JSON (primary)
- YAML (secondary)
- TOML (configuration)

**When to Use:**
- Configuration file validation
- API response testing
- Data transformation verification
- Schema evolution tracking

### 4. UI Component Snapshots (Jest)

**Setup:**
```bash
cd examples/optimus-prime-platform
npm install --save-dev jest @testing-library/react @testing-library/jest-dom
npm install --save-dev ts-jest @types/jest
```

**Usage:**
```typescript
import { render } from '@testing-library/react';

test('component snapshot', () => {
  const { container } = render(<MyComponent />);
  expect(container).toMatchSnapshot();
});
```

**Review Snapshots:**
```bash
# Update snapshots
npm test -- -u

# Interactive mode
npm test -- --watch
```

## Snapshot Review Workflow

### 1. Initial Snapshot Creation

1. Write test with `assert_snapshot!()` or `toMatchSnapshot()`
2. Run test suite
3. Review generated snapshot in `__snapshots__` directory
4. Commit snapshot files with code

### 2. Snapshot Updates

When code changes require snapshot updates:

1. **Identify Changes:**
   ```bash
   cargo test  # For Rust
   npm test    # For JavaScript/TypeScript
   ```

2. **Review Diffs:**
   - Use `cargo insta review` for Rust
   - Use Jest's interactive mode for JS/TS
   - Review similarity scores (>80% = minor change)

3. **Approve or Reject:**
   ```bash
   # Rust
   cargo insta accept [test_name]  # or reject

   # JavaScript
   npm test -- -u [test_name]
   ```

4. **Document Changes:**
   - Update snapshot metadata
   - Add change description in commit
   - Link to related issue/PR

### 3. Snapshot Metadata

Each snapshot update should include:
```rust
SnapshotMetadata {
    test_name: "scenario_result_snapshot",
    created_at: "2025-10-16T07:24:06Z",
    last_updated: Some("2025-10-16T08:15:32Z"),
    review_status: ReviewStatus::RequiresReview,
    reviewer: None,
    change_description: Some("Updated for new feature X"),
}
```

## Smart Diff Algorithms

### Similarity Score Calculation

```rust
pub fn calculate_similarity(old: &str, new: &str) -> f64 {
    let common_lines = count_common_lines(old, new);
    let total_lines = max(old.lines().count(), new.lines().count());
    common_lines as f64 / total_lines as f64
}
```

**Interpretation:**
- `1.0` - Identical
- `0.9-0.99` - Minor changes (likely safe to approve)
- `0.7-0.89` - Moderate changes (review carefully)
- `<0.7` - Major changes (requires detailed review)

### Diff Categories

1. **Added Lines** - New content
2. **Removed Lines** - Deleted content
3. **Modified Lines** - Changed content
4. **Structural Changes** - Format/indentation changes

## Best Practices

### DO:
- ✅ Create snapshots for all output formats
- ✅ Normalize dynamic values (timestamps, IDs, paths)
- ✅ Review snapshots before committing
- ✅ Keep snapshots small and focused
- ✅ Use descriptive snapshot names
- ✅ Document breaking changes
- ✅ Version control snapshots with code

### DON'T:
- ❌ Commit snapshots without review
- ❌ Include sensitive data in snapshots
- ❌ Mix multiple concerns in one snapshot
- ❌ Accept all snapshots blindly
- ❌ Leave snapshots unorganized
- ❌ Forget to update documentation

## Snapshot Update Commands

### Rust (using insta)
```bash
# Review all snapshots interactively
cargo insta review

# Accept specific snapshot
cargo insta accept test_name

# Test with snapshot generation
cargo test

# Force snapshot regeneration
cargo insta test --force-update-snapshots
```

### JavaScript/TypeScript (using Jest)
```bash
# Update all snapshots
npm test -- -u

# Update specific test snapshots
npm test -- -u -t "test_name"

# Interactive watch mode
npm test -- --watch

# View snapshot summary
npm test -- --verbose
```

## Integration with CI/CD

### GitHub Actions Example
```yaml
name: Snapshot Tests

on: [push, pull_request]

jobs:
  snapshots:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      # Rust snapshots
      - name: Run Rust snapshot tests
        run: |
          cargo test
          cargo insta test --check

      # UI snapshots
      - name: Run UI snapshot tests
        run: |
          cd examples/optimus-prime-platform
          npm ci
          npm test -- --ci

      # Fail if snapshots need updating
      - name: Check for snapshot changes
        run: |
          git diff --exit-code tests/snapshots/__snapshots__/
```

## Troubleshooting

### Snapshots Not Matching

1. Check for non-deterministic values
2. Verify normalization functions
3. Review timestamp/path replacements
4. Check for environment-specific output

### Snapshots Too Large

1. Break into smaller, focused tests
2. Use snapshot fragments
3. Filter unnecessary data
4. Consider structural testing instead

### Frequent Snapshot Updates

1. Review test stability
2. Check for flaky tests
3. Improve value normalization
3. Consider integration tests instead

## Memory Integration

Snapshot testing integrates with Claude-Flow memory:

```bash
# Store snapshot metadata
npx claude-flow@alpha hooks post-edit --file "snapshot.snap" \
  --memory-key "swarm/snapshot-testing/baseline"

# Retrieve snapshot history
npx claude-flow@alpha memory retrieve "swarm/snapshot-testing/*"
```

## Resources

- [insta Documentation](https://insta.rs/)
- [Jest Snapshot Testing](https://jestjs.io/docs/snapshot-testing)
- [Testing Library](https://testing-library.com/)
- [Snapshot Testing Best Practices](https://kentcdodds.com/blog/effective-snapshot-testing)
