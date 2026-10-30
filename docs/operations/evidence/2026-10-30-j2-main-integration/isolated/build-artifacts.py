import hashlib,json,pathlib,subprocess,tempfile
ROOT=pathlib.Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-j2-llgkp_le/worktree');E=ROOT.parent/'evidence'/'theft-correction';B=E/'baseline-71';BASE='2b3348eff0a6a7f07d6f2ba3520e4ca243e344a6'
def git(*args,cwd=ROOT,check=True):return subprocess.run(['git',*args],cwd=cwd,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=check)
def head(path):
 p=git('show',BASE+':'+path,check=False);return p.stdout if p.returncode==0 else None
def sha(data):return hashlib.sha256(data).hexdigest() if data is not None else None
prior=json.loads((B/'j2-reviewed-before-after-sha256.json').read_text())['files'];previous={f['path']:(B/'source'/f['path']).read_bytes() for f in prior}
for f in prior:assert sha(previous[f['path']])==f['after_sha256'],f['path']
tracked=git('diff','--name-only',BASE).stdout.decode().splitlines();untracked=git('ls-files','--others','--exclude-standard').stdout.decode().splitlines();files=sorted(set(tracked+[p for p in untracked if not p.startswith('.pi-lens') and not (ROOT/p).is_symlink() and (ROOT/p).is_file()]))
records=[]
for path in files:
 before=head(path);data=(ROOT/path).read_bytes();old=previous.get(path,before)
 records.append({'path':path,'base_sha256':sha(before),'prior_71_sha256':sha(old),'final_sha256':sha(data),'changed_since_71':old!=data,'bytes':len(data),'physical_lines':len(data.splitlines())})
protected=['crates/engine-native-kernel/src/catalog.rs','crates/engine-native-content/src/content/standard.rs','crates/engine-native-kernel/src/snap/validation.rs','crates/engine-native-kernel/src/snap/asks.rs','crates/engine-native-kernel/src/archive/resolution.rs']
for path in protected:assert (ROOT/path).read_bytes()==previous[path],path
assert not any(p.startswith('docs/') for p in files)
patch=git('diff','--binary',BASE,'--',*tracked).stdout
for path in files:
 if path not in tracked:
  p=git('diff','--no-index','--binary','--','/dev/null',path,check=False);assert p.returncode in [0,1];patch+=p.stdout
full=E/'j2-theft-total-vs-2b3348ef.patch';full.write_bytes(patch)
with tempfile.TemporaryDirectory(prefix='j2-theft-patch-check-') as folder:
 temp=pathlib.Path(folder);old=temp/'prior';new=temp/'final';base=temp/'base'
 for d in [old,new,base]:d.mkdir()
 for path in files:
  before=head(path)
  for parent,data in [(old,previous.get(path,before)),(base,before),(new,(ROOT/path).read_bytes())]:
   if data is not None:p=parent/path;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(data)
 d=git('diff','--no-index','--binary','--','prior','final',cwd=temp,check=False);assert d.returncode in [0,1]
 lines=[]
 for line in d.stdout.splitlines(keepends=True):
  if line.startswith((b'diff --git ',b'--- ',b'+++ ',b'Binary files ')):
   for prefix in [b'a/',b'b/']:
    for part in [b'prior/',b'final/']:line=line.replace(prefix+part,prefix)
  lines.append(line)
 incremental=E/'j2-theft-incremental-vs-71.patch';incremental.write_bytes(b''.join(lines))
 for parent,p in [(old,incremental),(base,full)]:
  r=git('apply','--check','--unsafe-paths',str(p),cwd=parent,check=False);assert r.returncode==0,r.stderr.decode()
  r=git('apply','--unsafe-paths',str(p),cwd=parent,check=False);assert r.returncode==0,r.stderr.decode()
  for f in records:assert sha((parent/f['path']).read_bytes())==f['final_sha256'],f['path']
assert not git('diff','--cached','--name-only').stdout
manifest={'base':BASE,'prior_paths':len(prior),'final_paths':len(files),'files':records,'protected_files_unchanged_since_71':protected,'total_patch_applies_and_matches':True,'incremental_patch_applies_and_matches':True,'no_staged_files':True}
(E/'j2-theft-before-after-sha256.json').write_text(json.dumps(manifest,indent=2)+'\n');(E/'j2-theft-changed-files.txt').write_text('\n'.join(files)+'\n');(E/'j2-theft-delta-files.txt').write_text('\n'.join(f['path'] for f in records if f['changed_since_71'])+'\n')
(E/'j2-theft-artifact-sha256.json').write_text(json.dumps({p.name:sha(p.read_bytes()) for p in [full,incremental,E/'j2-theft-before-after-sha256.json',E/'j2-theft-changed-files.txt',E/'j2-theft-delta-files.txt']},indent=2)+'\n')
print(json.dumps({'total_paths':len(files),'delta_paths':[f['path'] for f in records if f['changed_since_71']],'protected_files_unchanged':True,'both_patches_apply_and_match':True,'no_staged_files':True},indent=2))
