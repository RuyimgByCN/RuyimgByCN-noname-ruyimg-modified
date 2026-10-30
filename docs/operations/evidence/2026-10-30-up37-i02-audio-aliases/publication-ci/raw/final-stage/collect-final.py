import datetime
import json
import os
from pathlib import Path
import subprocess
import time

OUT = Path(__file__).resolve().parent
REPO = 'RuyimgByCN/RuyimgByCN-noname-ruyimg-modified'
BASE = 'repos/' + REPO
RUN = 41100475767
SHA = 'a1f7250815eee803d931301ab46207719cb9a15b'
ENV = {k: v for k, v in os.environ.items() if not k.startswith('GIT_TRACE') and k not in ('GH_DEBUG', 'GIT_CURL_VERBOSE')}
ENV.update(GODEBUG='http2client=0', GIT_TERMINAL_PROMPT='0')
START = time.monotonic()
STATE = {'started_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'run_id': RUN, 'head_sha': SHA, 'transport': 'command-local HTTP1', 'commands': [], 'jobs': [], 'missing': [], 'cleanup': 'no watch or background child'}


def save():
    STATE['elapsed_seconds'] = time.monotonic() - START
    (OUT / 'collection-state.json').write_text(json.dumps(STATE, indent=2) + '\n')


def request(endpoint, name, raw=False):
    for attempt in range(2):
        suffix = name if attempt == 0 else name + '-http1-retry'
        argv = ['gh', 'api', '--allow-escape-sequences', endpoint]
        item = {'name': suffix, 'command': argv, 'started_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'http1': True}
        STATE['commands'].append(item)
        save()
        with (OUT / (suffix + '.stdout')).open('xb') as stdout, (OUT / (suffix + '.stderr')).open('xb') as stderr:
            try:
                p = subprocess.run(argv, cwd=OUT, env=ENV, stdout=stdout, stderr=stderr, timeout=min(90, max(1, 1500 - (time.monotonic() - START))))
                item['exit'] = p.returncode
            except subprocess.TimeoutExpired:
                item['exit'] = 'timeout'
        item['finished_utc'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
        save()
        if item['exit'] == 0:
            path = suffix + '.stdout'
            return path if raw else json.loads((OUT / path).read_bytes())
        error = (OUT / (suffix + '.stderr')).read_bytes().lower()
        if attempt or not any(t in error for t in (b'502', b'eof', b'http2', b'http/2')):
            break
    raise RuntimeError(name + ': read failed after bounded attempt(s), original bytes retained')


def verify_run(run):
    assert run['id'] == RUN and run['head_sha'] == SHA and run['head_branch'] == 'native-convergence' and run['event'] == 'push' and run['run_attempt'] == 1
    assert run['status'] == 'completed' and run['conclusion'] == 'success'


try:
    run = request(BASE + '/actions/runs/' + str(RUN), 'run-final-fetch')
    verify_run(run)
    jobs = request(BASE + '/actions/runs/' + str(RUN) + '/jobs?per_page=100', 'jobs-final-fetch')
    artifacts = request(BASE + '/actions/runs/' + str(RUN) + '/artifacts?per_page=100', 'artifacts-final-fetch')
    assert jobs['total_count'] == len(jobs['jobs']) == 10
    assert artifacts['total_count'] == len(artifacts['artifacts']) == 3
    STATE['run'] = run
    STATE['artifacts'] = artifacts
    for listed in jobs['jobs']:
        ident = listed['id']
        job = {'listed': listed, 'id': ident, 'name': listed['name']}
        STATE['jobs'].append(job)
        assert listed['run_id'] == RUN and listed['head_sha'] == SHA and listed['status'] == 'completed' and listed['conclusion'] == 'success'
        try:
            detail = request(BASE + '/actions/jobs/' + str(ident), 'job-' + str(ident) + '-detail')
            assert detail['id'] == ident and detail['run_id'] == RUN and detail['head_sha'] == SHA and detail['conclusion'] == 'success'
            job['detail'] = detail
        except Exception as exc:
            job['detail_error'] = str(exc)
            STATE['missing'].append({'job_id': ident, 'kind': 'detail', 'error': str(exc)})
        try:
            job['raw_log'] = request(BASE + '/actions/jobs/' + str(ident) + '/logs', 'job-' + str(ident) + '-raw-log', raw=True)
            log = (OUT / job['raw_log']).read_bytes()
            job['log_bytes'] = len(log)
            job['log_crlf_count'] = log.count(b'\r\n')
        except Exception as exc:
            job['log_error'] = str(exc)
            STATE['missing'].append({'job_id': ident, 'kind': 'log', 'error': str(exc)})
        if time.monotonic() - START >= 1200 and 'checkpoint_20min' not in STATE:
            STATE['checkpoint_20min'] = {'at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'collected_jobs': len(STATE['jobs']), 'missing': STATE['missing']}
        save()
        if time.monotonic() - START >= 1500:
            STATE['stopped_for_report_deadline'] = True
            break
    STATE['collection_complete'] = len(STATE['jobs']) == 10 and not STATE['missing']
except Exception as exc:
    STATE['error'] = type(exc).__name__ + ': ' + str(exc)
finally:
    STATE['finished_utc'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
    save()
