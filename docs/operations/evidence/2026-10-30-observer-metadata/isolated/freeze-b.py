from pathlib import Path
import subprocess,json,hashlib,difflib,datetime
root=Path.cwd();out=root.parent/'evidence';base=json.loads((out/'b-baseline-sha256.json').read_text());prereq=json.loads((out/'parent-integration-prerequisite-manifest.json').read_text())
paths=list(dict.fromkeys(subprocess.check_output(['git','ls-files','-co','--exclude-standard','-z']).decode().split('\0')))
changes=[];excluded=[];deleted=[];final={};patch=[]
for path in sorted(set(paths)|set(base['files'])):
 if not path:continue
 p=root/path
 if not p.is_file():
  if path in base['files']:deleted.append(path)
  continue
 data=p.read_bytes();sha=hashlib.sha256(data).hexdigest();final[path]=sha
 if base['files'].get(path)==sha:continue
 if path.startswith('.pi-lens-probe-home/'):
  excluded.append(path);continue
 original=out/'b-originals'/path
 previous=original.read_bytes() if path in base['files'] else b''
 if path in base['files']:assert hashlib.sha256(previous).hexdigest()==base['files'][path],path
 assert path.startswith(('apps/','crates/')) or path=='Cargo.lock',path
 changes.append({'path':path,'status':'modified' if path in base['files'] else 'added','baseline_sha256':base['files'].get(path),'final_sha256':sha,'bytes':len(data),'original_snapshot':str(original) if path in base['files'] else None})
 patch.append(f'diff --git a/{path} b/{path}\n')
 if path not in base['files']:patch.append('new file mode 100644\n')
 patch.extend(difflib.unified_diff(previous.decode().splitlines(keepends=True),data.decode().splitlines(keepends=True),fromfile=f'a/{path}' if path in base['files'] else '/dev/null',tofile=f'b/{path}',n=3))
assert not deleted,deleted
text=''.join(patch);(out/'approved-b-checked.patch').write_text(text)
(out/'b-checked-sha256.json').write_text(json.dumps({'head':base['head'],'date':datetime.datetime.now().isoformat(),'files':final},indent=2)+'\n')
manifest={'head':base['head'],'base':'HEAD 100489f5 + exact 52-file parent reviewed PA09/Referee overlay','prerequisite_patch_sha256':prereq['patch_sha256'],'all_52_initial_hashes_match':all(base['files'][row['path']]==row['sha256'] for row in prereq['files']),'patch_sha256':hashlib.sha256(text.encode()).hexdigest(),'changed_files':changes,'excluded_automatic_observer_logs':excluded,'deleted_files':deleted,'source_versions':{'rules':328,'archive':105,'native_abi':753,'ws':5},'pi_lens_config_unchanged':base['files']['.pi-lens.json']==final['.pi-lens.json'],'no_staged_files':not subprocess.check_output(['git','diff','--cached','--name-only']).strip()}
(out/'b-checked-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
check=out/'b-patch-check-baseline';check.mkdir(exist_ok=True)
for row in changes:
 if row['original_snapshot']:
  p=check/row['path'];p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(Path(row['original_snapshot']).read_bytes())
result=subprocess.run(['git','apply','--check',str(out/'approved-b-checked.patch')],cwd=check,capture_output=True,text=True)
(out/'b-checked-patch-check.log').write_text(result.stdout+result.stderr+f'\nexit={result.returncode}\n');assert result.returncode==0
print(len(changes),'pure B files;',len(text),'patch chars; baseline dry-apply passed')
