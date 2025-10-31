# Testing Documentation Index

Complete index of clnrm testing documentation.

## Live-Check Test Suite

The comprehensive test harness for Weaver `registry live-check` integration.

### Quick Start
- **[Executive Summary](LIVE_CHECK_EXECUTIVE_SUMMARY.md)** - High-level overview and mission completion
- **[Quick Reference](../../LIVE-CHECK-TEST-SUITE-SUMMARY.md)** - One-page quick reference

### For Users
- **[Test Suite README](../../scripts/tests/README.md)** - Complete test documentation
- **[Developer Guide](LIVE_CHECK_TEST_GUIDE.md)** - Usage examples and troubleshooting

### For Contributors
- **[Deliverables](LIVE_CHECK_TEST_SUITE_DELIVERABLES.md)** - Complete technical documentation
- **[Architecture Diagrams](../architecture/)** - Visual documentation
  - [Test Suite Architecture](../architecture/live-check-test-architecture.puml)
  - [Validation Hierarchy](../architecture/validation-hierarchy.puml)

### Quick Commands
```bash
# Validate setup
./scripts/tests/validate_test_setup.sh

# Quick smoke test
./scripts/tests/run_test_subset.sh --quick

# Full suite
./scripts/tests/test_live_check_comprehensive.sh
```

## Other Testing Documentation

- **[TDD Specialist Deliverables](TDD_SPECIALIST_DELIVERABLES.md)** - London TDD implementation

## Validation Hierarchy

clnrm uses a 4-level validation hierarchy with Weaver as the source of truth:

```
Level 1: Schema Definition (weaver registry check)
    ↓
Level 2: Live-Check Capabilities (Test Suite)
    ↓
Level 3: Runtime Telemetry (weaver live-check + real data)
    ↓
Level 4: Traditional Tests (cargo test)
```

**Key Principle**: Only Weaver validation (Levels 1-3) proves features work. Traditional tests (Level 4) provide supporting evidence but can have false positives.

## Test Categories

### By Purpose
- **Infrastructure Tests**: Validate live-check tooling (Level 2)
- **Integration Tests**: Validate runtime telemetry (Level 3)
- **Unit Tests**: Validate code behavior (Level 4)

### By Duration
- **Quick** (~15s): Fast smoke tests
- **Basic** (~30s): Core functionality
- **Advanced** (~60s): Complex scenarios
- **Full** (~2-3min): Comprehensive validation

## CI/CD Integration

- **[GitHub Actions Workflow](../../.github/workflows/weaver-live-check-tests.yml)**
  - 6 jobs: validate, basic, advanced, concurrent, full, report
  - Parallel execution for fast feedback
  - Artifact upload for debugging

## Documentation Standards

All testing documentation follows these standards:
- Clear executive summaries
- Quick start sections
- Detailed usage examples
- Troubleshooting guides
- FAQ sections
- Integration examples

## Related Documentation

### Weaver Integration
- [Weaver User Guide](../WEAVER_USER_GUIDE.md)
- [Running Weaver Validation](../RUNNING_WEAVER_VALIDATION.md)
- [Weaver Integration Design](../architecture/WEAVER_INTEGRATION_DESIGN.md)

### Core Documentation
- [Main README](../../README.md)
- [CLAUDE.md](../../CLAUDE.md) - Project instructions
- [CHANGELOG](../../CHANGELOG.md)

## Support

For issues or questions:
1. Check relevant documentation above
2. Review troubleshooting sections
3. Check test logs in `validation_output/`
4. File issue: https://github.com/seanchatmangpt/clnrm/issues

---

**Last Updated**: 2025-10-30  
**Status**: ✅ All documentation current and validated
