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

before=json.loads((E/'source-before.json').read_text());after=snapshot('source-after');drift={f:{'before':before['files'].get(f),'after':after['files'].get(f)} for f in sorted(set(before['files'])|set(after['files'])) if before['files'].get(f)!=after['files'].get(f)}
ciBefore=json.loads((E/'ci4-before.json').read_text());ciAfter={f:hashlib.sha256((ROOT/f).read_bytes()).hexdigest() for f in ciBefore}
admission=json.loads((ROOT/'docs/operations/evidence/2026-10-30-adr0787-timeout/parent-source-admission.json').read_text());admitted={f:hashlib.sha256((ROOT/f).read_bytes()).hexdigest()==sha for f,sha in admission['runtime_paths_applied'].items()}
entry=json.loads((E/'entry.json').read_text());guard=hashlib.sha256((ROOT/'.pi-lens.json').read_bytes()).hexdigest()
extra=json.loads((E/'frozen-extra-sha256.json').read_text());extraAfter={f:hashlib.sha256((ROOT/f).read_bytes()).hexdigest() for f in extra}
aggregate=lambda d:hashlib.sha256(json.dumps(d,sort_keys=True,separators=(',',':')).encode()).hexdigest()
report={'time':datetime.datetime.now().isoformat(),'countBefore':before['count'],'countAfter':after['count'],'sourceDrift':drift,'aggregateBefore':aggregate(before['files']),'aggregateAfter':aggregate(after['files']),'admitted53Matches':admitted,'ci4Before':ciBefore,'ci4After':ciAfter,'ci4Unchanged':ciBefore==ciAfter,'guardUnchanged':entry['guardSha']==guard,'headBefore':before['head'],'headAfter':after['head'],'headUnchanged':before['head']==after['head'],'indexBefore':before['index'],'indexAfter':after['index'],'assetManifestBefore':before['files']['content/original/asset-manifest.json'],'assetManifestAfter':after['files']['content/original/asset-manifest.json'],'frozenExtraBefore':extra,'frozenExtraAfter':extraAfter,'frozenExtraUnchanged':extra==extraAfter}
(E/'hash-diff.json').write_text(json.dumps(report,indent=2)+'\n')
assert not drift and all(admitted.values()) and len(admitted)==53 and report['ci4Unchanged'] and report['guardUnchanged'] and report['headUnchanged'] and not before['index'] and not after['index'] and report['frozenExtraUnchanged']
print('final main source31808/53/CI4/frozen/Cargo/assets/guard/head/index: unchanged')
