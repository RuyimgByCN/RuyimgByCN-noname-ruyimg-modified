import datetime
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import time

ROOT = Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
OUT = ROOT / 'docs/operations/evidence/2026-10-30-native-ci-preparation/first-publication/execution'
SHA = '9dbc65530feb695688d3160dc38b6cf64ad39ecd'
TREE = '445e25cd7461da580e4bcd12c11a429420cb257f'
MAIN = '47347c6608d651660ae9459e59eb529b3cef277f'
URL = 'https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified.git'
ENV = {k: v for k, v in os.environ.items() if not k.startswith('GIT_TRACE') and k not in ('GH_DEBUG', 'GIT_CURL_VERBOSE')}
ENV.update(GIT_TERMINAL_PROMPT='0', GIT_OPTIONAL_LOCKS='0', GIT_NO_LAZY_FETCH='1')
GIT = ['git', '-c', 'credential.helper=', '-c', 'credential.helper=!gh auth git-credential', '-c', 'pack.threads=2']
started = time.monotonic()
result = {'started_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'accepted_commit': SHA, 'accepted_tree': TREE, 'push_attempts': 0}

def save():
    result['elapsed_seconds'] = time.monotonic() - started
    (OUT / 'result.json').write_text(json.dumps(result, indent=2) + '\n')

def run(argv, name, timeout=120):
    try:
        p = subprocess.run(argv, cwd=ROOT, env=ENV, capture_output=True, timeout=timeout)
        (OUT / (name + '.stdout')).write_bytes(p.stdout)
        (OUT / (name + '.stderr')).write_bytes(p.stderr)
        result[name + '_exit'] = p.returncode
        save()
        if p.returncode:
            raise RuntimeError(f'{name}: exit {p.returncode}; see saved stderr')
        return p.stdout
    except subprocess.TimeoutExpired as exc:
        (OUT / (name + '.stdout')).write_bytes(exc.stdout or b'')
        (OUT / (name + '.stderr')).write_bytes(exc.stderr or b'')
        result[name + '_exit'] = 'timeout'
        save()
        raise

def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, env=ENV)

def remote(name):
    raw = run(GIT + ['ls-remote', '--heads', URL, 'refs/heads/native-convergence', 'refs/heads/main'], name)
    refs = {line.split()[1]: line.split()[0] for line in raw.decode().splitlines()}
    result[name + '_refs'] = refs
    save()
    return refs

try:
    result['local_head_before'] = git('rev-parse', 'HEAD').decode().strip()
    result['local_main_before'] = git('rev-parse', 'refs/heads/main').decode().strip()
    result['index_sha256_before'] = hashlib.sha256((ROOT / '.git/index').read_bytes()).hexdigest()
    assert git('rev-parse', SHA + '^{tree}').decode().strip() == TREE
    assert not git('diff', '--cached', '--name-only')
    admission = json.loads(git('show', SHA + ':docs/operations/evidence/2026-10-30-native-ci-preparation/parent-corrected-ci-admission.json'))
    hashes = {p: hashlib.sha256(git('show', SHA + ':' + p)).hexdigest() for p in admission['reviewed_paths_exact']}
    assert len(hashes) == 4 and hashes == admission['reviewed_paths_exact']
    result['committed_ci_sha256'] = hashes
    paths = git('ls-tree', '-rz', '--name-only', SHA).decode().split('\0')[:-1]
    result['tree_counts'] = {'tracked': len(paths), 'dist': sum(p.startswith('apps/web-client/dist/') for p in paths), 'evidence': sum(p.startswith('docs/operations/evidence/') for p in paths)}
    assert result['tree_counts'] == {'tracked': 41879, 'dist': 37, 'evidence': 9207}
    assert 'docs/operations/evidence/2026-10-30-native-ci-preparation/parent-user-publication-decision25.json' not in paths
    assert 'docs/operations/evidence/2026-10-30-native-ci-preparation/parent-final-publication-scope.json' not in paths
    before = remote('remote_before')
    assert before.get('refs/heads/native-convergence') in (None, SHA), 'Unexpected native-convergence ref; stopped without push'
    assert before.get('refs/heads/main') == MAIN, 'Unexpected remote main; stopped without push'
    result['preflight_passed'] = True
    result['push_attempts'] = 1
    save()
    command = ['python3', '-B', '/tmp/noname-run-serialized.py'] + GIT + ['push', '--progress', URL, SHA + ':refs/heads/native-convergence']
    result['push_command'] = command
    with (OUT / 'push.stdout').open('xb') as stdout, (OUT / 'push.stderr').open('xb') as stderr:
        proc = subprocess.Popen(command, cwd=ROOT, env=ENV, stdout=stdout, stderr=stderr, start_new_session=True)
        result['push_pid'] = proc.pid
        save()
        checkpoint = False
        while proc.poll() is None:
            elapsed = time.monotonic() - started
            if elapsed >= 2700 and not checkpoint:
                checkpoint = True
                result['checkpoint_45min'] = {'elapsed_seconds': elapsed, 'push_running': True}
                save()
            if elapsed >= 3300:
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
    save()
    after = remote('remote_after')
    if result['push_exit'] != 0 or result.get('push_timed_out'):
        result['stopped_reason'] = 'Single push failed or timed out; remote state recorded; no retry or CLI switch'
    else:
        assert after.get('refs/heads/native-convergence') == SHA
        assert after.get('refs/heads/main') == MAIN
        result['remote_verified'] = True
        query = 'repos/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs?branch=native-convergence&head_sha=' + SHA + '&per_page=100'
        data = json.loads(run(['gh', 'api', query], 'actions_runs'))
        result['actions_total_count'] = data.get('total_count')
        result['actions_runs'] = [{k: r.get(k) for k in ('id', 'html_url', 'head_sha', 'head_branch', 'name', 'status', 'conclusion', 'run_number', 'run_attempt', 'jobs_url')} for r in data.get('workflow_runs', [])]
        assert all(r['head_sha'] == SHA and r['head_branch'] == 'native-convergence' for r in result['actions_runs'])
except Exception as exc:
    result['error'] = type(exc).__name__ + ': ' + str(exc)
finally:
    result['local_head_after'] = git('rev-parse', 'HEAD').decode().strip()
    result['local_main_after'] = git('rev-parse', 'refs/heads/main').decode().strip()
    result['index_sha256_after'] = hashlib.sha256((ROOT / '.git/index').read_bytes()).hexdigest()
    result['no_staged_files'] = not bool(git('diff', '--cached', '--name-only'))
    result['finished_utc'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
    save()
