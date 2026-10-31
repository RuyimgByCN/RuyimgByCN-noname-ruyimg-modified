import datetime
import gzip
import hashlib
import json
from pathlib import Path
import re
import zlib

ROOT = Path(__file__).resolve().parent
PUSH = ROOT / 'push-stage'
CI = ROOT / 'ci-stage'
SHA = 'd07b35f94a42b282222dda4ab7bbc5c39ab8fddb'
RUN = 41202703042
ANSI = re.compile(r'\x1b\[[0-?]*[ -/]*[@-~]')
RUST = re.compile(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;')


def read(path):
    return json.loads(path.read_bytes())


def section(lines, command):
    start = next(i for i, line in enumerate(lines) if line.startswith('##[group]Run ' + command))
    end = next((i for i in range(start + 1, len(lines)) if lines[i].startswith('##[group]Run ')), len(lines))
    return lines[start:end]


push = read(PUSH / 'result.json')
scope = read(PUSH / 'scope.json')
protected = read(PUSH / 'protected-admission.json')
receipt = read(PUSH / 'push.stdout')
lease = read(PUSH / 'lease-release.json')
parent = read(PUSH / 'stage-audit.raw.json')
stat = read(PUSH / 'accepted3618-tree-stat.json')
state = read(CI / 'observation-state.json')
run = read(CI / 'run-final.stdout')
assert push['accepted_commit'] == SHA and push['accepted_tree'] == '7cfa8e085e94e760999e14b90861857f8c9962bb'
assert push['push_attempts'] == 1 and push['push_exit'] == 0 and push['remote_verified']
assert not push.get('error') and not push.get('preservation_error') and push['before'] == push['after']
assert not push['after']['tracked_diff'] and not push['after']['staged_diff']
assert push['after']['original_hook_sha256'] == 'e349af1ec5ccc1742df7e7d91d387ed2ef086f7c864d8cf7372d0ef66dd49ca7'
assert push['after']['scoped_hook_sha256'] == scope['scoped_hook_sha256'] == 'c991082cb655afdf03af7998ca0fd5bdfb22bca876318d97aaf31de58468b308'
assert push['after']['core_hooksPath_exit'] == 1 and not push['after']['core_hooksPath_stdout']
assert len(scope['selftests']) == 8 and all(c['exit'] == c['expected'] for c in scope['selftests'])
assert push['committed_source6_sha256'] == protected['source6'] and len(protected['source6']) == 6
assert protected['composed_patch_sha256'] == 'c2ebaa41542c63f169b0c55367817a6d22cad22231401ed6af94859017016110'
assert push['after']['protected_sha256'] == {**protected['root_protected'], **protected['source6']}
assert len(protected['root_protected']) == 10
assert parent['status'] == 'PASS' and parent['raw_blob_mode_all_exact'] and parent['approved_pages_sha_exact']
assert len(stat['entries']) == len(push['committed_increment_paths']) == 3618
assert sum(item['bytes'] for item in stat['entries']) == stat['total_bytes'] == 204625141
assert set(item['path'] for item in stat['entries']) == set(push['committed_increment_paths'])
assert set(path for path in push['committed_increment_paths'] if not path.startswith('docs/')) == set(protected['source6'])
assert not push['new_compiled_binary_paths']
assert push['remote_before_refs'] == {'refs/heads/main': '47347c6608d651660ae9459e59eb529b3cef277f', 'refs/heads/native-convergence': '5fc0f77403d90fa4555156bf01722450cbe7bf3b'}
assert push['remote_after_refs'] == {'refs/heads/main': '47347c6608d651660ae9459e59eb529b3cef277f', 'refs/heads/native-convergence': SHA}
assert '--no-optional-locks' in push['push_command'] and SHA + ':refs/heads/native-convergence' in push['push_command']
assert run['id'] == RUN and run['head_sha'] == SHA and run['head_branch'] == 'native-convergence' and run['event'] == 'push' and run['run_attempt'] == 1
assert run['status'] == 'completed' and run['conclusion'] == 'failure'
assert state['completed'] and len(state['jobs']) == 10 and not state['pending_jobs'] and not state.get('error')
watches = [c for c in state['commands'] if c['name'].startswith('watch')]
assert len(watches) == 1 and watches[0]['exit'] == 1 and watches[0]['http1']
assert not (CI / 'watch.stderr').read_bytes()
assert all(c['exit'] == 0 for c in state['commands'] if not c['name'].startswith('watch'))
summary = {'verified_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'head_sha': SHA, 'tree': push['accepted_tree'], 'parent': scope['expected_old_remote'], 'run_id': RUN, 'run_url': run['html_url'], 'attempt': 1, 'run_status': run['status'], 'run_conclusion': run['conclusion'], 'run_created_at': run['created_at'], 'run_updated_at': run['updated_at'], 'observation_started_utc': state['started_utc'], 'observation_finished_utc': state['finished_utc'], 'lease_release': lease, 'push': {'exit': 0, 'wall_seconds_including_queue': push['push_seconds_including_queue'], 'locked_seconds': receipt['locked_seconds'], 'queue_and_startup_seconds': push['push_seconds_including_queue'] - receipt['locked_seconds'], 'pure_queue_seconds': None, 'exact_wire_bytes': None, 'new_objects': push['new_object_count'], 'new_max_blob': push['new_max_blob'], 'tracked_paths': push['tracked_tree_paths'], 'tracked_dist_paths': push['tracked_dist_paths']}, 'source6_sha256': protected['source6'], 'root_protected_sha256': protected['root_protected'], 'parent_full3618_audit': parent, 'fresh3618_audit_scope': 'Git tree mode/OID/type/size metadata and total bytes; body SHA freshly verified for source6 and root10, not a repeated 204MB evidence-body hash scan', 'watch_commands': watches, 'transport_failures': [], 'jobs': [], 'successful_jobs': [], 'failed_jobs': [], 'artifacts': state['artifacts'], 'private_originals': [], 'no_main_writes_or_reads_after_lease': True, 'checkpoint': state['checkpoint_15min']}
for job in state['jobs']:
    detail = job['detail']
    assert detail['id'] == job['id'] and detail['run_id'] == RUN and detail['head_sha'] == SHA and detail['conclusion'] == job['conclusion']
    assert not job.get('detail_error') and not job.get('log_error')
    by_number = {s['number']: s for s in job['listed_steps']}
    for step in job['steps']:
        old = by_number.get(step['number'])
        if old and old.get('conclusion') is not None and step.get('conclusion') is not None:
            assert old['conclusion'] == step['conclusion']
        if old is None or old.get('conclusion') is None or step.get('conclusion') is not None:
            by_number[step['number']] = step
    steps = [by_number[n] for n in sorted(by_number)]
    assert all(s['status'] == 'completed' and s.get('conclusion') is not None for s in steps)
    path = CI / job['raw_log']
    raw = gzip.decompress(path.read_bytes())
    private = ROOT / 'private-raw-downloads' / path.name.removesuffix('.gz')
    assert raw == private.read_bytes() and len(raw) == job['log_bytes']
    assert hashlib.sha256(raw).hexdigest() == job['log_sha256'] and f'{zlib.crc32(raw)&0xffffffff:08x}' == job['log_crc32']
    lines = [ANSI.sub('', line.split(' ', 1)[1]) for line in raw.decode(errors='replace').splitlines() if ' ' in line]
    item = {k: job.get(k) for k in ('id', 'name', 'html_url', 'status', 'conclusion', 'started_at', 'completed_at', 'raw_log', 'log_bytes', 'log_sha256', 'log_crc32', 'log_crlf_count')}
    item['steps'] = steps
    item['failed_steps'] = [s for s in steps if s['conclusion'] == 'failure']
    item['skipped_steps'] = [s for s in steps if s['conclusion'] == 'skipped']
    item['skip_reason_limit'] = 'Jobs API returns skip states, not textual causes. Windows product steps are observed skipped after checkout failure; Linux-dependency platform conditions remain separate.'
    item['error_annotations'] = [line for line in lines if '##[error]' in line]
    item['warning_lines'] = [line for line in lines if re.match(r'^(warning(?:\[[^]]+\])?:|##\[warning\])', line) or 'ExperimentalWarning:' in line]
    item['warning_line_count'] = len(item['warning_lines'])
    item['ignored_test_lines'] = [line for line in lines if re.match(r'^test .* \.\.\. ignored', line)]
    begin = datetime.datetime.fromisoformat(item['started_at'].replace('Z', '+00:00'))
    end = datetime.datetime.fromisoformat(item['completed_at'].replace('Z', '+00:00'))
    item['duration_seconds'] = (end - begin).total_seconds()
    tests = [tuple(map(int, m.groups())) for line in lines if (m := RUST.search(line))]
    if tests:
        item['rust_tests'] = dict(zip(('passed', 'failed', 'ignored', 'measured', 'filtered_out'), map(sum, zip(*tests))))
        assert item['rust_tests']['failed'] == 0
    if job['conclusion'] == 'success':
        assert not item['failed_steps'] and not item['error_annotations']
        summary['successful_jobs'].append(job['name'])
    else:
        summary['failed_jobs'].append(job['name'])
        assert job['name'] in ('build-windows-latest', 'desktop-windows-latest')
        assert len(item['failed_steps']) == 1 and item['failed_steps'][0]['name'] == 'Run actions/checkout@v4' and item['failed_steps'][0]['number'] == 2
        errors = [line for line in lines if 'Filename too long' in line]
        assert len(errors) == 5 and any('failed with exit code 128' in line for line in lines)
        label = 'windows-build' if job['name'] == 'build-windows-latest' else 'windows-desktop'
        assert gzip.decompress((CI / (label + '-early-log.stdout.gz')).read_bytes()) == raw
        item['checkout_failure'] = {'message': 'Filename too long', 'git_exit': 128, 'failed_step': 2, 'path_errors': errors, 'all_error_annotations': item['error_annotations'], 'early_and_final_raw_bytes_equal': True, 'product_tests_not_run': True}
    if job['name'] == 'native-web':
        assert item['rust_tests'] == {'passed': 1830, 'failed': 0, 'ignored': 75, 'measured': 0, 'filtered_out': 0}
        smoke = section(lines, 'pnpm smoke:native')
        passes = [line.split(' | ', 1)[0] for line in smoke if line.startswith('PASS ')]
        assert len(passes) == len(set(passes)) == 82 and not any(line.startswith('FAIL ') for line in smoke)
        item['worker_passes'] = passes
        frontend = section(lines, 'pnpm test')
        item['frontend_tap'] = [line for line in frontend if re.match(r'^# (tests|pass|fail|skipped|todo) ', line)]
        assert '# tests 179' in item['frontend_tap'] and '# pass 179' in item['frontend_tap'] and '# fail 0' in item['frontend_tap'] and '# skipped 0' in item['frontend_tap']
    if job['name'] == 'gates-ubuntu':
        assert item['rust_tests'] == {'passed': 6922, 'failed': 0, 'ignored': 77, 'measured': 0, 'filtered_out': 0}
        conform = section(lines, 'cargo run -p conformance')
        item['conformance_cases'] = [line.strip() for line in conform if re.match(r'^\s*OK\s', line)]
        assert len(item['conformance_cases']) == 19 and 'ALL PASS' in conform
        assert 'ALL CROSS-TARGET CHECKS PASSED' in section(lines, 'bash scripts/cross-target-hash.sh')
        item['cross_target_result'] = 'ALL CROSS-TARGET CHECKS PASSED'
        verify = section(lines, 'node --test scripts/test_original_content_order.mjs')
        item['resource_tap'] = [line for line in verify if re.match(r'^# (tests|pass|fail|skipped) ', line)]
        assert '# tests 17' in item['resource_tap'] and '# pass 17' in item['resource_tap'] and '# fail 0' in item['resource_tap']
    if job['name'] in ('gates-ubuntu', 'native-web'):
        own_tests = [line for line in lines if line.startswith('test nc10_public_own_plan::') and line.endswith(' ... ok')]
        assert len(own_tests) == 4 and 'test auto_step_uses_the_same_public_card_advice_and_submits_once ... ok' in lines
        item['public_own_plan_passed_tests'] = own_tests
        item['public_own_session_passed_test'] = 'auto_step_uses_the_same_public_card_advice_and_submits_once'
    if job['name'].startswith('build-') and job['conclusion'] == 'success':
        assert item['rust_tests'] == {'passed': 853, 'failed': 0, 'ignored': 0, 'measured': 0, 'filtered_out': 0}
    if job['name'].startswith('perf-'):
        metrics = []
        for line in lines:
            m = re.search(r'✓\s+(start-hash-p50|distance-query-p95|snapshot-restore|observer-projection)\s+(\d+) / (\d+) us', line)
            if m:
                metrics.append({'name': m[1], 'actual_us': int(m[2]), 'budget_us': int(m[3])})
        assert len(metrics) == 4 and all(m['actual_us'] <= m['budget_us'] for m in metrics) and 'result=PASS' in lines
        item['legacy_server_baseline_metrics'] = metrics
    summary['private_originals'].append({'path': str(private.relative_to(ROOT)), 'bytes': len(raw), 'sha256': job['log_sha256'], 'crc32': job['log_crc32'], 'copy_policy': 'Retained private; copy gzip evidence instead'})
    summary['jobs'].append(item)
assert len(summary['successful_jobs']) == 8 and sorted(summary['failed_jobs']) == ['build-windows-latest', 'desktop-windows-latest']
summary['step_count'] = sum(len(j['steps']) for j in summary['jobs'])
summary['skipped_step_count'] = sum(len(j['skipped_steps']) for j in summary['jobs'])
summary['failed_step_count'] = sum(len(j['failed_steps']) for j in summary['jobs'])
assert summary['step_count'] == 126 and summary['skipped_step_count'] == 14 and summary['failed_step_count'] == 2
summary['null_step_conclusions'] = 0
summary['raw_log_bytes'] = sum(j['log_bytes'] for j in summary['jobs'])
summary['warning_line_count'] = sum(j['warning_line_count'] for j in summary['jobs'])
artifacts = summary['artifacts']['artifacts']
assert len(artifacts) == summary['artifacts']['total_count'] == 2
assert sorted(a['name'] for a in artifacts) == ['installer-macos-latest', 'installer-ubuntu-latest']
for a in artifacts:
    assert a['workflow_run']['id'] == RUN and a['workflow_run']['head_sha'] == SHA and not a['expired']
    assert a['size_in_bytes'] > 0 and re.fullmatch(r'sha256:[0-9a-f]{64}', a['digest'])
assert read(CI / 'checkpoint-run.stdout')['head_sha'] == SHA
summary['checkpoint_manifest_limit'] = 'Checkpoint raw APIs remain immutable; checkpoint-copy-manifest.json is a dated live-stream index whose watch/state paths later changed. Final root copy-manifest and seal are current-copy authority.'
summary['cleanup'] = state['cleanup']
with (ROOT / 'verified-publication-ci-summary.json').open('x') as f:
    json.dump(summary, f, indent=2)
    f.write('\n')
files = []
for p in sorted(ROOT.rglob('*')):
    if p.is_file() and 'private-raw-downloads' not in p.relative_to(ROOT).parts:
        data = p.read_bytes()
        files.append({'path': str(p.relative_to(ROOT)), 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest(), 'crc32': f'{zlib.crc32(data)&0xffffffff:08x}'})
with (ROOT / 'copy-manifest.json').open('x') as f:
    json.dump({'source_root': str(ROOT), 'files': files, 'manifest_excludes_itself': True, 'private_originals_excluded': summary['private_originals'], 'extra_private_originals': ['private-raw-downloads/windows-build-early-log.stdout', 'private-raw-downloads/windows-desktop-early-log.stdout'], 'copy_instruction': 'Copy listed files plus manifest and later seal. Gzip carries exact original bytes/CRLF; do not stage private raw downloads.'}, f, indent=2)
    f.write('\n')
print(json.dumps({'checker': 'passed', 'run': RUN, 'successful_jobs': 8, 'failed_jobs': 2, 'steps': 126, 'skipped_steps': 14, 'null_step_conclusions': 0, 'raw_log_bytes': summary['raw_log_bytes'], 'warning_log_lines': summary['warning_line_count'], 'artifacts': 2, 'copy_files_including_manifest_before_seal': len(files) + 1}, indent=2))
