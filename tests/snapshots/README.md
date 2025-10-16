# Snapshot Testing Infrastructure

## Quick Start

### Generate All Baseline Snapshots
```bash
./tests/snapshots/baseline_generator.sh
```

### Run Snapshot Tests

#### Rust Tests
```bash
# Run all tests with snapshot validation
cargo test

# Review snapshot changes interactively
cargo insta review

# Accept all snapshot changes
cargo insta accept

# Reject all snapshot changes
cargo insta reject
```

#### CLI Tests
```bash
# Run CLI snapshot tests
cargo test --test cli_output_snapshots
```

#### Data Structure Tests
```bash
# Run data structure snapshot tests
cargo test --test data_structure_snapshots
```

#### UI Tests
```bash
cd examples/optimus-prime-platform

# Run all UI snapshot tests
npm test

# Update snapshots
npm test -- -u

# Watch mode
npm test -- --watch
```

## Directory Structure

```
tests/snapshots/
├── rust/                              # Rust code snapshots
│   ├── snapshot_infrastructure.rs     # Core infrastructure
│   ├── scenario_snapshots.rs         # Scenario tests
│   └── __snapshots__/                # Generated snapshots
├── cli/                               # CLI output snapshots
│   ├── cli_output_snapshots.rs       # CLI tests
│   └── __snapshots__/
├── data/                              # Data structure snapshots
│   ├── data_structure_snapshots.rs   # Data tests
│   └── __snapshots__/
├── ui/                                # UI regression tests
│   ├── visual_regression.rs          # Visual regression infrastructure
│   └── __snapshots__/
├── baseline_generator.sh              # Baseline generation script
├── SNAPSHOT_WORKFLOW.md              # Detailed workflow documentation
└── README.md                         # This file
```

## Snapshot Infrastructure Components

### 1. Rust Snapshot Testing (`snapshot_infrastructure.rs`)

- **SnapshotMetadata**: Track snapshot changes and reviews
- **SnapshotDiffEngine**: Smart diff algorithms
- **SnapshotManager**: Snapshot lifecycle management

### 2. CLI Output Snapshots (`cli_output_snapshots.rs`)

- Normalized timestamps and paths
- Exit code tracking
- Stdout/stderr separation

### 3. Data Structure Snapshots (`data_structure_snapshots.rs`)

- JSON/YAML snapshot comparisons
- Configuration validation
- Schema evolution tracking

### 4. Visual Regression (`visual_regression.rs`)

- Screenshot comparison
- Viewport testing
- Region-based ignoring
- Similarity scoring

## Integration with Testing Swarm

### Coordination Hooks

```bash
# Before work
npx claude-flow@alpha hooks pre-task --description "Snapshot Testing"

# After snapshot changes
npx claude-flow@alpha hooks post-edit \
  --file "snapshot.snap" \
  --memory-key "swarm/snapshot-testing/baseline"

# Notify swarm
npx claude-flow@alpha hooks notify \
  --message "Snapshot tests configured"

# After completion
npx claude-flow@alpha hooks post-task --task-id "snapshot-testing"
```

### Memory Storage

Snapshots and metadata are stored in swarm memory:
- `swarm/snapshot-testing/config` - Configuration
- `swarm/snapshot-testing/baseline` - Baseline snapshots
- `swarm/snapshot-testing/review` - Review status

## Best Practices

### DO ✅
- Review all snapshot changes before accepting
- Normalize dynamic values (timestamps, IDs)
- Keep snapshots small and focused
- Document breaking changes
- Version control snapshots with code
- Use descriptive snapshot names

### DON'T ❌
- Accept snapshots without review
- Include sensitive data in snapshots
- Mix multiple concerns in one snapshot
- Forget to update documentation
- Skip snapshot review in CI/CD

## Snapshot Review Workflow

1. **Run Tests**: Identify snapshot failures
2. **Review Changes**: Use interactive tools
3. **Approve/Reject**: Make informed decision
4. **Document**: Add change description
5. **Commit**: Include snapshots in PR

## Similarity Scoring

- `1.0` - Identical (no changes)
- `0.9-0.99` - Minor changes (safe to approve)
- `0.7-0.89` - Moderate changes (review carefully)
- `<0.7` - Major changes (requires detailed review)

## CI/CD Integration

```yaml
# GitHub Actions example
- name: Run Snapshot Tests
  run: |
    cargo test
    cargo insta test --check
    cd examples/optimus-prime-platform
    npm test -- --ci
```

## Troubleshooting

### Snapshots Not Matching
- Check for non-deterministic values
- Verify normalization functions
- Review environment-specific output

### Snapshots Too Large
- Break into smaller tests
- Use snapshot fragments
- Filter unnecessary data

### Frequent Updates
- Review test stability
- Improve normalization
- Consider structural testing

## Resources

- [Snapshot Workflow Documentation](SNAPSHOT_WORKFLOW.md)
- [insta Documentation](https://insta.rs/)
- [Jest Snapshot Testing](https://jestjs.io/docs/snapshot-testing)

## Support

For issues or questions, check:
- [GitHub Issues](https://github.com/seanchatmangpt/clnrm/issues)
- [Testing Documentation](../../docs/testing/)
