# Quick Start Guide - Autonomic Testing in 5 Minutes

Get your autonomic testing platform running in under 5 minutes!

---

## Prerequisites Check

Before starting, ensure you have:

- [ ] **Rust** installed (`rustc --version`)
- [ ] **Docker** running (`docker ps`)
- [ ] **8+ GB RAM** available
- [ ] **5+ GB disk space** free

---

## Step 1: Install Cleanroom CLI (30 seconds)

```bash
# Install from crates.io
cargo install clnrm

# Verify installation
clnrm --version
# Expected: clnrm 0.4.0
```

---

## Step 2: Install Ollama (2 minutes)

### macOS / Linux

```bash
# Install Ollama
curl -fsSL https://ollama.ai/install.sh | sh

# Start Ollama service (runs in background)
ollama serve &

# Pull recommended model
ollama pull qwen2.5-coder:7b

# Verify
ollama list
curl http://localhost:11434/api/version
```

### Windows

```powershell
# Download and install from https://ollama.ai/download
# Or use winget
winget install Ollama.Ollama

# Start Ollama
ollama serve

# Pull model
ollama pull qwen2.5-coder:7b
```

---

## Step 3: Create Your First Test (1 minute)

Create `cleanroom.toml`:

```toml
[framework]
name = "my-first-test"
version = "1.0.0"

[ai]
enabled = true
provider = "ollama"
model = "qwen2.5-coder:7b"

[[scenarios]]
name = "basic_test"
description = "My first autonomic test"

[[scenarios.steps]]
name = "step_1"
type = "shell"
command = "echo 'Hello, Autonomic World!'"
expected_stdout = "Hello, Autonomic World!"
```

---

## Step 4: Run AI-Powered Test Orchestration (30 seconds)

```bash
# Run with AI orchestration and failure prediction
clnrm ai-orchestrate --predict-failures --auto-optimize

# Expected output:
# 🤖 Starting AI-powered test orchestration
# 🔮 Predictive failure analysis: enabled
# ⚡ Autonomous optimization: enabled
# ✅ All tests passed!
# 📊 Performance Score: 1.0/1.0
```

---

## Step 5: Explore AI Features (1 minute)

### Get Predictive Analytics

```bash
# Analyze test patterns and predict failures
clnrm ai-predict --analyze-history --recommendations

# Output includes:
# - Historical analysis
# - Failure predictions
# - Optimization recommendations
# - Trend analysis
```

### Generate Optimization Report

```bash
# Get AI-driven optimization strategies
clnrm ai-optimize --execution-order --resource-allocation

# Output includes:
# - Execution order optimization (37% improvement)
# - Resource allocation strategies
# - Parallel execution plan
# - Implementation roadmap
```

---

## 🎉 You're Done!

You now have a fully functional autonomic testing platform with:

- ✅ **AI-powered test orchestration**
- ✅ **Predictive failure analysis**
- ✅ **Autonomous optimization**
- ✅ **Real-time performance insights**

---

## Next Steps

### Immediate Actions

1. **Add more tests** to your `cleanroom.toml`
2. **Run benchmark** to establish baseline: `clnrm ai-orchestrate --benchmark`
3. **Enable monitoring** (see full deployment guide)

### Within 24 Hours

1. **Configure services** (SurrealDB for advanced features)
2. **Tune performance** settings
3. **Set up CI/CD integration**

### Within 1 Week

1. **Deploy to production**
2. **Train team** on autonomic features
3. **Establish metrics** and alerting

---

## Common Quick Fixes

### Ollama Not Responding

```bash
# Check if running
ps aux | grep ollama

# Restart service
killall ollama
ollama serve &

# Test connection
curl http://localhost:11434/api/version
```

### Model Not Found

```bash
# List available models
ollama list

# Pull missing model
ollama pull qwen2.5-coder:7b

# Set in environment
export OLLAMA_MODEL=qwen2.5-coder:7b
```

### Memory Issues

```bash
# Use smaller model
ollama pull phi-3:mini
export OLLAMA_MODEL=phi-3:mini

# Reduce workers
export CLNRM_PARALLEL_WORKERS=2

# Run with limits
clnrm ai-orchestrate --max-workers 2
```

---

## Examples to Try

### Example 1: Database Testing

```toml
[[scenarios]]
name = "database_test"
description = "Test with PostgreSQL"

[[scenarios.services]]
type = "postgres"
version = "16"
port = 5432

[[scenarios.steps]]
name = "create_table"
type = "shell"
command = "psql -h localhost -U test -c 'CREATE TABLE users (id SERIAL PRIMARY KEY);'"
```

Run with AI:

```bash
clnrm ai-orchestrate --predict-failures
```

### Example 2: API Testing

```toml
[[scenarios]]
name = "api_test"
description = "Test REST API"

[[scenarios.services]]
type = "redis"
version = "7"
port = 6379

[[scenarios.steps]]
name = "api_health_check"
type = "shell"
command = "curl -f http://localhost:8080/health"
expected_exit_code = 0
```

Run with optimization:

```bash
clnrm ai-optimize --auto-apply
```

---

## Get Help

- **Documentation**: Full deployment guide in `docs/AUTONOMIC_SYSTEM_DEPLOYMENT.md`
- **Examples**: Check `examples/` directory
- **Issues**: https://github.com/seanchatmangpt/clnrm/issues
- **Community**: Join Discord for real-time help

---

## Quick Reference Commands

```bash
# AI Commands
clnrm ai-orchestrate       # Intelligent test orchestration
clnrm ai-predict           # Predictive failure analysis
clnrm ai-optimize          # AI-driven optimization

# Core Commands
clnrm validate            # Validate configuration
clnrm run                 # Run tests manually
clnrm watch              # Watch mode with auto-reload

# Utilities
clnrm --help             # Show all commands
clnrm diagnostics        # Generate diagnostic report
```

---

**Ready for more?** Check out the full **[Deployment Guide](./AUTONOMIC_SYSTEM_DEPLOYMENT.md)** for production setup, performance tuning, and advanced features.

---

**Version**: 1.0.0
**Last Updated**: 2025-10-16
