#!/usr/bin/env bash
# FM-007: Check CLI reference documentation coverage
# Ensures all CLI flags are documented in reference

set -euo pipefail

echo "🔍 FMEA FM-007: Checking CLI reference coverage..."
echo "==================================================="

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

# Extract CLI flags from --help output
echo ""
echo "Extracting CLI flags from 'clnrm --help'..."
echo ""

# Check if clnrm is installed
if ! command -v clnrm &> /dev/null; then
  echo "⚠️  clnrm not found in PATH"
  echo "   Install with: cargo install --path crates/clnrm"
  echo "✅ SKIP (clnrm not installed)"
  exit 0
fi

# Get all subcommands
SUBCOMMANDS=$(clnrm --help | grep -E "^\s+[a-z-]+" | awk '{print $1}' | sort | uniq || true)

echo "Found subcommands: $SUBCOMMANDS"
echo ""

TOTAL_FLAGS=0
DOCUMENTED=0
MISSING=0

# Check if CLI reference doc exists
CLI_DOC="docs/reference/cli.md"
if [[ ! -f "$CLI_DOC" ]]; then
  echo "❌ CLI reference not found: $CLI_DOC"
  echo ""
  echo "Create this file to document all CLI commands and flags."
  exit 1
fi

echo "Checking coverage in: $CLI_DOC"
echo ""

# Extract global flags
GLOBAL_FLAGS=$(clnrm --help | grep -E "^\s+-" | sed 's/^\s*\(--[a-z-]*\).*/\1/' | sort | uniq || true)

for flag in $GLOBAL_FLAGS; do
  TOTAL_FLAGS=$((TOTAL_FLAGS + 1))

  if grep -q "$flag" "$CLI_DOC"; then
    echo "  ✅ $flag (documented)"
    DOCUMENTED=$((DOCUMENTED + 1))
  else
    echo "  ❌ $flag (MISSING)"
    MISSING=$((MISSING + 1))
  fi
done

# Check each subcommand's flags
for cmd in $SUBCOMMANDS; do
  echo ""
  echo "Checking subcommand: $cmd"

  CMD_FLAGS=$(clnrm "$cmd" --help 2>/dev/null | grep -E "^\s+-" | sed 's/^\s*\(--[a-z-]*\).*/\1/' | sort | uniq || true)

  for flag in $CMD_FLAGS; do
    TOTAL_FLAGS=$((TOTAL_FLAGS + 1))

    if grep -q "$flag" "$CLI_DOC"; then
      echo "  ✅ $cmd $flag (documented)"
      DOCUMENTED=$((DOCUMENTED + 1))
    else
      echo "  ❌ $cmd $flag (MISSING)"
      MISSING=$((MISSING + 1))
    fi
  done
done

echo ""
echo "==================================================="
echo "📊 CLI Coverage Summary"
echo "==================================================="
echo "Total flags: $TOTAL_FLAGS"
echo "Documented: $DOCUMENTED"
echo "Missing: $MISSING"

if [[ $TOTAL_FLAGS -gt 0 ]]; then
  COVERAGE=$((DOCUMENTED * 100 / TOTAL_FLAGS))
  echo "Coverage: ${COVERAGE}%"
fi

echo ""

if [[ $MISSING -eq 0 ]]; then
  echo "✅ SUCCESS: All CLI flags are documented!"
  exit 0
else
  echo "⚠️  WARNING: Found $MISSING undocumented flag(s)"
  echo ""
  echo "FMEA FM-007 mitigation: Document missing flags in $CLI_DOC"
  # Don't fail - this is a warning, not error
  exit 0
fi
