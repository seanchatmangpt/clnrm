#!/usr/bin/env python3
"""
Fix schema files to add missing stability and span_kind fields.
"""

import yaml
import sys
from pathlib import Path

def fix_schema_file(file_path: Path):
    """Add stability fields to all groups and attributes in a schema file."""
    print(f"Fixing {file_path}...")

    with open(file_path, 'r') as f:
        data = yaml.safe_load(f)

    if not data or 'groups' not in data:
        print(f"  Skipping {file_path} - no groups found")
        return

    modified = False

    for group in data['groups']:
        # Add stability to group if missing
        if 'stability' not in group:
            group['stability'] = 'stable'
            modified = True

        # Add span_kind to span groups if missing
        if group.get('type') == 'span' and 'span_kind' not in group:
            group['span_kind'] = 'internal'
            modified = True

        # Fix attributes
        if 'attributes' in group:
            for attr in group['attributes']:
                if 'stability' not in attr:
                    attr['stability'] = 'stable'
                    modified = True

                # Fix enum members
                if 'type' in attr and isinstance(attr['type'], dict):
                    if 'members' in attr['type']:
                        for member in attr['type']['members']:
                            if 'stability' not in member:
                                member['stability'] = 'stable'
                                modified = True

    if modified:
        with open(file_path, 'w') as f:
            yaml.dump(data, f, default_flow_style=False, sort_keys=False, width=100)
        print(f"  Fixed {file_path}")
    else:
        print(f"  No changes needed for {file_path}")

def main():
    registry_dir = Path(__file__).parent

    # Fix all YAML files in registry subdirectories
    for yaml_file in registry_dir.rglob('*.yaml'):
        # Skip the manifest
        if yaml_file.name == 'registry_manifest.yaml':
            continue
        fix_schema_file(yaml_file)

    print("\nDone! Run 'weaver registry check -r registry/' to validate.")

if __name__ == '__main__':
    main()
