# Creating Custom Plugins

This chapter covers creating custom plugins for clnrm v2.0.0.

## Plugin Architecture

In v2.0.0, plugins implement the `ServicePlugin` trait:

```rust
use clnrm_core::{ServicePlugin, ServiceHandle, HealthStatus};
use std::collections::HashMap;

pub struct MyCustomPlugin {
    config: PluginConfig,
}

impl ServicePlugin for MyCustomPlugin {
    fn name(&self) -> &str {
        "my-plugin"
    }

    fn start(&self) -> Pin<Box<dyn Future<Output = Result<ServiceHandle>>>> {
        // Implementation
    }

    fn health_check(&self, handle: &ServiceHandle) -> HealthStatus {
        // Implementation
    }

    fn stop(&self, handle: ServiceHandle) -> Result<()> {
        // Implementation
    }
}
```

## v2.0.0 Improvements

- Simplified trait interface
- Better error handling
- Container lifecycle integration
- Environment variable persistence

## Next Steps

- [Plugin Lifecycle Management](plugin-lifecycle.md)
- [Plugin Examples](examples.md)