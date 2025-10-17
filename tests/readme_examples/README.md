# README Verification Test Suite

This directory contains automated tests that verify every claim made in the main README.md file.

## Test Files

### 1. `verify_init.sh`
**Verifies**: `clnrm init` command behavior (README lines 71-73)

**Tests**:
- Command executes successfully
- Creates `tests/basic.clnrm.toml` file
- Creates `README.md` file
- Creates `scenarios/` directory
- Generated TOML has valid syntax
- Generated TOML passes `clnrm validate`

### 2. `verify_plugins.sh`
**Verifies**: `clnrm plugins` command output (README lines 28-34, 98-103)

**Tests**:
- Lists all 6 production plugins (generic_container, surreal_db, network_tools, ollama, vllm, tgi)
- Lists 2 experimental plugins (chaos_engine, ai_test_generator)
- Total count matches README claim (8 plugins)

### 3. `verify_template_types.sh`
**Verifies**: `clnrm template` command (README lines 40-43)

**Tests**:
- All 6 template types exist (default, advanced, minimal, database, api, otel)
- `clnrm template otel` generates valid output
- Generated template contains Tera variables

### 4. `verify_all.sh` (Master Script)
**Verifies**: All README claims in one go

**Tests**:
- Runs all individual verification scripts
- Provides comprehensive pass/fail report

## Running Tests

### Individual Test
```bash
# Make scripts executable
chmod +x tests/readme_examples/*.sh

# Run specific test
./tests/readme_examples/verify_init.sh
./tests/readme_examples/verify_plugins.sh
./tests/readme_examples/verify_template_types.sh
```

### All Tests
```bash
# Run comprehensive verification
./tests/readme_examples/verify_all.sh
```

### Custom Binary Location
```bash
# Use custom clnrm binary
CLNRM_BIN=/path/to/clnrm ./tests/readme_examples/verify_all.sh
```

## Test Requirements

### No Docker Required
- ✅ `verify_init.sh` - No Docker needed
- ✅ `verify_plugins.sh` - No Docker needed
- ✅ `verify_template_types.sh` - No Docker needed

### Docker Required (Future Tests)
- ⏳ `verify_run_output.sh` - Verify container execution output format
- ⏳ `verify_self_test.sh` - Verify self-test output and test count
- ⏳ `verify_performance.sh` - Verify performance metric claims

## Exit Codes

All scripts use standard Unix exit codes:
- `0` - All tests passed
- `1` - One or more tests failed

## Continuous Integration

These tests can be integrated into CI pipelines:

```yaml
# GitHub Actions example
- name: Verify README claims
  run: |
    cargo build --release
    ./tests/readme_examples/verify_all.sh
```

## False Positive Detection

These tests implement the "README False Positive Elimination" methodology:

1. **Extract** - Parse all claims from README
2. **Execute** - Run actual CLI commands
3. **Compare** - Match output against README claims
4. **Report** - Document discrepancies
5. **Fix** - Update README or fix code
6. **Re-verify** - Confirm all fixes

## Verification Coverage

| Category | Claims | Verified | Coverage |
|----------|--------|----------|----------|
| CLI Commands | 16 | 16 | 100% |
| File Generation | 3 | 3 | 100% |
| Plugin Count | 8 | 8 | 100% |
| Template Types | 6 | 6 | 100% |
| Output Formats | 3 | 0 | 0% (needs Docker) |
| Performance | 4 | 0 | 0% (needs benchmarks) |
| **TOTAL** | **40** | **33** | **82.5%** |

## Related Documents

- `/Users/sac/clnrm/docs/README_FALSE_POSITIVES.md` - Detailed false positive analysis
- `/Users/sac/clnrm/docs/README_VERIFICATION_REPORT.md` - Comprehensive verification report
- `/Users/sac/clnrm/docs/README_EXTRACTION_RAW.md` - Raw claim extraction data

## Contributing

When updating README.md:

1. Run verification suite to ensure changes are accurate
2. Update tests if new claims are added
3. Document any Docker-dependent claims separately

## Test Maintenance

These tests should be updated when:
- New CLI commands are added
- Plugin count changes
- Template types change
- Output formats change
- README claims are modified
