# Advanced Features Examples

This directory contains advanced features examples that demonstrate the "fail then fix" approach to prove they actually work.

## 🎯 Purpose

These examples validate advanced Cleanroom features by:
1. **Intentionally breaking them** to show they don't work
2. **Fixing them** to prove they work correctly
3. **Providing copy-pasteable examples** for users

## 📁 Files

### Working Examples

- **`simple-test.toml`** - Minimal working example
- **`hermetic-isolation.toml`** - Demonstrates hermetic isolation capabilities
- **`concurrent-execution.toml`** - Demonstrates concurrent execution features

### Test Scripts

- **`test-fail-then-fix.sh`** - Comprehensive test script that validates all examples

## 🚀 Usage

### Run Individual Examples

```bash
# Simple test
clnrm run examples/advanced-features/simple-test.toml

# Hermetic isolation test
clnrm run examples/advanced-features/hermetic-isolation.toml

# Concurrent execution test
clnrm run examples/advanced-features/concurrent-execution.toml
```

### Validate All Examples

```bash
# Validate all TOML files
clnrm validate examples/advanced-features/

# Run comprehensive test
./examples/advanced-features/test-fail-then-fix.sh
```

## 🔧 What Was Fixed

### The Problem
The original examples were using the wrong TOML format:
- ❌ `[test.metadata]` instead of `[test]`
- ❌ Services as Vec instead of HashMap
- ❌ CLI validation using wrong config structure

### The Solution
Fixed the CLI to use consistent config structures:
- ✅ Updated validation to use CLI's own `TestConfig` structure
- ✅ Updated test execution to use CLI's own `TestConfig` structure
- ✅ All examples now use correct TOML format

## 📊 Validation Results

```
🎉 All advanced-features examples are working!

📊 Summary:
  ✅ Simple Test: Working
  ✅ Hermetic Isolation: Working
  ✅ Concurrent Execution: Working
  ✅ TOML Validation: Working
```

## 🎯 README Claims Validated

These examples validate the following README claims:

- **🔒 Hermetic Isolation ✅** - Complete isolation from host system and other tests
- **⚡ Concurrent Execution** - Parallel test execution capabilities
- **📝 TOML Configuration** - Declarative test configuration
- **🛠️ Professional CLI** - Command-line interface functionality

## 🔗 Copy-Paste Ready

All examples are ready for users to copy and paste:

1. **Copy the TOML file** to your project
2. **Run with CLI**: `clnrm run your-test.toml`
3. **See results** immediately

## 🧪 Fail Then Fix Approach

This directory demonstrates the "fail then fix" approach:

1. **Break intentionally** - Show examples don't work
2. **Fix systematically** - Address root causes
3. **Prove they work** - Validate with real tests
4. **Document the process** - Show what was fixed

This approach ensures examples are not just "toy examples" but real, working code that users can rely on.

## 📚 Related Documentation

- [Main README](../../README.md) - Framework overview
- [Examples README](../README.md) - All examples index
- [CLI Documentation](../../docs/cli.md) - CLI usage guide
