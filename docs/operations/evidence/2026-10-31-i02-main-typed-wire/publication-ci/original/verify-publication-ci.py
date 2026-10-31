import datetime
import gzip
import hashlib
import json
from pathlib import Path
import re
import zlib

ROOT = Path(__file__).resolve().parent
PUSH = ROOT/'push-stage'
CI = ROOT/'ci-stage'
SHA = '34e83f0550bb70ef4af6083697428de7be2f02f8'
OLD = 'd07b35f94a42b282222dda4ab7bbc5c39ab8fddb'
MAIN = '47347c6608d651660ae9459e59eb529b3cef277f'
RUN = 41214274772
ANSI = re.compile(r'\x1b\[[0-?]*[ -/]*[@-~]')
RUST = re.compile(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;')

def read(path):
    return json.loads(path.read_bytes())

def section(lines, command):
    start = next(i for i,line in enumerate(lines) if line.startswith('##[group]Run '+command))
    end = next((i for i in range(start+1,len(lines)) if lines[i].startswith('##[group]Run ')),len(lines))
    return lines[start:end]

def identity(data):
    return {'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest(),'crc32':f'{zlib.crc32(data)&0xffffffff:08x}'}

push=read(PUSH/'result.json');scope=read(PUSH/'scope.json');admission=read(PUSH/'protected-admission.json');audit=read(PUSH/'stage-audit.raw.json');plan=read(PUSH/'stage-plan.raw.json');state=read(CI/'observation-state.json');run=read(CI/'run-final.stdout');lease=read(PUSH/'lease-release.json')
assert push['accepted_commit']==SHA and push['accepted_tree']=='22e6eab72d104baf7a658d5ff2d832128ac32e4a' and push['expected_old_remote']==OLD
assert push['push_attempts']==1 and push['push_exit']==0 and push['remote_verified'] and not push.get('error') and not push.get('preservation_error')
assert push['before']==push['immediately_before_push']==push['after'] and not push['after']['tracked_diff'] and not push['after']['staged_diff']
assert push['after']['original_hook_sha256']=='e349af1ec5ccc1742df7e7d91d387ed2ef086f7c864d8cf7372d0ef66dd49ca7'
assert push['after']['core_hooksPath_exit']==1 and not push['after']['core_hooksPath_stdout']
assert push['committed_protected_sha256']==push['after']['protected_sha256']==admission['expected']
assert len(admission['production31'])==31 and len(admission['own6'])==6
assert admission['production31']['.github/workflows/ci.yml']=='b46fd58a50fe5a74c537929d66919fca694163588c19600226426d48f43c45e8'
assert len(scope['selftests'])==8 and all(t['exit']==t['expected'] for t in scope['selftests'])
assert next(t for t in scope['selftests'] if t['name']=='withdrawn-bdaf-new-SHA')['exit']==1
assert hashlib.sha256((PUSH/'hooks/pre-push').read_bytes()).hexdigest()==scope['scoped_hook_sha256']
assert audit['allBlobsModesExact'] and audit['files']==3433 and audit['bytes']==105988114
assert push['ancestry']['parent_d07_ancestor_exit']==0 and push['ancestry']['withdrawn_bdaf_ancestor_exit']==1
assert push['ancestry']['local_protect_sha']=='bdaf9868ef49de08dc66127eb82b05d922cc9eba'
assert len(plan['excludedLocalAliases'])==push['excluded552_verified']==552
assert len(read(PUSH/'exclusions552-path-stat.json')['excluded'])==552
stat=read(PUSH/'accepted3433-tree-stat.json');assert len(stat['entries'])==3433 and sum(i['bytes'] for i in stat['entries'])==105988114
assert set(i['path'] for i in stat['entries'])==set(push['committed_increment_paths'])
assert set(p for p in push['committed_increment_paths'] if not p.startswith('docs/'))==set(admission['production31'])
assert push['remote_before_refs']=={'refs/heads/main':MAIN,'refs/heads/native-convergence':OLD}
assert push['remote_after_refs']=={'refs/heads/main':MAIN,'refs/heads/native-convergence':SHA}
for suffix,key in (('raw-index','index_sha256'),('logical-index','logical_index_sha256'),('index-flags','index_flags_sha256'),('untracked-paths','untracked_paths_sha256')):
    bodies=[gzip.decompress((PUSH/(stage+'-'+suffix+'.gz')).read_bytes()) for stage in ('before','immediately_before_push','after')]
    assert bodies[0]==bodies[1]==bodies[2] and hashlib.sha256(bodies[0]).hexdigest()==push['after'][key]
assert run['id']==RUN and run['head_sha']==SHA and run['head_branch']=='native-convergence' and run['event']=='push' and run['run_attempt']==1
watches=[c for c in state['commands'] if c['name']=='watch'];assert len(watches)==1 and watches[0]['http1']
summary={'verified_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'head_sha':SHA,'tree':push['accepted_tree'],'parent':OLD,'run_id':RUN,'run_url':run['html_url'],'run_status':run['status'],'run_conclusion':run['conclusion'],'run_created_at':run['created_at'],'run_updated_at':run['updated_at'],'observation_started_utc':state['started_utc'],'observation_finished_utc':state['finished_utc'],'lease_release':lease,'push':push,'guard':scope,'parent_full3433_blob_mode_audit':audit,'fresh_audit_scope':'3433Gitpath/mode/OID/type/size metadata,46protected/sourcebodySHAs and552excludedpath/stat checks; parentall3433bodySHAaudit isseparate; excludedbodies notread','jobs':[],'watch_commands':watches,'watch_stderr':(CI/'watch.stderr').read_text(),'read_errors':[c for c in state['commands'] if c['name']!='watch' and c.get('exit')!=0],'successful_jobs':[],'failed_jobs':[],'pending_jobs':[],'artifacts':state['artifacts'],'private_originals':[],'warning_qualification':'Matchedwarning lines including repeats; complete original raw logs preserve all output. Counts across overlapping suites/jobs/platforms must not be summed asunique tests.','cleanup':state['cleanup'],'checkpoint':state.get('checkpoint_15min')}
for job in state['jobs']:
    item={k:job.get(k) for k in ('id','name','html_url','status','conclusion','started_at','completed_at','detail_error','log_error','raw_log','log_bytes','log_sha256','log_crc32','log_crlf_count')}
    detail=job.get('detail',{});assert not detail or (detail['id']==job['id'] and detail['run_id']==RUN and detail['head_sha']==SHA)
    steps={s['number']:s for s in job['listed_steps']}
    for step in job['steps']:
        old=steps.get(step['number'])
        if old and old.get('conclusion') is not None and step.get('conclusion') is not None:assert old['conclusion']==step['conclusion']
        if old is None or old.get('conclusion') is None or step.get('conclusion') is not None:steps[step['number']]=step
    item['steps']=[steps[n] for n in sorted(steps)];item['skipped_steps']=[s for s in item['steps'] if s.get('conclusion')=='skipped'];item['failed_steps']=[s for s in item['steps'] if s.get('conclusion')=='failure'];item['null_step_conclusions']=[s for s in item['steps'] if s.get('conclusion') is None]
    item['skip_reason_limit']='API textual skip causes unavailable; original states/names retained without filling missingmetadata.'
    if job['status']!='completed':summary['pending_jobs'].append(job['name'])
    elif job['conclusion']=='success':summary['successful_jobs'].append(job['name'])
    else:summary['failed_jobs'].append(job['name'])
    if item['started_at'] and item['completed_at']:item['duration_seconds']=(datetime.datetime.fromisoformat(item['completed_at'].replace('Z','+00:00'))-datetime.datetime.fromisoformat(item['started_at'].replace('Z','+00:00'))).total_seconds()
    if job.get('raw_log'):
        path=CI/job['raw_log'];raw=gzip.decompress(path.read_bytes());private=ROOT/'private-raw-downloads'/path.name.removesuffix('.gz');assert private.read_bytes()==raw
        original=identity(raw);assert original=={'bytes':job['log_bytes'],'sha256':job['log_sha256'],'crc32':job['log_crc32']}
        item['gzip_file_identity']=identity(path.read_bytes());item['original_log_identity']=original
        lines=[ANSI.sub('',line.split(' ',1)[1]) for line in raw.decode(errors='replace').splitlines() if ' ' in line]
        item['warning_lines']=[line for line in lines if re.search(r'\bwarning(?:\[[^]]+\])?:|ExperimentalWarning:|##\[warning\]|\bnpm warn\b|^\s*WARN\b',line,re.I)]
        item['ignored_test_lines']=[line for line in lines if re.match(r'^test .* \.\.\. ignored',line)]
        item['error_annotations']=[line for line in lines if '##[error]' in line]
        tests=[tuple(map(int,m.groups())) for line in lines if (m:=RUST.search(line))]
        if tests:item['rust_tests']={**dict(zip(('passed','failed','ignored','measured','filtered_out'),map(sum,zip(*tests)))),'summary_count':len(tests)}
        suites=[];suite_label='unlabeled'
        for line in lines:
            if line.strip().startswith(('Running ','Doc-tests ')):suite_label=line.strip()
            match=RUST.search(line)
            if match:suites.append({'suite':suite_label,**dict(zip(('passed','failed','ignored','measured','filtered_out'),map(int,match.groups())))})
        item['rust_suite_summaries']=suites
        if job['name']=='gates-ubuntu':
            selected=[suite for suite in suites if re.search(r'[/\\]engine_(?:api|protocol)-[0-9a-f]+|^Doc-tests engine_(?:api|protocol)$|^Running tests/presentation(?:_main5fc_reader)?\.rs ',suite['suite'])]
            item['api_protocol_suites']=selected
            item['api_protocol_totals']={key:sum(suite[key] for suite in selected) for key in ('passed','failed','ignored','measured','filtered_out')}
        if job['name']=='native-web':
            smoke=section(lines,'pnpm smoke:native');passes=[line.split(' | ',1)[0] for line in smoke if line.startswith('PASS ')];assert len(passes)==len(set(passes)) and not any(line.startswith('FAIL ') for line in smoke);item['worker_passes']=passes
            item['frontend_tap']=[line for line in section(lines,'pnpm test') if re.match(r'^# (tests|pass|fail|skipped|todo) ',line)]
        if job['name']=='gates-ubuntu':
            conform=section(lines,'cargo run -p conformance');item['conformance_cases']=[line.strip() for line in conform if re.match(r'^\s*OK\s',line)];item['conformance_all_pass']='ALL PASS' in conform
            item['cross_target_pass']='ALL CROSS-TARGET CHECKS PASSED' in section(lines,'bash scripts/cross-target-hash.sh')
            item['resource_tap']=[line for line in section(lines,'node --test scripts/test_original_content_order.mjs') if re.match(r'^# (tests|pass|fail|skipped) ',line)]
        if job['name'] in ('gates-ubuntu','native-web'):
            item['public_own_plan_tests']=[line for line in lines if line.startswith('test nc10_public_own_plan::') and line.endswith(' ... ok')]
            item['public_own_session_test_pass']='test auto_step_uses_the_same_public_card_advice_and_submits_once ... ok' in lines
        if job['name'].startswith('perf-'):
            metrics=[]
            for line in lines:
                m=re.search(r'✓\s+(start-hash-p50|distance-query-p95|snapshot-restore|observer-projection)\s+(\d+) / (\d+) us',line)
                if m:metrics.append({'name':m[1],'actual_us':int(m[2]),'budget_us':int(m[3])})
            item['legacy_server_baseline_metrics']=metrics;item['legacy_server_baseline_result_pass']='result=PASS' in lines
        if job['name'] in ('build-windows-latest','desktop-windows-latest'):
            checkout=next(s for s in item['steps'] if s['number']==2);item['actual_checkout_step2']=checkout
            label='windows-build' if job['name']=='build-windows-latest' else 'windows-desktop';early=read(CI/(label+'-early-detail.stdout'));assert early['id']==job['id'] and early['head_sha']==SHA
            assert next(s for s in early['steps'] if s['number']==2)['conclusion']==checkout['conclusion']=='success'
            item['checkout_env_log_lines']=[line for line in lines if any(k in line for k in ('GIT_CONFIG_COUNT:','GIT_CONFIG_KEY_0:','GIT_CONFIG_VALUE_0:'))]
        summary['private_originals'].append({'path':str(private.relative_to(ROOT)),**original,'copy_policy':'Private original excluded; gzip preserves identical bytes'})
    summary['jobs'].append(item)
summary['step_count']=sum(len(j['steps']) for j in summary['jobs']);summary['skipped_step_count']=sum(len(j['skipped_steps']) for j in summary['jobs']);summary['failed_step_count']=sum(len(j['failed_steps']) for j in summary['jobs']);summary['null_step_conclusion_count']=sum(len(j['null_step_conclusions']) for j in summary['jobs']);summary['raw_log_bytes']=sum(j.get('log_bytes') or 0 for j in summary['jobs']);summary['matched_warning_log_lines']=sum(len(j.get('warning_lines',[])) for j in summary['jobs'])
for a in summary['artifacts']['artifacts']:
    assert a['workflow_run']['id']==RUN and a['workflow_run']['head_sha']==SHA
    assert a['size_in_bytes']>0 and re.fullmatch(r'sha256:[0-9a-f]{64}',a['digest'])
summary['checkpoint_manifest_limit']='Dated checkpoint index binds then-live watch/state files. Current root copy-manifest/seal is final authority.'
with (ROOT/'verified-publication-ci-summary.json').open('x') as f:json.dump(summary,f,indent=2);f.write('\n')
files=[]
for path in sorted(ROOT.rglob('*')):
    if path.is_file() and 'private-raw-downloads' not in path.relative_to(ROOT).parts:
        data=path.read_bytes();record={'path':str(path.relative_to(ROOT)),**identity(data)}
        if path.suffix=='.gz':record['gzip_original_identity']=identity(gzip.decompress(data))
        files.append(record)
with (ROOT/'copy-manifest.json').open('x') as f:json.dump({'source_root':str(ROOT),'files':files,'private_originals_excluded':summary['private_originals'],'manifest_excludes_itself':True,'copy_instruction':'Copy listedfiles plusmanifest and laterseal. Do notstageprivate originals; nevernormalize gzip payloads.'},f,indent=2);f.write('\n')
print(json.dumps({'run_id':RUN,'run_status':summary['run_status'],'run_conclusion':summary['run_conclusion'],'successful_jobs':summary['successful_jobs'],'failed_jobs':summary['failed_jobs'],'pending_jobs':summary['pending_jobs'],'steps':summary['step_count'],'skips':summary['skipped_step_count'],'nulls':summary['null_step_conclusion_count'],'raw_log_bytes':summary['raw_log_bytes'],'warning_lines':summary['matched_warning_log_lines'],'artifact_count':summary['artifacts']['total_count'],'files_before_seal':len(files)+1},indent=2))
