import datetime
import gzip
import hashlib
import json
from pathlib import Path
import re
import zlib

ROOT = Path(__file__).resolve().parent
CI = ROOT / 'ci-stage'
PUSH = ROOT / 'push-stage'
RUN = 41183155464
SHA = '5fc0f77403d90fa4555156bf01722450cbe7bf3b'
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
receipt = read(PUSH / 'push.stdout')
lease = read(PUSH / 'lease-release.json')
state = read(CI / 'observation-state.json')
run = read(CI / 'run-final.stdout')
assert push['accepted_commit'] == SHA and push['accepted_tree'] == '9ba4253837da66457669bd19c5120a9163b2a05b'
assert push['push_attempts'] == 1 and push['push_exit'] == 0 and push['remote_verified']
assert not push.get('error') and not push.get('preservation_error') and push['before'] == push['after']
assert not push['after']['tracked_diff'] and not push['after']['staged_diff']
assert push['after']['original_hook_sha256'] == 'e349af1ec5ccc1742df7e7d91d387ed2ef086f7c864d8cf7372d0ef66dd49ca7'
assert push['after']['scoped_hook_sha256'] == scope['scoped_hook_sha256'] == 'ba0dbbd4e1d9d34fe6534aa0a716460a90ca9c29c2bafba8c59cccc1f8aa3d96'
assert push['after']['core_hooksPath_exit'] == 1 and not push['after']['core_hooksPath_stdout']
assert len(scope['selftests']) == 8 and all(t['exit'] == t['expected'] for t in scope['selftests'])
plan = read(PUSH / 'stage-plan.raw.json')
audit = read(PUSH / 'accepted477-committed-audit.json')
assert hashlib.sha256((PUSH / 'stage-plan.raw.json').read_bytes()).hexdigest() == scope['plan_sha256'] == 'a24db6895489866b42470a665e6b6516db59f0645c6fc35239928922ad16c4ba'
expected = {entry['path']: entry for entry in plan['paths']}
assert len(expected) == len(audit['files']) == len(push['committed_increment_paths']) == 477
assert set(push['committed_increment_paths']) == set(expected)
for entry in audit['files']:
    assert entry['bytes'] == expected[entry['path']]['bytes'] and entry['sha256'] == expected[entry['path']]['sha256']
assert audit['head'] == SHA and audit['tree'] == push['accepted_tree'] and audit['all_committed_bytes_match_plan']
assert sum(entry['bytes'] for entry in audit['files']) == audit['total_bytes'] == 81550091
assert push['before']['accepted_file_sha256'] == push['after']['accepted_file_sha256'] == {path: entry['sha256'] for path, entry in expected.items()}
assert len(plan['source33']) == 33 and push['committed_source33_sha256'] == {path: expected[path]['sha256'] for path in plan['source33']}
assert not push['new_compiled_binary_paths']
assert push['remote_before_refs'] == {'refs/heads/main': '47347c6608d651660ae9459e59eb529b3cef277f', 'refs/heads/native-convergence': '4dd0b286b995fe414128b6c52eb83a4d0d85f6cd'}
assert push['remote_after_refs'] == {'refs/heads/main': '47347c6608d651660ae9459e59eb529b3cef277f', 'refs/heads/native-convergence': SHA}
assert run['id'] == RUN and run['head_sha'] == SHA and run['head_branch'] == 'native-convergence' and run['event'] == 'push' and run['run_attempt'] == 1
assert run['status'] == 'completed' and run['conclusion'] == 'success'
assert state['completed'] and len(state['jobs']) == 10 and not state['pending_jobs'] and not state.get('error')
watch_commands = [c for c in state['commands'] if c['name'].startswith('watch')]
assert len(watch_commands) == 1 and watch_commands[0]['exit'] == 0 and not watch_commands[0]['http1']
assert not (CI / 'watch.stderr').read_bytes()
assert all(c['exit'] == 0 for c in state['commands'] if not c['name'].startswith('watch'))
summary = {'verified_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'run_id': RUN, 'run_url': run['html_url'], 'head_sha': SHA, 'run_attempt': 1, 'run_status': run['status'], 'run_conclusion': run['conclusion'], 'run_created_at': run['created_at'], 'run_updated_at': run['updated_at'], 'observation_started_utc': state['started_utc'], 'observation_finished_utc': state['finished_utc'], 'watch_commands': watch_commands, 'lease_release': lease, 'push': {'exit': 0, 'wall_seconds_including_queue': push['push_seconds_including_queue'], 'locked_seconds': receipt['locked_seconds'], 'queue_plus_startup_seconds': push['push_seconds_including_queue'] - receipt['locked_seconds'], 'pure_queue_seconds': None, 'exact_wire_bytes': None, 'new_object_count': push['new_object_count'], 'new_max_blob': push['new_max_blob'], 'tracked_tree_paths': push['tracked_tree_paths'], 'tracked_dist_paths': push['tracked_dist_paths']}, 'jobs': [], 'artifacts': state['artifacts'], 'successful_jobs': [], 'failed_jobs': [], 'private_originals': [], 'no_main_reads_or_writes_after_lease': True}
for job in state['jobs']:
    assert job['status'] == 'completed' and not job.get('detail_error') and not job.get('log_error')
    detail = job['detail']
    assert detail['id'] == job['id'] and detail['run_id'] == RUN and detail['head_sha'] == SHA and detail['conclusion'] == job['conclusion']
    steps = {s['number']: s for s in job['listed_steps']}
    for step in job['steps']:
        old = steps.get(step['number'])
        if old and old.get('conclusion') is not None and step.get('conclusion') is not None:
            assert old['conclusion'] == step['conclusion']
        if old is None or old.get('conclusion') is None or step.get('conclusion') is not None:
            steps[step['number']] = step
    steps = [steps[n] for n in sorted(steps)]
    assert all(s['status'] == 'completed' and s.get('conclusion') is not None for s in steps)
    compressed = CI / job['raw_log']
    raw = gzip.decompress(compressed.read_bytes())
    original_name = compressed.name.removesuffix('.gz')
    private = ROOT / 'private-raw-downloads' / original_name
    assert private.read_bytes() == raw
    assert len(raw) == job['log_bytes'] and hashlib.sha256(raw).hexdigest() == job['log_sha256'] and f'{zlib.crc32(raw) & 0xffffffff:08x}' == job['log_crc32']
    summary['private_originals'].append({'path': str(private.relative_to(ROOT)), 'bytes': len(raw), 'sha256': job['log_sha256'], 'crc32': job['log_crc32'], 'copy_policy': 'Retained private; publish/copy the gzip artifact instead to preserve original line endings'})
    lines = [ANSI.sub('', line.split(' ', 1)[1]) for line in raw.decode(errors='replace').splitlines() if ' ' in line]
    item = {k: job[k] for k in ('id', 'name', 'html_url', 'status', 'conclusion', 'started_at', 'completed_at', 'raw_log', 'log_bytes', 'log_sha256', 'log_crc32', 'log_crlf_count')}
    item['steps'] = steps
    item['failed_steps'] = [s for s in steps if s['conclusion'] == 'failure']
    item['skipped_steps'] = [s for s in steps if s['conclusion'] == 'skipped']
    item['raw_error_annotations'] = [line for line in lines if '##[error]' in line]
    item['warning_lines'] = [line for line in lines if re.match(r'^(warning(?:\[[^]]+\])?:|##\[warning\])', line) or 'ExperimentalWarning:' in line]
    item['warning_line_count'] = len(item['warning_lines'])
    item['ignored_test_lines'] = [line for line in lines if re.match(r'^test .* \.\.\. ignored', line)]
    start = datetime.datetime.fromisoformat(item['started_at'].replace('Z', '+00:00'))
    end = datetime.datetime.fromisoformat(item['completed_at'].replace('Z', '+00:00'))
    item['duration_seconds'] = (end - start).total_seconds()
    matches = [tuple(map(int, m.groups())) for line in lines if (m := RUST.search(line))]
    if matches:
        item['rust_tests'] = dict(zip(('passed', 'failed', 'ignored', 'measured', 'filtered_out'), map(sum, zip(*matches))))
        assert item['rust_tests']['failed'] == 0
    assert job['conclusion'] == 'success'
    assert not item['failed_steps'] and not item['raw_error_annotations']
    summary['successful_jobs'].append(job['name'])
    if job['name'] == 'native-web':
        assert item['rust_tests'] == {'passed': 1825, 'failed': 0, 'ignored': 75, 'measured': 0, 'filtered_out': 0}
        worker = section(lines, 'pnpm smoke:native')
        passes = [line.split(' | ', 1)[0] for line in worker if line.startswith('PASS ')]
        assert len(passes) == len(set(passes)) and not any(line.startswith('FAIL ') for line in worker)
        item['worker_passes'] = passes
        frontend = section(lines, 'pnpm test')
        item['frontend_tap'] = [line for line in frontend if re.match(r'^# (tests|pass|fail|skipped|todo) ', line)]
        assert '# tests 179' in item['frontend_tap'] and '# pass 179' in item['frontend_tap'] and '# fail 0' in item['frontend_tap'] and '# skipped 0' in item['frontend_tap']
    if job['name'] == 'gates-ubuntu':
        assert item['rust_tests'] == {'passed': 6917, 'failed': 0, 'ignored': 77, 'measured': 0, 'filtered_out': 0}
        conform = section(lines, 'cargo run -p conformance')
        item['conformance_cases'] = [line.strip() for line in conform if re.match(r'^\s*OK\s', line)]
        assert 'ALL PASS' in conform
        assert 'ALL CROSS-TARGET CHECKS PASSED' in section(lines, 'bash scripts/cross-target-hash.sh')
        item['cross_target_result'] = 'ALL CROSS-TARGET CHECKS PASSED'
        verify = section(lines, 'node --test scripts/test_original_content_order.mjs')
        item['resource_tap'] = [line for line in verify if re.match(r'^# (tests|pass|fail|skipped) ', line)]
        assert '# tests 17' in item['resource_tap'] and '# pass 17' in item['resource_tap'] and '# fail 0' in item['resource_tap']
        at = verify.index('original content verified:')
        item['original_verify_counts'] = verify[at:at + 5]
    if job['name'] in ('native-web', 'gates-ubuntu', 'desktop-ubuntu-latest', 'desktop-macos-latest', 'desktop-windows-latest'):
        assert 'git diff --exit-code -- content/original' in lines and 'original content resources verified:' in lines
        assert any('audio-compat: 5 files' in line for line in lines)
        item['compatibility_preparation_result'] = 'resources sync/verify and committed content/original diff guard logged; 5 fixed audio-compat files consumed'
    if job['name'].startswith('build-'):
        assert item['rust_tests'] == {'passed': 853, 'failed': 0, 'ignored': 0, 'measured': 0, 'filtered_out': 0}
    if job['name'] in ('gates-ubuntu', 'native-web'):
        source_lines = [line for line in lines if line.startswith('test ') and line.endswith(' ... ok') and any(token in line for token in ('nc10_source_provenance::', 'engine::choice_source::', 'source_actual_native', 'source_typed_restore', 'legacy_choice_errors::', 'native_ws_e2e::source_skill::'))]
        item['nc27_passing_test_lines'] = source_lines
        assert sum('nc10_source_provenance::' in line for line in source_lines) == 5
        assert sum('engine::choice_source::' in line for line in source_lines) == 4
        if job['name'] == 'gates-ubuntu':
            assert any('legacy_choice_errors_match_4dd_oracle' in line for line in source_lines)
            assert any('source_typed_restore_both_entries_never_reaches_pending_choice_json' in line for line in source_lines)
            assert any('source_actual_ws_roles_reconnect_deadline_and_wire_compatibility' in line for line in source_lines)
        else:
            assert len(passes) == 82
    if job['name'].startswith('perf-'):
        metrics = []
        for line in lines:
            m = re.search(r'✓\s+(start-hash-p50|distance-query-p95|snapshot-restore|observer-projection)\s+(\d+) / (\d+) us', line)
            if m:
                metrics.append({'name': m[1], 'actual_us': int(m[2]), 'budget_us': int(m[3])})
        assert len(metrics) == 4 and all(m['actual_us'] <= m['budget_us'] for m in metrics) and 'result=PASS' in lines
        item['tier'] = 'server-baseline'
        item['metrics'] = metrics
    summary['jobs'].append(item)
assert len(summary['successful_jobs']) == 10 and not summary['failed_jobs']
assert sum(len(j['steps']) for j in summary['jobs']) == 130
assert sum(len(j['skipped_steps']) for j in summary['jobs']) == 2
artifacts = summary['artifacts']['artifacts']
assert len(artifacts) == summary['artifacts']['total_count'] == 3
assert sorted(a['name'] for a in artifacts) == ['installer-macos-latest', 'installer-ubuntu-latest', 'installer-windows-latest']
for artifact in artifacts:
    assert artifact['workflow_run']['id'] == RUN and artifact['workflow_run']['head_sha'] == SHA and not artifact['expired']
    assert artifact['size_in_bytes'] > 0 and re.fullmatch(r'sha256:[0-9a-f]{64}', artifact['digest'])
summary['raw_log_bytes'] = sum(j['log_bytes'] for j in summary['jobs'])
summary['warning_line_count'] = sum(j['warning_line_count'] for j in summary['jobs'])
summary['step_count'] = 130
summary['null_step_conclusions'] = 0
summary['skipped_step_count'] = 2
summary['cleanup'] = state['cleanup']
windows = next(j for j in summary['jobs'] if j['name'] == 'desktop-windows-latest')
for name in ('Run ./noname-next/.github/actions/prepare-original-content', 'Produce installer (tauri beforeBuildCommand builds the web client)', 'Upload installer artifact'):
    assert next(s for s in windows['steps'] if s['name'] == name)['conclusion'] == 'success'
assert all(step['name'] == 'Install Linux Tauri dependencies' for j in summary['jobs'] for step in j['skipped_steps'])
summary['accepted_path_count'] = 477
summary['accepted_path_bytes'] = 81550091
summary['source33_sha256'] = push['committed_source33_sha256']
assert read(CI / 'checkpoint-run.stdout')['head_sha'] == SHA
summary['checkpoint'] = state['checkpoint_15min']
for name in ('run', 'jobs', 'artifacts'):
    assert next(c for c in state['commands'] if c['name'] == 'checkpoint-' + name)['exit'] == 0
summary['checkpoint_manifest_limit'] = 'ci-stage/checkpoint-copy-manifest.json is the dated live-stream index at the 15-minute observation checkpoint; watch.stdout and observation-state.json continued changing afterward. Use the final root copy-manifest.json for current immutable copying; raw checkpoint API responses remain retained.'
with (ROOT / 'verified-publication-ci-summary.json').open('x') as f:
    json.dump(summary, f, indent=2)
    f.write('\n')
files = []
for p in sorted(ROOT.rglob('*')):
    if p.is_file() and 'private-raw-downloads' not in p.relative_to(ROOT).parts:
        data = p.read_bytes()
        files.append({'path': str(p.relative_to(ROOT)), 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest(), 'crc32': f'{zlib.crc32(data) & 0xffffffff:08x}'})
with (ROOT / 'copy-manifest.json').open('x') as f:
    json.dump({'source_root': str(ROOT), 'files': files, 'manifest_excludes_itself': True, 'private_originals_excluded': summary['private_originals'], 'copy_instruction': 'Copy listed files plus this manifest. Gzip artifacts preserve original log bytes/CRLF; do not stage private raw Windows originals.'}, f, indent=2)
    f.write('\n')
print(json.dumps({'checker': 'passed', 'run_id': RUN, 'successful_jobs': 10, 'failed_jobs': 0, 'steps': 130, 'null_step_conclusions': 0, 'skipped_steps': 2, 'raw_log_bytes': summary['raw_log_bytes'], 'warning_lines_including_repetitions': summary['warning_line_count'], 'artifacts': len(artifacts), 'copy_file_count_including_manifest': len(files) + 1}, indent=2))
