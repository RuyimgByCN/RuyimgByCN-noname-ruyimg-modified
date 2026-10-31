import os,sys,json,hashlib,subprocess,time,datetime,shutil
from pathlib import Path
R=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next'); O=Path(json.load(open('/tmp/nc10-main-own-current.json'))['root'])
TARGET='/tmp/nc27-main-functional-458ed333/cargo-target'
ENV={**os.environ,'GIT_OPTIONAL_LOCKS':'0','CARGO_INCREMENTAL':'0','CARGO_BUILD_JOBS':'2','CARGO_TARGET_DIR':TARGET,'CARGO_NET_OFFLINE':'true','PYTHONDONTWRITEBYTECODE':'1','NC10_OWN_EVIDENCE':str(O/'main-own-cases')}
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest() if p.is_file() else None
def save(n,v):(O/n).write_text(json.dumps(v,indent=2)+'\n')
def git(*a):return subprocess.check_output(['git','--no-optional-locks',*a],cwd=R,env=ENV).decode().strip()
def bind():
 fs=json.load(open(O/'receipt.json'))['files'];actual={f['path']:sha(R/f['path']) for f in fs};assert all(actual[f['path']]==f['sha256'] for f in fs);return actual
def snapshot(n):
 paths=json.load(open(O/'protected-paths.json'));tracked=git('ls-files').splitlines();gd=Path(git('rev-parse','--absolute-git-dir'))
 x={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'head':git('rev-parse','HEAD'),'tree':git('rev-parse','HEAD^{tree}'),'source6':bind(),'index_sha256':sha(gd/'index'),'index_entries':git('ls-files','--stage'),'staged':git('diff','--cached','--name-only'),'status':git('status','--porcelain=v1'),'tracked_hashes':{p:sha(R/p) for p in tracked},'hooks':{str(p.relative_to(gd)):sha(p) for p in (gd/'hooks').rglob('*') if p.is_file()},'protected':{k:{p:sha(R/p) for p in v} for k,v in paths.items()},'free_disk_bytes':shutil.disk_usage(R).free}
 extra_paths=subprocess.check_output(['rg','--files','--hidden','-g','*.md','-g','*.rs','-g','*.ts','-g','*.vue','-g','*.mjs','-g','*.py','-g','*.toml','-g','*.json','-g','*.yaml','-g','*.yml','-g','*.dsl','-g','*.skill'],cwd=R,env=ENV,text=True).splitlines();x['maintained_and_markdown']={p:sha(R/p) for p in sorted(set(extra_paths))};shutil.copy2(gd/'index',O/(n+'-index.raw'));save(n+'.json',x);print(json.dumps({'snapshot':n,'tracked':len(tracked),'source6_exact':True,'free_disk_bytes':x['free_disk_bytes']}),flush=True)
def run(name,argv,cwd=R,extra=None):
 before=bind();env={**ENV,**(extra or {})};assert shutil.disk_usage(R).free>3*1024**3
 start=time.monotonic();actual=['python3','-B','/tmp/noname-run-serialized.py',*argv];save(name+'.running.json',{'argv':actual,'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source6':before})
 with (O/(name+'.stdout.log')).open('wb') as out,(O/(name+'.stderr.log')).open('wb') as err:
  p=subprocess.run(actual,cwd=cwd,env=env,stdout=out,stderr=err)
 after=bind();x={'name':name,'argv':argv,'actual_argv':actual,'cwd':str(cwd),'exit':p.returncode,'wall_seconds':time.monotonic()-start,'source6_pre':before,'source6_post':after,'environment':{k:env[k] for k in ['CARGO_TARGET_DIR','CARGO_BUILD_JOBS','CARGO_INCREMENTAL','CARGO_NET_OFFLINE','RUSTFLAGS','NODE_OPTIONS'] if k in env},'disk_free_after':shutil.disk_usage(R).free,'stdout_sha256':sha(O/(name+'.stdout.log')),'stderr_sha256':sha(O/(name+'.stderr.log'))};save(name+'.result.json',x);print(json.dumps(x),flush=True);return p.returncode
if __name__=='__main__':
 mode=sys.argv[1]
 if mode=='before':snapshot('before')
 elif mode=='after':snapshot('after')
 elif mode=='native':
  if run('native3',['cargo','test','--offline','-p','engine-native-kernel','-p','engine-native-content','-p','engine-native','--no-fail-fast','--','--test-threads=1']):sys.exit(1)
  if run('workspace',['cargo','test','--workspace','--offline','--no-fail-fast','--','--test-threads=1']):sys.exit(1)
  if run('gate1b',['cargo','check','--offline','-p','engine-native','--all-targets'],extra={'CARGO_TARGET_DIR':str(O/'strict-target'),'RUSTFLAGS':'-Dwarnings'}):sys.exit(1)
 elif mode=='front':
  if run('frontend-type',['pnpm','typecheck'],R/'apps/web-client'):sys.exit(1)
  if run('frontend-tests',['pnpm','test'],R/'apps/web-client'):sys.exit(1)
 elif mode=='build-native':sys.exit(run('native-wasm-build',['node','scripts/build-native.mjs'],R/'apps/web-client',{'NODE_OPTIONS':'--require='+str(O/'target-preload.cjs')}))
 elif mode=='products':
  if run('native-wasm-build',['node','scripts/build-native.mjs'],R/'apps/web-client',{'NODE_OPTIONS':'--require='+str(O/'target-preload.cjs')}):sys.exit(1)
  if run('worker-smoke',['pnpm','smoke:native'],R/'apps/web-client'):sys.exit(1)
  if run('frontend-type',['pnpm','typecheck'],R/'apps/web-client'):sys.exit(1)
  if run('frontend-tests',['pnpm','test'],R/'apps/web-client'):sys.exit(1)
  if run('vite-bundle',['node',str(O/'vite-build.mjs')],R/'apps/web-client'):sys.exit(1)
 elif mode=='run':sys.exit(run(sys.argv[2],sys.argv[3:]))
