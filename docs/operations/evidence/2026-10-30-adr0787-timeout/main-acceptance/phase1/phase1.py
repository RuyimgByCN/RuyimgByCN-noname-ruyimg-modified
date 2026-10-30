import os,sys,json,hashlib,subprocess,time,re,shutil,datetime
from pathlib import Path
ROOT=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
E=Path(__file__).resolve().parent
TARGET=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-f7-stage3.czy5e9gq/target')
ENV=dict(os.environ,CARGO_TARGET_DIR=str(TARGET),CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='2')
EXCLUDED=('docs/','target/','data/','apps/game-server/data/','apps/web-client/dist/','apps/web-client/src/worker/native-pkg/','apps/web-client/src/wasm-pkg/','apps/web-client/public/pkg/','.pi-lens-probe-home/')
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT)
def digest(path):
 if path.is_symlink():return {'kind':'symlink','sha256':hashlib.sha256(os.readlink(path).encode()).hexdigest(),'target':os.readlink(path)}
 return {'kind':'file','sha256':hashlib.sha256(path.read_bytes()).hexdigest(),'bytes':path.stat().st_size}
def snapshot(name):
 paths=set(git('ls-files','-z','--cached','--others','--exclude-standard').decode().split('\0'));paths.discard('')
 paths.add('content/original/asset-manifest.json')
 result={}
 for f in sorted(paths):
  if f.startswith(EXCLUDED) or 'node_modules' in Path(f).parts or 'evidence' in Path(f).parts:continue
  p=ROOT/f
  if p.is_file() or p.is_symlink():result[f]=digest(p)
 record={'time':datetime.datetime.now().isoformat(),'head':git('rev-parse','HEAD').decode().strip(),'index':git('diff','--cached','--name-only').decode().splitlines(),'definition':'tracked + nonignored untracked files except docs/evidence/node_modules and enumerated generated runtime paths; symlink text hashed without traversal','excludedPrefixes':EXCLUDED,'files':result,'count':len(result),'guard':digest(ROOT/'.pi-lens.json')}
 (E/(name+'.json')).write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n');return record
before=snapshot('source-before')
admission=json.loads((ROOT/'docs/operations/evidence/2026-10-30-adr0787-timeout/parent-source-admission.json').read_text())
matches={f:hashlib.sha256((ROOT/f).read_bytes()).hexdigest()==sha for f,sha in admission['runtime_paths_applied'].items()}
entry={'time':datetime.datetime.now().isoformat(),'sourceCount':before['count'],'head':before['head'],'indexEmpty':not before['index'],'admittedPaths':len(matches),'shaMatches':matches,'cache':str(TARGET),'env':{k:ENV[k] for k in ['CARGO_TARGET_DIR','CARGO_INCREMENTAL','CARGO_BUILD_JOBS']},'readonlyProfile':'readonly.sb','assetManifest':before['files'].get('content/original/asset-manifest.json'),'ciFiles':[f for f in before['files'] if f.startswith('.github/')],'guard':before['guard']}
(E/'entry.json').write_text(json.dumps(entry,indent=2)+'\n')
assert not before['index'] and len(matches)==53 and all(matches.values()),'entry mismatch'
assert json.loads((ROOT/'.pi-lens.json').read_text())['autofix']['enabled'] is False
steps=[
 ('native3',['cargo','test','--offline','-p','engine-native-kernel','-p','engine-native-content','-p','engine-native'],{}),
 ('affected6-tauri',['cargo','test','--offline','-p','engine-api','-p','engine-core','-p','engine-native','-p','engine-native-wasm','-p','game-server','-p','desktop-client','--features','desktop-client/tauri-shell'],{}),
 ('workspace',['cargo','test','--offline','--workspace','--no-fail-fast'],{}),
 ('strict-native3',['cargo','check','--offline','-p','engine-native-kernel','-p','engine-native-content','-p','engine-native','--all-targets'],{'RUSTFLAGS':'-Dwarnings'}),
 ('workspace-all-targets',['cargo','check','--offline','--workspace','--all-targets'],{}),
 ('native-layering',['python3','-B','scripts/check-native-layering.py'],{}),
 ('source-lines-unit',['python3','-B','-m','unittest','discover','-s','scripts','-p','test_check_source_lines.py'],{}),
 ('source-lines',['python3','-B','scripts/check-source-lines.py'],{}),
 ('conformance',['cargo','run','--offline','-p','conformance'],{}),
 ('frontend-node-typecheck',['bash','-c','cd apps/web-client && npm test && node_modules/.bin/vue-tsc --noEmit'],{})
]
results=[];started=time.monotonic()
try:
 for name,argv,extra in steps:
  if shutil.disk_usage(ROOT).free<3*1024**3:raise RuntimeError('disk below3GiB;queue stopped')
  env=dict(ENV,**extra)
  execution=['/usr/bin/sandbox-exec','-f',str(E/'readonly.sb'),*argv]
  command=['python3','-B','/tmp/noname-run-serialized.py',*execution]
  (E/'active.json').write_text(json.dumps({'name':name,'argv':argv,'execution':command,'elapsed':time.monotonic()-started,'status':'running or waiting on global lock'},indent=2)+'\n')
  print('START',name,flush=True)
  with (E/(name+'.log')).open('w') as log:r=subprocess.run(command,cwd=ROOT,env=env,stdout=log,stderr=subprocess.STDOUT)
  text=(E/(name+'.log')).read_text();m=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored',text)
  wrapper=None
  for line in reversed(text.splitlines()):
   if line.startswith('{"command"'):
    try:wrapper=json.loads(line);break
    except ValueError:pass
  item={'name':name,'argv':argv,'cwd':str(ROOT),'env':{k:env[k] for k in ['CARGO_TARGET_DIR','CARGO_INCREMENTAL','CARGO_BUILD_JOBS']},'extraEnv':extra,'execution':command,'exit':r.returncode,'log':name+'.log','rustTestCounts':[sum(int(x[i]) for x in m) for i in range(3)] if m else None,'warningLines':sum(1 for line in text.splitlines() if line.startswith('warning:')),'wrapper':wrapper,'diskFreeBytes':shutil.disk_usage(ROOT).free,'incrementalBytes':sum(p.stat().st_size for p in (TARGET/'debug/incremental').rglob('*') if p.is_file()),'cacheRetainedFor':'Parent Phase2 current-WASM/WS/UI acceptance; do not retire'}
  results.append(item);(E/'commands.json').write_text(json.dumps(results,indent=2)+'\n');print('FINISH',name,r.returncode,item['rustTestCounts'],flush=True)
  if r.returncode:
   (E/'failure.json').write_text(json.dumps(item,indent=2)+'\n');break
  if time.monotonic()-started>45*60:
   (E/'checkpoint.json').write_text(json.dumps({'elapsed':time.monotonic()-started,'results':results,'remaining':[s[0] for s in steps[len(results):]],'source':'source-before.json'},indent=2)+'\n');break
finally:
 after=snapshot('source-after');old=before['files'];new=after['files'];drift={f:{'before':old.get(f),'after':new.get(f)} for f in sorted(set(old)|set(new)) if old.get(f)!=new.get(f)}
 final={'time':datetime.datetime.now().isoformat(),'sourceCountBefore':before['count'],'sourceCountAfter':after['count'],'sourceDrift':drift,'headBefore':before['head'],'headAfter':after['head'],'indexBefore':before['index'],'indexAfter':after['index'],'guardUnchanged':before['guard']==after['guard'],'completed':len(results),'expected':len(steps),'allPassed':len(results)==len(steps) and all(r['exit']==0 for r in results),'elapsedSeconds':time.monotonic()-started,'diskFreeBytes':shutil.disk_usage(ROOT).free,'cacheRetainedForPhase2':True}
 (E/'final.json').write_text(json.dumps(final,indent=2)+'\n');(E/'active.json').write_text(json.dumps({'status':'finished','completed':len(results),'allPassed':final['allPassed']},indent=2)+'\n');print('FINAL',json.dumps(final),flush=True)
