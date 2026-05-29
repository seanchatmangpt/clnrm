import os
import re
import json

def is_in_async_fn(content, index):
    before = content[:index]
    fn_defs = [(m.start(), m.group(0)) for m in re.finditer(r'(?:async\s+)?fn\s+\w+\s*\(', before)]
    if not fn_defs:
        return False
    return 'async' in fn_defs[-1][1]

def add_await(content, start_idx):
    # start_idx is the index of 'std::fs::'
    prefix = 'std::fs::'
    
    # find the function name
    m = re.match(r'(\w+)\s*\(', content[start_idx + len(prefix):])
    if not m:
        # maybe it's not a function call, e.g. std::fs::File
        if content[start_idx + len(prefix):].startswith('File::'):
            # std::fs::File::create(path) -> tokio::fs::File::create(path).await
            m2 = re.match(r'File::(\w+)\s*\(', content[start_idx + len(prefix):])
            if m2:
                func_name = 'File::' + m2.group(1)
            else:
                return None
        else:
            return None
    else:
        func_name = m.group(1)
        
    call_start = start_idx + len(prefix) + len(func_name)
    # now balance parens
    depth = 0
    i = call_start
    while i < len(content):
        if content[i] == '(':
            depth += 1
        elif content[i] == ')':
            depth -= 1
            if depth == 0:
                end_idx = i + 1
                return end_idx
        i += 1
    return None

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
        fs_matches = list(re.finditer(r'\bstd::fs::', content))
        # reverse to avoid index shifting
        for match in reversed(fs_matches):
            if is_in_async_fn(content, match.start()):
                end_idx = add_await(content, match.start())
                if end_idx:
                    # check if it already has .await
                    if content[end_idx:end_idx+6] == '.await':
                        continue # already has await (maybe already tokio?)
                        
                    old_str = content[match.start():end_idx]
                    new_str = old_str.replace('std::fs::', 'tokio::fs::') + '.await'
                    
                    line_idx = content[:match.start()].count('\n')
                    start_line = max(0, line_idx - 3)
                    end_line = min(len(lines), line_idx + 4)
                    
                    old_chunk = '\n'.join(lines[start_line:end_line])
                    new_chunk = old_chunk.replace(old_str, new_str)
                    
                    if old_chunk != new_chunk:
                        calls.append({
                            "file": path,
                            "type": "fs",
                            "old": old_chunk,
                            "new": new_chunk
                        })
                        
                        # update lines for subsequent replacements in same file
                        lines = new_chunk.split('\n')
                        content = '\n'.join(lines)
                        
with open('calls_v2.json', 'w') as f:
    json.dump(calls, f, indent=2)

print(f"Total fs calls: {len([c for c in calls if c['type'] == 'fs'])}")
