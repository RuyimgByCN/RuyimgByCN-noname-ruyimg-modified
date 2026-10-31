from pathlib import Path
import json,subprocess,hashlib,tarfile,datetime,sys,io,difflib
root=Path(Path('/tmp/c331-phase7-root').read_text().strip());work=root/'worktree';ev=Path(__file__).parent;dest=ev/sys.argv[1];dest.mkdir(exist_ok=False);sha=lambda b:hashlib.sha256(b).hexdigest();base=json.loads((ev/'baseline-c8-repair.json').read_text());baseline={r['path']:r['sha256']for r in base['files']};paths=set(baseline)|set(subprocess.check_output(['git','ls-files','--others','--exclude-standard'],cwd=work,text=True).splitlines());rows=[]
for p in sorted(paths):
 q=work/p
 if q.is_file()and not q.is_symlink():
  h=sha(q.read_bytes())
  if h!=baseline.get(p):rows.append({'path':p,'baseline_c8_sha256':baseline.get(p),'sha256':h})
patch=b'';tar=tarfile.open(ev/'baseline-c8-repair.tar.gz');c7tar=tarfile.open(root/'evidence/phase8/admission-repair/full-source/composed-source.tar.gz');oldnames=set(tar.getnames());c7names=set(c7tar.getnames())
with tarfile.open(dest/'changed-source.tar.gz','w:gz')as out:
 for row in rows:
  p=row['path'];current=(work/p).read_bytes();out.add(work/p,arcname=p);old=b''
  if row['baseline_c8_sha256']is not None:
   if p in oldnames:old=tar.extractfile(p).read()
   elif p in c7names:old=c7tar.extractfile(p).read()
   else:old=subprocess.check_output(['git','show','HEAD:'+p],cwd=work)
   assert sha(old)==row['baseline_c8_sha256'],p
  lhs=dest/'baseline-changed'/p;lhs.parent.mkdir(parents=True,exist_ok=True);lhs.write_bytes(old)
  result=subprocess.run(['git','diff','--no-index','--binary','--',str(lhs)if row['baseline_c8_sha256']is not None else'/dev/null',str(work/p)],stdout=subprocess.PIPE);assert result.returncode in[0,1];part=result.stdout
  for prefix in ['a/','b/']:
   for tree in [dest/'baseline-changed',work]:part=part.replace((prefix+str(tree).lstrip('/')+'/').encode(),prefix.encode())
  patch+=part
(dest/'vs-c8-repair.patch').write_bytes(patch);manifest={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'base_c8_repair_manifest_sha256':base['source_manifest_sha256'],'files':rows,'scope':'C9 partial canonical damage runtime: source amount/settlement/continuation; exact remaining windows recorded separately','whole_c331':'BLOCK','source_proof':'262/290,remaining28,+0','no_staged_files':not bool(subprocess.check_output(['git','diff','--cached','--name-only'],cwd=work).strip())};assert manifest['no_staged_files'];(dest/'source-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');print(json.dumps({'path':str(dest),'changed_files':len(rows),'manifest_sha256':sha((dest/'source-manifest.json').read_bytes()),'tar_sha256':sha((dest/'changed-source.tar.gz').read_bytes()),'patch_sha256':sha(patch)}))
