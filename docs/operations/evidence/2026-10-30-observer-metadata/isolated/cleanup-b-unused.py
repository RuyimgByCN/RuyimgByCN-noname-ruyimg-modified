from pathlib import Path
import subprocess,shutil,json
root=Path.cwd();own=root/'target';private=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-referee-pbhzrko1/target')
assert not subprocess.check_output(['git','ls-files','target']).strip()
rows=subprocess.check_output(['ps','-Ao','pid,command']).decode().splitlines()[1:]
active=[]
for row in rows:
 parts=row.strip().split(None,1)
 if len(parts)<2 or parts[1].split()[0].rsplit('/',1)[-1] not in {'cargo','rustc'}:continue
 result=subprocess.run(['lsof','-a','-p',parts[0],'-d','cwd','-Fn'],capture_output=True,text=True)
 if 'n'+str(root) in result.stdout:active.append(parts[0])
assert not active,active
record={'authorization':'new explicit user cleanup direction relayed by parent','free_before':shutil.disk_usage(root).free,'removed_own_observer_target':None,'private_unused_removed':[]}
if own.is_dir():
 assert not own.is_symlink()
 record['removed_own_observer_target']={'path':str(own),'du':subprocess.check_output(['du','-sh',str(own)]).decode().strip()}
 shutil.rmtree(own)
for rel in ['debug/incremental','release/incremental','wasm32-unknown-unknown/release/incremental','tmp']:
 p=private/rel
 if p.is_dir():shutil.rmtree(p);record['private_unused_removed'].append(str(p))
record['free_after']=shutil.disk_usage(root).free
Path('../evidence/b-unused-cache-cleanup-final.json').write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps(record))
