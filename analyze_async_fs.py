import os
import re

def is_in_async_fn(content, index):
    # Search backwards from the index to find the nearest function definition
    before = content[:index]
    # Find all function definitions before this point
    fn_defs = [(m.start(), m.group(0)) for m in re.finditer(r'(?:async\s+)?fn\s+\w+\s*\(', before)]
    if not fn_defs:
        return False
    
    last_fn = fn_defs[-1]
    return 'async' in last_fn[1]

def analyze_dir(d):
    for root, dirs, files in os.walk(d):
        for file in files:
            if file.endswith('.rs'):
                path = os.path.join(root, file)
                with open(path, 'r') as f:
                    content = f.read()
                
                # Check for std::fs
                for match in re.finditer(r'\bstd::fs::\w+', content):
                    if is_in_async_fn(content, match.start()):
                        print(f"ASYNC_FS: {path}:{content.count(os.linesep, 0, match.start()) + 1} {match.group(0)}")
                
                # Check for println!
                lines = content.split('\n')
                for i, line in enumerate(lines):
                    if 'println!' in line and not line.strip().startswith('//'):
                        # Just print line content
                        print(f"PRINTLN: {path}:{i+1} {line.strip()}")

analyze_dir('crates/clnrm-core/src')
