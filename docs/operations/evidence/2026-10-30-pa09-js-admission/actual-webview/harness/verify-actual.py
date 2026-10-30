from pathlib import Path
import collections, datetime, hashlib, json, os, subprocess
out = Path('/tmp/pa09-webview-5b7ad3ab').resolve()
e = out / 'evidence'
root = Path('/tmp/pa09-js-bound-4306bc00').resolve()
main = Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
r = json.loads((e / 'actual-webview-report.json').read_text())
calls, trace = r['rawCalls'], r['trace']
assert r['passed'] and r['jsBounded'] and len(r['assertions']) == 33
assert all(row['passed'] for row in r['assertions']) and 'AppleWebKit' in r['userAgent']
burst = [row for row in calls if row['phase'] == 'burst128-status']
assert len(burst) == 8 and all(row['ok'] and row['response']['ok'] for row in burst)
accepted = [row['request']['id'] for row in trace if row['phase'] == 'burst128-status' and row['type'] == 'transport-accepted']
assert [row['args']['request']['id'] for row in burst] == accepted == list(range(2, 10))
rejected = [row for row in trace if row['phase'] == 'burst128-status' and row['type'] == 'transport-rejected']
assert len(rejected) == 120 and all(row['tailUnchanged'] and 'not submitted' in row['error'] and '未知' not in row['error'] for row in rejected)
counts = next(row['counts'] for row in trace if row['type'] == 'pending-counts')
assert len(counts) == 128 and max(row['pendingCount'] for row in counts) == 8
resources = [row['response'] for row in calls if row['command'] == 'native_engine_open']
assert len(set(resources)) == 4
for resource in resources:
    closed = [row for row in calls if row['command'] == 'native_engine_close' and row['args']['resourceId'] == resource]
    assert len(closed) == 1 and closed[0]['ok']
    probes = [row for row in calls if row['command'] == 'native_engine_request' and row['args']['resourceId'] == resource and row['args']['request']['id'] == 900001]
    assert len(probes) == 1 and not probes[0]['ok']
dispose = next(row for row in trace if row['type'] == 'old-disposed')
late = next(row for row in trace if row['type'] in ('invoke-reply', 'invoke-error') and row['call'] == dispose['activeCall'])
assert late['index'] > dispose['index'] and late['type'] == 'invoke-reply' and late['response']['ok']
assert not any(row['type'] == 'client-delivery' and row['transport'] == dispose['transport'] for row in trace)
freeze = json.loads(Path('/tmp/pa09-js-bound-evidence-4306bc00/freeze-manifest.json').read_text())
for row in freeze['changed_files']:
    p = root / row['path']
    assert hashlib.sha256(p.read_bytes()).hexdigest() == row['sha256'] and not p.stat().st_mode & 0o222
assert not subprocess.check_output(['git', 'diff', '--cached', '--name-only'], cwd=root)
assert set(subprocess.check_output(['git', 'diff', '--name-only'], cwd=root).decode().splitlines()) == {row['path'] for row in freeze['changed_files']}
loaded = json.loads((e / 'bundle-module-source-proof.json').read_text())['loadedModulesBeforeTransforms']
for name in ('src/engine/client.ts', 'src/engine/desktopEngineTransport.ts'):
    key = str(out / 'product-web' / name)
    assert loaded[key] == hashlib.sha256((root / 'apps/web-client' / name).read_bytes()).hexdigest()
identity = json.loads((e / 'runtime-product-identity.json').read_text())
assert hashlib.sha256(Path(identity['binary']).read_bytes()).hexdigest() == identity['packaged_binary_sha256']
served = json.loads((e / 'http-served-assets.json').read_text())
for row in served:
    name = row['path'].split('?')[0].lstrip('/') or 'index.html'
    assert row['sha256'] == identity['bundle_files'][name]
cleanup = json.loads((e / 'cleanup.json').read_text())
assert cleanup['owned_app_waited'] and cleanup['server_closed']
for row in cleanup['owned_state_archived_then_removed']:
    assert not Path(row['path']).exists()
    assert hashlib.sha256(Path(row['archive']).read_bytes()).hexdigest() == row['sha256']
owner = json.loads((e / 'run-owner.json').read_text())
owned_pids = [owner['app_pid'], owner['driver_pid']]
for pid in owned_pids:
    try: os.kill(pid, 0)
    except ProcessLookupError: continue
    raise AssertionError('Owned PID still exists: ' + str(pid))
summary = {
    'verified_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'assertions': len(r['assertions']), 'jsBounded_this_fixture': True,
    'actual_burst_sent_ids': accepted, 'not_submitted': len(rejected), 'max_pending': 8,
    'unresolved_at_turn_boundary': r['burst']['unresolvedAtTurnBoundary'],
    'raw_calls_by_command': dict(collections.Counter(row['command'] for row in calls)),
    'owned_resource_ids': resources, 'close_counts': {str(value): sum(row['command'] == 'native_engine_close' and row['args']['resourceId'] == value for row in calls) for value in resources},
    'old_disposed_trace_index': dispose['index'], 'old_late_raw_callback_trace_index': late['index'],
    'old_late_raw_result': r['active']['oldLateRawReply'],
    'source3_unchanged_readonly': True, 'candidate_diff_only_frozen3': True,
    'candidate_staged_files': [], 'actual_client_transport_modules_match_frozen_source': True,
    'owned_pids_stopped': owned_pids, 'private_state_absent_and_archived': True,
    'main_index_same_as_initial': hashlib.sha256((main / '.git/index').read_bytes()).hexdigest() == (e / 'main-before-index.sha256').read_text().strip(),
    'main_head_now': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=main).decode().strip(),
    'main_movement': 'External commit/index movement retained separately; this worker did not write Main, restore or stage anything.',
    'qualification': r['qualification'],
}
(e / 'offline-assertions.json').write_text(json.dumps(summary, indent=2))
print(json.dumps(summary, indent=2))
