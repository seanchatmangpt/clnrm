# AI Test Results Directory

This directory contains output from AI integration tests.

## Running Tests

Execute the integration test script:
```bash
cd /Users/sac/clnrm/examples/optimus-prime-platform
./clnrm-ai-tests.sh
```

## Expected Files

After running tests, this directory will contain:
- `ai-orchestrate-output.txt` - AI orchestration results
- `ai-predict-output.json` - Prediction analytics (JSON format)
- `ai-optimize-output.txt` - Optimization recommendations
- `ai-monitor-output.txt` - Monitoring system output
- `ai-real-output.txt` - Real AI intelligence results
- `integration-test-output.json` - Integration test results
- `health-check-output.txt` - System health check

## Viewing Results

```bash
# View orchestration results
cat ai-test-results/ai-orchestrate-output.txt

# View predictions (formatted JSON)
cat ai-test-results/ai-predict-output.json | jq .

# View all results
ls -lh ai-test-results/
```

## Documentation

Full test results and analysis:
- `/docs/AI_INTEGRATION_RESULTS.md` - Comprehensive results documentation
- `/docs/AI_QUICK_START.md` - Quick start guide

## Notes

Results are timestamped and can be archived for historical analysis.
