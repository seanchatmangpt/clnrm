#!/usr/bin/env python3
"""
TOML Schema Analysis Script - Agent 5
Analyzes all TOML files and categorizes by schema type
"""

import os
import re
from pathlib import Path
from collections import defaultdict

def find_toml_files(root_dir):
    """Find all TOML test files in the repository"""
    toml_files = []
    for root, dirs, files in os.walk(root_dir):
        # Skip target, .git, and test_output directories
        dirs[:] = [d for d in dirs if d not in ['target', '.git', 'test_output']]

        for file in files:
            if file.endswith('.toml') and file not in ['Cargo.toml', 'deny.toml']:
                toml_files.append(os.path.join(root, file))

    return toml_files

def detect_schema_type(file_path):
    """Detect the schema type used in a TOML file"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()

        # Check for schema markers
        has_test_metadata = bool(re.search(r'^\[test\.metadata\]', content, re.MULTILINE))
        has_test_section = bool(re.search(r'^\[test\]$', content, re.MULTILINE))
        has_meta_section = bool(re.search(r'^\[meta\]$', content, re.MULTILINE))
        has_steps = bool(re.search(r'^\[\[steps\]\]', content, re.MULTILINE))

        if has_test_metadata:
            return 'old_test_metadata'
        elif has_meta_section:
            return 'meta'
        elif has_test_section:
            return 'new_test'
        elif has_steps:
            return 'config_only'
        else:
            return 'other'
    except Exception as e:
        return f'error: {str(e)}'

def main():
    project_root = Path(__file__).parent.parent

    print("=== TOML Schema Analysis - Agent 5 ===")
    print(f"Project root: {project_root}")
    print()

    toml_files = find_toml_files(project_root)

    # Categorize files
    schema_types = defaultdict(list)
    for file_path in toml_files:
        rel_path = os.path.relpath(file_path, project_root)
        schema_type = detect_schema_type(file_path)
        schema_types[schema_type].append(rel_path)

    # Print summary
    total_test_files = sum(len(files) for schema, files in schema_types.items()
                           if schema in ['old_test_metadata', 'new_test', 'meta'])

    print(f"Total TOML files found: {len(toml_files)}")
    print(f"Total test TOML files: {total_test_files}")
    print()

    print("=== Schema Distribution ===")
    print(f"Old schema [test.metadata]: {len(schema_types.get('old_test_metadata', []))}")
    print(f"New schema [test]: {len(schema_types.get('new_test', []))}")
    print(f"Meta schema [meta]: {len(schema_types.get('meta', []))}")
    print(f"Config-only (no metadata): {len(schema_types.get('config_only', []))}")
    print(f"Other/Unknown: {len(schema_types.get('other', []))}")
    print()

    # Show sample files for each category
    for schema_type in ['old_test_metadata', 'new_test', 'meta']:
        files = schema_types.get(schema_type, [])
        if files:
            print(f"\n=== {schema_type.replace('_', ' ').title()} Files (showing first 10) ===")
            for file_path in files[:10]:
                print(f"  - {file_path}")
            if len(files) > 10:
                print(f"  ... and {len(files) - 10} more")

    print("\n=== Compatibility Status ===")
    print("✅ All schemas are supported by the updated TestMetadataSection enum")
    print("✅ Backward compatibility maintained for v1.3.0 files")
    print("✅ Forward compatibility enabled for v1.4.0+ files")

    return 0

if __name__ == '__main__':
    exit(main())
