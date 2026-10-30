from pathlib import Path
import runpy,json,time,sys,shutil,hashlib,subprocess,datetime,tarfile
m=runpy.run_path('/tmp/pa09-recovery.py');out=m['OUT'];root=m['ROOT'];target=Path(m['TARGET']);art=out/'recovery-artifacts'
started=time.monotonic()
while True:
 rows=json.loads((out/'recovery-command-results.json').read_text())
 row=next((r for r in rows if r['name']=='desktop-release-build'),None)
 if row:
  if row['exit']:sys.exit('Release failed; finalization/preflight not launched')
  break
 if time.monotonic()-started>3600:sys.exit('Release did not finish within one hour; finalization not launched')
 time.sleep(2)
source=target/'release/noname-next-desktop';dest=art/'release-noname-next-desktop'
shutil.copy2(source,dest);assert hashlib.sha256(source.read_bytes()).digest()==hashlib.sha256(dest.read_bytes()).digest()
cleanup=out/'recovery-post-compile-cleanup.json';cleanrows=json.loads(cleanup.read_text());cleanrows.append({'command':'desktop-release-build','at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'free_bytes':shutil.disk_usage(root).free,'incremental_entries':{str(p):len(list(p.iterdir())) if p.exists() else 0 for p in [target/'debug/incremental',target/'release/incremental',target/'wasm32-unknown-unknown/release/incremental']},'deleted':[],'retained':{'stage3_target':'Complete debug/release and actual WASM evidence secured; retain pending final preflight/handoff, then parent-unified retirement','main_generated_packages':'Shared normal main outputs; not cleanup-owned'},'release_sha256':hashlib.sha256(dest.read_bytes()).hexdigest(),'release_bytes':dest.stat().st_size});cleanup.write_text(json.dumps(cleanrows,indent=2)+'\n');print(json.dumps(cleanrows[-1]),flush=True)
# Preserve the initial generated build backup before retiring its private temporary copy.
backup=Path('/tmp/pa09-validation-dist-before')
if backup.exists():
 with tarfile.open(art/'generated-dist-before-validation.tar.gz','w:gz') as tar:tar.add(backup,arcname=backup.name)
 shutil.rmtree(backup)
# The task authorizes exact HEAD-byte restoration of tracked generated dist only.
restored={}
for name in filter(None,subprocess.check_output(['git','ls-files','-z','apps/web-client/dist'],cwd=root).decode().split('\0')):
 content=subprocess.check_output(['git','show','HEAD:'+name],cwd=root);p=root/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(content);assert p.read_bytes()==content
 restored[name]=hashlib.sha256(content).hexdigest()
(out/'recovery-dist-restored-exact-head.json').write_text(json.dumps(restored,indent=2)+'\n')
# Final repository gate follows every runtime check and the production release build.
code=m['run']('preflight-final',['python3','-B','scripts/preflight.py'])
print('final_preflight_exit',code,flush=True)
paths=subprocess.check_output(['git','ls-files','-c','-o','--exclude-standard','-z'],cwd=root).decode().split('\0')
after={p:hashlib.sha256((root/p).read_bytes()).hexdigest() for p in sorted(set(paths)) if p and (root/p).is_file() and not p.startswith(('docs/operations/evidence/','apps/web-client/dist/','apps/web-client/src/worker/native-pkg/','apps/web-client/src/worker/pkg/'))}
(out/'recovery-after-source-hashes.json').write_text(json.dumps(after,indent=2)+'\n')
before=json.loads((out/'validation-before-source-hashes.json').read_text());changed=[p for p in sorted(set(before)|set(after)) if before.get(p)!=after.get(p)]
staged=subprocess.check_output(['git','diff','--cached','--name-only'],cwd=root,text=True).splitlines()
(out/'recovery-source-difference.json').write_text(json.dumps({'before_files':len(before),'after_files':len(after),'changed':changed,'rust_ts_changes':[p for p in changed if p.endswith(('.rs','.ts','.tsx','.dsl'))],'non_doc_changes':[p for p in changed if not p.startswith('docs/')],'non_markdown_changes':[p for p in changed if not p.endswith('.md')],'staged_files':staged,'head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()},indent=2)+'\n')
(out/'recovery-git-status.txt').write_text(subprocess.check_output(['git','status','--short'],cwd=root,text=True))
hashes={str(p.relative_to(out)):{'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for p in art.rglob('*') if p.is_file()};(out/'recovery-artifact-hashes.json').write_text(json.dumps(hashes,indent=2)+'\n')
print('source_changed',changed,'staged',staged,flush=True)
sys.exit(code)
