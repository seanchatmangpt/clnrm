# Cleanroom AI - Autonomic Intelligence Module

**⚠️ EXPERIMENTAL**: This module contains AI and autonomic intelligence features that are under active development.

## Overview

This crate provides AI-powered capabilities for the Cleanroom Testing Framework through Ollama integration. All features include fallback mechanisms for environments where AI services are unavailable.

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

## License

MIT License - See LICENSE file for details.
