# Tutorials - Learn by Doing

Tutorials are **learning-oriented guides** that walk you step-by-step through complete examples. They're designed for newcomers to understand clnrm by doing.

## Tutorial Overview

Each tutorial takes 10-20 minutes and builds your skills progressively:

### [01 - Getting Started](./01-getting-started/)
**⏱ 15 minutes** | Prerequisites: None

Your first complete test from installation to running it. You'll:
- Install clnrm (Homebrew or Cargo)
- Initialize a test project
- Write a simple TOML test
- Run it and understand the output
- Explore results

**Start here if you're new to clnrm.**

### [02 - Container Pooling](./02-container-pooling/)
**⏱ 10 minutes** | Prerequisites: Tutorial 01

Speed up your tests 80% using container pooling. You'll:
- Understand why startup is slow (2-5 seconds)
- Enable pooling with one environment variable
- See how much faster tests run
- Configure pool size and timeout
- Monitor pool hit rate

**Do this next to get maximum performance.**

### [03 - Weaver Validation](./03-weaver-validation/)
**⏱ 15 minutes** | Prerequisites: Tutorial 01

Catch false-positive tests using schema validation. You'll:
- Understand the false-positive problem
- Install Weaver
- Write a validation schema
- Enable live-checking in your test
- See how Weaver catches broken tests

**Do this when you want to ensure tests validate actual behavior.**

### [04 - Custom Plugins](./04-custom-plugins/)
**⏱ 20 minutes** | Prerequisites: Tutorial 01

Create your own service plugin to extend clnrm. You'll:
- Understand the ServicePlugin trait
- Copy and modify an example plugin
- Implement start/stop/health_check
- Register your plugin
- Use it in a test TOML
- Test your plugin

**Do this when you need to test custom services.**

### [05 - OpenTelemetry Integration](./05-otel-integration/)
**⏱ 15 minutes** | Prerequisites: Tutorial 01

Export telemetry to an observability platform. You'll:
- Understand why observability matters
- Configure OTLP export in TOML
- Set up Jaeger (or DataDog/New Relic)
- Run tests with telemetry export
- Inspect traces in the observability UI
- Configure sampling and propagators

**Do this when you want to observe test behavior.**

---

## Learning Path

```
Tutorial 01: Getting Started (foundational)
     ↓
Choose your next tutorial based on your goal:

Speed up your tests?      → Tutorial 02: Container Pooling
Improve reliability?      → Tutorial 03: Weaver Validation
Extend with plugins?      → Tutorial 04: Custom Plugins
Add observability?        → Tutorial 05: OTEL Integration
```

## Diataxis: What Makes These Tutorials

Tutorials in Diataxis are:
- ✅ **Learning-oriented** — Goal is understanding, not just getting something working
- ✅ **Concrete examples** — Every step has a working example
- ✅ **Procedural** — Step-by-step instructions without skipping
- ✅ **Complete** — Each tutorial is self-contained and finishable
- ✅ **Realistic** — Examples match real usage patterns

They're **NOT**:
- ❌ How-to guides (those solve specific problems)
- ❌ Reference docs (those look up details)
- ❌ Explanations (those teach concepts)

---

## Next Steps

1. **Start with [Tutorial 01: Getting Started](./01-getting-started/)**
2. Choose your next tutorial based on your needs
3. Explore [How-To Guides](../how-to/) for specific problems
4. Read [Explanations](../explanation/) to understand concepts deeply
5. Use [Reference Docs](../reference/) to look up details

---

## Need Help?

- **Something doesn't work?** → See [Troubleshooting How-To Guide](../how-to/troubleshooting.md)
- **Want to understand more?** → See [Explanations](../explanation/)
- **Need technical details?** → See [Reference Docs](../reference/)
- **Have a question?** → [Open an issue](https://github.com/seanchatmangpt/clnrm/issues)
