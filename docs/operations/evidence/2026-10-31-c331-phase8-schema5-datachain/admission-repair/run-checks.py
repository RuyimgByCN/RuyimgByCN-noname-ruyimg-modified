from pathlib import Path
import subprocess,os,time,json,shutil,hashlib
r=Path('/tmp/c331-phase7-root').read_text().strip();r=Path(r);w=r/'worktree';ev=Path(__file__).parent;env=os.environ.copy();env.update(CARGO_TARGET_DIR='/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-c331-efhmbq7e/target',CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='2');commands=[('four-packages',['cargo','test','--offline','-p','content-schema','-p','content-compiler','-p','content-runtime','-p','content-native-loader']),('full-native',['cargo','test','--offline','-p','engine-native-kernel','-p','engine-native-content','-p','engine-native']),('gate1b',['cargo','check','--offline','-p','engine-native','--all-targets']),('source-cap-unit',['python3','-B','-m','unittest','discover','-s','scripts','-p','test_check_source_lines.py']),('source-caps',['python3','-B','scripts/check-source-lines.py']),('layering',['python3','-B','scripts/check-native-layering.py'])];results=[]
for name,args in commands:
 assert shutil.disk_usage(r).free>=3*1024**3;assert Path('/tmp/noname-phase4-heavy-command.lock').stat().st_ino==620205209
 current=env.copy()
 if name=='gate1b':current.update(CARGO_TARGET_DIR=str(r/'target-strict'),RUSTFLAGS='-Dwarnings')
 start=time.time()
 with(ev/(name+'.log')).open('w')as out:done=subprocess.run(['python3','/tmp/noname-run-serialized.py',*args],cwd=w,env=current,stdout=out,stderr=subprocess.STDOUT)
 results.append({'name':name,'argv':args,'exit':done.returncode,'seconds':time.time()-start,'target':current['CARGO_TARGET_DIR'],'RUSTFLAGS':current.get('RUSTFLAGS')});(ev/'check-results.json').write_text(json.dumps(results,indent=2)+'\n')
 if done.returncode:raise SystemExit(done.returncode)
