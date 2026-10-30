import json,shutil,subprocess,sys
from pathlib import Path
root=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-j2-llgkp_le/worktree')
args=sys.argv[1:]
free=shutil.disk_usage(root).free
print(json.dumps({'free_bytes_before_heavy_command':free,'minimum_free_bytes':3*1024**3,'actual_command':args}),flush=True)
if free <3*1024**3:
 print('STOP: less than3GiB free; no heavy command started.',flush=True);sys.exit(75)
assert 'CARGO_INCREMENTAL=0' in args
value=next((a.split('=',1)[1] for a in args if a.startswith('CARGO_TARGET_DIR=')),None)
target=Path(value) if value else None
if target:
 assert target.name in ['target-nc09','target-gate'] and target.parent.name=='noname-nc09-pe1axnwg'
 transient=[target/'debug/incremental',target/'release/incremental',target/'wasm32-unknown-unknown/release/incremental',target/'.tmp',target/'tmp']
else:transient=[]
before={p:set(p.iterdir()) if p.is_dir() else set() for p in transient}
result=subprocess.run(args,cwd=root)
removed=[];retained_tmp=[]
for directory in transient:
 if not directory.is_dir():continue
 for child in set(directory.iterdir())-before[directory]:
  if directory.name=='incremental':
   if child.is_dir():shutil.rmtree(child)
   else:child.unlink()
   removed.append(str(child))
  elif child.is_dir() and not any(child.iterdir()):child.rmdir();removed.append(str(child))
  else:retained_tmp.append(str(child))
print(json.dumps({'actual_command':args,'exit':result.returncode,'free_bytes_after_command':shutil.disk_usage(root).free,'new_private_incremental_or_empty_tmp_removed':removed,'temporary_test_outputs_retained_for_same_batch':retained_tmp,'retention_reason':'deps, build outputs, test executables and nonempty test fixtures are needed by remaining native/workspace/WASM checks; CARGO_INCREMENTAL=0 generates no reusable incremental cache. Parent will retire whole targets after the batch.'}),flush=True)
sys.exit(result.returncode)
