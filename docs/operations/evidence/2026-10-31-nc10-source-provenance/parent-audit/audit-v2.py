#!/usr/bin/env python3
"""Read retained NC27 inputs and results; never execute producer or product code."""
import concurrent.futures
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import time

ROOT = Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-nc10-source27-pkix8fs6')
E = ROOT / 'evidence'
F = E / 'final-freeze-v2/source'
W = ROOT / 'worktree'
OUT = Path('/tmp/noname-parent-nc27-audit')
BASE = '4dd0b286b995fe414128b6c52eb83a4d0d85f6cd'
MANIFEST_SHA = '48b6fa497c0ba22f512726a9566e496f5801b04e895c65ec725e8a66f3d0421e'
PATCH_SHA = '46edddeb016a61185888ad64bcbde359cde7b74a194aa7fe613651d1e6f18f43'
OUT.mkdir(exist_ok=True)

def git(*args):
    return subprocess.run(['git', '--no-optional-locks', '-C', str(W), *args], capture_output=True, check=True, env={**os.environ, 'GIT_OPTIONAL_LOCKS': '0'}).stdout

def digest(data):
    return hashlib.sha256(data).hexdigest()

def checked_bytes(path, record):
    data = os.readlink(path).encode() if path.is_symlink() else path.read_bytes()
    assert len(data) == record['bytes'], str(path)
    assert digest(data) == record['sha256'], str(path)
    return data

manifest_bytes = (E / 'final-freeze-v2/source-manifest.json').read_bytes()
assert digest(manifest_bytes) == MANIFEST_SHA
manifest = json.loads(manifest_bytes)
assert digest((E / 'final-freeze-v2/minimal.patch').read_bytes()) == PATCH_SHA
assert git('rev-parse', 'HEAD').decode().strip() == BASE
assert not git('diff', '--cached', '--name-only')
files = {r['path']: r for r in manifest['files']}
assert len(files) == manifest['completeSourceCount'] == 43867
changed = set(manifest['changedFiles'])
assert len(changed) == 31
assert changed == {p for p, r in files.items() if r['changed']}
base = {}
for raw in git('ls-tree', '-rz', '--full-tree', BASE).split(b'\0'):
    if raw:
        info, name = raw.split(b'\t', 1)
        mode, kind, oid = info.decode().split()
        assert kind == 'blob', name
        base[name.decode()] = (mode, oid)
assert set(files) == set(base) | changed
for name in changed:
    assert checked_bytes(F / name, files[name]) == checked_bytes(W / name, files[name])
check = subprocess.run(['git', '--no-optional-locks', '-C', str(W), 'apply', '--check', '--reverse', str(E / 'final-freeze-v2/minimal.patch')], capture_output=True)
assert check.returncode == 0, check.stderr.decode()
# A wrong recorded digest must be rejected by the identity checker.
bad = dict(files[next(iter(changed))], sha256='0' * 64)
try:
    checked_bytes(F / next(iter(changed)), bad)
except AssertionError:
    pass
else:
    raise AssertionError('negative SHA self-check did not fail')

start = time.monotonic()
def verify_source(name):
    data = checked_bytes(F / name, files[name])
    if name not in changed:
        oid = hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
        assert oid == base[name][1], name
    return len(data)
source_total = 0
with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
    source_total = sum(pool.map(verify_source, files))
assert source_total == manifest['logicalBytes']

runs = json.loads((E / 'command-index.json').read_bytes())['runs']
assert len(runs) == len({r['name'] for r in runs}) == 58
summaries = {}
for r in runs:
    log = (E / r['log']).read_bytes()
    assert digest(log) == r['logSha256'], r['name']
    recorded = json.loads((E / r['resultFile']).read_bytes())
    assert recorded['exit'] == r['exit'] and recorded['command'] == r['command']
    found = re.findall(rb'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;', log)
    if found:
        summaries[r['name']] = {'passed': sum(int(x[0]) for x in found), 'failed': sum(int(x[1]) for x in found), 'ignored': sum(int(x[2]) for x in found), 'suites': len(found)}
assert summaries['v2-workspace'] == {'passed': 6916, 'failed': 0, 'ignored': 77, 'suites': 164}
assert summaries['final-native-all-layers'] == {'passed': 1715, 'failed': 0, 'ignored': 75, 'suites': 17}
assert next(x for x in runs if x['name'] == 'v2-original-sync-verify')['exit'] == 1
assert b'asset manifest tree summaries mismatch' in (E / 'v2-original-sync-verify.log').read_bytes()
products = json.loads((E / 'products-manifest.json').read_bytes())
assert products['sourceManifestSha256'] == MANIFEST_SHA
product_missing = []
for item in products['products']:
    path = Path(item['path'])
    if path.exists() or path.is_symlink():
        checked_bytes(path, item)
    else:
        product_missing.append(item['path'])
assert len(products['products']) == 95
wk = {}
for name in ['actual-wk-final-report.json', 'actual-tauri-final-report.json']:
    value = json.loads((E / name).read_bytes())
    assert value['passed'] and len(value['cases']) == 4
    assert all(row['archiveUnchanged'] for row in value['cases'])
    wk[name] = value
v1 = json.loads((E / 'final-freeze/source-manifest.json').read_bytes())
v1_files = {x['path']: x for x in v1['files']}
v1_delta = [p for p in files if p not in v1_files or files[p]['sha256'] != v1_files[p]['sha256']]
assert v1_delta == ['crates/engine-native/tests/nc10_source_provenance.rs'], v1_delta
result = {'status': 'PASS retained-identity-and-results; resource gate STILL FAIL', 'base': BASE, 'manifest_sha256': MANIFEST_SHA, 'patch_sha256': PATCH_SHA, 'source_files_fresh_sha': len(files), 'source_bytes': source_total, 'unchanged_sources_fresh_git_blob_match': len(files) - len(changed), 'changed31_worktree_and_frozen_equal': True, 'base_new_paths': sorted(set(files) - set(base)), 'reverse_patch_check': check.returncode, 'negative_sha_selfcheck': 'PASS', 'source_audit_seconds': time.monotonic() - start, 'commands': len(runs), 'failures_retained': [{'name': x['name'], 'exit': x['exit'], 'log': x['log']} for x in runs if x['exit']], 'rust_summaries': summaries, 'product_records': len(products['products']), 'product_missing_now': product_missing, 'actual_wk_reports': wk, 'v1_v2_delta_only_test': v1_delta, 'qualification': 'Fresh bytes/Git identity/log/result parsing only. No producer/product execution, no tests/build/network. UI injected missing/null/escaped cases are compatibility/safe text, not natural source authority. No root/privacy/quality/global acceptance claim.'}
(OUT / 'independent-results.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
print(json.dumps({k: result[k] for k in ['status', 'source_files_fresh_sha', 'source_bytes', 'unchanged_sources_fresh_git_blob_match', 'commands', 'product_records', 'product_missing_now', 'source_audit_seconds']}, indent=2))
