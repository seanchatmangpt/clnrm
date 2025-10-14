# CLNRM-001: CLI Implementation

## Summary
Implement the core CLI functionality that is currently stubbed out. The CLI interface exists but most commands return "not implemented" errors.

## Status
🔴 **BLOCKED** - Waiting for framework self-testing infrastructure

## Priority
P0 - Critical (blocks user experience)

## Components Affected
- `src/cli.rs` - Main CLI implementation
- `src/bin/clnrm.rs` - Binary entry point

## Current State
The CLI has a comprehensive command structure with:
- `run` - Execute tests with parallel/watch options
- `validate` - Validate TOML config files
- `init` - Initialize new test projects
- `plugins` - List available plugins
- `services` - Service management (status/logs/restart)

All commands currently return "not implemented" errors except basic file validation.

## Implementation Plan

### Phase 1: Basic Test Execution
1. **Implement `run_tests()` function**
   - Parse TOML test files
   - Execute scenario DSL from TOML
   - Handle parallel execution
   - Return proper test results

2. **Connect CLI to Scenario Engine**
   - Map CLI arguments to scenario configuration
   - Execute scenarios from TOML files
   - Handle test discovery and execution

### Phase 2: Configuration Management
1. **Implement `validate_config()`**
   - TOML syntax validation
   - Schema validation against cleanroom config format
   - Policy validation

2. **Implement `init_project()`**
   - Create project template
   - Generate example TOML files
   - Set up directory structure

### Phase 3: Service Management
1. **Implement service status checking**
   - Query running services
   - Display service health
   - Show resource usage

2. **Implement service lifecycle**
   - Start/stop services via CLI
   - Service log viewing
   - Service restart functionality

## Dependencies
- CLNRM-002: CleanroomEnvironment Implementation
- CLNRM-003: TOML Configuration System
- CLNRM-004: Service Plugin System

## Acceptance Criteria
- [ ] `clnrm run <file>` executes tests successfully
- [ ] `clnrm validate <file>` validates TOML syntax and schema
- [ ] `clnrm init <project>` creates valid project structure
- [ ] `clnrm services status` shows service information
- [ ] All CLI commands provide meaningful error messages

## Testing
- Framework self-testing validates CLI functionality
- Integration tests for CLI commands
- End-to-end CLI workflow tests

## Related Issues
- CLNRM-002: CleanroomEnvironment Implementation
- CLNRM-003: TOML Configuration System
- CLNRM-004: Service Plugin System
