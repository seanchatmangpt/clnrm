import re
import os

def analyze_file(file_path):
    with open(file_path, 'r') as f:
        lines = f.readlines()
    
    methods = []
    current_method = None
    brace_count = 0
    start_line = 0
    
    # Regex for Rust function definitions
    # Matches 'fn name(...)', 'pub fn name(...)', etc.
    # Also handles async and generic parameters
    fn_re = re.compile(r'^\s*(?:pub\s+)?(?:async\s+)?fn\s+([a-zA-Z0-9_]+)')
    
    for i, line in enumerate(lines):
        # Look for function start if not in a method
        if current_method is None:
            match = fn_re.search(line)
            if match and '{' in line:
                current_method = match.group(1)
                start_line = i
                brace_count = line.count('{') - line.count('}')
                if brace_count == 0: # One liner or immediate close
                    methods.append((current_method, 1, file_path))
                    current_method = None
            elif match: # Multiline definition
                current_method = match.group(1)
                start_line = i
                # Need to find the opening brace
                j = i
                while j < len(lines) and '{' not in lines[j]:
                    j += 1
                if j < len(lines):
                    brace_count = lines[j].count('{') - lines[j].count('}')
                    if brace_count == 0:
                        methods.append((current_method, j - i + 1, file_path))
                        current_method = None
                else:
                    # Should not happen in valid Rust
                    current_method = None
        else:
            brace_count += line.count('{')
            brace_count -= line.count('}')
            if brace_count <= 0:
                methods.append((current_method, i - start_line + 1, file_path))
                current_method = None
                
    return methods

files = [
    'crates/clnrm-core/src/config/spec.rs',
    'crates/clnrm-core/src/validation/orchestrator.rs',
    'crates/clnrm-core/src/telemetry/live_check/orchestrator.rs',
    'crates/clnrm-core/src/chaos/orchestrator.rs'
]

all_methods = []
for f in files:
    if os.path.exists(f):
        all_methods.extend(analyze_file(f))

# Sort by line count descending
all_methods.sort(key=lambda x: x[1], reverse=True)

print("Top 10 Most Complex Methods (Technical Inventory Waste):")
for i, (name, lines, path) in enumerate(all_methods[:10]):
    print(f"{i+1}. {name} ({lines} lines) in {path}")
