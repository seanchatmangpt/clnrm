# Plugin Development

This section covers creating custom plugins for clnrm v2.0.0.

## Overview

Plugins extend clnrm's capabilities by providing custom service implementations. In v2.0.0, plugins use the `ServicePlugin` trait with `dyn` compatibility.

## Key Concepts

- **ServicePlugin Trait**: Core interface for custom services
- **Container Integration**: Seamless integration with container lifecycle
- **Health Checks**: Built-in health monitoring
- **Environment Persistence**: v2.0.0 environment variable handling

## Getting Started

1. [Creating Custom Plugins](creating-plugins.md)
2. [Plugin Lifecycle Management](plugin-lifecycle.md)
3. [Plugin Examples](examples.md)

## v2.0.0 Changes

- Simplified trait interface
- Better error handling
- Container execution model integration