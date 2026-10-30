from pathlib import Path
import hashlib
import json
import os
import subprocess
import sys
import time

ROOT = Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
OUT = ROOT / 'docs/operations/evidence/2026-10-30-native-ci-preparation'
try:
    EXPECTED = json.loads((OUT / 'parent-corrected-ci-admission.json').read_text())['reviewed_paths_exact']
except (OSError, ValueError, KeyError, TypeError) as error:
    raise SystemExit(f'Required CI source admission is missing or invalid: {error}') from error
assert len(EXPECTED) == 4


def verify_inputs():
    assert all(hashlib.sha256((ROOT / p).read_bytes()).hexdigest() == h for p, h in EXPECTED.items())
    assert not subprocess.check_output(['git', 'diff', '--name-only', '--', 'crates'], cwd=ROOT)
    assert not subprocess.check_output(['git', 'diff', '--name-only', '--', 'content/original'], cwd=ROOT)


verify_inputs()
checks = [
    ('source-line-selftests', ['python3', '-B', '-m', 'unittest', 'discover', '-s', 'scripts', '-p', 'test_check_source_lines.py']),
    ('source-line-gate', ['python3', '-B', 'scripts/check-source-lines.py']),
    ('strict-document-references', ['python3', '-B', 'scripts/check_doc_line_refs.py', '--strict']),
    ('full-preflight', ['python3', '-B', 'scripts/preflight.py']),
]
rows = []
env = dict(os.environ, GIT_OPTIONAL_LOCKS='0', GIT_NO_LAZY_FETCH='1', CARGO_INCREMENTAL='0')
for name, command in checks:
    log = OUT / ('parent-final-' + name + '.log')
    assert not log.exists(), 'Do not overwrite original evidence'
    started = time.monotonic()
    with log.open('w') as output:
        result = subprocess.run(['python3', '-B', '/tmp/noname-run-serialized.py', *command],
                                cwd=ROOT, env=env, stdout=output, stderr=subprocess.STDOUT)
    rows.append({'name': name, 'command': command, 'exit': result.returncode,
                 'elapsed_seconds_including_queue': time.monotonic() - started, 'log': log.name})
    (OUT / 'parent-final-command-index.json').write_text(json.dumps(rows, indent=2) + '\n')
    if result.returncode:
        break
verify_inputs()
passed = len(rows) == len(checks) and all(row['exit'] == 0 for row in rows)
(OUT / 'parent-final-local-validation.json').write_text(json.dumps({
    'passed': passed, 'commands': rows, 'four_reviewed_config_inputs_unchanged': True,
    'rust_and_tracked_original_content_unchanged': True,
    'github_run_count': 0, 'push_count': 0,
    'scope': 'Local configuration/input validation only; no Cargo/WASM build or external execution',
}, indent=2) + '\n')
sys.exit(0 if passed else 1)
