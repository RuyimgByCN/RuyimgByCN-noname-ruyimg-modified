import collections
import datetime
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import time

ROOT = Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
OUT = ROOT / 'docs/operations/evidence/2026-10-30-pa09-js-admission/final-gates'
PREVIOUS = OUT.parent / 'main-acceptance/entry.json'
ENV = {**os.environ, 'GIT_OPTIONAL_LOCKS': '0', 'GIT_NO_LAZY_FETCH': '1', 'CARGO_INCREMENTAL': '0', 'CARGO_BUILD_JOBS': '2'}
LIMIT = 3 * 1024**3
EXPECTED_HEAD = '777bf33969e5ecc04b63971adab26be32f12fd28'
EXPECTED_ATTRIBUTES = '9dbbbde6fb0c137b5a47b3d3738b3ca9399dec80a03f9acc1399ffc1d3619388'
COMMANDS = [
    ['python3', '-B', '-m', 'unittest', 'discover', '-s', 'scripts', '-p', 'test_check_source_lines.py'],
    ['python3', '-B', 'scripts/check-source-lines.py'],
    ['python3', '-B', 'scripts/check_doc_line_refs.py', '--strict'],
    ['python3', '-B', 'scripts/preflight.py'],
]

def sha(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        for block in iter(lambda: f.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()

def save(name, data):
    (OUT / name).write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n')

def git(*args):
    return subprocess.check_output(['git', '--no-optional-locks', *args], cwd=ROOT, env=ENV).decode()

def path_hashes(paths):
    return {p: sha(ROOT / p) if (ROOT / p).is_file() else None for p in sorted(paths)}

def snapshot():
    prior = json.loads(PREVIOUS.read_text())
    index = Path(git('rev-parse', '--git-path', 'index').strip())
    if not index.is_absolute():
        index = ROOT / index
    state = {'utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
             'head': git('rev-parse', 'HEAD').strip(), 'tree': git('rev-parse', 'HEAD^{tree}').strip(),
             'index_sha256': sha(index), 'staged': git('diff', '--cached', '--name-only').splitlines(),
             'tracked_status': git('status', '--porcelain=v1', '--untracked-files=no'),
             'attribute_sha256': sha(ROOT / '.gitattributes'), 'guard': json.loads((ROOT / '.pi-lens.json').read_text()),
             'free_disk_bytes': shutil.disk_usage(ROOT).free}
    for key in ['source3', 'protected', 'modules', 'dist']:
        state[key] = path_hashes(prior[key])
    state['groups'] = {k: sorted(v) for k, v in prior['groups'].items()}
    callers = subprocess.check_output(['rg', '-l', 'from ["\'].*/(engine/)?client["\']|new EngineClient|DesktopEngineTransport|desktopEngineChoice', 'apps/web-client/src', 'apps/web-client/scripts'], cwd=ROOT, env=ENV, text=True).splitlines()
    state['groups']['client_and_transport_callers'] = sorted(callers)
    state['group_hashes'] = {k: path_hashes(v) for k, v in state['groups'].items()}
    state['frozen_package_diff_sha256'] = hashlib.sha256(subprocess.check_output(['git', '--no-optional-locks', 'diff', 'HEAD', '--', 'content/original/ported', 'content/original/legacy-source', 'packages/extensions', 'Cargo.lock'], cwd=ROOT, env=ENV)).hexdigest()
    state['package_git_trees'] = {p: git('rev-parse', 'HEAD:' + p).strip() for p in prior['package_git_trees']}
    # Media bytes stay outside the broad source/docs manifest; the five compat inputs are covered by compat14.
    binary_suffixes = {'.mp3', '.m4a', '.wav', '.ogg', '.flac', '.mp4', '.webm', '.jpg', '.jpeg', '.png', '.gif', '.webp', '.svg', '.ico', '.icns', '.bmp', '.pdf', '.gz', '.zip', '.zst', '.xz', '.tar', '.wasm', '.bin', '.woff', '.woff2', '.ttf', '.otf', '.dylib', '.so', '.dll', '.exe', '.npack'}
    inventory = sorted(set(p for p in git('ls-files', '--cached', '--others', '--exclude-standard', '-z').split('\0') if p and not p.startswith(str(OUT.relative_to(ROOT)) + '/')))
    maintained = {}
    excluded = collections.Counter()
    for p in inventory:
        path = ROOT / p
        if path.suffix.lower() in binary_suffixes:
            excluded[path.suffix.lower()] += 1
        elif path.is_symlink():
            maintained[p] = {'symlink_target': os.readlink(path)}
        elif path.is_file():
            maintained[p] = {'sha256': sha(path), 'bytes': path.stat().st_size}
        else:
            maintained[p] = {'missing': True}
    state['maintained_source_docs'] = maintained
    state['binary_exclusions_by_suffix'] = dict(sorted(excluded.items()))
    state['scope_note'] = 'All Git tracked and nonignored untracked regular source/docs/text paths, excluding this final-gates directory and listed media/binary suffixes; narrow manifests additionally hash compat audio, 18 generated modules, and 37 dist paths. No bulk media hash.'
    return state

def entry_checks(state):
    prior = json.loads(PREVIOUS.read_text())
    admission = json.loads((OUT.parent / 'parent-main-source-admission.json').read_text())
    checks = {k: state[k] == prior[k] for k in ['head', 'tree', 'source3', 'protected', 'modules', 'dist', 'guard', 'index_sha256', 'staged', 'frozen_package_diff_sha256', 'package_git_trees']}
    checks['groups'] = all(set(state['groups'][k]) == set(v) for k, v in prior['groups'].items())
    checks['admitted_source3'] = state['source3'] == admission['source3']
    checks['attribute'] = state['attribute_sha256'] == EXPECTED_ATTRIBUTES
    checks['expected_head'] = state['head'] == EXPECTED_HEAD
    checks['no_staged_files'] = not state['staged']
    checks['disk_at_least_3GiB'] = state['free_disk_bytes'] >= LIMIT
    save('previous-functionality-comparison.json', {'invariants': checks, 'tracked_status_note': 'Parent documentation/evidence changes since functionality are legitimate; fresh final entry/exit manifest covers them.'})
    return checks

def run():
    OUT.mkdir(exist_ok=False)
    shutil.copyfile(Path(__file__), OUT / 'capture-final-gates.py')
    entry = snapshot()
    save('entry.json', entry)
    checks = entry_checks(entry)
    records = []
    save('commands.json', records)
    blocker = None
    if not all(checks.values()):
        blocker = 'Entry invariant failed: ' + ', '.join(k for k, v in checks.items() if not v)
    for n, argv in enumerate(COMMANDS, 1):
        if blocker:
            break
        if shutil.disk_usage(ROOT).free < LIMIT:
            blocker = 'Disk below 3GiB before gate ' + str(n)
            break
        actual = ['python3', '-B', '/tmp/noname-run-serialized.py', *argv]
        before = shutil.disk_usage(ROOT).free
        start_utc = datetime.datetime.now(datetime.timezone.utc).isoformat()
        start = time.monotonic()
        outname, errname = f'gate-{n}.stdout.log', f'gate-{n}.stderr.log'
        minimum = before
        stopped = False
        with (OUT / outname).open('xb') as stdout, (OUT / errname).open('xb') as stderr:
            child = subprocess.Popen(actual, cwd=ROOT, env=ENV, stdout=stdout, stderr=stderr, start_new_session=True)
            while child.poll() is None:
                free = shutil.disk_usage(ROOT).free
                minimum = min(minimum, free)
                if free < LIMIT:
                    stopped = True
                    os.killpg(child.pid, signal.SIGTERM)
                    try:
                        child.wait(timeout=10)
                    except subprocess.TimeoutExpired:
                        os.killpg(child.pid, signal.SIGKILL)
                        child.wait()
                    break
                time.sleep(0.5)
            code = child.wait()
        wall = time.monotonic() - start
        wrapper = None
        for line in reversed((OUT / outname).read_text(errors='replace').splitlines()):
            try:
                candidate = json.loads(line)
                if isinstance(candidate, dict) and candidate.get('command') == argv and 'locked_seconds' in candidate:
                    wrapper = candidate
                    break
            except json.JSONDecodeError:
                continue
        record = {'gate': n, 'argv': argv, 'serialized_argv': actual, 'cwd': str(ROOT), 'started_utc': start_utc,
                  'finished_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'exit': code,
                  'wall_seconds': wall, 'locked_seconds': wrapper['locked_seconds'] if wrapper else None,
                  'queue_plus_wrapper_seconds': max(0, wall - wrapper['locked_seconds']) if wrapper else None,
                  'disk_free_entry_bytes': before, 'disk_free_exit_bytes': shutil.disk_usage(ROOT).free,
                  'disk_free_minimum_sampled_bytes': minimum, 'disk_stop': stopped, 'wrapper_record': wrapper,
                  'stdout': outname, 'stderr': errname, 'stdout_sha256': sha(OUT / outname), 'stderr_sha256': sha(OUT / errname),
                  'environment': {'CARGO_INCREMENTAL': '0', 'CARGO_BUILD_JOBS': '2', 'GIT_OPTIONAL_LOCKS': '0', 'GIT_NO_LAZY_FETCH': '1'}}
        records.append(record)
        save('commands.json', records)
        print(json.dumps(record), flush=True)
        if code != 0 or stopped or wrapper is None:
            blocker = f'Gate {n} exit={code}, disk_stop={stopped}, serialization_record_present={wrapper is not None}'
    exit_state = snapshot()
    save('exit.json', exit_state)
    invariants = {k: entry[k] == exit_state[k] for k in entry if k not in {'utc', 'free_disk_bytes'}}
    changed = {p: {'entry': entry['maintained_source_docs'].get(p), 'exit': exit_state['maintained_source_docs'].get(p)} for p in sorted(set(entry['maintained_source_docs']) | set(exit_state['maintained_source_docs'])) if entry['maintained_source_docs'].get(p) != exit_state['maintained_source_docs'].get(p)}
    save('source-comparison.json', {'invariants': invariants, 'maintained_changed_paths': changed,
         'maintained_paths': len(entry['maintained_source_docs']), 'previous_functionality_invariants': checks,
         'group_comparison': 'Exact caller path sets, order-insensitive only; file hashes exact.',
         'counts': {k: len(entry[k]) for k in ['source3', 'protected', 'modules', 'dist']},
         'group_counts': {k: len(v) for k, v in entry['groups'].items()}})
    success = len(records) == 4 and all(r['exit'] == 0 and r['wrapper_record'] and not r['disk_stop'] for r in records) and all(invariants.values()) and all(checks.values()) and not exit_state['staged']
    if not all(invariants.values()):
        blocker = (blocker + '; ' if blocker else '') + 'Final window invariant failed: ' + ', '.join(k for k, v in invariants.items() if not v)
    preflight = None
    if len(records) == 4:
        output = (OUT / records[-1]['stdout']).read_text()
        preflight = {'expected_checks': 21, 'observed_success_lines': output.count('✅ ') ,
                     'observed_failure_lines': output.count('❌ '), 'child_logs': 'Suppressed by unmodified preflight; no individual child stdout/stderr claimed.',
                     'unmodified_preflight_source_sha256': entry['maintained_source_docs']['scripts/preflight.py']['sha256']}
    summary = {'result': 'PASS' if success else 'FAIL', 'literal_result': 'FINAL_GATE_RESULT=' + ('PASS' if success else 'FAIL'),
               'head': entry['head'], 'tree': entry['tree'], 'entry_utc': entry['utc'], 'exit_utc': exit_state['utc'],
               'commands_executed': len(records), 'commands_exit0': sum(r['exit'] == 0 for r in records),
               'source_invariance': all(invariants.values()), 'no_staged_files': not exit_state['staged'],
               'blocker': blocker, 'preflight': preflight,
               'write_scope': 'Only final-gates evidence and assigned report; no source/docs prose/config/index/staging/commit/push changes.',
               'lease': 'Main lease released after own evidence sealed.',
               'limitations': ['No new Rust/WASM/GUI/media/resource sync/builds beyond unmodified preflight.', 'Earlier frontend, typecheck, production bundle, LSP and WK evidence are inherited, not rerun.', 'New Windows CI remains outside this local final gate.']}
    save('summary.json', summary)
    save('artifact-sha256.json', {p.name: {'sha256': sha(p), 'bytes': p.stat().st_size} for p in sorted(OUT.iterdir()) if p.is_file() and p.name != 'artifact-sha256.json'})
    print(json.dumps(summary, ensure_ascii=False), flush=True)

if __name__ == '__main__':
    run()
