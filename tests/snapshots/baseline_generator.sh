#!/bin/bash
# Baseline Snapshot Generator
# Generates baseline snapshots for all test suites

set -e

echo "🎯 Generating Baseline Snapshots"
echo "================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Directories
RUST_SNAPSHOTS="/Users/sac/clnrm/tests/snapshots/rust/__snapshots__"
CLI_SNAPSHOTS="/Users/sac/clnrm/tests/snapshots/cli/__snapshots__"
DATA_SNAPSHOTS="/Users/sac/clnrm/tests/snapshots/data/__snapshots__"
UI_SNAPSHOTS="/Users/sac/clnrm/examples/optimus-prime-platform/tests/__snapshots__"

# Create snapshot directories
echo -e "${YELLOW}Creating snapshot directories...${NC}"
mkdir -p "$RUST_SNAPSHOTS"
mkdir -p "$CLI_SNAPSHOTS"
mkdir -p "$DATA_SNAPSHOTS"
mkdir -p "$UI_SNAPSHOTS"

# 1. Generate Rust snapshots
echo -e "\n${YELLOW}1. Generating Rust snapshots...${NC}"
cd /Users/sac/clnrm

# Run tests with snapshot generation (if insta is installed)
if command -v cargo-insta &> /dev/null; then
    echo "Running Rust tests with insta..."
    cargo test --test '*' -- --nocapture || true
    cargo insta accept || true
    echo -e "${GREEN}✓ Rust snapshots generated${NC}"
else
    echo -e "${YELLOW}⚠ cargo-insta not installed. Skipping Rust snapshots.${NC}"
    echo "  Install with: cargo install cargo-insta"
fi

# 2. Generate CLI snapshots
echo -e "\n${YELLOW}2. Generating CLI output snapshots...${NC}"

# Help output
echo "Capturing help output..."
cargo run --bin clnrm -- --help > "$CLI_SNAPSHOTS/help_output.txt" 2>&1 || true

# Version output
echo "Capturing version output..."
cargo run --bin clnrm -- --version > "$CLI_SNAPSHOTS/version_output.txt" 2>&1 || true

# Run output (if test project exists)
if [ -d "/Users/sac/clnrm/test-project" ]; then
    echo "Capturing run output..."
    cd /Users/sac/clnrm/test-project
    cargo run --bin clnrm -- run > "$CLI_SNAPSHOTS/run_output.txt" 2>&1 || true
    cd /Users/sac/clnrm
fi

echo -e "${GREEN}✓ CLI snapshots generated${NC}"

# 3. Generate data structure snapshots
echo -e "\n${YELLOW}3. Generating data structure snapshots...${NC}"

# Create sample config files
cat > "$DATA_SNAPSHOTS/sample_config.json" << 'EOF'
{
  "name": "integration_test",
  "backend": {
    "type": "testcontainer",
    "image": "postgres:15",
    "ports": [5432]
  },
  "steps": [
    {
      "name": "setup",
      "command": "psql",
      "args": ["-c", "CREATE TABLE users (id INT)"],
      "expected_exit_code": 0
    }
  ],
  "policy": {
    "security_level": "high",
    "network_access": true,
    "file_system_access": false
  }
}
EOF

cat > "$DATA_SNAPSHOTS/sample_config.yaml" << 'EOF'
name: integration_test
backend:
  type: testcontainer
  image: postgres:15
  ports:
    - 5432
steps:
  - name: setup
    command: psql
    args:
      - "-c"
      - "CREATE TABLE users (id INT)"
    expected_exit_code: 0
policy:
  security_level: high
  network_access: true
  file_system_access: false
EOF

echo -e "${GREEN}✓ Data structure snapshots generated${NC}"

# 4. Generate UI component snapshots
echo -e "\n${YELLOW}4. Generating UI component snapshots...${NC}"

if [ -d "/Users/sac/clnrm/examples/optimus-prime-platform" ]; then
    cd /Users/sac/clnrm/examples/optimus-prime-platform

    # Check if npm is available and dependencies are installed
    if command -v npm &> /dev/null; then
        if [ -f "package.json" ]; then
            echo "Installing dependencies (if needed)..."
            npm install --silent || echo "Dependencies already installed"

            echo "Running UI snapshot tests..."
            npm test -- --updateSnapshot --silent 2>&1 || true
            echo -e "${GREEN}✓ UI snapshots generated${NC}"
        else
            echo -e "${YELLOW}⚠ package.json not found${NC}"
        fi
    else
        echo -e "${YELLOW}⚠ npm not installed. Skipping UI snapshots.${NC}"
    fi

    cd /Users/sac/clnrm
else
    echo -e "${YELLOW}⚠ UI project not found${NC}"
fi

# 5. Generate snapshot metadata
echo -e "\n${YELLOW}5. Generating snapshot metadata...${NC}"

METADATA_FILE="/Users/sac/clnrm/tests/snapshots/snapshot_metadata.json"
cat > "$METADATA_FILE" << EOF
{
  "generated_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "version": "1.0.0",
  "snapshots": {
    "rust": {
      "location": "$RUST_SNAPSHOTS",
      "count": $(find "$RUST_SNAPSHOTS" -type f 2>/dev/null | wc -l | tr -d ' ')
    },
    "cli": {
      "location": "$CLI_SNAPSHOTS",
      "count": $(find "$CLI_SNAPSHOTS" -type f 2>/dev/null | wc -l | tr -d ' ')
    },
    "data": {
      "location": "$DATA_SNAPSHOTS",
      "count": $(find "$DATA_SNAPSHOTS" -type f 2>/dev/null | wc -l | tr -d ' ')
    },
    "ui": {
      "location": "$UI_SNAPSHOTS",
      "count": $(find "$UI_SNAPSHOTS" -type f 2>/dev/null | wc -l | tr -d ' ')
    }
  }
}
EOF

echo -e "${GREEN}✓ Metadata generated${NC}"

# Summary
echo -e "\n${GREEN}================================${NC}"
echo -e "${GREEN}Baseline Generation Complete!${NC}"
echo -e "${GREEN}================================${NC}"
echo ""
echo "Snapshot locations:"
echo "  Rust:  $RUST_SNAPSHOTS"
echo "  CLI:   $CLI_SNAPSHOTS"
echo "  Data:  $DATA_SNAPSHOTS"
echo "  UI:    $UI_SNAPSHOTS"
echo ""
echo "Metadata: $METADATA_FILE"
echo ""
echo "Next steps:"
echo "  1. Review generated snapshots"
echo "  2. Commit snapshots to version control"
echo "  3. Run tests to verify snapshots work"
echo ""
echo -e "${YELLOW}Commands:${NC}"
echo "  cargo test                    # Run Rust tests"
echo "  cargo insta review           # Review Rust snapshots"
echo "  npm test                     # Run UI tests"
echo "  npm test -- -u               # Update UI snapshots"
