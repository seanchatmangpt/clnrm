# CLNRM-003: TOML Configuration System

## Summary
Implement the TOML configuration file parsing and validation system. Currently, the CLI has basic file reading but no proper TOML parsing or schema validation.

## Status
🟡 **IN PROGRESS** - Basic file reading exists, needs TOML parsing

## Priority
P1 - High (blocks test execution)

## Components Affected
- `src/cli.rs` - Configuration loading in CLI
- `src/utils.rs` - TOML parsing utilities
- `src/cleanroom.rs` - Configuration loading in environment
- `src/config.rs` - Configuration structures (if exists)

## Current State
The system has:
- ✅ **Basic file reading** in CLI `run_tests()`
- ✅ **TOML dependency** in Cargo.toml
- ❌ **No TOML parsing** - `parse_toml_config()` returns null
- ❌ **No schema validation** - no validation of config structure
- ❌ **No configuration loading** in CleanroomEnvironment

## Implementation Plan

### Phase 1: TOML Parsing
1. **Implement `parse_toml_config()` in utils.rs**
   - Parse TOML content into structured data
   - Handle parsing errors gracefully
   - Return proper data structures

2. **Create configuration data structures**
   - Define TOML schema for test files
   - Define configuration for cleanroom settings
   - Handle optional vs required fields

### Phase 2: Schema Validation
1. **Implement configuration validation**
   - Validate TOML structure against schema
   - Validate field types and ranges
   - Provide meaningful validation errors

2. **Implement policy validation**
   - Validate security policies from TOML
   - Validate resource limits
   - Validate execution policies

### Phase 3: Configuration Loading
1. **Implement config loading in CleanroomEnvironment**
   - Load configuration from files
   - Merge with environment variables
   - Merge with CLI arguments

2. **Implement config loading in CLI**
   - Load config files for test execution
   - Handle multiple config files
   - Validate configs before execution

## Dependencies
- CLNRM-002: CleanroomEnvironment Implementation (for config loading)

## Acceptance Criteria
- [ ] `parse_toml_config()` correctly parses valid TOML
- [ ] Configuration validation catches invalid TOML structures
- [ ] CleanroomEnvironment loads configuration from TOML files
- [ ] CLI validates config files before execution
- [ ] Configuration supports all policy and execution settings

## Testing
- Unit tests for TOML parsing with various inputs
- Integration tests for configuration loading
- Validation tests for schema compliance
- Error handling tests for malformed configs

## Related Issues
- CLNRM-001: CLI Implementation (uses config system)
- CLNRM-002: CleanroomEnvironment Implementation (loads config)
