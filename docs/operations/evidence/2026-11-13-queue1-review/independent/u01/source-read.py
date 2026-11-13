from pathlib import Path
import re,sys
root=Path(__file__).resolve().parents[6]
source=root.parent/'archive/upstream/noname/apps/core/extension/3D精选/character/skill.js'
lines=source.read_text().splitlines()
start=int(sys.argv[1]);end=int(sys.argv[2]);skip=None
for i,line in enumerate(lines,1):
    if skip is not None:
        if line.startswith(skip+'},') or line==skip+'}': skip=None
        continue
    m=re.match(r'^(\s*)ai: \{',line)
    if m: skip=m.group(1);continue
    if start<=i<=end and line.strip(): print(f'{i}: {line.strip()}')
