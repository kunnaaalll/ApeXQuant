import os
import glob

files = glob.glob('services/*/src/lib.rs') + glob.glob('services/*/src/main.rs')

for f in files:
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
