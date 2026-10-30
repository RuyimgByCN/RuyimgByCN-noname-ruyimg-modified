import copy,hashlib,json,os,pathlib,shlex,subprocess
import yaml
root=pathlib.Path.cwd();ev=root/'../evidence/perf-profile';out=ev/'review';env={**os.environ,'GIT_OPTIONAL_LOCKS':'0','GIT_NO_LAZY_FETCH':'1'}
sha=lambda b:hashlib.sha256(b).hexdigest()
old=yaml.load((ev/'ci-before.yml').read_text(),Loader=yaml.BaseLoader)
new=yaml.load((root/'.github/workflows/ci.yml').read_text(),Loader=yaml.BaseLoader)
normalized=copy.deepcopy(new);changes=[]
for before,after in zip(old['jobs']['perf']['steps'],normalized['jobs']['perf']['steps']):
 if before==after:continue
 assert before['name']==after['name'] and set(before)==set(after)=={'name','run'}
 tokens=shlex.split(after['run']);assert tokens.count('--release')==1
 assert [t for t in tokens if t!='--release']==shlex.split(before['run'])
 changes.append({'name':after['name'],'before':before['run'],'after':after['run']})
 subprocess.run(['bash','-n'],input=after['run'],text=True,env=env,check=True)
 after['run']=before['run']
assert len(changes)==2 and normalized==old
assert len(new['jobs'])==5 and sum(len(j.get('strategy',{}).get('matrix',{}).get('os',['single'])) for j in new['jobs'].values())==10
added='Perf runs both commands with `--release`, matching the historical phase17\nmeasurement profile. The unchanged runner measures the legacy engine-core line;\nprofile diagnostics do not establish an engine-native performance improvement.\n\n'
assert (root/'docs/operations/native-ci.md').read_text().replace(added,'')==(ev/'native-ci-before.md').read_text()
base='9dbc65530feb695688d3160dc38b6cf64ad39ecd'
for path,filename in [('.github/workflows/ci.yml','ci-before.yml'),('docs/operations/native-ci.md','native-ci-before.md')]:
 assert subprocess.check_output(['git','show',base+':'+path],env=env)==(ev/filename).read_bytes()
for p,digest in json.loads((ev/'benchmark-source-sha256.json').read_text()).items():
 assert sha((root/p).read_bytes())==digest
 assert subprocess.check_output(['git','show',base+':'+p],env=env)==(root/p).read_bytes()
for p,digest in json.loads((ev/'original-failure-sha256.json').read_text()).items():assert sha((ev/p).read_bytes())==digest
for osname in ['ubuntu','macos']:
 log=(ev/f'run41053688083-perf-{osname}-raw.log').read_text()
 assert 'Run cargo run -q -p benchmark-runner -- tier-report server-baseline' in log
 assert 'result=FAIL' in log and 'Process completed with exit code 1.' in log
results=json.loads((ev/'diagnostic-results.json').read_text())
release=next(r for r in results if r['profile']=='release');tier=next(r for r in release['runs'] if r['argv'][-2:]==['tier-report','server-baseline'])
assert tier['exit']==1
text=(ev/'release-tier-report.log').read_text();assert '151 / 100 us' in text and 'result=FAIL' in text
assert not (root/'../target-perf').exists()
checks=[]
for command in [['git','apply','--reverse','--check',str(ev/'perf-profile-only.patch')],['git','diff','--check']]:
 r=subprocess.run(command,env=env,capture_output=True,text=True);assert r.returncode==0,r.stderr;checks.append({'command':' '.join(command),'exit':r.returncode})
entry=json.loads((out/'entry.json').read_text())
assert all(sha((root/p).read_bytes())==h for p,h in entry['files'].items())
assert sha((ev/'perf-profile-only.patch').read_bytes())==entry['patch_sha256']
assert sha(pathlib.Path(entry['index_path']).read_bytes())==entry['index_sha256']
for key,args in [('HEAD',['rev-parse','HEAD']),('diff',['diff','--binary']),('staged',['diff','--cached','--binary'])]:assert sha(subprocess.check_output(['git',*args],env=env))==entry[key]['sha256']
assert subprocess.check_output(['git','diff','--cached','--name-only'],env=env)==b''
report={'passed':True,'accepted_base':base,'two_command_changes':changes,'entire_normalized_workflow_equals_published_base':True,'only_three_doc_lines_plus_blank_added':True,'jobs':5,'matrix_executions':10,'benchmark_code_Cargo_lock_and_profile_definitions_equal_published_base':True,'original_debug_failure_logs_preserved':True,'release_diagnostic_exit':1,'release_start_us':151,'start_budget_us':100,'cleanup_target_absent':True,'commands':checks,'entry_exit_files_patch_index_diff_HEAD_unchanged':True,'no_staged_files':True,'no_build_or_platform_calls':True}
(out/'checks.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
