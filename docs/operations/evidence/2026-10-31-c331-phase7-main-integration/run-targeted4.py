from pathlib import Path
import os,json,subprocess,time,shutil
root=Path(__file__).resolve().parents[2];work=root/'worktree';ev=Path(__file__).parent;env=os.environ.copy();env.update(CARGO_TARGET_DIR='/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-c331-efhmbq7e/target',CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='2');commands=[('c-choice2',['cargo','test','--offline','-p','engine-native','--test','nc08']),('frontend-type',['pnpm','--dir','apps/web-client','typecheck']),('frontend-tests',['pnpm','--dir','apps/web-client','test'])];results=[]
for name,args in commands:
 assert shutil.disk_usage(work).free>=3*1024**3
 start=time.time()
 with(ev/(name+'.log')).open('w')as log:r=subprocess.run(['python3','/tmp/noname-run-serialized.py',*args],cwd=work,env=env,stdout=log,stderr=subprocess.STDOUT)
 results.append({'name':name,'argv':args,'exit':r.returncode,'seconds':time.time()-start});(ev/'targeted-results4.json').write_text(json.dumps(results,indent=2)+'\n')
 if r.returncode:raise SystemExit(r.returncode)
