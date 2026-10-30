import datetime,difflib,hashlib,json,pathlib,subprocess
root=pathlib.Path(__file__).resolve().parent
cwd=root.parent/'worktree'
frozen=json.loads((root/'frozen-manifest.json').read_text())['manifest']
paths=list(frozen['files'])+['crates/engine-native/tests/nc10_redaction_helper_contract.rs']
sha=lambda b:hashlib.sha256(b).hexdigest()
def patch(old,new,path):
 if old==new:return ''
 prefix=f'diff --git a/{path} b/{path}\n'
 if old is None:prefix+='new file mode 100644\n'
 return prefix+''.join(difflib.unified_diff((old or b'').decode().splitlines(True),new.decode().splitlines(True),fromfile=f'a/{path}' if old is not None else '/dev/null',tofile=f'b/{path}'))
complete='';delta='';files={};delta_paths=[]
for path in paths:
 now=(cwd/path).read_bytes()
 previous=root/'frozen-11'/path
 before=previous.read_bytes() if previous.exists() else None
 if path in frozen['files']:assert sha(before)==frozen['files'][path]['sha256'],path
 head=subprocess.run(['git','show',f'HEAD:{path}'],cwd=cwd,capture_output=True)
 complete+=patch(head.stdout if head.returncode==0 else None,now,path)
 change=patch(before,now,path);delta+=change
 if change:delta_paths.append(path)
 files[path]={'sha256':sha(now),'frozen_sha256':sha(before) if before is not None else None,'lines':len(now.splitlines()),'new':head.returncode!=0}
(root/'nc10-corrected-complete.patch').write_text(complete)
(root/'nc10-p2-vs-frozen-11.patch').write_text(delta)
borrowed=json.loads((root/'borrowed-prerequisites.json').read_text())
for f in borrowed['files']:assert sha((cwd/f['path']).read_bytes())==f['after_sha256'],f['path']
manifest={'recorded_at':datetime.datetime.now().astimezone().isoformat(),'head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=cwd,text=True).strip(),'ruleset':328,'archive':105,'native_abi':753,'pure_nc10_files':files,'p2_changed_files':delta_paths,'complete_patch_sha256':sha(complete.encode()),'p2_delta_patch_sha256':sha(delta.encode()),'borrowed_manifest_sha256':sha((root/'borrowed-prerequisites.json').read_bytes()),'borrowed_paths_verified':len(borrowed['files']),'borrowed_excluded_from_pure_patches':True,'no_staged_files':not subprocess.check_output(['git','diff','--cached','--name-only'],cwd=cwd,text=True).strip(),'cargo_incremental':'0 for every recovery Rust/WASM command; cache setting only; no performance conclusion'}
(root/'corrected-source-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps({'pure_files':len(files),'delta_paths':delta_paths,'complete_sha256':manifest['complete_patch_sha256'],'delta_sha256':manifest['p2_delta_patch_sha256'],'borrowed_verified':8,'no_staged_files':manifest['no_staged_files']},indent=2))
