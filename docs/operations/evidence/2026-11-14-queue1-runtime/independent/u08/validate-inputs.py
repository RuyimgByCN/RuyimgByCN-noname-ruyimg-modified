#!/usr/bin/env python3
"""Hash frozen U08 inputs without reading the collector's report/comparator."""
import hashlib
import json
from pathlib import Path

BASE = Path(__file__).resolve().parent
REPO = BASE.parents[5]
ORIGINAL = BASE.parents[1] / 'U08'
EXPECTED = {
    'review-input-manifest.json': 'a608d6906f7bd0d7776f32167c0f9f79124b5e9ac3a6ac7c9140684b94fa1c2e',
    'manifest.json': '7934ad115c63fd0ace23d3b3800d1e2703e621554e9db6391035f09a1185205c',
    'raw-transcript-index.json': '944370aaf9014f15708d72b583f0dc60c4c050c5077f5b306abdfa0c7e17f110',
    'browser-round4/results.json': 'f534d90678d2d98601d8cf98f4d378abff451387c94ff7f86485a72dba6ac2c7',
    'native-round2/native-results.json': '9b7a395c0878b3852065449ce7fedad7f5431118db46d6ffb141fd2337c05f8d',
    'native-probe/src/main.rs': '803814884294fab2d1bb4f428863afd225b45d737e67ff2a7eb799cede8874b5',
    'browser-round4-browser-runner.py': '1de870a02a1e6ca1000a930980e5db71ed0af38f9647425d73b17551215170f9',
    'browser-round4-upstream-setup.js': '64555a4e28c957af4c9ae8ffb56ddb6f440137c95836210912c4458bbf1a72a1',
    'browser-round4-control.py': '43c5aa7475205b12fa57e24862e69360e53fd905d9cccfa754619a22820a4210',
    'session-u08-1794594672096-requests.jsonl': '7e37da6009789b7e5bd0215f777f2dabb9b76ec4d7e5866316c697f3b004eac6',
    'session-u08-1794594672096-replies.jsonl': '4194b53e1b98207b77eca5c31d111b042f5c767391898d999fb8638aaef526b4',
}


def hashed(path):
    data = path.read_bytes()
    return {'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()}


records = []
for name, expected in EXPECTED.items():
    actual = hashed(ORIGINAL / name)
    records.append({'path': name, **actual, 'expected_sha256': expected, 'matches': actual['sha256'] == expected})
manifests = []
for name in ('review-input-manifest.json', 'manifest.json'):
    data = json.loads((ORIGINAL / name).read_text())
    rows = []
    for entry in data['files']:
        actual = hashed(ORIGINAL / entry['path'])
        rows.append({'path': entry['path'], **actual, 'matches': all(actual[k] == entry[k] for k in ('bytes', 'sha256'))})
    manifests.append({'path': name, 'count': len(rows), 'total_bytes': sum(r['bytes'] for r in rows), 'all_match': all(r['matches'] for r in rows), 'files': rows})
source_copies = []
for copy in sorted((ORIGINAL / 'source-inputs').rglob('*')):
    if not copy.is_file():
        continue
    rel = copy.relative_to(ORIGINAL / 'source-inputs').as_posix()
    live = REPO.parent / rel.removeprefix('upstream/') if rel.startswith('upstream/') else REPO / rel
    original_hash = hashed(copy)
    live_hash = hashed(live) if live.exists() else None
    source_copies.append({'copy': str(copy.relative_to(ORIGINAL)), 'live_path': str(live), 'copy_hash': original_hash, 'live_hash': live_hash, 'matches': original_hash == live_hash})
probe = json.loads((ORIGINAL / 'preserved-probe.json').read_text())
probe_actual = hashed(Path(probe['path']))
result = {'named_frozen_inputs': records, 'manifests': manifests, 'source_copies': source_copies,
          'preserved_binary': {'path': probe['path'], **probe_actual, 'matches': all(probe_actual[k] == probe[k] for k in ('bytes', 'sha256'))}}
result['all_pass'] = all(r['matches'] for r in records) and all(m['all_match'] for m in manifests) and all(r['matches'] for r in source_copies) and result['preserved_binary']['matches']
with (BASE / 'input-validation-first.json').open('x') as stream:
    json.dump(result, stream, ensure_ascii=False, indent=2)
    stream.write('\n')
print(json.dumps({'all_pass': result['all_pass'], 'named_inputs': len(records), 'manifest_counts': [m['count'] for m in manifests], 'source_copy_count': len(source_copies), 'binary': result['preserved_binary']}, ensure_ascii=False))
assert result['all_pass']
