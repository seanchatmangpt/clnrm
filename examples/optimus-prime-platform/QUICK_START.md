# Optimus Prime OTEL Testing - Quick Start

## The Correct Way (Hermetic)

```bash
# Everything runs in testcontainers (hermetic isolation)
cargo run --features otel-traces -- run examples/optimus-prime-platform/optimus-prime-otel.clnrm.toml
```

## What Happens

1. 🐳 OTEL Collector starts in container
2. 🐳 Optimus Prime (Next.js) starts in container
3. 🌐 Real HTTP requests sent to API
4. 📊 Real OTEL traces captured by collector
5. ✅ Spans validated → Test passes if traces prove functionality

## What This Validates

- ✅ POST /api/chat span exists → API works
- ✅ handleChildChat span exists → Child mode works  
- ✅ event.message_sent span exists → Event tracking works
- ✅ Span hierarchy correct → Context propagation works

**If these spans exist in captured traces → System works**

## Why Hermetic Matters

| Approach | Hermetic? | Real Traces? | Valid? |
|----------|-----------|--------------|--------|
| Run on host | ❌ No | ❌ No | ❌ **WRONG** |
| Static analysis | ❌ No | ❌ No | ❌ **WRONG** |
| Testcontainers + OTEL | ✅ Yes | ✅ Yes | ✅ **CORRECT** |

## Prerequisites

```bash
# Docker must be running
docker ps

# clnrm built with OTEL features
cargo build --features otel-traces
```

## Status

✅ Ready for hermetic testing
