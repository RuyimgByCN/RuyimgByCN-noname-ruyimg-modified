#!/usr/bin/env python3
"""Verify retained C331 phase6 bytes and test summaries without product execution."""
from pathlib import Path
import hashlib
import json
import re
import tarfile
import tempfile

E = Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-c331-efhmbq7e/evidence/phase6')
W = E.parent.parent / 'worktree'
OUT = Path(tempfile.mkdtemp(prefix='noname-parent-c331-phase6-'))
F = OUT / 'snapshot'
F.mkdir()
sha = lambda b: hashlib.sha256(b).hexdigest()
manifest_bytes = (E / 'final-freeze6/source-manifest.json').read_bytes()
assert sha(manifest_bytes) == '81d66d59f0ca336761f28629850f9daf7a955670e7c17725cde40b88a1ba21e7'
m = json.loads(manifest_bytes)
entries = {r['path']: r for r in m['files']}
assert len(entries) == 140
archive = E / 'final-freeze6/source-files.tar.gz'
assert sha(archive.read_bytes()) == 'b9cb5c97658750bc11becc8e23a0cd074112cf3f2d38d173aaa71be3fa2084b1'
seen = set()
with tarfile.open(archive) as tar:
    for member in tar:
        if member.isdir():
            continue
        assert member.isfile() and member.name in entries, member.name
        assert member.name not in seen
        data = tar.extractfile(member).read()
        assert sha(data) == entries[member.name]['sha256'], member.name
        assert sha((W / member.name).read_bytes()) == entries[member.name]['sha256'], member.name
        target = F / member.name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)
        seen.add(member.name)
assert seen == set(entries)
pre = json.loads((E / 'pre-full/source-manifest.json').read_bytes())
assert pre['files'] == m['files']
results = json.loads((E / 'validation-results.json').read_bytes())
assert all(r['exit'] == 0 for r in results)
counts = {}
for name in ['registered-group', 'full-native']:
    data = (E / (name + '.log')).read_bytes()
    matches = re.findall(rb'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;', data)
    counts[name] = {'passed': sum(int(x[0]) for x in matches), 'failed': sum(int(x[1]) for x in matches), 'ignored': sum(int(x[2]) for x in matches), 'summaries': len(matches)}
assert (counts['registered-group']['passed'], counts['registered-group']['failed'], counts['registered-group']['ignored']) == (177, 0, 0)
assert (counts['full-native']['passed'], counts['full-native']['failed'], counts['full-native']['ignored']) == (1749, 0, 75)
assert m['source_proven'] == 262 and m['total_required'] == 290 and m['new_proven'] == 0
result = {'status': 'PASS byte identity and retained result counts only', 'source_files': 140, 'source_manifest_sha256': sha(manifest_bytes), 'source_tar_sha256': sha(archive.read_bytes()), 'all140_tar_and_worktree_fresh_sha_match': True, 'pre_full_post_full_ordered_source_rows_equal': True, 'results': results, 'counts': counts, 'source_proof': '262/290 +0', 'snapshot': str(F), 'qualification': 'No new tests/build/network/producer execution; no semantic independent approval. Existing prompt dictionary uses UTF8 strings, not numeric prompt IDs. All C remains private/BLOCK.'}
(OUT / 'independent-results.json').write_text(json.dumps(result, indent=2) + '\n')
Path('/tmp/noname-parent-c331-phase6-current.json').write_text(json.dumps({'root': str(OUT), 'result': str(OUT / 'independent-results.json'), 'snapshot': str(F)}, indent=2) + '\n')
print(json.dumps(result, indent=2))
