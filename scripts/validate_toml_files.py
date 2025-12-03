#!/usr/bin/env python3
"""Validate TOML files for syntax errors."""

import sys
import tomllib
from pathlib import Path

def validate_toml_file(filepath: Path) -> tuple[bool, str]:
    """Validate a TOML file and return (is_valid, error_message)."""
    try:
        with open(filepath, 'rb') as f:
            tomllib.load(f)
        return True, "Valid"
    except Exception as e:
        return False, str(e)

def main():
    files = [
        "examples/readme-example-validation.clnrm.toml",
        "examples/template-workflow/otel-template-example.clnrm.toml",
        "examples/case-studies/redteam-otlp-env.clnrm.toml",
        "examples/behaviors.clnrm.toml",
        "examples/multi-service-demo.clnrm.toml",
        "examples/surrealdb-integration-demo.clnrm.toml",
        "examples/volume-mount-demo.clnrm.toml",
    ]

    results = {}
    for file in files:
        filepath = Path(file)
        if not filepath.exists():
            results[file] = (False, "File not found")
        else:
            results[file] = validate_toml_file(filepath)

    print("TOML Validation Results:")
    print("=" * 80)

    valid_files = []
    invalid_files = []

    for file, (is_valid, message) in results.items():
        status = "✅ VALID" if is_valid else "❌ INVALID"
        print(f"{status}: {file}")
        if not is_valid:
            print(f"   Error: {message}")
            invalid_files.append((file, message))
        else:
            valid_files.append(file)

    print("=" * 80)
    print(f"Valid: {len(valid_files)}, Invalid: {len(invalid_files)}")

    if invalid_files:
        print("\nFiles needing fixes:")
        for file, error in invalid_files:
            print(f"  - {file}")

    return 0 if not invalid_files else 1

if __name__ == "__main__":
    sys.exit(main())
