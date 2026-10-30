from pathlib import Path
import hashlib
import json
import os
import subprocess

MAIN = Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
ROOT = MAIN / 'docs/operations/evidence/2026-10-30-up37-i02-audio-aliases/final-gates'
OUT = Path('/tmp/parent-audio-alias-main-review')
index_record = json.loads((ROOT / 'artifact-sha256.json').read_text())
assert index_record['excludes_self'] is True
index = index_record['files']
assert len(index) == 69
checks = []
for name, expected in index.items():
    path = ROOT / name
    raw = path.read_bytes()
    actual = hashlib.sha256(raw).hexdigest()
    assert actual == expected['sha256'] and len(raw) == expected['bytes'], name
    checks.append({'path': name, 'bytes': len(raw), 'sha256': actual, 'equal': True})
actual_names = {str(path.relative_to(ROOT)) for path in ROOT.rglob('*') if path.is_file()}
extras = sorted(actual_names - set(index))
assert len(actual_names) == 70 and extras == ['artifact-sha256.json']
entry = json.loads((ROOT / 'narrow-entry.json').read_text())
exit_record = json.loads((ROOT / 'narrow-exit.json').read_text())
assert entry['groups'] == exit_record['groups']
current_checks = {}
doc_changes = []
for group, paths in exit_record['groups'].items():
    current_checks[group] = []
    for name, expected in paths.items():
        raw = (MAIN / name).read_bytes()
        actual = hashlib.sha256(raw).hexdigest()
        equal = actual == expected
        if not equal and group == 'parent_docs':
            doc_changes.append(name)
        else:
            assert equal, (group, name)
        current_checks[group].append({'path': name, 'bytes': len(raw), 'sha256': actual, 'equal_exit': equal})
env = {**os.environ, 'GIT_OPTIONAL_LOCKS': '0', 'GIT_NO_LAZY_FETCH': '1'}
def git(*args):
    return subprocess.check_output(['git', *args], cwd=MAIN, env=env, text=True).strip()
head = git('rev-parse', 'HEAD')
staged = git('diff', '--cached', '--name-only').splitlines()
index_path = Path(git('rev-parse', '--git-path', 'index'))
if not index_path.is_absolute():
    index_path = MAIN / index_path
index_sha = hashlib.sha256(index_path.read_bytes()).hexdigest()
assert head == entry['head'] == exit_record['head'] == 'da7e4c019461a6c467a9dd2de849f3b1a7494b86'
assert staged == entry['index_paths'] == exit_record['index_paths'] == []
assert index_sha == entry['index_sha256'] == exit_record['index_sha256']
report = {'all_assertions_passed': True, 'artifact_count': 69, 'actual_count': 70, 'missing': sorted(set(index) - actual_names), 'extras': extras, 'artifact_checks': checks, 'entry_exit_groups_equal': True, 'current_groups': current_checks, 'allowed_parent_doc_changes': doc_changes, 'head': head, 'index_paths': staged, 'index_sha256': index_sha, 'scope': '69 indexed gate artifacts and exact narrow recorded groups only; no gates rerun or protected/media tree scan.'}
(OUT / 'final-gates-independent-results.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps({'all_assertions_passed': True, 'artifact_count': 69, 'actual_count': 70, 'groups': {key: len(rows) for key, rows in current_checks.items()}, 'parent_doc_changes': doc_changes, 'head': head, 'index_empty': not staged}, indent=2))
