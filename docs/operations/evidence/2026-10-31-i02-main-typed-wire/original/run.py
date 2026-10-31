import os,sys,json,hashlib,subprocess,time,datetime,shutil
from pathlib import Path
R=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');E=Path('/tmp/i02-main-typed-wire-functional-33d21ba5');W=Path('/tmp/nc27-main-functional-458ed333/cargo-target');ENV={**os.environ,'GIT_OPTIONAL_LOCKS':'0','CARGO_TARGET_DIR':str(W),'CARGO_BUILD_JOBS':'2','CARGO_INCREMENTAL':'0','CARGO_NET_OFFLINE':'true','PYTHONDONTWRITEBYTECODE':'1','NC10_OWN_EVIDENCE':str(E/'main-own-cases')}
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest() if p.is_file() else None
def save(n,x):(E/n).write_text(json.dumps(x,indent=2)+'\n')
def git(*a):return subprocess.check_output(['git','--no-optional-locks',*a],cwd=R,env=ENV)
def bind():
 src=json.load(open(E/'source36.json'));assert all(sha(R/p)==h for p,h in src.items());lock=json.load(open(E/'integration-receipt.json'))['protected_Own6_and_Main_lock']['Cargo.lock'];assert sha(R/'Cargo.lock')==lock;return {'source36':src,'MainCargoLock':lock}
def snapshot(name):
 bind();paths=json.load(open(E/'protection-paths.json'));gd=Path(git('rev-parse','--absolute-git-dir').decode().strip());tracked=git('ls-files','-z').decode().split('\0');tracked=[p for p in tracked if p];maint=[p for p in tracked if p.split('/')[0] in ['crates','apps','packages','content','scripts','fuzz','tests'] and Path(p).suffix in ['.rs','.ts','.vue','.mjs','.cjs','.py','.toml','.json','.yaml','.yml','.skill','.dsl','.lock','.md']];docs=[p for p in tracked if p.endswith('.md')];md=sorted(set(maint+docs+list(json.load(open(E/'source36.json')))))
 x={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'head':git('rev-parse','HEAD').decode().strip(),'tree':git('rev-parse','HEAD^{tree}').decode().strip(),'indexSha256':sha(gd/'index'),'indexEntries':git('ls-files','--stage','-z').decode(),'headEntries':git('ls-tree','-rz','HEAD').decode(),'staged':git('diff','--cached','--name-only').decode(),'trackedStatus':git('status','--porcelain=v1','--untracked-files=no').decode(),'untrackedNames':git('ls-files','--others','--exclude-standard','-z').decode(),'sourceBinding':bind(),'maintainedAndMarkdown':{p:sha(R/p) for p in md},'protectedSets':{k:{p:sha(R/p) for p in v} for k,v in paths.items()},'protected99Source30Overlap':sorted(set(paths['protected'])&set(json.load(open(E/'integration-receipt.json'))['source30'])),'hooks':{str(p.relative_to(gd)):sha(p) for p in (gd/'hooks').rglob('*') if p.is_file()},'freeDiskBytes':shutil.disk_usage(R).free,'trackedPathsCount':len(tracked),'maintainedAndMarkdownCount':len(md)}
 shutil.copy2(gd/'index',E/(name+'-index.raw'));save(name+'.json',x);print(json.dumps({'snapshot':name,'head':x['head'],'tracked':len(tracked),'maintainedMarkdown':len(md),'index':x['indexSha256'],'freeDisk':x['freeDiskBytes']}),flush=True)
def run(name,argv,cwd=R,extra=None):
 start=time.monotonic();before=bind();env={**ENV,**(extra or {})};assert shutil.disk_usage(R).free>=3*1024**3
 scripts={str(p):sha(p) for a in argv if (p:=(Path(a) if Path(a).is_absolute() else Path(cwd)/a)).is_file() and p.suffix in ['.py','.mjs','.cjs','.toml','.rs']};actual=['python3','-B','/tmp/noname-run-serialized.py',*argv];save(name+'.running.json',{'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'argv':actual,'sourceBinding':before,'commandSourceHashes':scripts})
 with (E/(name+'.stdout.log')).open('wb') as out,(E/(name+'.stderr.log')).open('wb') as err:p=subprocess.run(actual,cwd=cwd,env=env,stdout=out,stderr=err)
 after=bind();assert all(sha(Path(p))==h for p,h in scripts.items());x={'name':name,'argv':argv,'serializedArgv':actual,'cwd':str(cwd),'exit':p.returncode,'seconds':time.monotonic()-start,'environment':{k:env[k] for k in ['CARGO_TARGET_DIR','CARGO_BUILD_JOBS','CARGO_INCREMENTAL','CARGO_NET_OFFLINE','GIT_OPTIONAL_LOCKS','RUSTFLAGS','NODE_OPTIONS','TAURI_CONFIG'] if k in env},'commandSourceHashes':scripts,'bindingBefore':before,'bindingAfter':after,'stdoutSha256':sha(E/(name+'.stdout.log')),'stderrSha256':sha(E/(name+'.stderr.log')),'freeDiskBytesAfter':shutil.disk_usage(R).free};save(name+'.result.json',x);print(json.dumps({'name':name,'exit':p.returncode,'seconds':x['seconds']}),flush=True);return p.returncode
if __name__=='__main__':
 mode=sys.argv[1]
 if mode=='before':snapshot('before')
 elif mode=='native':
  commands=[('api-protocol',['cargo','test','--offline','-p','engine-api','-p','engine-protocol']),('affected-consumers',['cargo','check','--offline','-p','game-server','-p','desktop-client','--tests']),('native3',['cargo','test','--offline','-p','engine-native-kernel','-p','engine-native-content','-p','engine-native','--no-fail-fast','--','--test-threads=1']),('workspace',['cargo','test','--workspace','--offline','--no-fail-fast','--','--test-threads=1'])]
  for n,a in commands:
   if run(n,a):sys.exit(1)
  if run('gate1b',['cargo','check','--offline','-p','engine-native','--all-targets'],extra={'CARGO_TARGET_DIR':str(E/'strict-target'),'RUSTFLAGS':'-Dwarnings'}):sys.exit(1)
 elif mode=='run':sys.exit(run(sys.argv[2],sys.argv[3:]))
