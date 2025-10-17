# AI Commands Reference Guide

Complete reference for all AI-powered commands in the Cleanroom autonomic system.

---

## Overview

The Cleanroom CLI provides three powerful AI commands:

| Command | Purpose | Key Features |
|---------|---------|--------------|
| **`ai-orchestrate`** | Intelligent test execution | Prediction, optimization, autonomous execution |
| **`ai-predict`** | Predictive analytics | Historical analysis, failure prediction, trends |
| **`ai-optimize`** | Performance optimization | Execution order, resources, parallelization |

---

## `clnrm ai-orchestrate`

**Purpose**: AI-powered test orchestration with predictive failure analysis and autonomous optimization.

### Usage

```bash
clnrm ai-orchestrate [OPTIONS]
```

### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--predict-failures` | flag | false | Enable predictive failure analysis |
| `--auto-optimize` | flag | false | Enable autonomous optimization |
| `--confidence-threshold` | float | 0.8 | Minimum confidence for predictions (0.0-1.0) |
| `--max-workers` | int | 4 | Maximum parallel workers |
| `--benchmark` | flag | false | Run in benchmark mode |
| `--output` | path | - | Output file for results (JSON) |
| `--verbose` | flag | false | Enable verbose logging |
| `--debug` | flag | false | Enable debug mode |

### Examples

#### Basic AI Orchestration

```bash
clnrm ai-orchestrate
```

**Output**:
```
🤖 Starting AI-powered test orchestration
📊 Phase 1: Intelligent Test Discovery & Analysis
🔍 Discovered 3 test files for AI orchestration
🚀 Phase 2: Intelligent Test Execution
✅ All tests passed!
📊 Success Rate: 100.0%
```

#### With Failure Prediction

```bash
clnrm ai-orchestrate --predict-failures --confidence-threshold 0.85
```

**Output**:
```
🤖 Starting AI-powered test orchestration
🔮 Predictive failure analysis: enabled
📊 Phase 1: Intelligent Test Discovery & Analysis
🔍 Discovered 5 test files for AI orchestration

🔮 Phase 2: Predictive Failure Analysis
⚠️ High-risk tests predicted:
   • complex_integration_test: 78% failure probability (confidence: 88%)
   • performance_stress_test: 62% failure probability (confidence: 85%)

💡 Recommendations:
   • Run high-risk tests first for early feedback
   • Allocate additional resources to stress tests
   • Consider breaking complex tests into smaller units
```

#### With Auto-Optimization

```bash
clnrm ai-orchestrate --predict-failures --auto-optimize --max-workers 8
```

**Output**:
```
🤖 Starting AI-powered test orchestration
🔮 Predictive failure analysis: enabled
⚡ Autonomous optimization: enabled
📊 Phase 1: Intelligent Test Discovery & Analysis
🔍 Discovered 10 test files for AI orchestration

⚡ Phase 3: AI-Driven Test Optimization
📊 Estimated Improvement: 45.0%
💾 Resource Optimization: 32.4% efficiency gain
👥 Parallel Execution: 8 workers optimized

🚀 Phase 4: Intelligent Test Execution
🧠 AI-optimized execution for: integration_tests
📊 Complexity score: 0.85
💾 Resource requirements: 2.4 CPU, 1024 MB RAM
⏱️ Estimated duration: 45s

✅ Phase 5: AI-Powered Results Analysis
📊 Success Rate: 95.0%
⚡ Performance Score: 0.92/1.0
🛡️ Reliability Score: 0.95/1.0
⏱️ Total Duration: 142s

💡 AI Insights:
   • Excellent test reliability - 90%+ success rate
   • Performance is good - minor optimization opportunities
   • 2 flaky tests detected - recommend quarantine
```

#### Benchmark Mode

```bash
clnrm ai-orchestrate --benchmark --iterations 10 --output benchmark.json
```

**Output**:
```json
{
  "iterations": 10,
  "metrics": {
    "avg_duration_ms": 145.3,
    "p50_duration_ms": 142.0,
    "p95_duration_ms": 168.5,
    "p99_duration_ms": 175.2,
    "success_rate": 0.96,
    "ai_inference_avg_ms": 2847.3,
    "container_startup_avg_ms": 4532.1
  },
  "optimization_impact": {
    "execution_order": 0.375,
    "resource_allocation": 0.286,
    "parallel_execution": 0.25
  }
}
```

### Return Codes

| Code | Meaning |
|------|---------|
| 0 | All tests passed |
| 1 | Some tests failed |
| 2 | Configuration error |
| 3 | AI service unavailable |
| 4 | Timeout |

---

## `clnrm ai-predict`

**Purpose**: Predictive analytics for failure forecasting, trend analysis, and optimization recommendations.

### Usage

```bash
clnrm ai-predict [OPTIONS]
```

### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--analyze-history` | flag | false | Analyze historical test data |
| `--predict-failures` | flag | false | Predict test failures |
| `--recommendations` | flag | false | Generate optimization recommendations |
| `--trends` | flag | false | Show trend analysis |
| `--days` | int | 30 | Days of history to analyze |
| `--confidence-threshold` | float | 0.8 | Minimum confidence for predictions |
| `--format` | string | human | Output format: human, json, csv |
| `--output` | path | - | Output file |

### Examples

#### Comprehensive Analysis

```bash
clnrm ai-predict --analyze-history --predict-failures --recommendations --trends
```

**Output**:
```
🔮 Starting AI-powered predictive analytics

📊 Phase 1: Historical Data Analysis
📈 Time Range: Last 30 days
🔢 Total Executions: 450
✅ Successful: 387 (86.0%)
❌ Failed: 63 (14.0%)
⏱️ Average Execution Time: 8543ms

🔍 Failure Patterns:
   • database_integration_test: 22.5% failure rate (High)
   • performance_load_test: 18.2% failure rate (Medium)
   • ui_automation_test: 12.8% failure rate (Medium)

🔮 Phase 2: Failure Prediction Analysis

⚠️ High-Risk Tests (next 24 hours):
   1. database_integration_test: 65% failure probability
      Confidence: 92%
      Risk Factors: ["Database connection pool exhaustion", "Schema migration conflicts"]
      Mitigation: ["Increase connection pool size", "Run migrations in isolation"]

   2. performance_load_test: 48% failure probability
      Confidence: 87%
      Risk Factors: ["Memory pressure", "Network latency spikes"]
      Mitigation: ["Scale up resources", "Add retry logic"]

💡 Phase 3: Optimization Recommendations

🎯 Top Recommendations:
   1. Enable Parallel Execution (High Impact, Medium Effort)
      • Reduce execution time by 40-60%
      • Estimated savings: 180s per run

   2. Implement Container Reuse (Medium Impact, Low Effort)
      • Reduce startup overhead by 20-30%
      • Estimated savings: 45s per run

   3. Optimize Database Test Isolation (High Impact, High Effort)
      • Improve reliability by 50%
      • Reduce flaky test failures

📈 Phase 4: Trend Analysis

📊 30-Day Trends:
   • Success Rate: ↗️ Improving (+5.2%)
   • Performance: → Stable (±2.1%)
   • Reliability: ↗️ Improving (+3.8%)
   • Resource Usage: ↘️ Degrading (+12.5%)

🔍 Key Insights:
   • Success rate improved after implementing retry logic (Day 15)
   • Resource usage increased due to more complex tests
   • Database tests show weekly pattern (fail on Mondays)
   • Performance stable despite increased load

🧠 Phase 5: Predictive Insights

🔮 Next 7 Days Forecast:
   • Overall Success Rate: 88-92% (Confidence: 89%)
   • Performance Degradation Risk: Low (Confidence: 91%)
   • Failure Spike Probability: Monday AM (Confidence: 85%)

💡 Recommended Actions:
   1. Schedule maintenance before Monday (database cleanup)
   2. Increase resource allocation for complex tests
   3. Implement connection pooling improvements
   4. Add monitoring for resource usage trends
```

#### JSON Output for Automation

```bash
clnrm ai-predict --analyze-history --predict-failures --format json --output predictions.json
```

**Output** (`predictions.json`):
```json
{
  "generated_at": "2025-10-16T08:00:00Z",
  "analysis_period_days": 30,
  "historical_analysis": {
    "total_executions": 450,
    "success_count": 387,
    "failure_count": 63,
    "success_rate": 0.86,
    "avg_duration_ms": 8543,
    "failure_patterns": [
      {
        "test_name": "database_integration_test",
        "failure_rate": 0.225,
        "severity": "high"
      }
    ]
  },
  "failure_predictions": [
    {
      "test_name": "database_integration_test",
      "probability": 0.65,
      "confidence": 0.92,
      "timeframe_hours": 24,
      "risk_factors": [
        "Database connection pool exhaustion",
        "Schema migration conflicts"
      ],
      "mitigation_steps": [
        "Increase connection pool size",
        "Run migrations in isolation"
      ]
    }
  ],
  "trends": {
    "success_rate": {"direction": "improving", "change_percent": 5.2},
    "performance": {"direction": "stable", "change_percent": 2.1},
    "reliability": {"direction": "improving", "change_percent": 3.8},
    "resource_usage": {"direction": "degrading", "change_percent": 12.5}
  }
}
```

#### Quick Failure Check

```bash
clnrm ai-predict --predict-failures --days 7 --format human
```

**Output**:
```
🔮 Failure Predictions (Next 24 hours):

⚠️ 2 high-risk tests detected:
   1. database_integration_test (65% probability)
   2. performance_load_test (48% probability)

💡 Quick Action: Review database connection settings before next run.
```

### Use Cases

1. **CI/CD Pre-Check**: Run before deployments to assess risk
2. **Sprint Planning**: Identify tests needing attention
3. **Capacity Planning**: Forecast resource needs
4. **Incident Prevention**: Proactive failure mitigation

---

## `clnrm ai-optimize`

**Purpose**: AI-driven optimization for test execution order, resource allocation, and parallelization.

### Usage

```bash
clnrm ai-optimize [OPTIONS]
```

### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--execution-order` | flag | false | Optimize test execution order |
| `--resource-allocation` | flag | false | Optimize resource allocation |
| `--parallel-execution` | flag | false | Optimize parallelization |
| `--auto-apply` | flag | false | Auto-apply optimizations |
| `--max-workers` | int | 4 | Target worker count |
| `--output` | path | - | Save optimization plan |
| `--dry-run` | flag | false | Show plan without applying |

### Examples

#### Comprehensive Optimization

```bash
clnrm ai-optimize --execution-order --resource-allocation --parallel-execution
```

**Output**:
```
⚡ Starting AI-powered test optimization

📊 Phase 1: Analyzing Current Configuration
📊 Current Test Configuration:
🔢 Total Tests: 15
📋 Total Steps: 47
🔧 Total Services: 8
💻 Total CPU Requirements: 12.4 cores
💾 Total Memory Requirements: 8.5 GB
⏱️ Total Execution Time: 385s
👥 Current Parallel Workers: 4

🔄 Phase 2: Execution Order Optimization

🧠 AI Analysis:
   • Critical path tests identified: 3
   • Independent test clusters: 5
   • Sequential dependencies: 7

🔄 Optimized Execution Order:
   Priority 1 (Run First):
   • critical_integration_test (high impact, fast)
   • security_audit_test (high impact, fast)
   • api_contract_test (blocks 5 other tests)

   Priority 2 (Parallel Group):
   • unit_tests_suite_1
   • unit_tests_suite_2
   • unit_tests_suite_3
   • performance_benchmarks

   Priority 3 (Run Last):
   • ui_smoke_tests (slow, low impact)
   • load_stress_tests (resource intensive)

📈 Expected Improvement:
   • Time Savings: 37.5% (144s → 90s)
   • Earlier Failure Detection: 45% faster
   • Resource Utilization: +28% efficiency

💾 Phase 3: Resource Allocation Optimization

🧠 AI Analysis:
   • Resource contention detected in 4 tests
   • Over-provisioned resources in 6 tests
   • Opportunity for memory pooling: 2.1 GB savings

💾 Optimized Resource Allocation:
   Test: database_integration_test
   Before: 2.0 CPU, 1024 MB RAM
   After:  1.4 CPU, 768 MB RAM
   Reasoning: Database I/O bound, not CPU intensive

   Test: performance_load_test
   Before: 1.5 CPU, 512 MB RAM
   After:  2.5 CPU, 1024 MB RAM
   Reasoning: CPU intensive, needs more resources

📈 Resource Savings:
   • CPU: -1.8 cores (14.5%)
   • Memory: -2.1 GB (24.7%)
   • Network: -450 MB (32.1%)

👥 Phase 4: Parallel Execution Optimization

🧠 AI Analysis:
   • Parallelizable tests: 12 of 15
   • Optimal worker count: 6 (up from 4)
   • Estimated speedup: 2.4x

👥 Parallel Execution Plan:
   Worker 1: [test_1, test_5, test_9]
   Worker 2: [test_2, test_6, test_10]
   Worker 3: [test_3, test_7, test_11]
   Worker 4: [test_4, test_8, test_12]
   Worker 5: [test_13, test_14]
   Worker 6: [test_15]

   Sequential (dependencies): [critical_integration_test, api_contract_test]

📈 Expected Improvement:
   • Total Time: 385s → 160s (58.4% faster)
   • Resource Utilization: 45% → 78%
   • Throughput: 0.039 tests/s → 0.094 tests/s

📋 Phase 5: Optimization Summary

🎯 Total Optimization Potential:
   • Time Savings: 225s (58.4%)
   • Resource Efficiency: +52.3%
   • Cost Savings: ~$45/month (estimated)

⚠️ Risk Assessment:
   • Overall Risk: Low
   • Parallel execution risk: Low (tests are independent)
   • Resource optimization risk: Low (conservative estimates)

💡 Implementation Roadmap:
   Phase 1: Quick Wins (1-2 days)
   • Enable parallel execution (6 workers)
   • Reorder critical tests to run first
   • Expected impact: 40% time reduction

   Phase 2: Resource Tuning (3-5 days)
   • Adjust CPU/memory allocations
   • Implement connection pooling
   • Expected impact: 25% resource savings

   Phase 3: Advanced Optimization (1-2 weeks)
   • Container reuse strategy
   • Caching improvements
   • Expected impact: 15% additional speedup

🤖 Auto-apply: disabled (use --auto-apply to apply changes)
```

#### Auto-Apply Optimizations

```bash
clnrm ai-optimize --execution-order --resource-allocation --auto-apply
```

**Output**:
```
⚡ Starting AI-powered test optimization
🤖 Auto-apply: enabled

[... analysis output ...]

✅ Applying Optimizations:
   ✓ Updated test execution order in cleanroom.toml
   ✓ Adjusted resource allocations for 8 tests
   ✓ Configured parallel workers: 6
   ✓ Saved optimization profile

🎉 Optimizations applied successfully!

💡 Next Steps:
   1. Run tests to validate: clnrm ai-orchestrate
   2. Monitor performance: clnrm ai-predict --trends
   3. Fine-tune if needed: clnrm ai-optimize --dry-run
```

#### Dry Run (Preview Only)

```bash
clnrm ai-optimize --parallel-execution --max-workers 8 --dry-run
```

**Output**:
```
⚡ Starting AI-powered test optimization (DRY RUN)

[... analysis output ...]

📋 Optimization Plan (NOT APPLIED):
   • Parallel workers: 4 → 8
   • Parallelizable tests: 12
   • Expected speedup: 1.8x

🤖 This was a dry run. Use --auto-apply to apply changes.
```

### Optimization Strategies

The AI uses multiple strategies:

1. **Execution Order**:
   - Critical path analysis
   - Dependency graph optimization
   - Fail-fast prioritization

2. **Resource Allocation**:
   - Historical usage patterns
   - Dynamic scaling based on load
   - Container memory optimization

3. **Parallel Execution**:
   - Independence analysis
   - Worker pool sizing
   - Load balancing

---

## Best Practices

### 1. Use Appropriate Confidence Thresholds

```bash
# Conservative (fewer predictions, higher confidence)
clnrm ai-predict --confidence-threshold 0.9

# Balanced (recommended)
clnrm ai-predict --confidence-threshold 0.8

# Aggressive (more predictions, lower confidence)
clnrm ai-predict --confidence-threshold 0.65
```

### 2. Combine Commands for Maximum Impact

```bash
# Step 1: Analyze and get recommendations
clnrm ai-predict --analyze-history --recommendations --output analysis.json

# Step 2: Optimize based on recommendations
clnrm ai-optimize --execution-order --resource-allocation --auto-apply

# Step 3: Run with AI orchestration
clnrm ai-orchestrate --predict-failures --auto-optimize
```

### 3. Use Benchmark Mode for Baselining

```bash
# Before optimization
clnrm ai-orchestrate --benchmark --output baseline.json

# After optimization
clnrm ai-orchestrate --benchmark --output optimized.json

# Compare results
diff <(jq '.metrics' baseline.json) <(jq '.metrics' optimized.json)
```

### 4. Automate in CI/CD

```yaml
# .github/workflows/ai-testing.yml
- name: Predictive Analysis
  run: clnrm ai-predict --predict-failures --format json --output predictions.json

- name: Fail if High Risk
  run: |
    RISK=$(jq '.failure_predictions | length' predictions.json)
    if [ "$RISK" -gt 2 ]; then
      echo "High failure risk detected. Review before deploying."
      exit 1
    fi

- name: Run Optimized Tests
  run: clnrm ai-orchestrate --predict-failures --auto-optimize
```

---

## Troubleshooting

### AI Service Connection Issues

```bash
# Verify Ollama is running
curl http://localhost:11434/api/version

# Test AI inference
ollama run qwen2.5-coder:7b "test"

# Check logs
clnrm ai-orchestrate --debug 2>&1 | grep -i "ollama"
```

### Low Confidence Predictions

```bash
# Provide more historical data
clnrm ai-predict --days 60 --analyze-history

# Use more accurate model
export OLLAMA_MODEL=qwen2.5-coder:14b

# Lower threshold temporarily
clnrm ai-predict --confidence-threshold 0.6
```

### Performance Issues

```bash
# Use faster model
ollama pull phi-3:mini
export OLLAMA_MODEL=phi-3:mini

# Reduce parallel workers
clnrm ai-orchestrate --max-workers 2

# Increase timeout
export OLLAMA_TIMEOUT_SECONDS=600
```

---

## Advanced Usage

### Custom Model Configuration

```toml
# cleanroom.toml
[ai]
models = [
  { name = "fast", model = "phi-3:mini", timeout = 10 },
  { name = "balanced", model = "qwen2.5-coder:7b", timeout = 30 },
  { name = "accurate", model = "qwen2.5-coder:14b", timeout = 60 }
]
default_model = "balanced"
```

### Programmatic Access

```rust
use clnrm_core::services::ai_intelligence::AiIntelligenceService;

let ai = AiIntelligenceService::new(config)?;
let predictions = ai.predict_failures(&tests).await?;
let optimizations = ai.optimize_execution(&tests).await?;
```

---

## API Reference

For detailed API documentation, see:
- **Rust API**: https://docs.rs/clnrm-core/latest/clnrm_core/services/ai_intelligence/
- **REST API**: Coming soon

---

**Version**: 1.0.0
**Last Updated**: 2025-10-16
