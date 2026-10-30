import pathlib,hashlib,json,re,subprocess,os
import yaml
root=pathlib.Path.cwd();out=root/'../evidence/review-corrections';env={**os.environ,'GIT_OPTIONAL_LOCKS':'0','GIT_NO_LAZY_FETCH':'1'}
sha=lambda b:hashlib.sha256(b).hexdigest()
workflow=yaml.load((root/'.github/workflows/ci.yml').read_text(),Loader=yaml.BaseLoader)
prior=yaml.load((root/'../evidence/corrections/original/.github/workflows/ci.yml').read_text(),Loader=yaml.BaseLoader)
action=yaml.load((root/'.github/actions/prepare-original-content/action.yml').read_text(),Loader=yaml.BaseLoader)
assert set(workflow['jobs'])==set(prior['jobs'])
for name,job in prior['jobs'].items():
 now=workflow['jobs'][name];assert job.get('strategy')==now.get('strategy') and job['runs-on']==now['runs-on']
 for step in job['steps']:
  if 'run' in step:assert any(x.get('run')==step['run'] for x in now['steps'])
 for step in now['steps']:
  if step.get('uses','').startswith('./'):
   assert step['uses']=='./noname-next/.github/actions/prepare-original-content'
   assert step.get('with',{}).get('integration-sources','false')==('true' if name in ['gates','native-web'] else 'false')
  if step.get('uses')=='Swatinem/rust-cache@v2':assert step['with']['workspaces']=='noname-next'
assert workflow['on']==prior['on']=={'push':'','pull_request':''}
assert workflow['permissions']=={'contents':'read'}
assert sum(len(j.get('strategy',{}).get('matrix',{}).get('os',['single'])) for j in workflow['jobs'].values())==10
assert workflow['jobs']['desktop']['steps'][-1]['with']['path'].strip()=='noname-next/target/release/bundle/**'
steps=action['runs']['steps'];core=steps[0];bind=steps[1]
assert core['with']['ref']=='1a548c95acdf8ac1ef44d74473403eda2ac86cf0'
assert bind['working-directory']=='archive/upstream/noname'
assert bind['run']=='git remote set-url origin https://github.com/libnoname/noname.git'
liuli=steps[2];restore=steps[3]
assert liuli['with']['ref']=='8485dbb29ff16f9b258a766b1a17ef75f883aade'
assert liuli['with']['sparse-checkout-cone-mode']=='false'
assert liuli['if']==restore['if']=="inputs.integration-sources == 'true'"
patterns=liuli['with']['sparse-checkout'].splitlines();assert len(patterns)==12 and all(p.startswith('/noname/') and not any(c in p for c in '*?[') for p in patterns)
readers={};targets=set()
for p in (root/'crates').rglob('*.rs'):
 text='\n'.join(l for l in p.read_text().splitlines() if not l.lstrip().startswith('//'))
 found=re.findall(r'"../../../archive/upstream/noname-liuli/([^"\n]+)"',text)
 if not found:continue
 concrete=[]
 for f in found:
  if '{name}' in f:
   args=re.findall(r'upstream_data\("([^"\n]+)"\)',text);assert args
   concrete += [f.replace('{name}',a) for a in args]
  else:concrete.append(f)
 concrete=sorted(set(concrete));readers[str(p.relative_to(root))]=concrete;targets.update(concrete)
assert len(readers)==8 and targets=={p[1:] for p in patterns}
for p in readers:
 stem=pathlib.Path(p).stem
 assert any(f'mod {stem};' in g.read_text() for g in (root/'crates/engine-native/tests/groups').glob('*.rs'))
patch=root/'.github/actions/prepare-original-content/liuli-longxue.patch';lines=patch.read_bytes().splitlines(keepends=True)
assert sum(l.startswith(b'diff --git ') for l in lines)==1
hunk=next(i for i,l in enumerate(lines) if l.startswith(b'@@ '));assert lines[hunk]==b'@@ -1,509 +1 @@\n'
old=[];new=[];previous=None
for line in lines[hunk+1:]:
 if line.startswith(b'-'):old.append(line[1:])
 elif line.startswith(b'+'):new.append(line[1:])
 elif line.startswith(b' '):old.append(line[1:]);new.append(line[1:])
 elif line.startswith(b'\\ No newline'):
  if previous in (b'-',b' '):old[-1]=old[-1].removesuffix(b'\n')
  if previous in (b'+',b' '):new[-1]=new[-1].removesuffix(b'\n')
 else:raise AssertionError(repr(line[:50]))
 previous=line[:1]
old=b''.join(old);new=b''.join(new)
pre=sha(old);post=sha(new)
assert pre=='cf953876295d884bf2a766dd3a74e953d30073dc7a513ecb16cd59d50ad0db76'
assert post=='e3fad7cf47bfb5a6f6b55d21bd6dbdd7b44753a6b92d7407a2a1c88bb71d2db7'
assert pre in restore['run'] and post in restore['run']
blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
assert f'index {blob(old)}..{blob(new)} 100644\n'.encode() in lines
assert new==(pathlib.Path('/Users/ruyimg/Documents/Claude/sanguosha/archive/upstream/noname-liuli')/'noname/extension/龙血玄黄/extension.js').read_bytes()
assert b'jsjiami' not in old and b'jsjiami' in new and b'mbqianlong_info' not in new
# Verify only this path can be affected, using Git's own patch parser in read-only summary mode.
parsed=subprocess.check_output(['git','-c','core.quotepath=false','apply','--numstat','-z',str(patch)],env=env)
assert parsed==b'1\t509\tnoname/extension/'+ '龙血玄黄'.encode()+b'/extension.js\0',parsed
for step in steps:
 if 'run' in step:
  assert step['shell']=='bash' and step['working-directory'] in ['archive/upstream/noname','archive/upstream/noname-liuli','noname-next']
  subprocess.run(['bash','-n'],input=step['run'],text=True,check=True,env=env)
assert "path.join(process.env.GITHUB_WORKSPACE," in restore['run']
assert "'noname-next/.github/actions/prepare-original-content/liuli-longxue.patch'" in restore['run']
assert "execFileSync('git', ['apply'," in restore['run']
for label in ['gates','native-web','desktop']:
 seq=workflow['jobs'][label]['steps'];install=next(i for i,s in enumerate(seq) if s.get('name')=='Install wasm-pack')
 assert seq[install]['run']=='cargo install wasm-pack --version 0.15.0 --locked'
 if label=='gates':assert install<next(i for i,s in enumerate(seq) if s.get('run')=='bash scripts/cross-target-hash.sh')
commands=[]
for p in ['native-ci-only.patch','correction-only.patch']:
 command=['git','apply','--reverse','--check',str(root/'../evidence/corrections'/p)]
 r=subprocess.run(command,env=env,capture_output=True,text=True);assert r.returncode==0,r.stderr;commands.append({'command':' '.join(command),'exit':r.returncode})
subprocess.run(['git','diff','--check'],env=env,check=True)
result={'passed':True,'jobs':5,'matrix_executions':10,'all_prior_run_commands_preserved':True,'actual_raw_reader_files':readers,'sparse_inputs':len(targets),'patch_scope':parsed.decode().rstrip('\0'),'preimage_sha256':pre,'postimage_sha256':post,'blob_oids_match_patch':True,'postimage_equals_actual_canonical_bytes':True,'shell_syntax':'passed','origin_cwd_and_url':'passed','patch_path_and_exec_argument_array':'passed','wasm_pack_ordering':'passed','patch_reverse_checks':commands,'limits':'No Git init/add/commit, restore, metadata write, Cargo/build, or platform execution. Existing real sparse/apply fixtures reviewed, not repeated.'}
(out/'checks.json').write_text(json.dumps(result,indent=2,ensure_ascii=False)+'\n');print(json.dumps(result,indent=2,ensure_ascii=False))
