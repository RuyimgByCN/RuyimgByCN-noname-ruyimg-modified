from pathlib import Path
import hashlib
import json
import subprocess

MAIN = Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
BASE = MAIN / 'docs/operations/evidence/2026-10-30-up37-i02-audio-aliases'
EVIDENCE = BASE / 'main-acceptance'
OUT = Path('/tmp/parent-audio-alias-main-review')
OUT.mkdir(exist_ok=True)
index = json.loads((EVIDENCE / 'artifact-sha256.json').read_text())
assert len(index) == 107
checks = []
missing = []
for name, expected in index.items():
    path = EVIDENCE / name
    if not path.is_file():
        missing.append(name)
        continue
    raw = path.read_bytes()
    actual = hashlib.sha256(raw).hexdigest()
    checks.append({'path': name, 'bytes': len(raw), 'sha256': actual, 'equal': actual == expected})
actual_names = {str(path.relative_to(EVIDENCE)) for path in EVIDENCE.rglob('*') if path.is_file()}
extras = sorted(actual_names - set(index))
admission = json.loads((BASE / 'parent-main-source-admission.json').read_text())
source9 = {name: hashlib.sha256((MAIN / name).read_bytes()).hexdigest() for name in admission['paths']}
assert len(source9) == 9
aggregate = hashlib.sha256(json.dumps(source9, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
recorded_aggregate = json.loads((EVIDENCE / 'source-aggregate-sha256.json').read_text())['source9']
head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=MAIN, text=True).strip()
report = {'artifact_index_entries': len(index), 'actual_file_count': len(actual_names), 'missing': missing, 'extra_files': extras, 'index_self_excluded': 'artifact-sha256.json' not in index, 'artifact_checks': checks, 'source9': source9, 'source9_equal_admission': source9 == admission['paths'], 'source9_aggregate': aggregate, 'recorded_source9_aggregate': recorded_aggregate, 'aggregate_equal': aggregate == recorded_aggregate, 'head': head, 'read_only_scope': '107 indexed evidence files and source9 only; no protected tree/media hash, tests, build, CI or final-gates execution.'}
assert not missing and all(row['equal'] for row in checks)
assert extras == ['artifact-sha256.json']
assert report['source9_equal_admission'] and report['aggregate_equal']
assert head == 'da7e4c019461a6c467a9dd2de849f3b1a7494b86'
report['all_assertions_passed'] = True
(OUT / 'independent-results.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps({key: report[key] for key in ['artifact_index_entries', 'actual_file_count', 'missing', 'extra_files', 'source9_equal_admission', 'source9_aggregate', 'head', 'all_assertions_passed']}, indent=2))
