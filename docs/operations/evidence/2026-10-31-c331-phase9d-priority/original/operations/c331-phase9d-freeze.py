from pathlib import Path
import json,hashlib,tarfile,subprocess,sys,datetime
r=Path(Path('/tmp/c331-phase7-root').read_text().strip());w=r/'worktree';e=r/'evidence/phase9d';d=e/sys.argv[1];d.mkdir(exist_ok=False);sha=lambda b:hashlib.sha256(b).hexdigest()
b=r/'evidence/phase9c/full-source/source-manifest.json';assert sha(b.read_bytes())=='0ab2b44eac80e7c1d591ebe560e8be69ab4ff72a4ac35b7db92edc5c4f4925ae';m=json.loads(b.read_text());base={x['path']:x['sha256']for x in m['files']};t=tarfile.open(b.parent/'composed-source.tar.gz');names=set(t.getnames())
paths=set(base)|set(subprocess.check_output(['git','diff','--name-only','HEAD'],cwd=w,text=True).splitlines())|set(subprocess.check_output(['git','ls-files','--others','--exclude-standard'],cwd=w,text=True).splitlines());rows=[];patch=b''
with tarfile.open(d/'changed-source.tar.gz','w:gz')as out:
 for p in sorted(paths):
  q=w/p
  if not q.is_file()or q.is_symlink():continue
  old=t.extractfile(p).read()if p in names else subprocess.run(['git','show','HEAD:'+p],cwd=w,capture_output=True).stdout
  cur=q.read_bytes()
  if cur==old:continue
  rows.append({'path':p,'baseline_c9c_sha256':sha(old)if old else None,'sha256':sha(cur)})
  out.add(q,arcname=p);dest=d/'source'/p;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(cur);prev=d/'baseline-changed'/p;prev.parent.mkdir(parents=True,exist_ok=True);prev.write_bytes(old)
  result=subprocess.run(['git','diff','--no-index','--binary','--',str(prev)if old else'/dev/null',str(q)],capture_output=True);assert result.returncode in[0,1];part=result.stdout
  for prefix in ['a/','b/']:
   for tree in[d/'baseline-changed',w]:part=part.replace((prefix+str(tree).lstrip('/')+'/').encode(),prefix.encode())
  patch+=part
(d/'vs-c9c.patch').write_bytes(patch)
manifest={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'base_c9c_manifest_sha256':sha(b.read_bytes()),'files':rows,'scope':'Existing damage-window native source-unit competition; genuine changeHp queue explicitly deferred by supervisor','whole_c331':'BLOCK;262/290 remaining28 +0'};(d/'source-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
check=subprocess.run(['git','apply','--reverse','--check',str(d/'vs-c9c.patch')],cwd=w,capture_output=True,text=True);assert check.returncode==0,check.stderr
assert not subprocess.check_output(['git','diff','--cached','--name-only'],cwd=w).strip()
print(json.dumps({'path':str(d),'files':len(rows),'manifest':sha((d/'source-manifest.json').read_bytes()),'tar':sha((d/'changed-source.tar.gz').read_bytes()),'patch':sha(patch),'reverse_check':True}))
