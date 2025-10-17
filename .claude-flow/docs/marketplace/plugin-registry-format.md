# Plugin Registry Format Specification

## Overview

The CLNRM Plugin Registry uses a JSON-based format for storing plugin metadata, installation records, and community information. This document describes the complete registry format specification.

## Registry Structure

### Top-Level Registry Format

```json
{
  "version": "1.0",
  "updated_at": "2025-10-16T12:00:00Z",
  "installed": {
    "plugin-name": { /* PluginMetadata */ }
  },
  "available": {
    "plugin-name": { /* PluginMetadata */ }
  },
  "installations": {
    "plugin-name": { /* InstallationRecord */ }
  }
}
```

### PluginMetadata Format

```json
{
  "name": "string",
  "version": "string (semver)",
  "description": "string",
  "author": "string",
  "license": "string (SPDX identifier)",
  "homepage": "string (optional URL)",
  "repository": "string (optional URL)",
  "documentation": "string (optional URL)",
  "keywords": ["string"],
  "capabilities": [
    {
      "name": "string",
      "category": "database|cache|message-queue|web|ai-ml|storage|observability|security|testing|custom:name",
      "description": "string",
      "config_schema": {
        "field_name": "type"
      }
    }
  ],
  "dependencies": [
    {
      "name": "string",
      "version_constraint": "string (semver range)",
      "optional": boolean
    }
  ],
  "min_cleanroom_version": "string (semver)",
  "community": {
    "average_rating": number,
    "rating_count": number,
    "download_count": number,
    "created_at": "ISO 8601 timestamp",
    "updated_at": "ISO 8601 timestamp",
    "reviews": ["string"]
  },
  "custom_fields": {
    "key": "value"
  }
}
```

### InstallationRecord Format

```json
{
  "plugin_name": "string",
  "version": "string (semver)",
  "installed_at": "ISO 8601 timestamp",
  "install_path": "string (absolute path)",
  "active": boolean
}
```

## Plugin Categories

### Supported Categories

| Category | Description | Example Plugins |
|----------|-------------|-----------------|
| `database` | Database connectivity and testing | PostgreSQL, MySQL, MongoDB |
| `cache` | Caching and session management | Redis, Memcached |
| `message-queue` | Message queuing and streaming | Kafka, RabbitMQ, NATS |
| `web` | Web servers and API testing | HTTP servers, REST APIs |
| `ai-ml` | AI/ML model testing | Ollama, vLLM, TGI |
| `storage` | Storage and file systems | S3, MinIO, local storage |
| `observability` | Monitoring and tracing | Prometheus, Jaeger |
| `security` | Security and authentication | OAuth, JWT validators |
| `testing` | Testing utilities | Test generators, mocks |
| `custom:<name>` | Custom category | User-defined |

## Capability Configuration Schema

### Standard Schemas

#### Database Capability
```json
{
  "name": "database",
  "category": "database",
  "description": "Database connectivity",
  "config_schema": {
    "host": "string",
    "port": "integer",
    "database": "string",
    "username": "string (optional)",
    "password": "string (optional)"
  }
}
```

#### Cache Capability
```json
{
  "name": "cache",
  "category": "cache",
  "description": "Caching system",
  "config_schema": {
    "host": "string",
    "port": "integer",
    "ttl": "integer (optional)"
  }
}
```

#### AI/ML Capability
```json
{
  "name": "ai_ml",
  "category": "ai-ml",
  "description": "AI/ML model integration",
  "config_schema": {
    "endpoint": "string",
    "model": "string",
    "api_key": "string (optional)"
  }
}
```

## Version Constraints

### Semver Ranges

Plugin dependencies support standard semantic versioning constraints:

- `^1.0.0` - Compatible with version 1.x.x
- `~1.2.3` - Compatible with version 1.2.x
- `>=1.0.0 <2.0.0` - Explicit range
- `1.0.0` - Exact version
- `*` - Any version (not recommended)

## File Locations

### Default Paths

| Platform | Cache Directory | Install Directory |
|----------|----------------|-------------------|
| Linux | `/tmp/cleanroom/marketplace` | `./plugins` |
| macOS | `/var/folders/.../cleanroom/marketplace` | `./plugins` |
| Windows | `%TEMP%\cleanroom\marketplace` | `.\plugins` |

### Custom Paths

Override default paths using environment variables:

```bash
export CLNRM_CACHE_DIR=/custom/cache/path
export CLNRM_PLUGIN_DIR=/custom/plugin/path
```

## Security and Validation

### Required Fields

The following fields are required for all plugins:

- `name`: Unique identifier
- `version`: Semantic version
- `description`: Human-readable description
- `author`: Author name or organization
- `capabilities`: At least one capability

### Validation Rules

1. **Name**: Must be lowercase, alphanumeric with hyphens
2. **Version**: Must be valid semver (e.g., 1.0.0)
3. **License**: Should be valid SPDX identifier
4. **Dependencies**: Version constraints must be valid semver ranges
5. **URLs**: Homepage, repository, and documentation must be valid URLs

## Remote Registry Protocol

### Registry Endpoints

Remote registries must support the following HTTP endpoints:

```
GET  /api/v1/plugins           # List all plugins
GET  /api/v1/plugins/:name     # Get specific plugin
GET  /api/v1/search?q=query    # Search plugins
POST /api/v1/plugins           # Publish plugin (authenticated)
PUT  /api/v1/plugins/:name     # Update plugin (authenticated)
```

### Authentication

Remote registries use Bearer token authentication:

```
Authorization: Bearer <token>
```

## Example Plugin Manifest

### Complete Example

```json
{
  "name": "postgres-testing-plugin",
  "version": "1.2.3",
  "description": "Production-ready PostgreSQL testing plugin with advanced features",
  "author": "Cleanroom Core Team",
  "license": "MIT",
  "homepage": "https://plugins.cleanroom.dev/postgres",
  "repository": "https://github.com/cleanroom/postgres-plugin",
  "documentation": "https://docs.cleanroom.dev/plugins/postgres",
  "keywords": ["database", "postgresql", "sql", "testing"],
  "capabilities": [
    {
      "name": "database",
      "category": "database",
      "description": "PostgreSQL database connectivity and testing",
      "config_schema": {
        "host": "string",
        "port": "integer",
        "database": "string",
        "username": "string",
        "password": "string",
        "ssl_mode": "string (optional)"
      }
    }
  ],
  "dependencies": [
    {
      "name": "database-commons",
      "version_constraint": "^2.0.0",
      "optional": false
    }
  ],
  "min_cleanroom_version": "0.3.0",
  "community": {
    "average_rating": 4.8,
    "rating_count": 156,
    "download_count": 5420,
    "created_at": "2025-01-15T10:00:00Z",
    "updated_at": "2025-10-15T08:30:00Z",
    "reviews": [
      "Excellent plugin for PostgreSQL testing!",
      "Works perfectly with our CI/CD pipeline"
    ]
  },
  "custom_fields": {
    "support_email": "support@cleanroom.dev",
    "discord": "https://discord.gg/cleanroom"
  }
}
```

## Migration and Compatibility

### Version 1.0 (Current)

- Initial registry format
- Support for all core features

### Future Versions

The registry format will maintain backward compatibility through version negotiation:

```json
{
  "version": "2.0",
  "compatibility": ["1.0", "1.1", "1.2"]
}
```

## Best Practices

### Plugin Naming

- Use lowercase letters and hyphens
- Be descriptive but concise
- Avoid generic names
- Example: `postgres-testing-plugin` (good) vs `plugin` (bad)

### Version Management

- Follow semantic versioning strictly
- Document breaking changes in CHANGELOG
- Use pre-release versions for testing (e.g., `1.0.0-beta.1`)

### Metadata Quality

- Write clear, comprehensive descriptions
- Provide all optional URLs (homepage, repo, docs)
- Use relevant keywords for discoverability
- Keep dependencies minimal

### Security

- Never include secrets in metadata
- Use environment variables for sensitive config
- Validate all inputs in plugin code
- Follow security best practices

## See Also

- [Plugin Developer Guide](./plugin-developer-guide.md)
- [Marketplace API Reference](./marketplace-api.md)
- [Security Guidelines](./security-guidelines.md)
