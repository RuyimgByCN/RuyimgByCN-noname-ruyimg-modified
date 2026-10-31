from pathlib import Path
import os,subprocess,shutil,json,time
root=Path(__file__).resolve().parents[2];work=root/'worktree';ev=Path(__file__).parent;target=root/'target-strict';assert not target.exists(),'independent strict target must be new';assert shutil.disk_usage(root).free>=3*1024**3;env=os.environ.copy();env.update(CARGO_TARGET_DIR=str(target),CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='2',RUSTFLAGS='-Dwarnings');args=['cargo','check','--offline','-p','engine-native','--all-targets'];start=time.time()
with(ev/'gate1b-strict.log').open('w')as log:r=subprocess.run(['python3','/tmp/noname-run-serialized.py',*args],cwd=work,env=env,stdout=log,stderr=subprocess.STDOUT)
(ev/'gate1b-result.json').write_text(json.dumps({'argv':args,'exit':r.returncode,'seconds':time.time()-start,'independent_target':str(target),'RUSTFLAGS':'-Dwarnings','incremental':0,'jobs':2},indent=2)+'\n');raise SystemExit(r.returncode)
