from pathlib import Path
import datetime,hashlib,json,subprocess,sys
BASE=Path(__file__).resolve().parent
ROOT=BASE.parents[4]
name=sys.argv[1]
assert '/' not in name and name.endswith('.json')
files=[]
for p in ['crates/engine-native/src','crates/engine-native-content/src','crates/engine-native-kernel/src','crates/content-native-loader/src']:
 files.extend((ROOT/p).rglob('*.rs'))
files.append(ROOT/'packages/extensions/wudian-upstream-noname/assets/cards.json')
up=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-queue1-runtime-modern-b789ktt1/apps/core')
for p in ['extension/玩点论杀/character/skill.js','extension/玩点论杀/card/card.js','extension/玩点论杀/card/skill.js']:
 files.append(up/p)
binary=ROOT/'target/debug/queue1-u07-runtime-probe'
if binary.exists():files.append(binary)
row={'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'tracked_dirty':subprocess.check_output(['git','status','--porcelain=v1','--untracked-files=no'],cwd=ROOT,text=True),'rules':333,'ngsa':107,'schema':5,'abi':759,'ws':8,'sha256':{str(p):hashlib.sha256(p.read_bytes()).hexdigest()for p in sorted(files)}}
with (BASE/name).open('x')as out:out.write(json.dumps(row,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'record':name,'files':len(files),'head':row['head'],'dirty':row['tracked_dirty']},ensure_ascii=False))
