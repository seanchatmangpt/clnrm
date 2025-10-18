#!/usr/bin/env python3
"""
Script to remove all #[cfg(test)] modules from Rust source files.
This removes test functions while preserving testing infrastructure.
"""

import os
import re
import sys
from pathlib import Path

def remove_test_modules(file_path):
    """Remove #[cfg(test)] modules from a Rust file."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Pattern to match #[cfg(test)] modules
        # This matches from #[cfg(test)] to the closing brace of the module
        pattern = r'#\[cfg\(test\)\]\s*\n(?:\s*#!\[allow\([^)]*\)\]\s*\n)?(?:\s*use[^;]*;\s*\n)*\s*mod\s+\w+\s*\{[^{}]*(?:\{[^{}]*\}[^{}]*)*\}'
        
        # More comprehensive pattern that handles nested braces
        lines = content.split('\n')
        new_lines = []
        i = 0
        in_test_module = False
        brace_count = 0
        
        while i < len(lines):
            line = lines[i]
            
            # Check if this line starts a test module
            if re.match(r'^\s*#\[cfg\(test\)\]', line):
                in_test_module = True
                brace_count = 0
                # Skip this line and continue to find the module start
                i += 1
                continue
            
            if in_test_module:
                # Count braces to find the end of the module
                for char in line:
                    if char == '{':
                        brace_count += 1
                    elif char == '}':
                        brace_count -= 1
                        if brace_count == 0:
                            # End of test module found
                            in_test_module = False
                            i += 1
                            continue
                
                # Skip lines inside test module
                i += 1
                continue
            
            # Not in test module, keep the line
            new_lines.append(line)
            i += 1
        
        new_content = '\n'.join(new_lines)
        
        # If content changed, write it back
        if new_content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(new_content)
            print(f"Removed test modules from: {file_path}")
            return True
        
        return False
        
    except Exception as e:
        print(f"Error processing {file_path}: {e}")
        return False

def main():
    """Main function to process all Rust files."""
    if len(sys.argv) > 1:
        root_dir = Path(sys.argv[1])
    else:
        root_dir = Path.cwd()
    
    rust_files = []
    
    # Find all Rust files in crates/
    for crate_dir in root_dir.glob("crates/*"):
        if crate_dir.is_dir():
            for rust_file in crate_dir.rglob("*.rs"):
                rust_files.append(rust_file)
    
    # Also process other Rust files mentioned in the plan
    for pattern in ["tests/**/*.rs", "examples/**/*.rs", "swarm/**/*.rs"]:
        for rust_file in root_dir.glob(pattern):
            rust_files.append(rust_file)
    
    processed_count = 0
    modified_count = 0
    
    for rust_file in rust_files:
        if rust_file.is_file():
            processed_count += 1
            if remove_test_modules(rust_file):
                modified_count += 1
    
    print(f"\nProcessed {processed_count} Rust files")
    print(f"Modified {modified_count} files")

if __name__ == "__main__":
    main()
