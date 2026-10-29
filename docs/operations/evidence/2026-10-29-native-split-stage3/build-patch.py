from pathlib import Path
import difflib,hashlib,json
baseline=Path(__file__).parent/'baseline';root=Path.cwd();changes=[];chunks=[]
for folder in ['crates/engine-native/src','crates/engine-native/tests','scripts']:
 old={str(p.relative_to(baseline)):p for p in (baseline/folder).rglob('*') if p.is_file() and p.suffix in ['.py','.rs']}
 cur={str(p.relative_to(root)):p for p in (root/folder).rglob('*') if p.is_file() and p.suffix in ['.py','.rs']}
 for name in sorted(old.keys()|cur.keys()):
  before=old[name].read_bytes() if name in old else b'';after=cur[name].read_bytes() if name in cur else b''
  if before==after:continue
  action='modified' if name in old and name in cur else 'added' if name in cur else 'deleted'
  changes.append({'path':name,'action':action,'beforeSha256':hashlib.sha256(before).hexdigest() if name in old else None,'afterSha256':hashlib.sha256(after).hexdigest() if name in cur else None})
  chunks.append(f'diff --git a/{name} b/{name}\n')
  if action=='added':chunks.append('new file mode 100644\n')
  elif action=='deleted':chunks.append('deleted file mode 100644\n')
  chunks.extend(difflib.unified_diff(before.decode().splitlines(keepends=True),after.decode().splitlines(keepends=True),fromfile='a/'+name if name in old else '/dev/null',tofile='b/'+name if name in cur else '/dev/null'))
out=Path(__file__).parent;(out/'f7-stage3.patch').write_text(''.join(chunks));(out/'changed-files.json').write_text(json.dumps(changes,indent=2)+'\n');print('files',len(changes));print({action:sum(c['action']==action for c in changes) for action in ['added','modified','deleted']});print('patch sha256',hashlib.sha256((out/'f7-stage3.patch').read_bytes()).hexdigest())
