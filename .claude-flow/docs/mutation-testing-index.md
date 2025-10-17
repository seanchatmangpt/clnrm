# Mutation Testing Implementation - File Index

## Overview
Complete mutation testing infrastructure for the CLNRM project.

**Status**: ✅ Complete and Ready for Use
**Date**: 2025-10-16
**Agent**: Mutation Testing Specialist

## File Locations

### Configuration Files

| File | Location | Description |
|------|----------|-------------|
| Master Configuration | `/Users/sac/clnrm/docs/mutation-testing-config.toml` | Central mutation testing settings |
| Rust Configuration | `/Users/sac/clnrm/docs/cargo-mutants-config.toml` | cargo-mutants specific configuration |
| TypeScript Configuration | `/Users/sac/clnrm/examples/optimus-prime-platform/stryker.conf.json` | Stryker mutation testing configuration |

### Scripts

| File | Location | Description |
|------|----------|-------------|
| Main Test Runner | `/Users/sac/clnrm/scripts/run-mutation-tests.sh` | Execute all mutation tests (Rust + TypeScript) |

**Usage:**
```bash
# Run all tests
./scripts/run-mutation-tests.sh

# Rust only
./scripts/run-mutation-tests.sh --rust-only

# TypeScript only
./scripts/run-mutation-tests.sh --typescript-only
```

### Documentation

| File | Location | Description |
|------|----------|-------------|
| User Guide | `/Users/sac/clnrm/docs/MUTATION_TESTING_GUIDE.md` | Complete usage instructions and best practices |
| Analysis Report | `/Users/sac/clnrm/docs/mutation-testing-analysis.md` | Detailed project analysis and expected results |
| Recommendations | `/Users/sac/clnrm/docs/mutation-testing-recommendations.md` | Actionable improvement steps |
| Summary | `/Users/sac/clnrm/docs/MUTATION_TESTING_SUMMARY.md` | Executive summary and quick start |
| File Index | `/Users/sac/clnrm/docs/mutation-testing-index.md` | This document |

### Report Directories

| Directory | Location | Purpose |
|-----------|----------|---------|
| Reports Root | `/Users/sac/clnrm/docs/mutation-reports/` | All mutation testing reports |
| Rust Reports | `/Users/sac/clnrm/docs/mutation-reports/rust/` | cargo-mutants output |
| TypeScript Reports | `/Users/sac/clnrm/docs/mutation-reports/typescript/` | Stryker output |
| Report README | `/Users/sac/clnrm/docs/mutation-reports/README.md` | Report directory documentation |

## Quick Reference

### Rust Mutation Testing

**Install Tool:**
```bash
cargo install cargo-mutants --locked
```

**Run Tests:**
```bash
# All crates
cargo mutants

# Specific crate
cargo mutants -p clnrm-core

# Specific file
cargo mutants --file crates/clnrm-core/src/backend/testcontainer.rs

# With configuration
cargo mutants --config docs/cargo-mutants-config.toml
```

**View Reports:**
```bash
# Latest report
ls -t docs/mutation-reports/rust/*.json | head -1

# View with jq
jq '.mutation_score' docs/mutation-reports/rust/report_latest.json
```

### TypeScript Mutation Testing

**Install Tool:**
```bash
cd examples/optimus-prime-platform
npm install --save-dev \
    @stryker-mutator/core \
    @stryker-mutator/typescript-checker \
    @stryker-mutator/jest-runner
```

**Run Tests:**
```bash
cd examples/optimus-prime-platform
npx stryker run

# With specific config
npx stryker run --config stryker.conf.json
```

**View Reports:**
```bash
# HTML report
open docs/mutation-reports/typescript/optimus-prime-platform.html

# JSON report
cat docs/mutation-reports/typescript/optimus-prime-platform.json
```

## Configuration Summary

### Mutation Operators

**Rust (cargo-mutants):**
- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Logical: `&&`, `||`, `!`
- Relational: `<`, `>`, `<=`, `>=`, `==`, `!=`
- Conditional: if/else branches
- Return values
- Assignments

**TypeScript (Stryker):**
- ArithmeticOperator
- LogicalOperator
- ConditionalExpression
- EqualityOperator
- BooleanLiteral
- UnaryOperator
- UpdateOperator
- ArrowFunction
- MethodExpression

### Exclusions

**Rust:**
```toml
exclude_globs = [
    "*/tests/*",
    "*/examples/*",
    "**/bin.rs",
    "**/main.rs"
]

exclude_re = [
    "test_.*",
    ".*_test",
    "fmt",
    "clone",
    "default"
]
```

**TypeScript:**
```json
"mutate": [
    "src/**/*.ts",
    "src/**/*.tsx",
    "!src/**/*.test.ts",
    "!src/**/*.spec.ts",
    "!src/**/__tests__/**"
]
```

### Target Scores

| Component | Minimum | Target | Stretch |
|-----------|---------|--------|---------|
| Core Modules | 75% | 85% | 90% |
| Utilities | 65% | 75% | 80% |
| CLI Commands | 60% | 70% | 75% |
| Examples | 50% | 60% | 70% |

## Workflow Integration

### Local Development

```bash
# Before committing
./scripts/run-mutation-tests.sh --rust-only

# Review results
cat docs/mutation-reports/comprehensive_report_*.md | tail -1
```

### CI/CD Integration

```yaml
# Example GitHub Action
jobs:
  mutation-testing:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install cargo-mutants
        run: cargo install cargo-mutants --locked
      - name: Run mutation tests
        run: ./scripts/run-mutation-tests.sh
      - name: Check threshold
        run: |
          SCORE=$(jq '.mutation_score' docs/mutation-reports/rust/report_latest.json)
          if [ "$SCORE" -lt 70 ]; then exit 1; fi
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Get changed Rust files
CHANGED_RS=$(git diff --cached --name-only --diff-filter=ACM | grep '\.rs$')

if [ -n "$CHANGED_RS" ]; then
    echo "Running mutation tests on changed files..."
    for file in $CHANGED_RS; do
        cargo mutants --file "$file" || exit 1
    done
fi
```

## Key Features

### Parallel Execution
- Rust: 4 parallel jobs by default
- TypeScript: 4 concurrent test runners
- Configurable via config files

### Timeout Management
- Rust: 3.0x timeout multiplier
- TypeScript: 1.5x timeout factor
- Prevents false timeouts

### Comprehensive Reporting
- HTML (interactive visualization)
- JSON (machine-readable)
- Markdown (human-readable)
- Log files (detailed execution)

### Quality Gates
- Block: <50% mutation score
- Warn: 50-69%
- Pass: 70-79%
- Good: 80-89%
- Excellent: 90-100%

## Troubleshooting

### cargo-mutants not found
```bash
# Reinstall
cargo install cargo-mutants --locked

# Check PATH
which cargo-mutants

# Add to PATH if needed
export PATH="$HOME/.cargo/bin:$PATH"
```

### Tests timeout
```bash
# Increase timeout
cargo mutants --timeout-multiplier 5.0
```

### Out of memory
```bash
# Reduce parallelism
cargo mutants --jobs 2
```

### Too many mutants
```bash
# Test incrementally
cargo mutants --file path/to/file.rs

# Skip low-value mutations
cargo mutants --skip-calls-unsafe
```

## Support and Resources

### Documentation Links
- [Main Guide](/Users/sac/clnrm/docs/MUTATION_TESTING_GUIDE.md)
- [Analysis](/Users/sac/clnrm/docs/mutation-testing-analysis.md)
- [Recommendations](/Users/sac/clnrm/docs/mutation-testing-recommendations.md)
- [Summary](/Users/sac/clnrm/docs/MUTATION_TESTING_SUMMARY.md)

### External Resources
- [cargo-mutants Documentation](https://mutants.rs/)
- [Stryker Documentation](https://stryker-mutator.io/)
- [Mutation Testing Best Practices](https://en.wikipedia.org/wiki/Mutation_testing)

### Project Resources
- GitHub: https://github.com/seanchatmangpt/clnrm
- Issues: https://github.com/seanchatmangpt/clnrm/issues

## Next Steps

1. **Verify Installation**: `cargo mutants --version`
2. **Run Baseline Tests**: `./scripts/run-mutation-tests.sh`
3. **Review Reports**: Check `docs/mutation-reports/`
4. **Implement Improvements**: Follow recommendations document
5. **Integrate CI/CD**: Add to your pipeline

---

**Created**: 2025-10-16
**Last Updated**: 2025-10-16
**Version**: 1.0.0
**Status**: ✅ Complete
