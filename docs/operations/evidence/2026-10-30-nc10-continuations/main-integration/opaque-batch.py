import runpy,sys,json,time,shutil,tarfile,hashlib
from pathlib import Path
m=runpy.run_path('/tmp/nc10-main-run.py');run=m['run'];out=m['OUT'];tmp=Path(Path('/tmp/nc10-main-tmp-path.txt').read_text());h=tmp/'opaque-planner';pkg=tmp/'opaque-web-pkg';artifact=out/'artifacts';artifact.mkdir(exist_ok=True)
started=time.monotonic()
while True:
 index=out/'recovery-command-results.json';rows=json.loads(index.read_text()) if index.exists() else []
 if any(r['exit'] for r in rows):sys.exit('Prior acceptance command failed; opaque matrix held for diagnosis')
 if any(r['name']=='native-wasm-trajectory' for r in rows):break
 if time.monotonic()-started>5400:sys.exit('Prior acceptance chain not complete within90minutes; opaque commands not launched')
 time.sleep(2)
shutil.copy2('/tmp/nc10-main-opaque-verify.mjs',out/'opaque-harness/verify.mjs')
if run('opaque-native',['cargo','run','--offline','--manifest-path',str(h/'Cargo.toml'),'--',str(out/'opaque-native-output.json')]):sys.exit(1)
shutil.copy2(h/'Cargo.lock',out/'opaque-harness/Cargo.lock')
if run('opaque-wasm-build',['wasm-pack','build',str(h),'--target','web','--release','--no-opt','--out-dir',str(pkg)]):sys.exit(1)
if run('opaque-wasm-compare',['node','/tmp/nc10-main-opaque-verify.mjs',str(pkg),str(out/'opaque-native-output.json'),str(out/'opaque-wasm-output.json'),str(out/'opaque-wasm-report.json')]):sys.exit(1)
packages=[(pkg,'opaque-planner-web.tar.gz'),(m['ROOT']/'apps/web-client/src/worker/native-pkg','native-engine-web.tar.gz')]
manifest={}
for folder,name in packages:
 manifest[name]={str(p.relative_to(folder)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'bytes':p.stat().st_size} for p in folder.rglob('*') if p.is_file()}
 with tarfile.open(artifact/name,'w:gz') as tar:tar.add(folder,arcname=folder.name)
 manifest[name]['archive']={'sha256':hashlib.sha256((artifact/name).read_bytes()).hexdigest(),'bytes':(artifact/name).stat().st_size}
(out/'wasm-artifact-hashes.json').write_text(json.dumps(manifest,indent=2)+'\n')
# This is the private generated package only; main shared package and sources are retained.
shutil.rmtree(pkg)
(out/'opaque-post-use-cleanup.json').write_text(json.dumps({'removed_private_generated_package':str(pkg),'evidence_archives_and_hashes_preserved':True,'free_bytes':shutil.disk_usage(m['ROOT']).free,'retained':'Main normal package remains usable by parent; private target deps retained pending source-audit/handoff, and parent may reuse stage3 for next integration.'},indent=2)+'\n')
print('opaque actual WASM planner matrix passed; artifact copies secured; private generated package retired',flush=True)
