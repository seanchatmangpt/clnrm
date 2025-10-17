#!/bin/bash
# Master test runner for README verification tests
# Runs all 30 README example tests and generates comprehensive report

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_DIR="${SCRIPT_DIR}/reports/${TIMESTAMP}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  README Verification Test Suite${NC}"
echo -e "${BLUE}  30 comprehensive tests covering all README claims${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Create report directory
mkdir -p "${REPORT_DIR}"

# Test counters
total_tests=0
passed_tests=0
failed_tests=0

# Test categories
declare -a categories=(
  "cmd:CLI Commands:8"
  "feature:Features:8"
  "example:Code Examples:6"
  "metric:Metrics:4"
  "install:Installation:3"
  "plugin:Plugins:2"
  "service:Services:1"
  "documentation:Documentation:1"
  "template:Templates:1"
  "advanced:Advanced:1"
  "version:Version:1"
  "prerequisites:Prerequisites:1"
)

# Function to run test category
run_category() {
  local prefix=$1
  local name=$2
  local expected=$3

  echo -e "${YELLOW}▶ Running ${name} Tests${NC}"
  echo ""

  local count=0
  local failed=0

  for test_file in "${SCRIPT_DIR}/${prefix}"_*.clnrm.toml*; do
    if [ -f "$test_file" ]; then
      count=$((count + 1))
      total_tests=$((total_tests + 1))

      test_name=$(basename "$test_file" .clnrm.toml)
      echo -n "  Testing: ${test_name}... "

      if clnrm run "$test_file" > "${REPORT_DIR}/${test_name}.log" 2>&1; then
        echo -e "${GREEN}✓ PASS${NC}"
        passed_tests=$((passed_tests + 1))
      else
        echo -e "${RED}✗ FAIL${NC}"
        failed_tests=$((failed_tests + 1))
        failed=1
      fi
    fi
  done

  echo ""
  if [ $failed -eq 0 ]; then
    echo -e "${GREEN}✓ ${name}: All ${count} tests passed${NC}"
  else
    echo -e "${RED}✗ ${name}: Some tests failed${NC}"
  fi
  echo ""
}

# Run all test categories
for category_info in "${categories[@]}"; do
  IFS=: read -r prefix name expected <<< "$category_info"
  run_category "$prefix" "$name" "$expected"
done

# Generate summary report
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  Test Summary${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "  Total Tests:  ${total_tests}"
echo -e "  Passed:       ${GREEN}${passed_tests}${NC}"
echo -e "  Failed:       ${RED}${failed_tests}${NC}"
echo ""

# Calculate pass rate
if [ $total_tests -gt 0 ]; then
  pass_rate=$((passed_tests * 100 / total_tests))
  echo "  Pass Rate:    ${pass_rate}%"
fi

echo ""
echo "  Report Dir:   ${REPORT_DIR}"
echo ""

# Generate JSON report
cat > "${REPORT_DIR}/summary.json" << EOF
{
  "timestamp": "${TIMESTAMP}",
  "total_tests": ${total_tests},
  "passed": ${passed_tests},
  "failed": ${failed_tests},
  "pass_rate": ${pass_rate},
  "categories": [
$(for category_info in "${categories[@]}"; do
  IFS=: read -r prefix name expected <<< "$category_info"
  echo "    {\"name\": \"${name}\", \"prefix\": \"${prefix}\", \"expected\": ${expected}},"
done | sed '$ s/,$//')
  ]
}
EOF

# Exit with appropriate code
if [ $failed_tests -eq 0 ]; then
  echo -e "${GREEN}✓ All README verification tests passed!${NC}"
  echo ""
  exit 0
else
  echo -e "${RED}✗ Some tests failed. Check logs in ${REPORT_DIR}${NC}"
  echo ""
  exit 1
fi
