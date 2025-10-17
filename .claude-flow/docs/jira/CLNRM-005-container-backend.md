# CLNRM-005: Container Backend Integration

## Summary
Implement proper container backend integration using testcontainers-rs. Currently, the TestcontainerBackend exists but has limited functionality and the TODO comment indicates volume mounting needs implementation.

## Status
🟡 **IN PROGRESS** - Backend exists, needs feature completion

## Priority
P1 - High (enables containerized testing)

## Components Affected
- `src/backend/testcontainer.rs` - Main backend implementation
- `src/backend/mod.rs` - Backend trait and types
- `src/containers.rs` - Container wrapper implementations

## Current State
The system has:
- ✅ **TestcontainerBackend** - Basic structure and initialization
- ✅ **Backend trait** - Well-defined interface
- ✅ **GenericContainer** - Basic container wrapper
- ✅ **Command execution** - Basic command running capability
- ❌ **Volume mounting not implemented** (TODO comment)
- ❌ **Advanced container features** not implemented
- ❌ **Resource monitoring** in containers

## Implementation Plan

### Phase 1: Volume Mounting Implementation
1. **Implement volume mounting in TestcontainerBackend**
   - Add volume configuration to Cmd struct
   - Implement volume mounting with testcontainers API
   - Handle volume cleanup and resource management

2. **Enhance container configuration**
   - Support for custom volumes
   - Host path to container path mapping
   - Read-only vs read-write volumes

### Phase 2: Advanced Container Features
1. **Implement container networking**
   - Custom network creation
   - Port mapping and exposure
   - Network isolation controls

2. **Implement resource limits**
   - CPU and memory limits per container
   - Disk space limits
   - Network bandwidth limits

3. **Implement health checks**
   - Container readiness checks
   - Health endpoint monitoring
   - Automatic restart on failure

### Phase 3: Container Lifecycle Management
1. **Implement container reuse optimization**
   - Singleton pattern for container reuse
   - Container registry for tracking
   - Automatic cleanup on test completion

2. **Implement container dependency management**
   - Service dependency resolution
   - Startup order management
   - Health check coordination

## Dependencies
- None (foundational component)

## Acceptance Criteria
- [ ] Volume mounting works correctly with testcontainers API
- [ ] Container networking supports custom networks and port mapping
- [ ] Resource limits are properly enforced
- [ ] Health checks work for container readiness
- [ ] Container reuse pattern provides performance benefits
- [ ] Container cleanup works properly without resource leaks

## Testing
- Unit tests for each backend method
- Integration tests for container lifecycle
- Performance tests for container reuse
- Resource usage tests for limits enforcement
- Network isolation tests

## Related Issues
- CLNRM-002: CleanroomEnvironment Implementation (uses backend)
- CLNRM-004: Service Plugin System (uses containers)
