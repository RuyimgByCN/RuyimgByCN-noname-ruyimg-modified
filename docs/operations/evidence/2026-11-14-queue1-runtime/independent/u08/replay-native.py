#!/usr/bin/env python3
"""Rerun the frozen linked probe into this review's exclusive output directory."""
import datetime
import hashlib
import json
import os
from pathlib import Path
import subprocess

BASE = Path(__file__).resolve().parent
REPO = BASE.parents[5]
ORIGINAL = BASE.parents[1] / 'U08'
OUT = BASE / 'native-replay01'
OUT.mkdir(exist_ok=False)
probe = json.loads((ORIGINAL / 'preserved-probe.json').read_text())
data = Path(probe['path']).read_bytes()
assert hashlib.sha256(data).hexdigest() == probe['sha256']
started = datetime.datetime.now(datetime.timezone.utc).isoformat()
env = os.environ.copy()
env.update({'U08_EVIDENCE': str(OUT), 'U08_REPO': str(REPO)})
with (BASE / 'native-replay01.raw.log').open('xb') as log:
    process = subprocess.run([probe['path']], cwd=REPO, env=env, stdout=log, stderr=subprocess.STDOUT)
rows = []
for path in sorted(OUT.rglob('*')):
    if path.is_file():
        relative = path.relative_to(OUT)
        prior = ORIGINAL / 'native-round2' / relative
        current = path.read_bytes()
        rows.append({'path': str(relative), 'bytes': len(current), 'sha256': hashlib.sha256(current).hexdigest(),
                     'original_exists': prior.is_file(), 'byte_identical': prior.is_file() and prior.read_bytes() == current})
inputs = json.loads((BASE / 'input-context-first.json').read_text())
unchanged = []
for row in inputs['product_rust_sources']:
    current = Path(row['path']).read_bytes()
    unchanged.append({'path': row['path'], 'sha256': hashlib.sha256(current).hexdigest(), 'unchanged': hashlib.sha256(current).hexdigest() == row['sha256']})
result = {
    'started_at_utc': started, 'finished_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=REPO, text=True).strip(),
    'tracked_product_status': subprocess.check_output(['git', 'status', '--short', '--untracked-files=no', '--', 'crates', 'packages', 'apps', 'scripts'], cwd=REPO, text=True).strip(),
    'argv': [probe['path']], 'environment_overrides': {k: env[k] for k in ('U08_EVIDENCE', 'U08_REPO')},
    'binary_sha256': hashlib.sha256(data).hexdigest(), 'exit_code': process.returncode,
    'files': rows, 'files_count': len(rows), 'all_replayed_files_byte_identical': all(r['byte_identical'] for r in rows),
    'product_rust_sources': unchanged, 'all_product_rust_sources_unchanged': all(r['unchanged'] for r in unchanged),
    'not_executed': ['Cargo compile', 'browser rerun', 'shared target cleanup', 'product modification'],
}
with (BASE / 'native-replay01-validation.json').open('x') as stream:
    json.dump(result, stream, ensure_ascii=False, indent=2)
    stream.write('\n')
print(json.dumps({k: result[k] for k in ('exit_code', 'files_count', 'all_replayed_files_byte_identical', 'all_product_rust_sources_unchanged')}, ensure_ascii=False))
assert process.returncode == 0 and result['all_replayed_files_byte_identical'] and result['all_product_rust_sources_unchanged']
