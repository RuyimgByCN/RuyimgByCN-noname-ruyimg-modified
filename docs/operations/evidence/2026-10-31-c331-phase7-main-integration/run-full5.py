from pathlib import Path
import os,json,subprocess,time,shutil,hashlib
root=Path(__file__).resolve().parents[2];work=root/'worktree';ev=Path(__file__).parent;env=os.environ.copy();env.update(CARGO_TARGET_DIR='/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-c331-efhmbq7e/target',CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='2');env.pop('C331_PHASE3_EVIDENCE',None)
subprocess.run(['python3',str(ev/'freeze.py'),'pre-full4'],check=True,stdout=(ev/'pre-full4-result.json').open('w'))
commands=[('full-native5',['cargo','test','--offline','-p','engine-native-kernel','-p','engine-native-content','-p','engine-native']),('source-cap-unit',['python3','-B','-m','unittest','discover','-s','scripts','-p','test_check_source_lines.py']),('source-caps',['python3','-B','scripts/check-source-lines.py']),('layering',['python3','-B','scripts/check-native-layering.py'])];results=[]
for name,args in commands:
 assert shutil.disk_usage(work).free>=3*1024**3
 saved=json.loads((ev/'pre-full4/source-manifest.json').read_text());assert all(hashlib.sha256((work/r['path']).read_bytes()).hexdigest()==r['sha256']for r in saved['files'])
 start=time.time();(ev/'active-command.json').write_text(json.dumps({'name':name,'argv':args,'driver':os.getpid(),'state':'running'},indent=2)+'\n')
 with(ev/(name+'.log')).open('w')as log:r=subprocess.run(['python3','/tmp/noname-run-serialized.py',*args],cwd=work,env=env,stdout=log,stderr=subprocess.STDOUT)
 results.append({'name':name,'argv':args,'exit':r.returncode,'seconds':time.time()-start});(ev/'full-results5.json').write_text(json.dumps(results,indent=2)+'\n')
 if r.returncode:raise SystemExit(r.returncode)
(ev/'active-command.json').write_text(json.dumps({'state':'completed','driver':os.getpid()},indent=2)+'\n')
