from pathlib import Path
import subprocess,os,time,json,shutil,hashlib
r=Path(Path('/tmp/c331-phase7-root').read_text().strip());w=r/'worktree';ev=Path(__file__).parent;env=os.environ.copy();env.update(CARGO_TARGET_DIR='/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-c331-efhmbq7e/target',CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='2');source=json.loads((ev/'../final-freeze1/complete-source-manifest.json').read_text());commands=[('predicate-traces',['cargo','test','--offline','-p','engine-native','--test','nc_rounds','c331_','--','--nocapture']),('full-native',['cargo','test','--offline','-p','engine-native-kernel','-p','engine-native-content','-p','engine-native','--no-fail-fast']),('gate1b',['cargo','check','--offline','-p','engine-native','--all-targets']),('golden-traces',['cargo','test','--offline','-p','engine-native','--test','kernel','replay_scenarios_match_golden','--','--test-threads=1']),('source-cap-unit',['python3','-B','-m','unittest','discover','-s','scripts','-p','test_check_source_lines.py']),('source-caps',['python3','-B','scripts/check-source-lines.py']),('layering',['python3','-B','scripts/check-native-layering.py'])];results=[]
for name,args in commands:
 assert time.time()<1793467896.0, 'No new heavy producer after17:31:36Z'
 assert hashlib.sha256(Path('/tmp/noname-run-serialized.py').read_bytes()).hexdigest()=='d7f246f1fdbd4ec6d85deb22643d7ca8137be62539ebd5e75d49c779febb8e34'
 lock=Path('/tmp/noname-phase4-heavy-command.lock').stat();assert lock.st_ino==620205209 and lock.st_dev==16777231
 assert shutil.disk_usage(r).free>=3*1024**3;assert all(hashlib.sha256((w/x['path']).read_bytes()).hexdigest()==x['sha256']for x in source['files']);current=env.copy()
 if name=='predicate-traces':current.update(C331_PREDICATE_TRACE_DIR=str(ev/'predicate-traces'))
 if name=='golden-traces':current.update(C331_GOLDEN_TRACE_DIR=str(ev/'current-final-traces-v3'),C331_SEMANTIC_TRACE_DIR=str(ev/'current-command-traces-v3'))
 if name=='gate1b':current.update(CARGO_TARGET_DIR=str(r/'target-strict'),RUSTFLAGS='-Dwarnings')
 start=time.time()
 with(ev/(name+'.log')).open('w')as out:done=subprocess.run(['python3','/tmp/noname-run-serialized.py',*args],cwd=w,env=current,stdout=out,stderr=subprocess.STDOUT)
 assert all(hashlib.sha256((w/x['path']).read_bytes()).hexdigest()==x['sha256']for x in source['files'])
 results.append({'name':name,'argv':args,'exit':done.returncode,'seconds':time.time()-start,'target':current['CARGO_TARGET_DIR'],'RUSTFLAGS':current.get('RUSTFLAGS')});(ev/'check-results.json').write_text(json.dumps(results,indent=2)+'\n')
