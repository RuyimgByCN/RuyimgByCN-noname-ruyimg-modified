from pathlib import Path
import subprocess,os,json,time,shutil,hashlib
r=Path(__file__).resolve().parents[2];w=r/'worktree';ev=Path(__file__).parent;env=os.environ.copy();env.update(CARGO_TARGET_DIR='/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-c331-efhmbq7e/target',CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='2');rows=json.loads((ev/'pre-full2/source-manifest.json').read_text())['files'];commands=[('full-native3',['cargo','test','--offline','-p','engine-native-kernel','-p','engine-native-content','-p','engine-native']),('legacy-admission',['cargo','test','--offline','-p','engine-core','c331_schema_tests']),('source-cap-unit',['python3','-B','-m','unittest','discover','-s','scripts','-p','test_check_source_lines.py']),('source-caps',['python3','-B','scripts/check-source-lines.py']),('layering',['python3','-B','scripts/check-native-layering.py'])];results=[]
for name,args in commands:
 assert shutil.disk_usage(r).free>=3*1024**3;assert all(hashlib.sha256((w/x['path']).read_bytes()).hexdigest()==x['sha256']for x in rows)
 start=time.time()
 with(ev/(name+'.log')).open('w')as out:done=subprocess.run(['python3','/tmp/noname-run-serialized.py',*args],cwd=w,env=env,stdout=out,stderr=subprocess.STDOUT)
 results.append({'name':name,'argv':args,'exit':done.returncode,'seconds':time.time()-start});(ev/'final-check-results3.json').write_text(json.dumps(results,indent=2)+'\n')
 if done.returncode:raise SystemExit(done.returncode)
