import datetime
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import time

ROOT = Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
OUT = Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-ci-d07b35f9-own-br49nqpd/push-stage')
SCOPE_PATH = OUT / 'scope.json'
SCOPE = json.loads(SCOPE_PATH.read_text())
SHA = 'd07b35f94a42b282222dda4ab7bbc5c39ab8fddb'
TREE = '7cfa8e085e94e760999e14b90861857f8c9962bb'
OLD = '5fc0f77403d90fa4555156bf01722450cbe7bf3b'
MAIN = '47347c6608d651660ae9459e59eb529b3cef277f'
URL = 'https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified.git'
REPO = 'RuyimgByCN/RuyimgByCN-noname-ruyimg-modified'
ENV = {k: v for k, v in os.environ.items() if not k.startswith('GIT_TRACE') and k not in ('GH_DEBUG', 'GIT_CURL_VERBOSE')}
ENV.update(GIT_TERMINAL_PROMPT='0', GIT_OPTIONAL_LOCKS='0', GIT_NO_LAZY_FETCH='1', CARGO_INCREMENTAL='0', CARGO_BUILD_JOBS='2')
GIT = ['git', '--no-optional-locks', '-c', 'core.hooksPath=' + SCOPE['scoped_hooks_path'], '-c', 'credential.helper=', '-c', 'credential.helper=!gh auth git-credential', '-c', 'pack.threads=2']
SCOPE_TIME = datetime.datetime.fromisoformat(SCOPE['created_utc']).timestamp()
started = time.monotonic()
result = {'started_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'accepted_commit': SHA, 'accepted_tree': TREE, 'expected_old_remote': OLD, 'push_attempts': 0}

def save():
    result['elapsed_seconds'] = time.monotonic() - started
    (OUT / 'result.json').write_text(json.dumps(result, indent=2) + '\n')

def run(argv, name, timeout=60, env=ENV):
    try:
        p = subprocess.run(argv, cwd=ROOT, env=env, capture_output=True, timeout=timeout)
        (OUT / (name + '.stdout')).write_bytes(p.stdout)
        (OUT / (name + '.stderr')).write_bytes(p.stderr)
        result[name + '_exit'] = p.returncode
        save()
        return p
    except subprocess.TimeoutExpired as exc:
        (OUT / (name + '.stdout')).write_bytes(exc.stdout or b'')
        (OUT / (name + '.stderr')).write_bytes(exc.stderr or b'')
        result[name + '_exit'] = 'timeout'
        save()
        raise

def git(*args):
    return subprocess.check_output(['git', '--no-optional-locks', *args], cwd=ROOT, env=ENV)

def snapshot(stage):
    original = hashlib.sha256((ROOT / '.git/hooks/pre-push').read_bytes()).hexdigest()
    scoped = hashlib.sha256((Path(SCOPE['scoped_hooks_path']) / 'pre-push').read_bytes()).hexdigest()
    config = subprocess.run(['git', '--no-optional-locks', 'config', '--get-all', 'core.hooksPath'], cwd=ROOT, env=ENV, capture_output=True)
    state = {'head': git('rev-parse', 'HEAD').decode().strip(), 'tree': git('rev-parse', 'HEAD^{tree}').decode().strip(), 'index_sha256': hashlib.sha256((ROOT / '.git/index').read_bytes()).hexdigest(), 'tracked_diff': git('diff', '--name-only').decode(), 'staged_diff': git('diff', '--cached', '--name-only').decode(), 'original_hook_sha256': original, 'scoped_hook_sha256': scoped, 'core_hooksPath_exit': config.returncode, 'core_hooksPath_stdout': config.stdout.decode()}
    protected = json.loads((OUT / 'protected-admission.json').read_bytes())
    expected = {**protected['root_protected'], **protected['source6']}
    state['protected_sha256'] = {path: hashlib.sha256((ROOT / path).read_bytes()).hexdigest() for path in expected}
    state['local_branch'] = git('symbolic-ref', '--short', 'HEAD').decode().strip()
    assert state['protected_sha256'] == expected, 'Exact source6/root protected drift; no baseline refresh allowed'
    assert state['local_branch'] == 'main'
    result[stage] = state
    save()
    assert state['head'] == SHA and state['tree'] == TREE
    assert not state['tracked_diff'] and not state['staged_diff']
    assert original == SCOPE['original_hook_sha256'] == 'e349af1ec5ccc1742df7e7d91d387ed2ef086f7c864d8cf7372d0ef66dd49ca7'
    assert scoped == SCOPE['scoped_hook_sha256'] == 'c991082cb655afdf03af7998ca0fd5bdfb22bca876318d97aaf31de58468b308'
    assert config.returncode == 1 and not config.stdout and not config.stderr

def api(endpoint, name):
    p = run(['gh', 'api', endpoint], name)
    if p.returncode and (b'EOF' in p.stderr or b'http2' in p.stderr.lower() or b'HTTP/2' in p.stderr):
        p = run(['gh', 'api', endpoint], name + '-http1', env={**ENV, 'GODEBUG': 'http2client=0'})
    assert p.returncode == 0, name + ': API failed; original logs retained'
    return json.loads(p.stdout)

try:
    receipt = json.loads(Path('/tmp/noname-own-accepted-commit.json').read_text())
    assert receipt['head'] == SHA and receipt['tree'] == TREE and receipt['parent'] == OLD
    assert SCOPE['repository'] == URL and SCOPE['branch'] == 'refs/heads/native-convergence'
    assert SCOPE['accepted_commit'] == SHA and SCOPE['accepted_tree'] == TREE and SCOPE['expected_old_remote'] == OLD
    assert len(SCOPE['selftests']) == 8 and all(t['exit'] == t['expected'] for t in SCOPE['selftests'])
    snapshot('before')
    assert git('rev-parse', SHA + '^').decode().strip() == OLD
    subprocess.run(['git', '--no-optional-locks', 'merge-base', '--is-ancestor', OLD, SHA], cwd=ROOT, env=ENV, check=True)
    protected = json.loads((OUT / 'protected-admission.json').read_bytes())
    result['committed_source6_sha256'] = {path: hashlib.sha256(git('show', SHA + ':' + path)).hexdigest() for path in protected['source6']}
    assert len(result['committed_source6_sha256']) == 6 and result['committed_source6_sha256'] == protected['source6']
    changed = git('diff', '--name-only', '-z', OLD, SHA).decode().split('\0')[:-1]
    result['committed_increment_paths'] = changed
    assert len(changed) == 3618 and set(protected['source6']).issubset(changed)
    raw_tree = git('ls-tree', '-rz', SHA).decode().split('\0')[:-1]
    tree_entries = {}
    for row in raw_tree:
        meta, path = row.split('\t', 1)
        mode, kind, oid = meta.split()
        tree_entries[path] = {'path': path, 'mode': mode, 'type': kind, 'oid': oid}
    checked = subprocess.run(['git', '--no-optional-locks', 'cat-file', '--batch-check=%(objectname) %(objecttype) %(objectsize)'], input=('\n'.join(tree_entries[path]['oid'] for path in changed) + '\n').encode(), cwd=ROOT, env=ENV, capture_output=True, timeout=60, check=True)
    rows = []
    for path, line in zip(changed, checked.stdout.decode().splitlines()):
        oid, kind, size = line.split()
        assert kind == 'blob' and oid == tree_entries[path]['oid']
        rows.append({**tree_entries[path], 'bytes': int(size)})
    assert len(rows) == 3618 and sum(row['bytes'] for row in rows) == 204625141
    result['increment_blob_bytes'] = 204625141
    result['tracked_tree_paths'] = len(tree_entries)
    result['tracked_dist_paths'] = sum(path.startswith('apps/web-client/dist/') for path in tree_entries)
    result['new_compiled_binary_paths'] = [path for path in changed if path.endswith(('.wasm', '.exe', '.dmg', '.msi', '.dll', '.dylib', '.so'))]
    assert not result['new_compiled_binary_paths'] and '.pi-lens.json' not in tree_entries
    (OUT / 'accepted3618-tree-stat.json').write_text(json.dumps({'head': SHA, 'tree': TREE, 'entries': rows, 'total_bytes': 204625141, 'qualification': 'Fresh Git tree/mode/OID/size metadata; parent all3618byte/mode audit retained separately; source6 body SHA freshly verified'}, indent=2) + '\n')
    newly_reachable = git('rev-list', '--objects', SHA, '^' + OLD).decode().splitlines()
    objects = {line.partition(' ')[0]: line.partition(' ')[2] for line in newly_reachable}
    checked = subprocess.run(['git', '--no-optional-locks', 'cat-file', '--batch-check=%(objectname) %(objecttype) %(objectsize)'], input=('\n'.join(objects) + '\n').encode(), cwd=ROOT, env=ENV, capture_output=True, check=True)
    records = []
    for line in checked.stdout.decode().splitlines():
        oid, kind, size = line.split()
        records.append({'oid': oid, 'type': kind, 'bytes': int(size), 'path': objects[oid]})
    blobs = [r for r in records if r['type'] == 'blob']
    result['new_object_count'] = len(records)
    result['new_object_uncompressed_bytes_not_wire'] = sum(r['bytes'] for r in records)
    result['new_max_blob'] = max(blobs, key=lambda r: r['bytes'])
    assert sum(r['type'] == 'commit' for r in records) == 1
    assert result['new_max_blob']['bytes'] <= 100 * 1024 * 1024
    (OUT / 'new-objects-audit.json').write_text(json.dumps({'base': OLD, 'new': SHA, 'scope': 'newly reachable objects only; no history filtering or rewrite', 'objects': records}, indent=2) + '\n')
    pre = run(GIT + ['ls-remote', '--heads', URL, 'refs/heads/native-convergence', 'refs/heads/main'], 'remote-before')
    assert pre.returncode == 0
    refs = {line.split()[1]: line.split()[0] for line in pre.stdout.decode().splitlines()}
    result['remote_before_refs'] = refs
    assert refs == {'refs/heads/main': MAIN, 'refs/heads/native-convergence': OLD}, 'Unexpected remote refs; no push allowed'
    snapshot('immediately_before_push')
    result['preflight_passed'] = True
    result['push_attempts'] = 1
    command = ['python3', '-B', '/tmp/noname-run-serialized.py'] + GIT + ['push', '--progress', URL, SHA + ':refs/heads/native-convergence']
    result['push_command'] = command
    push_started = time.monotonic()
    with (OUT / 'push.stdout').open('xb') as stdout, (OUT / 'push.stderr').open('xb') as stderr:
        proc = subprocess.Popen(command, cwd=ROOT, env=ENV, stdout=stdout, stderr=stderr, start_new_session=True)
        result['push_pid'] = proc.pid
        save()
        while proc.poll() is None:
            elapsed = time.monotonic() - started
            if time.time() >= SCOPE_TIME + 14 * 60 and 'checkpoint_pre20min' not in result:
                result['checkpoint_pre20min'] = {'elapsed_seconds': elapsed, 'push_running': True}
                save()
            if time.time() >= SCOPE_TIME + 40 * 60:
                result['push_timed_out'] = True
                os.killpg(proc.pid, signal.SIGTERM)
                try:
                    proc.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    os.killpg(proc.pid, signal.SIGKILL)
                    proc.wait()
                break
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                pass
        result['push_exit'] = proc.returncode
        result['push_seconds_including_queue'] = time.monotonic() - push_started
    save()
    if result['push_exit'] != 0 or result.get('push_timed_out'):
        post = run(GIT + ['ls-remote', '--heads', URL, 'refs/heads/native-convergence', 'refs/heads/main'], 'remote-after-failure')
        if post.returncode == 0:
            result['remote_after_refs'] = {line.split()[1]: line.split()[0] for line in post.stdout.decode().splitlines()}
        result['stopped_reason'] = 'Single push failed/timed out; no retry or CLI switch'
    else:
        branch = api('repos/' + REPO + '/branches/native-convergence', 'branch-after')
        main = api('repos/' + REPO + '/branches/main', 'main-after')
        result['remote_after_refs'] = {'refs/heads/native-convergence': branch['commit']['sha'], 'refs/heads/main': main['commit']['sha']}
        assert result['remote_after_refs'] == {'refs/heads/native-convergence': SHA, 'refs/heads/main': MAIN}
        result['remote_verified'] = True
except Exception as exc:
    result['error'] = type(exc).__name__ + ': ' + str(exc)
finally:
    try:
        snapshot('after')
        assert result['before']['index_sha256'] == result['after']['index_sha256']
    except Exception as exc:
        result['preservation_error'] = type(exc).__name__ + ': ' + str(exc)
    result['finished_utc'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
    save()
    if result.get('push_exit') == 0 and result.get('remote_verified') and not result.get('error') and not result.get('preservation_error'):
        release = {'at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'lease': 'Read/push lease released after successful immutable source6/root/index snapshots; no Main writes by worker', 'head_sha': SHA, 'push_exit': 0, 'push_completed_utc': result['finished_utc'], 'remote_refs': result['remote_after_refs']}
        (OUT / 'lease-release.json').write_text(json.dumps(release, indent=2) + '\n')
