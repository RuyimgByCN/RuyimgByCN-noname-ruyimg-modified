import hashlib
import json
import pathlib
import re

BASE = pathlib.Path(__file__).resolve().parent
NATIVE = BASE / 'native-round2'
rows = json.loads((NATIVE / 'native-results.json').read_text())
checks = []
for row in rows:
    name = row['case']
    d, r = row['direct'], row['restored']
    for mode in ['direct', 'restored']:
        for c in row[mode]['pending_archive_checks']:
            assert c.get('error') is None, (name, mode, c)
            assert c['state_hash_identical'], (name, mode, c)
            assert c['snapshot_equal_except_auth_generation'], (name, mode, c)
            assert not c['roundtrip_bytes_identical'], (name, mode, c)
            assert c['auth_after'] == c['auth_before'] + 1, (name, mode, c)
            assert (NATIVE / c['file']).stat().st_size == c['bytes']
            checks.append({'case': name, 'mode': mode, **c})
    assert d['final'] == r['final'], name
    project = lambda s: [(x['request'], x['reply'], x['log_delta']) for x in s['commands']]
    assert project(d) == project(r), name
    normalize = lambda x: re.sub(r'auth_generation: \d+', 'auth_generation: NORMALIZED', x)
    assert normalize((NATIVE / f'{name}-direct-final-snapshot-debug.txt').read_text()) == normalize((NATIVE / f'{name}-restored-final-snapshot-debug.txt').read_text()), name
files = [{'path': str(p.relative_to(BASE)), 'bytes': p.stat().st_size, 'sha256': hashlib.sha256(p.read_bytes()).hexdigest(),
          'first_8_bytes_hex': p.read_bytes()[:8].hex()} for p in sorted(NATIVE.glob('*.ngsa'))]
report = {'cases': len(rows), 'all_assertions_satisfied': True, 'pending_checks': len(checks),
          'final_complete_snapshots_compared_after_only_auth_generation_normalization': len(rows),
          'raw_ngsa_file_count': len(files), 'pending_roundtrip_bytes_identical': 0,
          'explanation': 'Restore increments auth_generation once. Raw archives remain different. Pending snapshot equality is the actual Rust Snapshot comparison captured by checkpoint(); final Debug snapshots are compared after replacing exactly auth_generation digits, and request/reply/log arrays are compared without normalization.',
          'checks': checks, 'files': files}
(BASE / 'native-validation-audit.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
print(json.dumps({k: report[k] for k in ['cases', 'pending_checks', 'raw_ngsa_file_count', 'all_assertions_satisfied']}))
