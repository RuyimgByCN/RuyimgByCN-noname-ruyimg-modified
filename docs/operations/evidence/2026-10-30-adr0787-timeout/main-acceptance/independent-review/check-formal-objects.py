from pathlib import Path
import hashlib
import json

SOURCE = Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-a330-main-phase2-in4xi44k')
MAIN = Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
OUT = Path(__file__).parent
CHECKPOINT = MAIN / 'docs/operations/evidence/2026-10-30-adr0787-timeout/main-acceptance/phase2/production-artifacts-checkpoint.json'

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def load(relative):
    return json.loads((SOURCE / relative).read_text())

def differences(left, right, path='$'):
    if type(left) is not type(right):
        return [path + ': type differs']
    if isinstance(left, dict):
        found = [path + '.' + key + ': missing' for key in left.keys() ^ right.keys()]
        for key in left.keys() & right.keys():
            found.extend(differences(left[key], right[key], path + '.' + key))
        return found
    if isinstance(left, list):
        found = [] if len(left) == len(right) else [path + ': length differs']
        for index, (a, b) in enumerate(zip(left, right)):
            found.extend(differences(a, b, f'{path}[{index}]'))
        return found
    return [] if left == right else [path + ': value differs']

locator = str(MAIN / 'apps/web-client/src/worker/native-pkg/engine_native_wasm_bg.wasm')
checkpoint = json.loads(CHECKPOINT.read_text())
expected_sha = checkpoint['packageCopiesPreserved']['native-pkg']['engine_native_wasm_bg.wasm']['sha256']
actual_sha = digest(Path(locator))
assert actual_sha == expected_sha
result = {'read_only_inputs': True, 'comparisons': [], 'locator': {'path': locator, 'sha256': actual_sha, 'checkpoint_sha256': digest(CHECKPOINT), 'matches_checkpoint': True}}
for native, wasm in [('formal/native.json', 'formal/wasm.json'), ('formal/native-package.json', 'formal/wasm-package.json')]:
    a, b = load(native), load(wasm)
    assert 'wasm' not in a
    assert set(b) == set(a) | {'wasm'}
    assert type(b['wasm']) is str and b['wasm'] == locator
    # Only the verified top-level build locator is absent from native output.
    payload = {key: value for key, value in b.items() if key != 'wasm'}
    delta = differences(a, payload)
    result['comparisons'].append({'native': native, 'wasm': wasm, 'native_sha256': digest(SOURCE / native), 'wasm_sha256': digest(SOURCE / wasm), 'native_keys': sorted(a), 'wasm_keys': sorted(b), 'excluded_exact_field': '$.wasm', 'differences': delta, 'equal': not delta, 'direct_count': len(a['direct']), 'restore_count': len(a['restore'])})
    assert not delta, delta
for a_path, b_path in [('planner/native-output.json', 'planner/wasm-output.json'), ('referee-external-corrected/report.json.before.json', 'referee-external-corrected/report.json.after.json')]:
    delta = differences(load(a_path), load(b_path))
    result['comparisons'].append({'left': a_path, 'right': b_path, 'left_sha256': digest(SOURCE / a_path), 'right_sha256': digest(SOURCE / b_path), 'excluded_fields': [], 'differences': delta, 'equal': not delta})
    assert not delta, delta
rows = load('catalog-version/report.json')
assert len(rows) == 14
versions = []
for row in rows:
    for key in ['lock', 'catalogRules', 'headerRules']:
        assert type(row[key]) is str and str(int(row[key])) == row[key]
    lock = int(row['lock'])
    assert row['base'] == 330 and row['archiveVersion'] == 105
    assert (row['actualPacks'] == 0) == (lock == 0)
    expected = str(330 if lock == 0 else 330 ^ (lock | (1 << 63)))
    assert row['catalogRules'] == row['headerRules'] == expected
    versions.append({'name': row['name'], 'lock': row['lock'], 'expected_rules_string': expected, 'equal': True})
assert sum(row['actualPacks'] == 0 for row in rows) == 11
result['versions'] = {'source_sha256': digest(SOURCE / 'catalog-version/report.json'), 'standard_count': 11, 'package_count': 3, 'rows': versions}
result['all_equal'] = True
result['scope'] = 'All recorded JSON keys, JSON types and values compared; formal payload itself records revision/hash/codes, not full engine state. No general metadata removal. No rerun or source/fixture modification.'
(OUT / 'formal-object-check.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({'all_equal': True, 'formal_pairs': 2, 'planner_pair': 1, 'referee_state_pair': 1, 'versions': len(rows), 'output': str(OUT / 'formal-object-check.json')}))
