"""Run the four unmodified gates once, preserving wrapper output and current identity."""
import datetime
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import time

from narrow_identity import ENV, OUT, ROOT, save, snapshot

GATES = [
    ('source-line-tests', ['python3', '-B', '-m', 'unittest', 'discover', '-s', 'scripts', '-p', 'test_check_source_lines.py']),
    ('source-lines', ['python3', '-B', 'scripts/check-source-lines.py']),
    ('strict-docrefs', ['python3', '-B', 'scripts/check_doc_line_refs.py', '--strict']),
    ('full-preflight', ['python3', '-B', 'scripts/preflight.py']),
]

def main():
    deadline = time.monotonic() + 1200
    records = []
    save('commands.json', records)
    for name, argv in GATES:
        assert shutil.disk_usage(ROOT).free >= 3 * 1024**3
        actual = ['python3', '-B', '/tmp/noname-run-serialized.py', *argv]
        record = {'name': name, 'argv': argv, 'wrapper_argv': actual, 'cwd': str(ROOT),
                  'env_overrides': {k: ENV[k] for k in ['GIT_OPTIONAL_LOCKS', 'GIT_NO_LAZY_FETCH', 'CARGO_INCREMENTAL', 'CARGO_BUILD_JOBS']},
                  'started_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
                  'stdout': name + '.stdout.raw.txt', 'stderr': name + '.stderr.raw.txt',
                  'free_disk_before_bytes': shutil.disk_usage(ROOT).free, 'status': 'running'}
        records.append(record)
        save('commands.json', records)
        start = time.monotonic()
        with (OUT / record['stdout']).open('x') as out, (OUT / record['stderr']).open('x') as err:
            process = subprocess.Popen(actual, cwd=ROOT, env=ENV, stdout=out, stderr=err, start_new_session=True)
            try:
                code = process.wait(timeout=max(1, deadline - time.monotonic()))
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGTERM)
                try:
                    code = process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    code = process.wait()
                record['deadline_exceeded'] = True
        wall = time.monotonic() - start
        wrapper = None
        for line in reversed((OUT / record['stdout']).read_text().splitlines()):
            try:
                candidate = json.loads(line)
            except ValueError:
                continue
            if isinstance(candidate, dict) and 'locked_seconds' in candidate:
                wrapper = candidate
                break
        record.update({'exit': code, 'wall_seconds': wall,
                       'locked_seconds': wrapper['locked_seconds'] if wrapper else None,
                       'queue_estimate_seconds': max(0, wall - wrapper['locked_seconds']) if wrapper else None,
                       'queue_estimate_includes_wrapper_startup': True,
                       'wrapper_result': wrapper, 'free_disk_after_bytes': shutil.disk_usage(ROOT).free,
                       'status': 'PASS' if code == 0 and wrapper and wrapper['exit'] == 0 else 'FAIL'})
        save('commands.json', records)
        print(json.dumps(record, ensure_ascii=False), flush=True)
        try:
            snapshot(name + '-identity.json')
        except Exception as error:
            save('identity-failure.json', {'after': name, 'error': str(error)})
            raise
        if record.get('deadline_exceeded'):
            break
    snapshot('narrow-exit.json')
    print('DONE', flush=True)

if __name__ == '__main__':
    main()
