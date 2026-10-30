from pathlib import Path
import hashlib
import importlib.util
import json
import sys

ROOT = Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
OUT = ROOT / 'docs/operations/evidence/2026-10-30-j2-main-integration/combined-acceptance'
manifest = json.loads((OUT / 'maintained-source-hashes-final.json').read_text())
assert len(manifest) == 20752


def verify_source():
    drift = [name for name, expected in manifest.items()
             if not (ROOT / name).is_file()
             or hashlib.sha256((ROOT / name).read_bytes()).hexdigest() != expected]
    assert not drift, f'Unexpected source changes: {drift}'


verify_source()
spec = importlib.util.spec_from_file_location('recorded_runner', '/tmp/b-j2-main-run.py')
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)
checks = [
    ('parent-line-self-tests', ['python3', '-B', '-m', 'unittest', 'discover', '-s', 'scripts', '-p', 'test_check_source_lines.py']),
    ('parent-source-lines', ['python3', '-B', 'scripts/check-source-lines.py']),
    ('parent-final-preflight', ['python3', '-B', 'scripts/preflight.py']),
]
results = []
for name, command in checks:
    code = runner.run(name, command)
    results.append({'name': name, 'exit': code})
    if code:
        break
verify_source()
passed = len(results) == len(checks) and all(row['exit'] == 0 for row in results)
(OUT / 'parent-final-gates.json').write_text(json.dumps({
    'passed': passed, 'results': results, 'maintained_files_unchanged': len(manifest),
    'prior_failed_full_preflight_preserved': True,
    'source_versions': {'rules': 329, 'archive': 105, 'abi': 758, 'ws': 5},
    'cache_retention_reason': 'Private stage3 dependencies needed immediately by these final checks; no incremental compilation.',
}, indent=2) + '\n')
sys.exit(0 if passed else 1)
