import os
import re
import json

def is_in_async_fn(content, index):
    before = content[:index]
    fn_defs = [(m.start(), m.group(0)) for m in re.finditer(r'(?:async\s+)?fn\s+\w+\s*\(', before)]
    if not fn_defs:
        return False
    return 'async' in fn_defs[-1][1]

def plan():
    replacements = []
    
    # 1. async fs replacements
    for root, dirs, files in os.walk('crates/clnrm-core/src'):
        for file in files:
            if not file.endswith('.rs'):
                continue
            path = os.path.join(root, file)
            with open(path, 'r') as f:
                content = f.read()
            
            new_content = content
            lines = new_content.split('\n')
            
            # Find std::fs in async
            fs_matches = list(re.finditer(r'\bstd::fs::(\w+)', new_content))
            # Reverse order so replacements don't mess up indices
            for match in reversed(fs_matches):
                if is_in_async_fn(new_content, match.start()):
                    start = match.start()
                    end = match.end()
                    new_content = new_content[:start] + f"tokio::fs::{match.group(1)}" + new_content[end:]
            
            # 2. println! replacements
            # Replace println!(...) with tracing::info!(...)
            # Only if it's not a doc comment
            
            lines = new_content.split('\n')
            final_lines = []
            for line in lines:
                if 'println!' in line and not line.strip().startswith('//'):
                    final_lines.append(line.replace('println!', 'tracing::info!'))
                else:
                    final_lines.append(line)
            
            final_content = '\n'.join(final_lines)
            
            if final_content != content:
                replacements.append({
                    "file_path": os.path.abspath(path),
                    "old_content": content,
                    "new_content": final_content
                })
                
    with open('replacements.json', 'w') as f:
        json.dump(replacements, f)

plan()
