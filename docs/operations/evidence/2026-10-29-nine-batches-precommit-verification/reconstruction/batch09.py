from pathlib import Path
import json,shutil,hashlib
snap=Path('../final-files');manifest=json.loads((snap/'snapshot-manifest.json').read_text())
for row in manifest:
 p=Path(row['path'])
 if row['sha256'] is None:
  if p.exists():p.unlink()
 else:
  assert hashlib.sha256((snap/p).read_bytes()).hexdigest()==row['sha256'],str(p)
  p.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(snap/p,p);p.touch()
print('Copied exactly the 757 frozen manifest paths, including the one deletion; no main-worktree product file was copied.')
