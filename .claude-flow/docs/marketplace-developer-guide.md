# Marketplace Developer Guide

## Quick Start

### Using the Marketplace CLI

```bash
# Search for plugins
clnrm marketplace search "database"

# List all available plugins
clnrm marketplace list

# Get detailed information
clnrm marketplace info postgres-plugin

# Install a plugin (Phase 2)
clnrm marketplace install postgres-plugin

# Rate and review
clnrm marketplace rate postgres-plugin 5
clnrm marketplace review postgres-plugin "Excellent plugin!"
```

## Architecture Overview

### Core Components

```rust
// Main marketplace client
use clnrm_core::marketplace::{Marketplace, MarketplaceConfig};

let config = MarketplaceConfig::default();
let marketplace = Marketplace::new(config)?;

// Search for plugins
let results = marketplace.search("test").await?;

// Get plugin info
let plugin = marketplace.get_plugin_info("postgres-plugin")?;

// List installed
let installed = marketplace.list_installed()?;
```

### Plugin Metadata Structure

```rust
use clnrm_core::marketplace::metadata::PluginMetadata;

let mut plugin = PluginMetadata::new(
    "my-plugin",
    "1.0.0",
    "My awesome plugin",
    "Author Name"
)?;

// Add capabilities
plugin.capabilities.push(PluginCapability::new(
    "database",
    PluginCategory::Database,
    "Provides database testing"
));

// Add dependencies
plugin.dependencies.push(PluginDependency::new(
    "postgres-plugin",
    "^1.0"
)?);
```

## File Locations

### Source Files
- **Main Module**: `/Users/sac/clnrm/crates/clnrm-core/src/marketplace/mod.rs`
- **Metadata**: `/Users/sac/clnrm/crates/clnrm-core/src/marketplace/metadata.rs`
- **Registry**: `/Users/sac/clnrm/crates/clnrm-core/src/marketplace/registry.rs`
- **Discovery**: `/Users/sac/clnrm/crates/clnrm-core/src/marketplace/discovery.rs`
- **Commands**: `/Users/sac/clnrm/crates/clnrm-core/src/marketplace/commands.rs`
- **Package**: `/Users/sac/clnrm/crates/clnrm-core/src/marketplace/package.rs`
- **Security**: `/Users/sac/clnrm/crates/clnrm-core/src/marketplace/security.rs`
- **Community**: `/Users/sac/clnrm/crates/clnrm-core/src/marketplace/community.rs`

### CLI Integration
- **Types**: `/Users/sac/clnrm/crates/clnrm-core/src/cli/types.rs` (line 231-233)
- **Handler**: `/Users/sac/clnrm/crates/clnrm-core/src/cli/mod.rs` (line 190-194)

## Testing

### Running Tests

```bash
# Run all marketplace tests
cargo test --lib marketplace

# Run specific test
cargo test --lib marketplace::registry::tests::test_registry_creation

# Run with output
cargo test --lib marketplace -- --nocapture
```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_my_feature() -> Result<()> {
        // Use temporary directories for tests
        let temp_dir = std::env::temp_dir()
            .join(format!("clnrm-test-{}", uuid::Uuid::new_v4()));

        let config = MarketplaceConfig {
            cache_dir: temp_dir.join("cache"),
            install_dir: temp_dir.join("plugins"),
            // ...
        };

        // Your test code here

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }
}
```

## Common Patterns

### Creating a Plugin Registry

```rust
use clnrm_core::marketplace::{MarketplaceConfig, registry::PluginRegistry};

let config = MarketplaceConfig {
    registry_urls: vec!["https://registry.example.com".to_string()],
    cache_dir: PathBuf::from("/tmp/marketplace/cache"),
    install_dir: PathBuf::from("/tmp/marketplace/plugins"),
    community_enabled: true,
    auto_update: false,
};

let registry = PluginRegistry::new(&config)?;
```

### Searching Plugins

```rust
use clnrm_core::marketplace::discovery::PluginDiscovery;

let discovery = PluginDiscovery::new(&config)?;
let results = discovery.search_plugins("database").await?;

for plugin in results {
    println!("{} v{} - {}", plugin.name, plugin.version, plugin.description);
}
```

### Validating Plugin Security

```rust
use clnrm_core::marketplace::security::PluginSecurityValidator;

let validator = PluginSecurityValidator::new();
let result = validator.validate(&metadata).await?;

if result.is_safe {
    println!("Plugin is safe to install");
} else {
    println!("Security issues: {:?}", result.issues);
}
```

## Error Handling

All marketplace operations return `Result<T, CleanroomError>`:

```rust
use clnrm_core::error::{CleanroomError, Result};

match marketplace.install("plugin-name").await {
    Ok(metadata) => println!("Installed: {}", metadata.name),
    Err(CleanroomError::ValidationError { message, .. }) => {
        eprintln!("Validation error: {}", message);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Stub Functions (Phase 2)

These functions currently return placeholder data:

### 1. Remote Registry Fetching
```rust
// src/marketplace/registry.rs:292
async fn fetch_registry_catalog(&self, _registry_url: &str) -> Result<Vec<PluginMetadata>> {
    // TODO: Implement HTTP fetch
    Ok(Vec::new())
}
```

### 2. Plugin Installation
```rust
// src/marketplace/package.rs:74
pub async fn install_plugin(&self, metadata: &PluginMetadata) -> Result<PluginMetadata> {
    // TODO: Implement actual download and installation
    Ok(metadata.clone())
}
```

### 3. Update Checking
```rust
// src/marketplace/package.rs:197
pub async fn check_updates(&self, current: &PluginMetadata) -> Result<Option<PluginMetadata>> {
    // TODO: Implement remote version check
    Ok(None)
}
```

## Best Practices

1. **Always use temp directories in tests**
   ```rust
   let temp_dir = std::env::temp_dir().join(format!("clnrm-test-{}", uuid::Uuid::new_v4()));
   ```

2. **Use multi-threaded runtime for async tests**
   ```rust
   #[tokio::test(flavor = "multi_thread")]
   ```

3. **Validate metadata before registration**
   ```rust
   metadata.validate()?;
   registry.register_plugin(metadata).await?;
   ```

4. **Clean up test directories**
   ```rust
   let _ = std::fs::remove_dir_all(&temp_dir);
   ```

5. **Handle errors gracefully**
   ```rust
   match result {
       Ok(data) => { /* success */ }
       Err(e) => { /* appropriate error handling */ }
   }
   ```

## Integration Examples

### CLI Command Handler
```rust
// src/cli/mod.rs
Commands::Marketplace { command } => {
    use crate::marketplace::{MarketplaceConfig, execute_marketplace_command};
    let config = MarketplaceConfig::default();
    execute_marketplace_command(&config, command).await
}
```

### Custom Plugin Discovery
```rust
let discovery = PluginDiscovery::new(&config)?;

// Search by category
let database_plugins = discovery.search_by_category("database").await?;

// Get trending plugins
let trending = discovery.get_trending_plugins(10).await?;

// Get recommendations based on installed plugins
let recommendations = discovery.get_recommendations(&installed).await?;
```

## Debugging

### Enable Logging
```bash
RUST_LOG=clnrm_core::marketplace=debug cargo test
```

### Inspect Registry State
```rust
let db_path = config.cache_dir.join("registry.json");
let content = std::fs::read_to_string(db_path)?;
println!("{}", content);
```

### Check Installation Records
```rust
let installed = registry.list_installed_plugins()?;
for plugin in installed {
    println!("{}: {} at {:?}",
        plugin.name,
        plugin.version,
        plugin.community.installed_at
    );
}
```

## Performance Tips

1. **Use caching** - The discovery module caches search results
2. **Batch operations** - Register multiple plugins in a single transaction
3. **Async operations** - All I/O operations are async for better performance
4. **Lazy loading** - Registry loads on-demand, not at startup

## Contributing

When adding new features:

1. Add comprehensive tests
2. Update this documentation
3. Follow existing patterns
4. Use proper error types
5. Add TODO comments for Phase 2 items

## Support

For issues or questions:
- Check the compilation report: `/Users/sac/clnrm/docs/marketplace-compilation-report.md`
- Review test cases in `src/marketplace/*/tests`
- Examine error handling in `src/error.rs`
