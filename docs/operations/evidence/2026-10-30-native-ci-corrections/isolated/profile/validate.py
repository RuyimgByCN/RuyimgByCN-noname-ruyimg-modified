import hashlib,json,shlex,subprocess
from pathlib import Path
import yaml
root=Path.cwd();out=root/'../evidence/perf-profile'
workflow='.github/workflows/ci.yml';doc='docs/operations/native-ci.md'
old=yaml.load((out/'ci-before.yml').read_text(),Loader=yaml.BaseLoader)
new=yaml.load(Path(workflow).read_text(),Loader=yaml.BaseLoader)
assert set(old)==set(new)
for key in old:
    if key!='jobs':assert old[key]==new[key]
assert set(old['jobs'])==set(new['jobs'])
for key in old['jobs']:
    if key!='perf':assert old['jobs'][key]==new['jobs'][key]
a=old['jobs']['perf'];b=new['jobs']['perf']
for key in a:
    if key!='steps':assert a[key]==b[key]
assert a['steps'][:-2]==b['steps'][:-2]
for before,after in zip(a['steps'][-2:],b['steps'][-2:]):
    assert before['name']==after['name']
    tokens=shlex.split(after['run']);assert tokens.count('--release')==1
    assert [s for s in tokens if s!='--release']==shlex.split(before['run'])
    subprocess.run(['bash','-n'],input=after['run'],text=True,check=True)
assert sum(len(j.get('strategy',{}).get('matrix',{}).get('os',['single'])) for j in new['jobs'].values())==10
base=json.loads((out/'ci-before-sha.json').read_text())
for name in ['.github/actions/prepare-original-content/action.yml','.github/actions/prepare-original-content/liuli-longxue.patch']:
    assert hashlib.sha256(Path(name).read_bytes()).hexdigest()==base[name]
overlay=json.loads(Path('../evidence/borrowed-resource-overlay.json').read_text())['not_part_of_ci_patch']
assert all(hashlib.sha256(Path(name).read_bytes()).hexdigest()==sha for name,sha in overlay.items())
patch=b'';files={}
for name,oldfile in [(workflow,'ci-before.yml'),(doc,'native-ci-before.md')]:
    original=out/oldfile
    d=subprocess.run(['git','diff','--no-index','--binary','--',str(original),name],capture_output=True)
    assert d.returncode==1
    lines=d.stdout.decode().splitlines(keepends=True);lines[0]=f'diff --git a/{name} b/{name}\n'
    for i,line in enumerate(lines):
        if line.startswith('--- '):lines[i]=f'--- a/{name}\n';break
    patch+=''.join(lines).encode()
    files[name]={'base_sha256':hashlib.sha256(original.read_bytes()).hexdigest(),'final_sha256':hashlib.sha256(Path(name).read_bytes()).hexdigest()}
(out/'perf-profile-only.patch').write_bytes(patch)
assert len(patch.splitlines())<=50,len(patch.splitlines())
checks=[['git','apply','--reverse','--check',str((out/'perf-profile-only.patch').resolve())],['git','diff','--check'],['python3','-B','-m','unittest','discover','-s','scripts','-p','test_check_source_lines.py'],['python3','-B','scripts/check-source-lines.py']]
commands=[]
for i,cmd in enumerate(checks):
    p=subprocess.run(cmd,capture_output=True,text=True);assert p.returncode==0,p.stderr
    (out/f'static-check-{i}.log').write_text(p.stdout+p.stderr)
    commands.append({'command':cmd,'exit':p.returncode})
assert subprocess.check_output(['git','diff','--cached','--name-only'])==b''
result={'passed':True,'files':files,'patch_sha256':hashlib.sha256(patch).hexdigest(),'patch_lines':len(patch.splitlines()),'only_perf_argv_profile_changed':True,'all_other_jobs_steps_inputs_versions_budgets_unchanged':True,'five_jobs_ten_matrix_executions':True,'six_resource_premise_files_unchanged':True,'commands':commands,'no_staged_files':True,'actionlint':'not installed; YAML parsed with existing PyYAML and shell syntax checked'}
(out/'static-validation.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result,indent=2))
