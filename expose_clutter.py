#!/usr/bin/env python3
import os
import time
import glob
import sys
from pathlib import Path

# Ghost file extensions to track
EXTENSIONS = ['.stdout', '.lock', '.json', '.tar']

# Standard locations to scan
SCAN_PATHS = [
    '/tmp',
    os.path.expanduser('~/.cache/clnrm'),
    os.path.expanduser('~/Library/Caches/clnrm'),
    os.getcwd()
]

def get_file_info(path):
    """Returns (age_seconds, size_bytes) for a given path."""
    try:
        stat = os.stat(path)
        age = time.time() - stat.st_mtime
        return age, stat.st_size
    except Exception:
        return None, None

def format_age(seconds):
    """Formats age into a human-readable string."""
    if seconds is None: return "???"
    if seconds < 60: return f"{seconds:.1f}s"
    if seconds < 3600: return f"{seconds/60:.1f}m"
    if seconds < 86400: return f"{seconds/3600:.1f}h"
    return f"{seconds/86400:.1f}d"

def expose_clutter():
    """Scans and prints ghost files."""
    print(f"{'GHOST FILE':<70} | {'AGE':<10} | {'SIZE':<10}")
    print("=" * 95)
    
    clutter_count = 0
    for base in SCAN_PATHS:
        if not os.path.exists(base):
            continue
            
        # Search for artifacts matching extensions
        for ext in EXTENSIONS:
            # Use glob to find files recursively
            pattern = os.path.join(base, f"**/*{ext}")
            for path in glob.iglob(pattern, recursive=True):
                # Ignore noisy directories
                if any(x in path for x in ['/target/', '/.git/', '/node_modules/', '/Library/Caches/com.apple']):
                    continue
                
                # Filter specifically for artifacts in project root if scanning cwd
                if base == os.getcwd():
                    # Only root level or specific subdirs that are known to leak
                    depth = len(Path(path).relative_to(base).parts)
                    if depth > 2: # Keep it focused on top-level clutter
                        continue

                age, size = get_file_info(path)
                if age is not None:
                    print(f"{path[:68]:<70} | {format_age(age):<10} | {size:<10}")
                    clutter_count += 1

    print("=" * 95)
    print(f"Total Ghost Files Detected: {clutter_count}")
    
    if clutter_count > 0:
        print("\nSYSTEM FAILURE: The workspace fails to 'Shine' due to persistent temporary state leakage.")
    else:
        print("\nWorkspace is clean.")

if __name__ == "__main__":
    expose_clutter()
