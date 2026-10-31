import os,sys,json,hashlib,subprocess,time,datetime,shutil,signal
from pathlib import Path
R=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next'); O=Path('/tmp/nc27-main-functional-458ed333'); O.mkdir(exist_ok=True)
E={**os.environ,'GIT_OPTIONAL_LOCKS':'0','GIT_NO_LAZY_FETCH':'1','CARGO_INCREMENTAL':'0','CARGO_BUILD_JOBS':'2','CARGO_TARGET_DIR':str(O/'cargo-target'),'PYTHONDONTWRITEBYTECODE':'1'}
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def save(n,x): (O/n).write_text(json.dumps(x,indent=2)+'\n')
def git(*a): return subprocess.check_output(['git','--no-optional-locks',*a],cwd=R,env=E).decode().strip()
def hashes(paths): return {p:sha(R/p) if (R/p).is_file() else None for p in sorted(set(paths))}
def snapshot(name):
 prior=json.loads((R/'docs/operations/evidence/2026-10-30-pa09-js-admission/main-acceptance/entry.json').read_text())
 admission=json.loads(Path('/tmp/noname-parent-nc27-main-admission.json').read_text())
 inputs=[]
 for d in ['crates/engine-native','crates/engine-native-kernel','crates/engine-native-content','crates/engine-api','apps/web-client/src','apps/web-client/scripts']:
  inputs += [str(p.relative_to(R)) for p in (R/d).rglob('*') if p.is_file()]
 inputs += subprocess.check_output(['rg','--files','-g','Cargo.toml','-g','Cargo.lock','-g','*.config.*','-g','package.json','-g','pnpm-lock.yaml','-g','tsconfig*.json','scripts','.github','apps/web-client','crates','packages'],cwd=R,text=True).splitlines()
 index=Path(git('rev-parse','--git-path','index')); index=index if index.is_absolute() else R/index
 x={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'head':git('rev-parse','HEAD'),'tree':git('rev-parse','HEAD^{tree}'),'index_sha256':sha(index),'staged':git('diff','--cached','--name-only'),'tracked_status':git('status','--porcelain=v1','--untracked-files=no'),'admission31':hashes(admission['paths']),'admission_mismatches':[p for p,v in admission['paths'].items() if sha(R/p)!=v['candidate_sha256']],'input_hashes':hashes(inputs),'guard':json.loads((R/'.pi-lens.json').read_text()),'attribute_sha256':sha(R/'.gitattributes'),'free_disk_bytes':shutil.disk_usage(R).free,'root_gates':hashes(['.github/workflows/ci.yml','scripts/preflight.py','scripts/check-source-lines.py','scripts/check-native-layering.py','scripts/check-native-wasm-cross-target.py','Cargo.toml','Cargo.lock']), 'docs_dirty':hashes(admission['before_docs_dirty_preserved'])}
 for k in ['source3','protected','modules','dist']:x[k]=hashes(prior[k])
 x['pa09_admission_sha256']=sha(R/'docs/operations/evidence/2026-10-30-pa09-js-admission/parent-main-source-admission.json')
 save(name,x); return x

def run(name,argv,cwd=R):
 actual=['python3','-B','/tmp/noname-run-serialized.py',*argv]; start=time.monotonic(); free=shutil.disk_usage(R).free; assert free>=3*1024**3
 with (O/(name+'.stdout.log')).open('wb') as out,(O/(name+'.stderr.log')).open('wb') as err:
  p=subprocess.Popen(actual,cwd=cwd,env=E,stdout=out,stderr=err,start_new_session=True)
  stopped=False
  while p.poll() is None:
   free=min(free,shutil.disk_usage(R).free)
   if free<3*1024**3: os.killpg(p.pid,signal.SIGTERM); stopped=True; break
   time.sleep(.5)
  code=p.wait()
 x={'name':name,'argv':argv,'serialized_argv':actual,'cwd':str(cwd),'exit':code,'wall_seconds':time.monotonic()-start,'minimum_free_bytes':free,'disk_stop':stopped,'environment':{k:E[k] for k in ['CARGO_INCREMENTAL','CARGO_BUILD_JOBS','CARGO_TARGET_DIR']},'stdout_sha256':sha(O/(name+'.stdout.log')),'stderr_sha256':sha(O/(name+'.stderr.log'))}
 save(name+'.result.json',x); print(json.dumps(x),flush=True); return code
if __name__=='__main__':
 mode=sys.argv[1]
 if mode=='before': snapshot('before.json')
 elif mode=='after': snapshot('after.json')
 elif mode=='native':
  code=run('native-all-layers',['cargo','test','--offline','-p','engine-native-kernel','-p','engine-native-content','-p','engine-native']);sys.exit(code)
 elif mode=='frontend':
  for name,argv in [('frontend-tests',['pnpm','test']),('frontend-typecheck',['pnpm','typecheck']),('frontend-build',['node',str(O/'vite-build.mjs')])]:run(name,argv,R/'apps/web-client')
 elif mode=='source':
  for name,argv in [('artifact-native-source',['node','--test','scripts/artifact-native-source.test.mjs']),('artifact-any-source',['node','--test','scripts/artifact-any-source.test.mjs'])]:run(name,argv)
