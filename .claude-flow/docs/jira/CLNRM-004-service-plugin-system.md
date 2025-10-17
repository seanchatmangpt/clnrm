# CLNRM-004: Service Plugin System

## Summary
Implement the service plugin system that allows extensible service management. The MockDatabasePlugin exists but all methods return "not implemented" errors.

## Status
🟡 **IN PROGRESS** - Plugin architecture exists, needs implementation

## Priority
P1 - High (enables service management)

## Components Affected
- `src/cleanroom.rs` - Service plugin implementation
- `src/services/` - Service plugin implementations (if exists)
- `src/backend/` - Backend service integration

## Current State
The system has:
- ✅ **ServicePlugin trait** - Well-defined plugin interface
- ✅ **ServiceRegistry** - Plugin registration and management
- ✅ **ServiceHandle** - Service instance management
- ✅ **Health status system** - Health checking infrastructure
- ❌ **MockDatabasePlugin methods not implemented**
- ❌ **No actual service implementations**
- ❌ **No plugin discovery/loading system**

## Implementation Plan

### Phase 1: MockDatabasePlugin Implementation
1. **Implement `start()` method**
   - Create actual database container using testcontainers
   - Return proper ServiceHandle with container info
   - Handle startup errors and timeouts

2. **Implement `stop()` method**
   - Stop and cleanup database container
   - Release resources properly
   - Handle cleanup errors gracefully

3. **Implement `health_check()` method**
   - Check if database is responsive
   - Verify database connectivity
   - Return proper HealthStatus

### Phase 2: Service Plugin Infrastructure
1. **Implement plugin discovery**
   - Dynamic plugin loading from files/directories
   - Plugin registry with version management
   - Plugin dependency resolution

2. **Implement service lifecycle management**
   - Service startup orchestration
   - Dependency order resolution
   - Graceful shutdown handling

### Phase 3: Real Service Implementations
1. **Create PostgreSQL plugin**
   - Full PostgreSQL container management
   - Database initialization and migration support
   - Connection pooling integration

2. **Create Redis plugin**
   - Redis container management
   - Configuration and persistence
   - Cluster support if needed

3. **Create web server plugin**
   - Generic web server container
   - Health check endpoints
   - Log aggregation

## Dependencies
- CLNRM-005: Container Backend Integration (for actual container management)

## Acceptance Criteria
- [ ] MockDatabasePlugin starts/stops actual database containers
- [ ] Service health checks return accurate status
- [ ] Plugin registry can load and manage multiple services
- [ ] Service dependencies are properly resolved
- [ ] Service logs and metrics are properly collected

## Testing
- Unit tests for each service plugin method
- Integration tests for service lifecycle
- End-to-end tests for multi-service scenarios
- Performance tests for service startup/shutdown

## Related Issues
- CLNRM-002: CleanroomEnvironment Implementation (uses service plugins)
- CLNRM-005: Container Backend Integration (required for containers)
