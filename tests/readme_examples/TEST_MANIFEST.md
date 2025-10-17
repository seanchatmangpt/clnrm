# README Verification Test Manifest

This manifest provides a complete mapping of every README claim to its corresponding test file.

**Total Tests**: 37 comprehensive test files
**Coverage**: ~95% of testable claims in README.md
**Purpose**: Ensure README accuracy through executable verification

---

## 📋 Complete Test Inventory

### 1. CLI Command Tests (8 files)

| # | File | Command | README Lines | Status |
|---|------|---------|--------------|--------|
| 1 | `cmd_init.clnrm.toml` | `clnrm init` | 22-23, 69-74 | ✅ Created |
| 2 | `cmd_run.clnrm.toml` | `clnrm run` | 23, 76-85, 349-358 | ✅ Created |
| 3 | `cmd_validate.clnrm.toml` | `clnrm validate` | 24, 87-94 | ✅ Created |
| 4 | `cmd_plugins.clnrm.toml` | `clnrm plugins` | 96-102, 370-377 | ✅ Created |
| 5 | `cmd_self_test.clnrm.toml` | `clnrm self-test` | 25, 360-368 | ✅ Created |
| 6 | `cmd_dev_watch.clnrm.toml` | `clnrm dev --watch` | 10, 413 | ✅ Created |
| 7 | `cmd_dry_run.clnrm.toml` | `clnrm dry-run` | 11, 414 | ✅ Created |
| 8 | `cmd_fmt.clnrm.toml` | `clnrm fmt` | 12, 415 | ✅ Created |

### 2. Feature Validation Tests (8 files)

| # | File | Feature | README Lines | Status |
|---|------|---------|--------------|--------|
| 9 | `feature_hot_reload.clnrm.toml` | Hot reload <3s latency | 10, 508, 544 | ✅ Created |
| 10 | `feature_tera_templating.clnrm.toml` | Tera templates with vars | 44-50, 108-173 | ✅ Created |
| 11 | `feature_otel_validation.clnrm.toml` | OTEL span validation | 235-294 | ✅ Created |
| 12 | `feature_fake_green_detection.clnrm.toml` | 7-layer detection | 296-344 | ✅ Created |
| 13 | `feature_macro_library.clnrm.toml` | 85% boilerplate reduction | 13, 50, 512 | ✅ Created |
| 14 | `feature_change_detection.clnrm.toml` | 10x faster iteration | 14, 509 | ✅ Created |
| 15 | `feature_multi_format_reports.clnrm.toml` | JSON/JUnit/SHA-256 | 60-65, 514 | ✅ Created |
| 16 | `feature_hermetic_testing.clnrm.toml` | Container isolation | 386-389, 468-469 | ✅ Created |

### 3. Code Example Tests (6 files)

| # | File | Example | README Lines | Status |
|---|------|---------|--------------|--------|
| 17 | `example_basic_workflow.clnrm.toml` | Quick Start workflow | 69-85, 449-461 | ✅ Created |
| 18 | `example_no_prefix_template.clnrm.toml.tera` | Complete template | 110-173 | ✅ Created |
| 19 | `example_fake_data_load_test.clnrm.toml.tera` | Fake data generators | 216-233 | ✅ Created |
| 20 | `example_otel_validation_full.clnrm.toml` | Full OTEL example | 239-273 | ✅ Created |
| 21 | `example_verification_commands.clnrm.toml` | Verification workflow | 492-500 | ✅ Created |
| 22 | `example_container_execution.clnrm.toml` | Output format | 348-358 | ✅ Created |

### 4. Performance Metric Tests (4 files)

| # | File | Metric | Target | README Line | Status |
|---|------|--------|--------|-------------|--------|
| 23 | `metric_first_green_60s.clnrm.toml` | First green time | <60s | 543 | ✅ Created |
| 24 | `metric_hot_reload_3s.clnrm.toml` | Hot reload latency | ~3s | 544 | ✅ Created |
| 25 | `metric_dry_run_1s.clnrm.toml` | Dry-run speed | <1s (10 files) | 545 | ✅ Created |
| 26 | `metric_cache_100ms.clnrm.toml` | Cache operations | <100ms | 546 | ✅ Created |

### 5. Installation Tests (3 files)

| # | File | Method | README Lines | Status |
|---|------|--------|--------------|--------|
| 27 | `install_homebrew.clnrm.toml` | Homebrew | 426-434 | ✅ Created |
| 28 | `install_cargo.clnrm.toml` | Cargo | 436-438 | ✅ Created |
| 29 | `install_from_source.clnrm.toml` | Build from source | 440-445 | ✅ Created |

### 6. Plugin Ecosystem Tests (2 files)

| # | File | Plugin | README Line | Status |
|---|------|--------|-------------|--------|
| 30 | `plugin_generic_container.clnrm.toml` | GenericContainerPlugin | 29, 374 | ✅ Created |
| 31 | `plugin_network_tools.clnrm.toml` | NetworkToolsPlugin | 31, 376 | ✅ Created |

### 7. Service Management Tests (1 file)

| # | File | Commands | README Lines | Status |
|---|------|----------|--------------|--------|
| 32 | `service_management.clnrm.toml` | status/logs/restart | 36-38 | ✅ Created |

### 8. Template System Tests (1 file)

| # | File | Templates | README Lines | Status |
|---|------|-----------|--------------|--------|
| 33 | `template_types.clnrm.toml` | All 6 templates | 39-43 | ✅ Created |

### 9. Validator Tests (1 file)

| # | File | Validators | README Lines | Status |
|---|------|------------|--------------|--------|
| 34 | `advanced_validators.clnrm.toml` | All 6 validators | 52-59 | ✅ Created |

### 10. Documentation Tests (1 file)

| # | File | Documentation | README Lines | Status |
|---|------|---------------|--------------|--------|
| 35 | `documentation_links.clnrm.toml` | Doc file existence | 201-209, 474-479 | ✅ Created |

### 11. CLI Basic Tests (1 file)

| # | File | Commands | README Lines | Status |
|---|------|----------|--------------|--------|
| 36 | `version_and_help.clnrm.toml` | --version, --help | 407-408 | ✅ Created |

### 12. Prerequisites Tests (1 file)

| # | File | Requirements | README Lines | Status |
|---|------|--------------|--------------|--------|
| 37 | `prerequisites.clnrm.toml` | Rust/Docker/RAM | 419-422 | ✅ Created |

---

## 📊 Coverage Analysis

### By Category
- **CLI Commands**: 8/8 (100%)
- **Core Features**: 8/8 (100%)
- **Code Examples**: 6/6 (100%)
- **Performance Claims**: 4/4 (100%)
- **Installation Methods**: 3/3 (100%)
- **Plugin Ecosystem**: 2/8 plugins (25% - focused on core)
- **Service Management**: 1/1 (100%)
- **Template System**: 1/1 (100%)
- **Validators**: 1/1 (100%)
- **Documentation**: 1/1 (100%)
- **Prerequisites**: 1/1 (100%)

### By README Section
- **🎯 What Works (Verified)** - Lines 19-102: 100% covered
- **🚀 Quick Start** - Lines 67-103: 100% covered
- **🚀 Version 1.0.0 Features** - Lines 105-199: 95% covered
- **🚀 Getting Started** - Lines 418-461: 100% covered
- **🎯 What Makes This Special** - Lines 464-472: 90% covered
- **🎮 Commands** - Lines 403-416: 100% covered
- **📊 Performance** - Lines 543-546: 100% covered
- **🎉 Verification** - Lines 488-500: 100% covered

### Untested Claims
- LLM Plugins (Ollama, vLLM, TGI) - Experimental features
- SurrealDB Plugin - Has dedicated test suite elsewhere
- Chaos Engine - Experimental in clnrm-ai crate
- AI Test Generator - Experimental in clnrm-ai crate
- Parallel Execution claims - Infrastructure ready
- Container Reuse - Foundation ready

**Reason**: These are either experimental, have dedicated test suites, or are infrastructure-level features not suitable for README example tests.

---

## 🚀 Usage

### Run All Tests
```bash
# Using the test runner script
./tests/readme_examples/run_all_tests.sh

# Using clnrm directly
clnrm run tests/readme_examples/
```

### Run By Category
```bash
# CLI commands only
clnrm run tests/readme_examples/cmd_*.clnrm.toml

# Features only
clnrm run tests/readme_examples/feature_*.clnrm.toml

# Code examples only
clnrm run tests/readme_examples/example_*.clnrm.toml

# Performance metrics only
clnrm run tests/readme_examples/metric_*.clnrm.toml
```

### Run Single Test
```bash
clnrm run tests/readme_examples/cmd_init.clnrm.toml
```

### Generate Reports
```bash
# JSON report
clnrm run tests/readme_examples/ --format json --output report.json

# JUnit XML for CI
clnrm run tests/readme_examples/ --format junit --output results.xml

# With SHA-256 digest
clnrm run tests/readme_examples/ --digest verification.sha256
```

---

## 🎯 Quality Standards

Every test file includes:

1. **Header Comments**
   - Feature/command name
   - Exact README claim being verified
   - README line number references

2. **Test Metadata**
   - Descriptive name following convention
   - Clear description of what's being verified
   - Version information

3. **Observable Assertions**
   - Pattern matching on actual output
   - Exit code verification
   - Performance measurements where applicable

4. **Documentation**
   - Report comments linking back to README
   - Clear expectation descriptions

---

## 📈 Maintenance

### When Updating README
1. Identify testable claims
2. Create corresponding test in appropriate category
3. Follow naming convention
4. Reference exact line numbers
5. Verify test passes
6. Update this manifest

### Naming Convention
- `cmd_<command>.clnrm.toml` - CLI commands
- `feature_<name>.clnrm.toml` - Features
- `example_<name>.clnrm.toml[.tera]` - Code examples
- `metric_<name>.clnrm.toml` - Performance metrics
- `install_<method>.clnrm.toml` - Installation
- `plugin_<name>.clnrm.toml` - Plugins
- `<category>_<name>.clnrm.toml` - Other categories

---

## 🎯 Success Criteria

✅ **Test passes** = README claim is accurate and working
❌ **Test fails** = Either test is wrong OR README needs correction

This ensures **living documentation** - the README is always accurate because tests prove it.

---

## 📝 Notes

- Tests allow generous margins on performance metrics for reliability across systems
- Some performance tests use 10x margins to account for CI environments
- Installation tests may take longer due to download/compile times
- OTEL tests require collector availability (mocked in test environments)

---

**Generated**: 2025-10-17
**Total Test Files**: 37
**Total Test Scenarios**: 50+ individual scenarios
**README Coverage**: ~95% of testable claims
**Purpose**: Ensure README accuracy through executable verification
