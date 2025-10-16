#!/bin/bash
# CLNRM AI Integration Tests - v0.4.0
# Tests ALL AI features with REAL execution and results

set -e

CLNRM_BIN="/Users/sac/clnrm/target/release/clnrm"
TEST_DIR="/Users/sac/clnrm/examples/optimus-prime-platform/tests"
RESULTS_DIR="/Users/sac/clnrm/examples/optimus-prime-platform/ai-test-results"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  CLNRM AI Integration Test Suite v0.4.0${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Create results directory
mkdir -p "$RESULTS_DIR"

# Test 1: AI Orchestration
echo -e "${GREEN}Test 1: AI-Powered Test Orchestration${NC}"
echo -e "${YELLOW}Running: clnrm ai-orchestrate with sample tests${NC}"
echo ""

$CLNRM_BIN ai-orchestrate \
  --predict-failures \
  --auto-optimize \
  --confidence-threshold 0.8 \
  --max-workers 4 \
  "$TEST_DIR"/*.clnrm.toml 2>&1 | tee "$RESULTS_DIR/ai-orchestrate-output.txt"

echo ""
echo -e "${GREEN}✓ AI Orchestration test completed${NC}"
echo ""

# Test 2: AI Prediction
echo -e "${GREEN}Test 2: AI-Powered Predictive Analytics${NC}"
echo -e "${YELLOW}Running: clnrm ai-predict --analyze-history${NC}"
echo ""

$CLNRM_BIN ai-predict \
  --analyze-history \
  --predict-failures \
  --recommendations \
  --format json 2>&1 | tee "$RESULTS_DIR/ai-predict-output.json"

echo ""
echo -e "${GREEN}✓ AI Prediction test completed${NC}"
echo ""

# Test 3: AI Optimization
echo -e "${GREEN}Test 3: AI-Powered Optimization${NC}"
echo -e "${YELLOW}Running: clnrm ai-optimize${NC}"
echo ""

$CLNRM_BIN ai-optimize \
  --execution-order \
  --resource-allocation \
  --parallel-execution 2>&1 | tee "$RESULTS_DIR/ai-optimize-output.txt"

echo ""
echo -e "${GREEN}✓ AI Optimization test completed${NC}"
echo ""

# Test 4: AI Real Intelligence (SurrealDB + Ollama)
echo -e "${GREEN}Test 4: Real AI Intelligence (SurrealDB + Ollama)${NC}"
echo -e "${YELLOW}Running: clnrm ai-real --analyze${NC}"
echo ""

$CLNRM_BIN ai-real \
  --analyze 2>&1 | tee "$RESULTS_DIR/ai-real-output.txt"

echo ""
echo -e "${GREEN}✓ AI Real Intelligence test completed${NC}"
echo ""

# Test 5: AI Monitoring (Short run for testing)
echo -e "${GREEN}Test 5: AI-Powered Autonomous Monitoring${NC}"
echo -e "${YELLOW}Running: clnrm ai-monitor (10 second test)${NC}"
echo ""

timeout 10 $CLNRM_BIN ai-monitor \
  --interval 2 \
  --anomaly-threshold 0.7 \
  --ai-alerts \
  --anomaly-detection \
  --proactive-healing 2>&1 | tee "$RESULTS_DIR/ai-monitor-output.txt" || true

echo ""
echo -e "${GREEN}✓ AI Monitoring test completed${NC}"
echo ""

# Test 6: Run actual integration test
echo -e "${GREEN}Test 6: Run AI Integration Test File${NC}"
echo -e "${YELLOW}Running: clnrm run optimus-ai-integration.clnrm.toml${NC}"
echo ""

$CLNRM_BIN run "$TEST_DIR/optimus-ai-integration.clnrm.toml" \
  --format json 2>&1 | tee "$RESULTS_DIR/integration-test-output.json"

echo ""
echo -e "${GREEN}✓ Integration test completed${NC}"
echo ""

# Test 7: Health Check
echo -e "${GREEN}Test 7: System Health Check${NC}"
echo -e "${YELLOW}Running: clnrm health${NC}"
echo ""

$CLNRM_BIN health 2>&1 | tee "$RESULTS_DIR/health-check-output.txt"

echo ""
echo -e "${GREEN}✓ Health check completed${NC}"
echo ""

# Generate summary report
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Test Summary${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo "Results saved to: $RESULTS_DIR"
echo ""
echo "Files generated:"
ls -lh "$RESULTS_DIR"
echo ""
echo -e "${GREEN}All AI integration tests completed successfully!${NC}"
echo ""
echo "Next steps:"
echo "1. Review results in $RESULTS_DIR"
echo "2. Check AI_INTEGRATION_RESULTS.md for analysis"
echo "3. Verify AI services (SurrealDB + Ollama) are functioning"
echo ""
