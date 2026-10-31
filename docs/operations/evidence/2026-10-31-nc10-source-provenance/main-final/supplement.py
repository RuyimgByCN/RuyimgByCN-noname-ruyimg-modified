import hashlib,json,os,subprocess,sys,datetime
from pathlib import Path
R=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');O=Path('/tmp/nc27-main-final-v3-8e1fad00')
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def git(*argv):return subprocess.check_output(['git','--no-optional-locks',*argv],cwd=R,env={**os.environ,'GIT_OPTIONAL_LOCKS':'0'}).decode()
prior=json.loads((R/'docs/operations/evidence/2026-10-30-pa09-js-admission/main-acceptance/entry.json').read_text())
md=subprocess.check_output(['rg','--files','docs','-g','*.md'],cwd=R,text=True).splitlines()
x={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'docs_markdown':{p:sha(R/p) for p in sorted(md)},'package_git_trees':{p:git('rev-parse','HEAD:'+p).strip() for p in prior['package_git_trees']},'package_tracked_diff_sha256':hashlib.sha256(subprocess.check_output(['git','--no-optional-locks','diff','HEAD','--','content/original/ported','content/original/legacy-source','packages/extensions','Cargo.lock'],cwd=R)).hexdigest(),'package_status':git('status','--porcelain=v1','--untracked-files=all','--','content/original/ported','content/original/legacy-source','packages/extensions','Cargo.lock'),'main_target':{}}
for s in ['target','target/debug','target/release','target/wasm32-unknown-unknown','target/wasm32-unknown-unknown/release','target/.rustc_info.json','apps/web-client/node_modules/.vite-temp']:
 p=R/s;x['main_target'][s]={'exists':p.exists()}
 if p.exists():x['main_target'][s].update({'mtime_ns':p.stat().st_mtime_ns,'size':p.stat().st_size})
 if p.is_file():x['main_target'][s]['sha256']=sha(p)
(O/sys.argv[1]).write_text(json.dumps(x,indent=2)+'\n');print(json.dumps({'snapshot':sys.argv[1],'docs_md':len(md),'package_diff_sha256':x['package_tracked_diff_sha256'],'package_status':x['package_status']}))
