import datetime
import json
import os
from pathlib import Path
import shutil
import subprocess
import time
from narrow_identity import ROOT, OUT, snapshot

TARGET = Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-f7-stage3.czy5e9gq/target')
ENV = {**os.environ, 'GIT_OPTIONAL_LOCKS': '0', 'GIT_NO_LAZY_FETCH': '1', 'CARGO_INCREMENTAL': '0', 'CARGO_BUILD_JOBS': '2', 'CARGO_TARGET_DIR': str(TARGET), 'PYTHONDONTWRITEBYTECODE': '1', 'PYTHONPATH': str(OUT), 'FINAL_GATES_OUT': str(OUT), 'PATH': str(OUT / 'bin') + os.pathsep + os.environ['PATH']}
COMMANDS = [
    ('01-source-line-tests', ['python3', '-B', '-m', 'unittest', 'discover', '-s', 'scripts', '-p', 'test_check_source_lines.py']),
    ('02-source-lines', ['python3', '-B', 'scripts/check-source-lines.py']),
    ('03-doc-line-refs', ['python3', '-B', 'scripts/check_doc_line_refs.py', '--strict']),
    ('04-preflight', ['python3', '-B', 'scripts/preflight.py']),
]

def save(name, value):
    (OUT / name).write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')

snapshot('narrow-entry.json')
lease = json.loads(Path('/tmp/noname-ci-stage3-lease.json').read_text())
assert lease['handoff_allowed'] and lease['owners'] == [] and lease['target'] == str(TARGET)
lsof = subprocess.run(['lsof', '-nP', '+D', str(TARGET)], capture_output=True, text=True)
save('exclusive-stage3-lease.json', {'time': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'prior_lease': lease, 'prior_lease_sha256': __import__('hashlib').sha256(Path('/tmp/noname-ci-stage3-lease.json').read_bytes()).hexdigest(), 'lsof_exit': lsof.returncode, 'lsof_stdout': lsof.stdout, 'lsof_stderr': lsof.stderr, 'exclusive_owner_pid': os.getpid(), 'purpose': 'main alias9 final four gates only', 'released': False, 'dependencies_retained_for': 'parent next explicitly authorized acceptance'})
assert lsof.returncode == 1 and not lsof.stdout and not lsof.stderr
incremental_before = {str(p): p.stat().st_mtime_ns for p in TARGET.rglob('incremental') if p.is_dir()}
save('incremental-before.json', incremental_before)
help_result = subprocess.run(['python3', '-B', 'scripts/check_doc_line_refs.py', '--help'], cwd=ROOT, env=ENV, capture_output=True, text=True)
(OUT / 'docrefs-help.log').write_text(help_result.stdout + help_result.stderr)
assert help_result.returncode == 0 and '--strict' in help_result.stdout
records = []
for name, argv in COMMANDS:
    assert shutil.disk_usage(ROOT).free >= 3 * 1024**3
    actual = ['python3', '-B', '/tmp/noname-run-serialized.py', *argv]
    started_at = datetime.datetime.now(datetime.timezone.utc).isoformat()
    started = time.monotonic()
    with (OUT / (name + '.log')).open('x') as stream:
        result = subprocess.run(actual, cwd=ROOT, env=ENV, stdout=stream, stderr=subprocess.STDOUT)
    wall = time.monotonic() - started
    lines = (OUT / (name + '.log')).read_text().splitlines()
    wrapper = json.loads(lines[-1]) if lines else {}
    locked = wrapper.get('locked_seconds')
    record = {'name': name, 'argv': argv, 'wrapper_argv': actual, 'cwd': str(ROOT), 'started_at': started_at, 'exit': result.returncode, 'wall_seconds': wall, 'locked_seconds': locked, 'queue_seconds': None, 'queue_plus_launch_overhead_seconds': max(0, wall-locked) if locked is not None else None, 'disk_free_bytes': wrapper.get('disk_free_bytes'), 'log': name + '.log'}
    records.append(record)
    save('command-index.json', records)
    print(json.dumps(record, ensure_ascii=False), flush=True)
    expected = json.loads((OUT.parent / 'parent-main-source-admission.json').read_text())['paths']
    assert all(__import__('hashlib').sha256((ROOT / p).read_bytes()).hexdigest() == h for p, h in expected.items()), 'source9 drift'

snapshot('narrow-exit.json')
incremental_after = {str(p): p.stat().st_mtime_ns for p in TARGET.rglob('incremental') if p.is_dir()}
save('incremental-after.json', incremental_after)
assert incremental_after == incremental_before, 'incremental directories changed despite inc0'
lease_record = json.loads((OUT / 'exclusive-stage3-lease.json').read_text())
lease_record.update({'released': True, 'release_time': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'target_preserved': True, 'cleanup_performed': False})
save('exclusive-stage3-lease.json', lease_record)
save('runner-result.json', {'all_four_exit0': all(r['exit'] == 0 for r in records), 'count': len(records), 'finished_at': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'free_disk_bytes': shutil.disk_usage(ROOT).free, 'source9_exact': True, 'index_empty': True, 'incremental_unchanged': incremental_after == incremental_before})
