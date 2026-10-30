import datetime
import hashlib
import json
from pathlib import Path

OUT = Path(__file__).resolve().parent
SHA = 'a1f7250815eee803d931301ab46207719cb9a15b'
RUN = 41100475767
REPO = 'RuyimgByCN/RuyimgByCN-noname-ruyimg-modified'


def read(relative):
    return json.loads((OUT / relative).read_bytes())


def source(relative):
    p = OUT / relative
    return {'path': relative, 'bytes': p.stat().st_size, 'sha256': hashlib.sha256(p.read_bytes()).hexdigest(), 'captured_file_mtime_utc': datetime.datetime.fromtimestamp(p.stat().st_mtime, datetime.timezone.utc).isoformat()}


push = read('push-snapshot/result.json')
scope = read('push-snapshot/scope.json')
assert push['accepted_commit'] == SHA and push['accepted_tree'] == 'abb64b4d7aeabc3f1e2525887b88bca46b0e9133'
assert push['push_attempts'] == 1 and push['push_exit'] == 0 and push['remote_verified']
assert not push.get('error') and not push.get('preservation_error')
assert push['before'] == push['after']
assert push['after']['head'] == SHA and not push['after']['tracked_diff'] and not push['after']['staged_diff']
assert push['after']['core_hooksPath_exit'] == 1 and not push['after']['core_hooksPath_stdout']
assert push['after']['original_hook_sha256'] == 'e349af1ec5ccc1742df7e7d91d387ed2ef086f7c864d8cf7372d0ef66dd49ca7'
assert push['after']['scoped_hook_sha256'] == scope['scoped_hook_sha256']
assert len(scope['selftests']) == 8 and all(t['exit'] == t['expected'] for t in scope['selftests'])
assert len(push['committed_source9_sha256']) == 9 and len(push['committed_increment_paths']) == 245
assert push['excluded_local_products_not_committed'] == 39
assert push['remote_after_refs'] == {'refs/heads/native-convergence': SHA, 'refs/heads/main': '47347c6608d651660ae9459e59eb529b3cef277f'}
latest = 'http1-final-authorized-watch'
run = read(latest + '/run-final.stdout')
jobs = read(latest + '/jobs-initial.stdout')
artifacts = read('http1-observation/artifacts-final.stdout')
assert run['id'] == RUN and run['head_sha'] == SHA and run['head_branch'] == 'native-convergence' and run['event'] == 'push'
assert run['status'] == 'in_progress' and run['conclusion'] is None
assert jobs['total_count'] == len(jobs['jobs']) == 10
assert all(j['run_id'] == RUN and j['head_sha'] == SHA and j['status'] == 'in_progress' and j['conclusion'] is None and not j['steps'] for j in jobs['jobs'])
assert artifacts['total_count'] == 0 and artifacts['artifacts'] == []
attempts = []
for prefix in ('', 'http1-observation/', 'http1-final-authorized-watch/'):
    state = read(prefix + 'observation-state.json')
    watched = next(c for c in state['commands'] if c['name'] == 'watch')
    assert watched['exit'] == 1
    assert str(RUN) in watched['command'] and REPO in watched['command']
    assert b'HTTP 502' in (OUT / (prefix + 'watch.stderr')).read_bytes()
    assert 'exited/reaped' in state['cleanup']
    attempts.append({'state': source(prefix + 'observation-state.json'), 'watch': watched, 'stderr': source(prefix + 'watch.stderr'), 'cleanup': state['cleanup'], 'error': state.get('error')})
assert attempts[0]['watch']['http1'] is False and all(t['watch']['http1'] for t in attempts[1:])
final_failure = next(c for c in read(latest + '/observation-state.json')['commands'] if c['name'] == 'jobs-final')
assert final_failure['exit'] == 1 and b'HTTP 502' in (OUT / (latest + '/jobs-final.stderr')).read_bytes()
receipt = json.loads((OUT / 'push-snapshot/push.stdout').read_text())
assert receipt['exit'] == 0
pack_lines = [line for line in (OUT / 'push-snapshot/push.stderr').read_text().replace('\r', '\n').splitlines() if line.startswith('Writing objects: 100%') and line.endswith('done.')]
assert len(pack_lines) == 1 and '234/234' in pack_lines[0] and '7.17 MiB' in pack_lines[0]
checkpoint = {'checkpoint_created_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'checkpoint_reason': 'Immediate transport-blocked checkpoint after third watch HTTP502, as explicitly directed by parent; before normal45/55minute deadline', 'accepted_commit': SHA, 'accepted_tree': push['accepted_tree'], 'run_id': RUN, 'run_url': run['html_url'], 'run_status': run['status'], 'run_conclusion': run['conclusion'], 'run_snapshot': source(latest + '/run-final.stdout'), 'job_snapshot': source(latest + '/jobs-initial.stdout'), 'jobs': [{k: j.get(k) for k in ('id', 'name', 'html_url', 'status', 'conclusion', 'steps', 'head_sha', 'run_id')} for j in jobs['jobs']], 'artifact_snapshot': source('http1-observation/artifacts-final.stdout'), 'observed_artifacts': artifacts, 'watch_attempts': attempts, 'final_jobs_api_failure': {'command': final_failure, 'stderr': source(latest + '/jobs-final.stderr'), 'stdout': source(latest + '/jobs-final.stdout')}, 'push': {'exit': 0, 'completed_utc': push['finished_utc'], 'remote_refs': push['remote_after_refs'], 'pack_progress': pack_lines[0], 'push_seconds_including_queue': push['push_seconds_including_queue'], 'locked_seconds': receipt['locked_seconds'], 'wall_minus_locked_queue_and_startup_seconds': push['push_seconds_including_queue'] - receipt['locked_seconds'], 'pure_queue_seconds': None, 'exact_wire_bytes': None}, 'lease': 'Main evidence lease released after successful push snapshot; no later main writes or HEAD/index/source snapshots by worker', 'unobserved': ['All10final job conclusions', 'Per-step states/numeric exits/skips', 'CI test counts/ignored tests/warnings', 'Release budget metrics', 'Final artifact IDs/sizes/digests and Windows raw CRLF job logs'], 'cleanup': 'All3watch processes exited/reaped and all3observers finished; no local compilation or workflow cancellation', 'parent_next_step': 'Resume read-only observation of the same exact run/head when transport recovers; no push/rerun/dispatch required'}
with (OUT / 'transport-blocked-checkpoint.json').open('x') as f:
    json.dump(checkpoint, f, indent=2)
    f.write('\n')
files = [source(str(p.relative_to(OUT))) for p in sorted(OUT.rglob('*')) if p.is_file()]
manifest = {'source_root': str(OUT), 'manifest_excludes_itself': True, 'files': files, 'file_count_without_manifest': len(files), 'total_bytes_without_manifest': sum(f['bytes'] for f in files)}
with (OUT / 'copy-manifest.json').open('x') as f:
    json.dump(manifest, f, indent=2)
    f.write('\n')
print(json.dumps({'push_exit': 0, 'run_id': RUN, 'latest_run_status': run['status'], 'job_count': len(jobs['jobs']), 'job_states': ['in_progress'] * 10, 'observed_artifacts_at_last_successful_snapshot': 0, 'watch_transport_failures': 3, 'manifest_files_plus_manifest': len(files) + 1, 'self_check': 'passed'}, indent=2))
