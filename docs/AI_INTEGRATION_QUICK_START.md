# AI Integration Quick Start Guide

## 🚀 Quick Start

### Prerequisites

1. **Install Ollama:**
```bash
# macOS
brew install ollama

# Linux
curl -fsSL https://ollama.com/install.sh | sh
```

2. **Pull the AI model:**
```bash
ollama pull llama3.2:3b
```

3. **Start Ollama:**
```bash
ollama serve
```

4. **Verify it's working:**
```bash
curl http://localhost:11434/api/generate \
  -d '{"model":"llama3.2:3b","prompt":"Hello, AI!","stream":false}'
```

## 🎯 Using Real AI Commands

### 1. AI Orchestrate

**Command:**
```bash
clnrm ai-orchestrate \
  --predict-failures \
  --auto-optimize \
  --confidence-threshold 0.7 \
  --max-workers 4
```

**What it does:**
- Analyzes test files with REAL AI
- Predicts potential failures using Ollama
- Generates optimization strategies
- Provides AI-powered insights

**Example output:**
```
🤖 Starting REAL AI-powered test orchestration
🧠 Using Ollama AI for genuine intelligence
✅ Real AI service initialized with Ollama
🧠 Phase 5: AI-Powered Results Analysis
🧠 Real AI insights generated
   • Excellent test reliability - 90%+ success rate
   • Performance optimization opportunities detected
   • Consider parallelization for faster execution
🎉 AI orchestration completed successfully!
```

### 2. AI Predict

**Command:**
```bash
clnrm ai-predict \
  --analyze-history \
  --predict-failures \
  --recommendations \
  --format human
```

**What it does:**
- Analyzes historical test data with AI
- Predicts future test failures
- Generates optimization recommendations
- Provides trend analysis

**Example output:**
```
🔮 Starting REAL AI-powered predictive analytics
🧠 Using Ollama AI for genuine predictions
✅ Real AI service initialized with Ollama
🧠 Generating predictive insights using REAL Ollama AI
🔮 Failure Predictions:
   • performance_test: 35.0% failure probability
     Confidence: 85.0%
     Risk Factors: ["Resource intensive", "Timing sensitive"]
     Mitigation: ["Optimize resource allocation", "Add retry logic"]
```

### 3. AI Optimize

**Command:**
```bash
clnrm ai-optimize \
  --execution-order \
  --resource-allocation \
  --parallel-execution \
  --auto-apply
```

**What it does:**
- Analyzes test configuration with AI
- Optimizes execution order
- Recommends resource allocation
- Generates parallel execution strategies

**Example output:**
```
⚡ Starting REAL AI-powered test optimization
🧠 Using Ollama AI for genuine optimization
✅ Real AI service initialized with Ollama
📋 Generating comprehensive optimization report using REAL AI
🧠 Real AI optimization report generated
📈 Expected Overall Improvement: 68.2%
💡 Key Optimization Opportunities:
   • Parallel Execution: 40-60% improvement (High impact, Medium effort)
   • Resource Optimization: 20-30% improvement (Medium impact, Low effort)
```

## 🔧 Without Ollama (Fallback Mode)

If Ollama is not running, commands automatically fall back to simulated AI:

```bash
clnrm ai-orchestrate --predict-failures
```

**Output:**
```
🤖 Starting REAL AI-powered test orchestration
⚠️ Ollama unavailable, using simulated AI: Connection refused
💡 To enable real AI, ensure Ollama is running at http://localhost:11434
📊 Phase 1: Intelligent Test Discovery & Analysis
[... continues with simulated AI ...]
```

## 📊 Comparison: Real AI vs Simulated

| Feature | Real AI (Ollama) | Simulated AI |
|---------|------------------|--------------|
| Analysis Quality | Context-aware, adaptive | Rule-based, static |
| Insights | AI-generated, unique | Pre-programmed patterns |
| Learning | Learns from patterns | Fixed algorithms |
| Recommendations | Dynamic, contextual | Template-based |
| Accuracy | High (85%+ confidence) | Medium (heuristic) |
| Performance | Depends on model | Fast, consistent |

## 🛠️ Developer Integration

### Adding Real AI to New Commands

**Pattern to follow:**

```rust
use crate::cleanroom::ServicePlugin;
use crate::services::ai_intelligence::AIIntelligenceService;

pub async fn my_ai_command() -> Result<()> {
    // 1. Initialize AI service
    let ai_service = AIIntelligenceService::new();
    let (use_real_ai, ai_handle) = match ai_service.start().await {
        Ok(handle) => {
            info!("✅ Real AI service initialized");
            (true, Some(handle))
        },
        Err(e) => {
            warn!("⚠️ Ollama unavailable: {}", e);
            (false, None)
        }
    };

    // 2. Use AI (with fallback)
    let results = if use_real_ai {
        generate_with_real_ai().await?
    } else {
        generate_with_simulation().await?
    };

    // 3. Cleanup
    if let Some(handle) = ai_handle {
        ai_service.stop(handle).await?;
    }

    Ok(())
}

// Helper function for Ollama queries
async fn query_ollama_direct(prompt: &str) -> Result<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&serde_json::json!({
            "model": "llama3.2:3b",
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": 0.7,
                "top_p": 0.9,
                "max_tokens": 500
            }
        }))
        .send()
        .await?;

    let data: serde_json::Value = response.json().await?;
    Ok(data["response"].as_str().unwrap_or("").to_string())
}
```

### Best Practices

1. **Always provide fallback:**
   - Commands should work without Ollama
   - Clear error messages
   - Graceful degradation

2. **Proper error handling:**
   - No `unwrap()` or `expect()`
   - Return `Result` types
   - Log errors appropriately

3. **Resource management:**
   - Always cleanup AI service
   - Use proper async/await
   - Handle timeouts

4. **User feedback:**
   - Indicate AI mode (real vs simulated)
   - Show progress
   - Provide actionable insights

## 🧪 Testing

### Test with Ollama:
```bash
# Start Ollama
ollama serve &

# Run command
clnrm ai-orchestrate --predict-failures

# Verify "Real AI" in output
```

### Test without Ollama:
```bash
# Stop Ollama
killall ollama

# Run command
clnrm ai-orchestrate --predict-failures

# Verify fallback mode
```

### Test AI response quality:
```bash
# Enable verbose logging
RUST_LOG=info clnrm ai-predict --analyze-history

# Check for "Real AI insights generated"
```

## 📈 Performance Tips

1. **Model Selection:**
   - `llama3.2:3b` - Fast, good for most tasks
   - `llama3.2:7b` - Better quality, slower
   - Configure in code or via environment

2. **Caching:**
   - Cache common queries
   - Reduce API calls
   - Improve response time

3. **Batch Processing:**
   - Group similar queries
   - Reduce overhead
   - Better throughput

4. **Timeout Configuration:**
   - Default: 120 seconds
   - Adjust based on model size
   - Balance quality vs speed

## 🔍 Troubleshooting

### Ollama Connection Failed

**Problem:**
```
⚠️ Ollama unavailable: Connection refused
```

**Solution:**
1. Check if Ollama is running: `ps aux | grep ollama`
2. Start Ollama: `ollama serve`
3. Verify endpoint: `curl http://localhost:11434/api/version`

### Model Not Found

**Problem:**
```
⚠️ Ollama API error: model 'llama3.2:3b' not found
```

**Solution:**
```bash
ollama pull llama3.2:3b
```

### Slow Response

**Problem:** AI queries taking too long

**Solution:**
1. Use smaller model (3b instead of 7b)
2. Reduce max_tokens
3. Increase timeout
4. Check system resources

### Empty AI Response

**Problem:** AI returns empty or invalid response

**Solution:**
- Commands automatically fall back to simulated mode
- Check Ollama logs: `ollama logs`
- Verify model is loaded: `ollama list`

## 📚 Additional Resources

- **Ollama Documentation:** https://ollama.com/docs
- **AIIntelligenceService Code:** `/Users/sac/clnrm/crates/clnrm-core/src/services/ai_intelligence.rs`
- **Command Examples:** `/Users/sac/clnrm/crates/clnrm-core/src/cli/commands/`
- **Integration Summary:** `/Users/sac/clnrm/docs/REAL_AI_INTEGRATION_SUMMARY.md`

## 🎓 Learning Path

1. **Start with fallback mode:** Understand how commands work without AI
2. **Install Ollama:** Set up the AI service locally
3. **Run commands with real AI:** See the difference in output quality
4. **Explore customization:** Adjust prompts and parameters
5. **Integrate into workflows:** Use AI commands in CI/CD pipelines

---

**Quick Commands Reference:**

```bash
# Orchestrate with AI
clnrm ai-orchestrate --predict-failures --auto-optimize

# Predict failures with AI
clnrm ai-predict --analyze-history --predict-failures

# Optimize tests with AI
clnrm ai-optimize --execution-order --parallel-execution

# Check AI service status
curl http://localhost:11434/api/version
```

**Status:** ✅ Ready to use
