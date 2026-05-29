import os
import re
import json

def is_in_async_fn(content, index):
    before = content[:index]
    fn_defs = [(m.start(), m.group(0)) for m in re.finditer(r'(?:async\s+)?fn\s+\w+\s*\(', before)]
    if not fn_defs:
        return False
    return 'async' in fn_defs[-1][1]

calls = []

for root, dirs, files in os.walk('crates/clnrm-core/src'):
    for file in files:
        if not file.endswith('.rs'):
            continue
        path = os.path.join(root, file)
        with open(path, 'r') as f:
            content = f.read()
            
        lines = content.split('\n')
        
        # 1. fs matches
        fs_matches = list(re.finditer(r'\bstd::fs::(\w+)', content))
        for match in fs_matches:
            if is_in_async_fn(content, match.start()):
                line_idx = content[:match.start()].count('\n')
                start_line = max(0, line_idx - 3)
                end_line = min(len(lines), line_idx + 4)
                
                old_chunk = '\n'.join(lines[start_line:end_line])
                new_chunk = old_chunk.replace(match.group(0), f"tokio::fs::{match.group(1)}")
                
                if old_chunk != new_chunk:
                    calls.append({
                        "file": path,
                        "type": "fs",
                        "old": old_chunk,
                        "new": new_chunk
                    })
                    
        # Update lines array if we made fs matches to not conflict, actually
        # it's better to process files entirely if there are many.
        # But wait, we can just replace 'println!' globally in the file.
        has_println = False
        for l in lines:
            if 'println!' in l and not l.strip().startswith('//'):
                has_println = True
                break
        
        if has_println:
            calls.append({
                "file": path,
                "type": "println",
                "old": "println!",
                "new": "tracing::info!"
            })

with open('calls.json', 'w') as f:
    json.dump(calls, f, indent=2)

print(f"Total fs calls: {len([c for c in calls if c['type'] == 'fs'])}")
print(f"Total println calls: {len([c for c in calls if c['type'] == 'println'])}")
