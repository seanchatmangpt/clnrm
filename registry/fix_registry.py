import yaml
from pathlib import Path

def fix_dict(d):
    if not isinstance(d, dict):
        return
    
    # Process attributes list if it exists
    if 'attributes' in d and isinstance(d['attributes'], list):
        for attr in d['attributes']:
            if not isinstance(attr, dict):
                continue
            
            # 1. Fix requirement_level
            if attr.get('requirement_level') == 'optional':
                attr['requirement_level'] = 'opt_in'
            
            # 2. Fix enum type and members
            if attr.get('type') == 'enum':
                members = attr.pop('members', [])
                attr['type'] = {'members': members}
            
            # Recurse into attribute if needed
            fix_dict(attr)
            
    # Recurse into all dictionary values
    for k, v in d.items():
        if isinstance(v, dict):
            fix_dict(v)
        elif isinstance(v, list):
            for item in v:
                if isinstance(item, dict):
                    fix_dict(item)

def fix_file(file_path):
    print(f"Fixing {file_path}")
    with open(file_path, 'r') as f:
        data = yaml.safe_load(f)
        
    if not data:
        return
        
    fix_dict(data)
    
    with open(file_path, 'w') as f:
        yaml.dump(data, f, default_flow_style=False, sort_keys=False, width=100)

def main():
    registry_dir = Path("/Users/sac/clnrm/registry")
    for yaml_file in registry_dir.rglob('*.yaml'):
        if yaml_file.name == 'registry_manifest.yaml':
            continue
        fix_file(yaml_file)

if __name__ == '__main__':
    main()
