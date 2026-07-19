import os
import glob
from pathlib import Path

# Find all files in tests/ directories across the workspace
test_files = []
for root, dirs, files in os.walk('.'):
    if 'tests' in root.split(os.sep):
        for f in files:
            if f.endswith('.rs'):
                test_files.append(os.path.join(root, f))

for f in test_files:
    with open(f, 'r') as file:
        content = file.read()
    
    lines = content.split('\n')
    new_lines = []
    
    for line in lines:
        if not line.startswith('#![allow('):
            new_lines.append(line)
            
    inject = "#![allow(warnings, clippy::all, deprecated)]"
    final_lines = [inject] + new_lines
    
    with open(f, 'w') as file:
        file.write('\n'.join(final_lines))
    print(f"Updated {f}")
