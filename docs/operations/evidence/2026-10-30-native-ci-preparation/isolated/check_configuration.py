import hashlib,json,subprocess
from pathlib import Path
import yaml
root=Path.cwd()
evidence=(root/'../evidence').resolve()
paths=['.github/workflows/ci.yml','.github/actions/prepare-original-content/action.yml','docs/operations/native-ci.md']
old_text=subprocess.check_output(['git','show','HEAD:.github/workflows/ci.yml']).decode()
old=yaml.load(old_text,Loader=yaml.BaseLoader)
new=yaml.load(Path(paths[0]).read_text(),Loader=yaml.BaseLoader)
action=yaml.load(Path(paths[1]).read_text(),Loader=yaml.BaseLoader)
assert old['on'] == new['on'] == {'push':'','pull_request':''}
assert set(old['jobs']) == set(new['jobs'])
assert new['permissions'] == {'contents':'read'}
assert new['defaults']['run']['working-directory']=='noname-next'
changed_runs={
 'pnpm install --frozen-lockfile',
 'pnpm --dir apps/web-client install --frozen-lockfile',
 'node --test scripts/test_original_content_order.mjs\nnode scripts/sync-original-content.mjs --verify-only\n'
}
for name,job in old['jobs'].items():
    current=new['jobs'][name]
    assert job['runs-on']==current['runs-on']
    assert job.get('strategy')==current.get('strategy')
    oldruns=[step['run'] for step in job['steps'] if 'run' in step]
    newruns=[step['run'] for step in current['steps'] if 'run' in step]
    for command in oldruns:
        assert command in newruns or command in changed_runs,(name,command)
    assert current['steps'][0]['with']['path']=='noname-next'
    for step in current['steps']:
        if step.get('uses')=='Swatinem/rust-cache@v2':
            assert step['with']['workspaces']=='noname-next'
assert new['jobs']['native-web']['defaults']['run']['working-directory']=='noname-next/apps/web-client'
for name in ['gates','native-web','desktop']:
    assert any(s.get('uses')=='./noname-next/.github/actions/prepare-original-content' for s in new['jobs'][name]['steps'])
checkout=action['runs']['steps'][0]['with']
manifest=json.loads(Path('content/original/asset-manifest.json').read_text())
assert checkout['ref']==manifest['sources']['core']['head']
assert checkout['path']=='archive/upstream/noname'
assert checkout['repository']=='libnoname/noname'
assert checkout['sparse-checkout']=='apps/core'
assert checkout['persist-credentials']=='false'
assert manifest['sources']['skin'] is None
assert action['runs']['steps'][1]['with']['version']=='9'
assert action['runs']['steps'][2]['run']=='pnpm --dir apps/web-client install --frozen-lockfile'
assert action['runs']['steps'][3]['run'].splitlines()==[
 'node scripts/sync-original-content.mjs --resources-only',
 'node scripts/sync-original-content.mjs --resources-only --verify-only',
 'git diff --exit-code -- content/original']
for step in action['runs']['steps']:
    if 'run' in step:
        assert step['working-directory']=='noname-next'
        assert step['shell']=='bash'
        subprocess.run(['bash','-n'],input=step['run'],text=True,check=True)
assert new['jobs']['desktop']['steps'][-1]['with']['path'].strip()=='noname-next/target/release/bundle/**'
files={}
patch=b''
for name in paths:
    base=subprocess.run(['git','show','HEAD:'+name],capture_output=True)
    final=Path(name).read_bytes()
    files[name]={'base_sha256':hashlib.sha256(base.stdout).hexdigest() if base.returncode==0 else None,'final_sha256':hashlib.sha256(final).hexdigest()}
    if base.returncode==0:
        patch+=subprocess.check_output(['git','diff','--binary','--',name])
    else:
        diff=subprocess.run(['git','diff','--no-index','--binary','--','/dev/null',name],capture_output=True)
        assert diff.returncode==1
        patch+=diff.stdout
(evidence/'native-ci-only.patch').write_bytes(patch)
(evidence/'ci-file-sha256.json').write_text(json.dumps(files,indent=2)+'\n')
overlay=json.loads((evidence/'borrowed-resource-overlay.json').read_text())['not_part_of_ci_patch']
for name,digest in overlay.items():
    assert hashlib.sha256(Path(name).read_bytes()).hexdigest()==digest,name
tracked=set(subprocess.check_output(['git','ls-files','-z']).decode().strip('\0').split('\0'))
candidate=sorted(tracked|set(paths)|set(overlay))
(evidence/'proposed-native-tree-paths.txt').write_text('\n'.join(candidate)+'\n')
staged=subprocess.check_output(['git','diff','--cached','--name-only']).decode().strip()
assert not staged
result={'yaml_parsed':paths[:2],'all_existing_jobs_and_matrices_preserved':True,'all_gate_commands_preserved_or_test_extended':True,'checkout_paths_within_workspace':True,'source_head_matches_adopted_manifest':True,'bootstrap_shell_syntax':'passed','overlay_six_files_unchanged':True,'ci_changed_files':files,'proposed_tree_paths':len(candidate),'proposed_tree_sha256':hashlib.sha256((evidence/'proposed-native-tree-paths.txt').read_bytes()).hexdigest(),'no_staged_files':not staged,'actionlint':'not installed; not run'}
(evidence/'configuration-validation.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result,indent=2))
