# Cleanroom AI - Autonomic Intelligence Module

**⚠️ EXPERIMENTAL**: This module contains AI and autonomic intelligence features that are under active development.

**🔒 ISOLATED**: This crate is intentionally **excluded from default workspace builds** to keep the main CLI production-ready.

## Overview

This crate provides AI-powered capabilities for the Cleanroom Testing Framework through Ollama integration. All features include fallback mechanisms for environments where AI services are unavailable.

This crate is separated from the core framework to ensure stability and allow experimental features to evolve without impacting production deployments.

## Features

- **AI Orchestration** - Autonomous test execution with intelligent analysis
- **Predictive Analytics** - Failure pattern prediction and trend analysis
- **Intelligent Optimization** - AI-driven test execution optimization
- **Real-time Monitoring** - Anomaly detection and performance analysis

## Requirements

- **Ollama** (optional) - For real AI capabilities
  - Without Ollama: Falls back to rule-based algorithms
  - With Ollama: Enables genuine AI-powered features

## Installation

```bash
# Install Ollama (optional)
brew install ollama  # macOS
# or
curl -fsSL https://ollama.com/install.sh | sh  # Linux

# Pull AI model
ollama pull llama3.2:3b

# Start Ollama service
ollama serve &
```

## Usage

```bash
# AI orchestration
clnrm ai-orchestrate --predict-failures --auto-optimize

# Predictive analytics
clnrm ai-predict --analyze-history --recommendations

# Test optimization
clnrm ai-optimize --execution-order --resource-allocation

# Real-time monitoring
clnrm ai-monitor status
```

## Development Status

This is an **experimental module**. Features may change and some capabilities are still under development. All AI features gracefully degrade to deterministic algorithms when Ollama is unavailable.

## Testing & Building

This crate is **excluded from default workspace operations** to prevent experimental code from affecting main CLI stability:

```bash
# Default commands exclude clnrm-ai
cargo build           # Builds: clnrm, clnrm-core, clnrm-shared (NOT clnrm-ai)
cargo test            # Tests: clnrm, clnrm-core, clnrm-shared (NOT clnrm-ai)
cargo check           # Checks: clnrm, clnrm-core, clnrm-shared (NOT clnrm-ai)

# To work with AI crate explicitly
cargo test -p clnrm-ai      # Test only AI features
cargo build -p clnrm-ai     # Build only AI crate
cargo check -p clnrm-ai     # Check only AI crate

# To include all crates (including clnrm-ai)
cargo build --workspace     # Builds ALL workspace members
cargo test --workspace      # Tests ALL workspace members
```

## Integration with Main CLI

The main CLI (clnrm) does **not** depend on this crate. When users try to use AI commands, they receive helpful guidance:

```
$ clnrm ai-orchestrate tests/
Error: AI orchestration is an experimental feature in the clnrm-ai crate.
To use this feature, enable the 'ai' feature flag or use the clnrm-ai crate directly.
```

## Architecture

- **Services**: `AIIntelligenceService`, `AITestGeneratorPlugin`
- **Commands**: `ai-orchestrate`, `ai-predict`, `ai-optimize`, `ai-monitor`, `ai-real`
- **Dependencies**: surrealdb, uuid, toml (separate from core)

## License

MIT License - See LICENSE file for details.
