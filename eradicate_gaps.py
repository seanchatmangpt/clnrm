import os
import re

directories_to_scan = [
    "crates/clnrm-core/src",
    "crates/clnrm-cli/src"
]

def process_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    original_content = content

    # 1. Flag "For now," in comments
    # Avoid double prefixing
    content = re.sub(r'//(?!\s*EXAMPLE-ONLY:)\s*For now,', '// EXAMPLE-ONLY: For now,', content)
    
    # 2. Flag "In a real implementation" / "In a full implementation"
    content = re.sub(r'//(?!\s*EXAMPLE-ONLY:)\s*In a real implementation', '// EXAMPLE-ONLY: In a real implementation', content)
    content = re.sub(r'//(?!\s*EXAMPLE-ONLY:)\s*In a full implementation', '// EXAMPLE-ONLY: In a full implementation', content)
    content = re.sub(r'//(?!\s*EXAMPLE-ONLY:)\s*In a future version', '// EXAMPLE-ONLY: In a future version', content)
    
    # 3. Flag TODOs
    content = re.sub(r'//\s*TODO:?', '// ORACLE-GAP Refusal:', content)

    # 4. Flag raw unimplemented!()
    content = re.sub(r'unimplemented!\(\)', 'unimplemented!("ORACLE-GAP Refusal: Path not fully mapped")', content)

    if content != original_content:
        print(f"Updated {filepath}")
        with open(filepath, 'w') as f:
            f.write(content)

def main():
    for dir_path in directories_to_scan:
        for root, dirs, files in os.walk(dir_path):
            for file in files:
                if file.endswith('.rs'):
                    process_file(os.path.join(root, file))

if __name__ == "__main__":
    main()
