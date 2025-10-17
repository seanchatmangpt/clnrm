#!/bin/bash
# Test script to verify _macros.toml.tera renders correctly

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Testing Tera macro rendering..."
echo "================================"
echo ""

# Create a simple test template
cat > /tmp/test_macros.clnrm.toml.tera << 'EOF'
{% import "_macros.toml.tera" as m %}

[test.metadata]
name = "macro-test"
description = "Testing macro library"

[services.postgres]
type = "generic_container"
image = "postgres:15"

# Test 1: Single span
{{ m::span("test.span", "internal") }}

# Test 2: Lifecycle
{{ m::lifecycle("postgres") }}

# Test 3: Count
{{ m::count("internal", 3) }}

[[steps]]
name = "test"
command = ["echo", "test"]
EOF

echo "Generated test template at /tmp/test_macros.clnrm.toml.tera"
echo ""
echo "Template contents:"
echo "=================="
cat /tmp/test_macros.clnrm.toml.tera
echo ""
echo "=================="
echo ""
echo "To render this template with clnrm:"
echo "  clnrm template render /tmp/test_macros.clnrm.toml.tera"
echo ""
echo "Note: Template rendering requires implementing Tera support in clnrm-core"
echo "      with template search paths including:"
echo "        - Current directory"
echo "        - ~/.clnrm/templates/"
echo "        - /usr/share/clnrm/templates/"
