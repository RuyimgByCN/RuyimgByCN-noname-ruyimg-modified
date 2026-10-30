import datetime
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import time

OUT = Path(__file__).resolve().parent
REPO = 'RuyimgByCN/RuyimgByCN-noname-ruyimg-modified'
SHA = 'a1f7250815eee803d931301ab46207719cb9a15b'
RUN = 41100475767
BASE = 'repos/' + REPO
ENV = {k: v for k, v in os.environ.items() if not k.startswith('GIT_TRACE') and k not in ('GH_DEBUG', 'GIT_CURL_VERBOSE')}
ENV.update(GODEBUG='http2client=0')
STARTED = time.monotonic()
SCOPE = json.loads((OUT.parent / 'push-snapshot/scope.json').read_text())
SCOPE_TIME = datetime.datetime.fromisoformat(SCOPE['created_utc']).timestamp()
# Reserve five minutes for pre-guard inspection, and stop early enough to harvest/report by 55 minutes.
CHECKPOINT_AT = SCOPE_TIME + 40 * 60
WATCH_STOP_AT = SCOPE_TIME + 44 * 60
HARD_STOP_AT = SCOPE_TIME + 48 * 60
STATE = {'run_id': RUN, 'head_sha': SHA, 'run_url': 'https://github.com/' + REPO + '/actions/runs/' + str(RUN), 'started_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'commands': [], 'cleanup': 'no child running', 'deadline_basis': 'Conservative five-minute allowance before scope creation; checkpoint at scope+40min, watch stop at scope+44min, harvest stop scope+48min', 'checkpoint_at_utc': datetime.datetime.fromtimestamp(CHECKPOINT_AT, datetime.timezone.utc).isoformat(), 'watch_stop_at_utc': datetime.datetime.fromtimestamp(WATCH_STOP_AT, datetime.timezone.utc).isoformat()}

def save():
    STATE['elapsed_seconds'] = time.monotonic() - STARTED
    (OUT / 'observation-state.json').write_text(json.dumps(STATE, indent=2) + '\n')

def command(argv, name, env=ENV, timeout=60):
    entry = {'command': argv, 'name': name, 'http1': env.get('GODEBUG') == 'http2client=0'}
    STATE['commands'].append(entry)
    save()
    with (OUT / (name + '.stdout')).open('xb') as stdout, (OUT / (name + '.stderr')).open('xb') as stderr:
        try:
            p = subprocess.run(argv, cwd=OUT, env=env, stdout=stdout, stderr=stderr, timeout=min(timeout, max(1, HARD_STOP_AT-time.time())))
            entry['exit'] = p.returncode
        except subprocess.TimeoutExpired:
            entry['exit'] = 'timeout'
    save()
    return entry

def transport_error(name):
    text = (OUT / (name + '.stderr')).read_bytes().lower()
    return any(t in text for t in (b'eof', b'http2', b'http/2'))

def api(endpoint, name, raw=False):
    argv = ['gh', 'api', '--allow-escape-sequences', endpoint]
    entry = command(argv, name)
    if entry['exit'] != 0 and transport_error(name):
        name += '-http1'
        entry = command(argv, name, env={**ENV, 'GODEBUG': 'http2client=0'})
    if entry['exit'] != 0:
        raise RuntimeError(name + ': API failure; original logs preserved')
    if raw:
        return name + '.stdout'
    return json.loads((OUT / (name + '.stdout')).read_bytes())

def verify_run(run):
    assert run['id'] == RUN and run['head_sha'] == SHA and run['head_branch'] == 'native-convergence' and run['event'] == 'push'


def watch(name, env):
    argv = ['gh', 'run', 'watch', str(RUN), '--repo', REPO, '--exit-status', '--interval', '30', '--compact']
    remaining = max(1, WATCH_STOP_AT - time.time())
    entry = {'command': argv, 'name': name, 'http1': env.get('GODEBUG') == 'http2client=0'}
    STATE['commands'].append(entry)
    with (OUT / (name + '.stdout')).open('xb') as stdout, (OUT / (name + '.stderr')).open('xb') as stderr:
        proc = subprocess.Popen(argv, cwd=OUT, env=env, stdout=stdout, stderr=stderr, stdin=subprocess.DEVNULL, start_new_session=True)
        STATE['watch_pid'] = proc.pid
        STATE['cleanup'] = 'watch process running'
        save()
        try:
            while proc.poll() is None:
                if time.time() >= CHECKPOINT_AT and 'checkpoint_45min' not in STATE:
                    STATE['checkpoint_45min'] = {'at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat()}
                    save()
                    try:
                        checkpoint_run = api(BASE + '/actions/runs/' + str(RUN), 'checkpoint-run')
                        verify_run(checkpoint_run)
                        checkpoint_jobs = api(BASE + '/actions/runs/' + str(RUN) + '/jobs?per_page=100', 'checkpoint-jobs')
                        api(BASE + '/actions/runs/' + str(RUN) + '/artifacts?per_page=100', 'checkpoint-artifacts')
                        STATE['checkpoint_45min']['pending_jobs'] = [{'id': j['id'], 'name': j['name'], 'status': j['status']} for j in checkpoint_jobs['jobs'] if j['status'] != 'completed']
                        files = [{'path': str(p.relative_to(OUT)), 'bytes': p.stat().st_size, 'sha256': hashlib.sha256(p.read_bytes()).hexdigest()} for p in sorted(OUT.rglob('*')) if p.is_file()]
                        (OUT / 'checkpoint-copy-manifest.json').write_text(json.dumps(files, indent=2) + '\n')
                    except Exception as exc:
                        STATE['checkpoint_45min']['error'] = str(exc)
                    save()
                remaining = WATCH_STOP_AT - time.time()
                if remaining <= 0:
                    STATE['watch_deadline_reached'] = True
                    os.killpg(proc.pid, signal.SIGTERM)
                    try:
                        proc.wait(timeout=10)
                    except subprocess.TimeoutExpired:
                        os.killpg(proc.pid, signal.SIGKILL)
                        proc.wait()
                    break
                try:
                    proc.wait(timeout=min(30, remaining))
                except subprocess.TimeoutExpired:
                    pass
        finally:
            if proc.poll() is None:
                os.killpg(proc.pid, signal.SIGTERM)
                try:
                    proc.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    os.killpg(proc.pid, signal.SIGKILL)
                    proc.wait()
            entry['exit'] = proc.returncode
            STATE['cleanup'] = 'watch process exited/reaped; no local compiled products or background child remaining'
            save()
    return entry

try:
    run = api(BASE + '/actions/runs/' + str(RUN), 'run-initial')
    verify_run(run)
    initial_jobs = api(BASE + '/actions/runs/' + str(RUN) + '/jobs?per_page=100', 'jobs-initial')
    STATE['initial_jobs'] = [{'id': j['id'], 'name': j['name'], 'status': j['status'], 'conclusion': j['conclusion']} for j in initial_jobs['jobs']]
    save()
    if run['status'] != 'completed':
        watched = watch('watch', ENV)
    final_run = api(BASE + '/actions/runs/' + str(RUN), 'run-final')
    verify_run(final_run)
    jobs = api(BASE + '/actions/runs/' + str(RUN) + '/jobs?per_page=100', 'jobs-final')
    artifacts = api(BASE + '/actions/runs/' + str(RUN) + '/artifacts?per_page=100', 'artifacts-final')
    STATE['run_status'] = final_run['status']
    STATE['run_conclusion'] = final_run['conclusion']
    STATE['jobs'] = []
    for job in jobs['jobs']:
        item = {k: job.get(k) for k in ('id', 'name', 'html_url', 'status', 'conclusion', 'started_at', 'completed_at', 'runner_name', 'runner_group_name', 'labels')}
        item['failed_steps'] = [s for s in job.get('steps', []) if s.get('conclusion') == 'failure']
        item['skipped_steps'] = [s for s in job.get('steps', []) if s.get('conclusion') == 'skipped']
        item['steps'] = job.get('steps', [])
        STATE['jobs'].append(item)
        if job['status'] == 'completed' and time.time() < HARD_STOP_AT:
            try:
                item['raw_log'] = api(BASE + '/actions/jobs/' + str(job['id']) + '/logs', 'job-' + str(job['id']) + '-raw-log', raw=True)
                logbytes = (OUT / item['raw_log']).read_bytes()
                item['log_bytes'] = len(logbytes)
                item['log_crlf_count'] = logbytes.count(b'\r\n')
            except Exception as exc:
                item['log_error'] = type(exc).__name__ + ': ' + str(exc)
        save()
    STATE['artifacts'] = artifacts
    STATE['pending_jobs'] = [{'id': j['id'], 'name': j['name'], 'status': j['status']} for j in STATE['jobs'] if j['status'] != 'completed']
    STATE['job_count'] = len(STATE['jobs'])
    STATE['completed'] = final_run['status'] == 'completed' and len(STATE['jobs']) == 10 and not STATE['pending_jobs']
    (OUT / 'job-analyzer.json').write_text(json.dumps({'run_id': RUN, 'head_sha': SHA, 'status': final_run['status'], 'conclusion': final_run['conclusion'], 'jobs': STATE['jobs'], 'artifacts': artifacts}, indent=2) + '\n')
except Exception as exc:
    STATE['error'] = type(exc).__name__ + ': ' + str(exc)
finally:
    STATE['finished_utc'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
    save()
