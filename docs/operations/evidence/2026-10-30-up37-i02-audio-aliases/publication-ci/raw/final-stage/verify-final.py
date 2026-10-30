import datetime
import hashlib
import json
from pathlib import Path
import re

OUT = Path(__file__).resolve().parent
ROOT = OUT.parent
SHA = 'a1f7250815eee803d931301ab46207719cb9a15b'
RUN = 41100475767
ANSI = re.compile(r'\x1b\[[0-?]*[ -/]*[@-~]')
RUST = re.compile(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;')


def read(name):
    return json.loads((OUT / name).read_bytes())


def section(lines, command):
    start = next(i for i, line in enumerate(lines) if line.startswith('##[group]Run ' + command))
    end = next((i for i in range(start + 1, len(lines)) if lines[i].startswith('##[group]Run ')), len(lines))
    return lines[start:end]


state = read('collection-state.json')
assert state['collection_complete'] and not state['missing'] and not state.get('error')
assert len(state['jobs']) == 10
run = state['run']
parent = read('parent-api-snapshot/summary.json')
assert run['id'] == RUN and run['head_sha'] == SHA and run['event'] == 'push' and run['head_branch'] == 'native-convergence'
assert run['run_attempt'] == 1 and run['status'] == 'completed' and run['conclusion'] == 'success'
assert parent['run']['id'] == RUN and parent['run']['head_sha'] == SHA and parent['run']['conclusion'] == 'success'
assert all(c.get('exit') == 0 for c in state['commands'])
# The earlier transport checkpoint remains historical evidence; final success comes from APIs, not watch exit0.
old_manifest = json.loads((ROOT / 'copy-manifest.json').read_bytes())
for item in old_manifest['files']:
    p = ROOT / item['path']
    assert p.stat().st_size == item['bytes'] and hashlib.sha256(p.read_bytes()).hexdigest() == item['sha256']
watch_exits = []
for prefix in ('', 'http1-observation/', 'http1-final-authorized-watch/'):
    old = json.loads((ROOT / (prefix + 'observation-state.json')).read_bytes())
    watched = next(c for c in old['commands'] if c['name'] == 'watch')
    assert watched['exit'] == 1 and b'HTTP 502' in (ROOT / (prefix + 'watch.stderr')).read_bytes()
    watch_exits.append({'directory': prefix or '.', 'exit': watched['exit'], 'http1': watched['http1']})
summary = {'verified_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'run_id': RUN, 'head_sha': SHA, 'run_url': run['html_url'], 'run_attempt': 1, 'status': run['status'], 'conclusion': run['conclusion'], 'run_updated_at': run['updated_at'], 'parent_capture_utc': parent['utc'], 'new_fetch_started_utc': state['started_utc'], 'new_fetch_finished_utc': state['finished_utc'], 'earlier_watch_exits': watch_exits, 'final_success_evidence': 'Completed run/job APIs plus full raw job logs; no successful watch claimed', 'jobs': [], 'artifacts': state['artifacts'], 'api_step_limitations': [], 'local_source_not_inspected': True}
no_cache = read('attempt1-jobs-no-cache.stdout')
assert len(no_cache['jobs']) == 10
attempt_jobs = {j['id']: j for j in no_cache['jobs']}
for collected in state['jobs']:
    job = collected['detail']
    assert job['id'] == collected['id'] and job['run_id'] == RUN and job['head_sha'] == SHA and job['status'] == 'completed' and job['conclusion'] == 'success'
    raw = (OUT / collected['raw_log']).read_bytes()
    lines = []
    for line in raw.decode(errors='replace').splitlines():
        if ' ' in line:
            lines.append(ANSI.sub('', line.split(' ', 1)[1]))
    observations = [{'source': 'jobs-final-fetch.stdout', 'steps': collected['listed']['steps']}, {'source': 'job-' + str(job['id']) + '-detail.stdout', 'steps': job['steps']}, {'source': 'attempt1-jobs-no-cache.stdout', 'steps': attempt_jobs[job['id']]['steps']}]
    follow = OUT / ('job-' + str(job['id']) + '-detail-no-cache.stdout')
    if follow.exists():
        detail = json.loads(follow.read_bytes())
        assert detail['id'] == job['id'] and detail['head_sha'] == SHA and detail['conclusion'] == 'success'
        observations.append({'source': follow.name, 'steps': detail['steps']})
    by_number = {}
    for observation in observations:
        for step in observation['steps']:
            old = by_number.get(step['number'])
            if old is None or old['step'].get('conclusion') is None or step.get('conclusion') is not None:
                by_number[step['number']] = {'step': step, 'source': observation['source']}
    steps = [by_number[n] for n in sorted(by_number)]
    null_steps = [s for s in steps if s['step'].get('conclusion') is None]
    failures = [s for s in steps if s['step'].get('conclusion') == 'failure']
    assert not failures
    item = {k: job.get(k) for k in ('id', 'name', 'html_url', 'status', 'conclusion', 'started_at', 'completed_at')}
    item.update(raw_log=collected['raw_log'], log_bytes=len(raw), log_sha256=hashlib.sha256(raw).hexdigest(), log_crlf_count=raw.count(b'\r\n'), step_observations=observations, merged_step_records=steps, unresolved_api_step_records=null_steps, failed_step_records=failures, skipped_step_records=[s for s in steps if s['step'].get('conclusion') == 'skipped'])
    if null_steps:
        summary['api_step_limitations'].append({'job': job['name'], 'null_conclusion_steps': [s['step']['name'] for s in null_steps]})
    if item['started_at'] and item['completed_at']:
        begin = datetime.datetime.fromisoformat(item['started_at'].replace('Z', '+00:00'))
        end = datetime.datetime.fromisoformat(item['completed_at'].replace('Z', '+00:00'))
        item['duration_seconds'] = (end - begin).total_seconds()
    matches = [tuple(map(int, m.groups())) for line in lines if (m := RUST.search(line))]
    if matches:
        item['rust_tests'] = dict(zip(('passed', 'failed', 'ignored', 'measured', 'filtered_out'), map(sum, zip(*matches))))
        assert item['rust_tests']['failed'] == 0
    item['ignored_test_lines'] = [line for line in lines if re.match(r'^test .* \.\.\. ignored', line)]
    item['warning_lines'] = [line for line in lines if re.match(r'^(warning(?:\[[^]]+\])?:|##\[warning\])', line) or 'ExperimentalWarning:' in line]
    item['warning_line_count'] = len(item['warning_lines'])
    item['raw_error_annotations'] = [line for line in lines if '##[error]' in line]
    assert not item['raw_error_annotations']
    if job['name'].startswith('build-'):
        assert item['rust_tests'] == {'passed': 838, 'failed': 0, 'ignored': 0, 'measured': 0, 'filtered_out': 0}
    if job['name'] == 'native-web':
        assert item['rust_tests'] == {'passed': 1815, 'failed': 0, 'ignored': 75, 'measured': 0, 'filtered_out': 0}
        worker = section(lines, 'pnpm smoke:native')
        passes = [line.split(' | ', 1)[0] for line in worker if line.startswith('PASS ')]
        assert len(passes) == len(set(passes)) == 82 and not any(line.startswith('FAIL ') for line in worker)
        item['worker_passes'] = passes
        frontend = section(lines, 'pnpm test')
        item['frontend_tap'] = [line for line in frontend if re.match(r'^# (tests|pass|fail|skipped|todo) ', line)]
        assert '# tests 167' in item['frontend_tap'] and '# pass 167' in item['frontend_tap'] and '# fail 0' in item['frontend_tap'] and '# skipped 0' in item['frontend_tap']
        item['audio_alias_frontend_cases'] = [line for line in frontend if re.match(r'^ok \d+ - ', line) and any(t in line for t in ('declared aliases', 'N01 declaration', 'B current character', 'B real tempname'))]
        assert len(item['audio_alias_frontend_cases']) == 4
    if job['name'] == 'gates-ubuntu':
        assert item['rust_tests'] == {'passed': 6894, 'failed': 0, 'ignored': 77, 'measured': 0, 'filtered_out': 0}
        conformance = section(lines, 'cargo run -p conformance')
        item['conformance_cases'] = [line.strip() for line in conformance if re.match(r'^\s*OK\s', line)]
        assert len(item['conformance_cases']) == 19 and 'ALL PASS' in conformance
        assert 'ALL CROSS-TARGET CHECKS PASSED' in section(lines, 'bash scripts/cross-target-hash.sh')
        item['cross_target_result'] = 'ALL CROSS-TARGET CHECKS PASSED'
        verify = section(lines, 'node --test scripts/test_original_content_order.mjs scripts/test_original_resources.mjs')
        item['original_verify_tap'] = [line for line in verify if re.match(r'^# (tests|pass|fail|skipped) ', line)]
        assert '# tests 2' in item['original_verify_tap'] and '# fail 0' in item['original_verify_tap']
        at = verify.index('original content verified:')
        item['original_verified_counts'] = verify[at:at + 4]
    if job['name'] in ('native-web', 'gates-ubuntu') or job['name'].startswith('desktop-'):
        assert 'git diff --exit-code -- content/original' in lines and 'original content resources verified:' in lines
        item['resource_preparation_logged'] = {'resources_sync': 'original content resources synced:' in lines, 'resources_verify': True, 'committed_manifest_diff_guard': True}
    if job['name'].startswith('perf-'):
        metrics = []
        for line in lines:
            match = re.search(r'✓\s+(start-hash-p50|distance-query-p95|snapshot-restore|observer-projection)\s+(\d+) / (\d+) us', line)
            if match:
                metrics.append({'name': match[1], 'actual_us': int(match[2]), 'budget_us': int(match[3])})
        assert len(metrics) == 4 and all(m['actual_us'] <= m['budget_us'] for m in metrics) and 'result=PASS' in lines
        item['tier'] = 'server-baseline'
        item['metrics'] = metrics
    summary['jobs'].append(item)
assert len(summary['jobs']) == 10
assert sum(len(j['skipped_step_records']) for j in summary['jobs']) == 2
assert all(s['step']['name'] == 'Install Linux Tauri dependencies' for j in summary['jobs'] for s in j['skipped_step_records'])
expected = {12998118328: (32802940, 'sha256:add7868181f107bc5ff3c2d80e05df7e78816d8b876de54c4b3203fadf35161f'), 12997584073: (266340325, 'sha256:4ac4d2e21fd712f1f73aee7887bfb3c7761c9325767fda3323d6b4b7c5a3bf06'), 12997069742: (26272381, 'sha256:4b3e9a8044bd475ed7a3766c9a78e17cb0846d4a0325939b90472d944fde8c4a')}
for artifact in summary['artifacts']['artifacts']:
    assert artifact['id'] in expected and (artifact['size_in_bytes'], artifact['digest']) == expected[artifact['id']]
    assert artifact['workflow_run']['id'] == RUN and artifact['workflow_run']['head_sha'] == SHA and not artifact['expired']
assert len(summary['artifacts']['artifacts']) == 3
summary['raw_log_bytes'] = sum(j['log_bytes'] for j in summary['jobs'])
summary['log_warning_lines'] = sum(j['warning_line_count'] for j in summary['jobs'])
summary['step_api_null_conclusion_records'] = sum(len(j['unresolved_api_step_records']) for j in summary['jobs'])
with (OUT / 'verified-final-summary.json').open('x') as f:
    json.dump(summary, f, indent=2)
    f.write('\n')
files = [{'path': str(p.relative_to(ROOT)), 'bytes': p.stat().st_size, 'sha256': hashlib.sha256(p.read_bytes()).hexdigest()} for p in sorted(ROOT.rglob('*')) if p.is_file()]
with (OUT / 'copy-manifest.json').open('x') as f:
    json.dump({'source_root': str(ROOT), 'manifest_excludes_itself': True, 'files': files, 'file_count_without_manifest': len(files)}, f, indent=2)
    f.write('\n')
print(json.dumps({'jobs': 10, 'successful_jobs': 10, 'artifact_count': 3, 'raw_log_bytes': summary['raw_log_bytes'], 'warning_lines': summary['log_warning_lines'], 'null_step_records_preserved': summary['step_api_null_conclusion_records'], 'old_manifest_files_unchanged': len(old_manifest['files']), 'copy_set_files_including_manifest': len(files) + 1, 'checker': 'passed'}, indent=2))
