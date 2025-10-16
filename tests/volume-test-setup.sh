#!/bin/bash
# Setup script for volume mount TOML tests
#
# This script prepares the test environment for volume mount tests
# by creating necessary directories and test files.

set -e

echo "Setting up volume mount test environment..."

# Create test directories
mkdir -p /tmp/clnrm-test-input
mkdir -p /tmp/clnrm-test-output
mkdir -p /tmp/clnrm-test-config

# Create test input files
echo "Input data from host" > /tmp/clnrm-test-input/test-input.txt
echo "More input data" > /tmp/clnrm-test-input/data.txt
echo '{"test": "json data"}' > /tmp/clnrm-test-input/data.json

# Create test config files
echo "Config data from host" > /tmp/clnrm-test-config/test-config.txt
echo "app_name=test_app" > /tmp/clnrm-test-config/config.ini
echo "LOG_LEVEL=debug" > /tmp/clnrm-test-config/.env

# Set appropriate permissions
chmod -R 755 /tmp/clnrm-test-input
chmod -R 755 /tmp/clnrm-test-config
chmod -R 777 /tmp/clnrm-test-output  # Output needs write permissions

echo "Setup complete!"
echo ""
echo "Test directories created:"
echo "  - /tmp/clnrm-test-input (read-only mount)"
echo "  - /tmp/clnrm-test-output (read-write mount)"
echo "  - /tmp/clnrm-test-config (read-only mount)"
echo ""
echo "Run test with:"
echo "  cargo test --test integration_volume"
echo "  OR"
echo "  cargo run -- run tests/volume-mount-test.clnrm.toml"
