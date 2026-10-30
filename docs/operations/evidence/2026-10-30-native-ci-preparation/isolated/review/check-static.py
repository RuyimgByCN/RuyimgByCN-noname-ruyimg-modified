import json, pathlib, subprocess, os
import yaml
root=pathlib.Path.cwd(); env={**os.environ,'GIT_OPTIONAL_LOCKS':'0','GIT_NO_LAZY_FETCH':'1'}
old=yaml.load(subprocess.check_output(['git','show','HEAD:.github/workflows/ci.yml'],env=env),Loader=yaml.BaseLoader)
new=yaml.load((root/'.github/workflows/ci.yml').read_text(),Loader=yaml.BaseLoader)
action=yaml.load((root/'.github/actions/prepare-original-content/action.yml').read_text(),Loader=yaml.BaseLoader)
assert old['on']==new['on']=={'push':'','pull_request':''}
assert set(old['jobs'])==set(new['jobs'])
changed={'pnpm install --frozen-lockfile','pnpm --dir apps/web-client install --frozen-lockfile','node --test scripts/test_original_content_order.mjs\nnode scripts/sync-original-content.mjs --verify-only\n'}
for name,job in old['jobs'].items():
 current=new['jobs'][name];assert job.get('strategy')==current.get('strategy');assert job['runs-on']==current['runs-on']
 for step in job['steps']:
  if 'run' in step: assert step['run'] in changed or any(s.get('run')==step['run'] for s in current['steps'])
 for step in current['steps']:
  if step.get('uses','').startswith('./'):assert pathlib.Path(step['uses'].removeprefix('./noname-next/'),'action.yml').is_file()
  if step.get('uses')=='Swatinem/rust-cache@v2':assert step['with']['workspaces']=='noname-next'
for step in action['runs']['steps']:
 if 'run' in step:
  assert step['shell']=='bash' and step['working-directory']=='noname-next'
  subprocess.run(['bash','-n'],input=step['run'],text=True,check=True)
assert new['permissions']=={'contents':'read'}
assert new['jobs']['desktop']['steps'][-1]['with']['path'].strip()=='noname-next/target/release/bundle/**'
manifest=json.loads((root/'content/original/asset-manifest.json').read_text())
assert action['runs']['steps'][0]['with']['ref']==manifest['sources']['core']['head']
source='/Users/ruyimg/Documents/Claude/sanguosha/archive/upstream/noname'
tracked=set(subprocess.check_output(['git','-C',source,'ls-files','-z'],env=env).decode().split('\0'))
missing=[a['source_path'] for a in manifest['assets'] if 'apps/core/'+a['source_path'] not in tracked]
assert not missing,missing
# Full tests actually register these unconditional reads; no cargo invocation needed.
assert 'mod l10_epic_cards_eight;' in pathlib.Path('crates/engine-native/tests/groups/l_rounds.rs').read_text()
assert 'mod nc08_upstream_boss_zhugeliang;' in pathlib.Path('crates/engine-native/tests/groups/nc08_upstream_a_k.rs').read_text()
assert all('noname-liuli' not in s.get('with',{}).get('repository','') for s in action['runs']['steps'])
assert not any('wasm-pack' in s.get('run','') for s in new['jobs']['gates']['steps'])
assert 'version = "0.2.126"' in pathlib.Path('Cargo.lock').read_text()
assert '"--version", "0.2.100"' in pathlib.Path('apps/web-client/scripts/build-wasm.mjs').read_text()
print(json.dumps({'passed':True,'jobs':list(new['jobs']),'matrix_executions':sum(len(j.get('strategy',{}).get('matrix',{}).get('os', ['single'])) for j in new['jobs'].values()),'existing_commands_preserved':True,'composite_shell_and_local_uses':'passed','cache_and_artifact_paths':'passed','all_manifest_assets_tracked_in_adopted_source':len(manifest['assets']),'unconditional_liuli_callers_registered':True,'gates_no_wasm_pack':True,'fallback_and_lock_versions':['0.2.100','0.2.126']},indent=2))
