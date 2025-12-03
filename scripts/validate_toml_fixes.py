#!/usr/bin/env python3
"""Validate the TOML fixes"""

import tomllib
import sys
from pathlib import Path

def validate_file(path: Path) -> tuple[bool, str]:
    """Validate a single TOML file"""
    try:
        with open(path, 'rb') as f:
            tomllib.load(f)
        return True, "✅ Valid TOML"
    except Exception as e:
        return False, f"❌ Error: {e}"

def main():
    """Validate specific fixed files"""
    files = [
        "examples/behaviors.clnrm.toml",
        "examples/live-check/ci-cd.clnrm.toml",
    ]

    root = Path("/Users/sac/clnrm")
    print("Validating fixed TOML files...\n")

    all_valid = True
    for file in files:
        path = root / file
        if not path.exists():
            print(f"❌ {file}: File not found")
            all_valid = False
            continue

        valid, msg = validate_file(path)
        print(f"{msg}: {file}")
        if not valid:
            all_valid = False

    print()
    if all_valid:
        print("✅ All fixes validated successfully!")
        return 0
    else:
        print("❌ Some files still have errors")
        return 1

if __name__ == "__main__":
    sys.exit(main())
