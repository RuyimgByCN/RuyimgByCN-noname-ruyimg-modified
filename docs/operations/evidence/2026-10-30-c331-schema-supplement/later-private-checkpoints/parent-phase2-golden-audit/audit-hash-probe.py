from pathlib import Path
import ast
import difflib
import hashlib
import json
import re
import subprocess
import tarfile

ROOT = Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-c331-efhmbq7e/evidence/phase2')
OUT = Path('/tmp/parent-c331-golden-review')
PROBE = ROOT.parent.parent / 'golden-probe'
SNAPSHOT = Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-c331-phase2-review-u32ezani/snapshot')
prepare = (ROOT / 'golden-lab-prepare.py').read_text()
syntax = ast.parse(prepare)
append = [node.value.value for node in ast.walk(syntax) if isinstance(node, ast.AugAssign) and isinstance(node.op, ast.Add) and isinstance(node.target, ast.Name) and node.target.id == 's' and isinstance(node.value, ast.Constant)]
assert len(append) == 1
replacements = [node.args[1] for node in ast.walk(syntax) if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and node.func.attr == 'replace']
assert len(replacements) == 1 and isinstance(replacements[0], ast.BinOp)
assert isinstance(replacements[0].left, ast.Constant) and isinstance(replacements[0].right, ast.Name) and replacements[0].right.id == 'needle'
instrument = replacements[0].left.value
needle = '        let snap = engine.save();'
paths = ['crates/engine-native/src/engine/debug.rs', 'crates/engine-native/tests/replay_scenarios.rs']
original = {}
with tarfile.open(ROOT / 'p1-freeze/source-files.tar.gz') as bundle:
    for name in paths:
        try:
            original[name] = bundle.extractfile(name).read().decode()
        except KeyError:
            original[name] = subprocess.check_output(['git', 'show', '38ab1678df9b4b7b991d3b3122b1d937fd2648a1:' + name], cwd='/Users/ruyimg/Documents/Claude/sanguosha/noname-next').decode()
expected = {paths[0]: original[paths[0]] + append[0], paths[1]: original[paths[1]].replace(needle, instrument + needle, 1)}
source_checks = []
for name in paths:
    actual = (PROBE / name).read_text()
    assert actual == expected[name], name
    diff = ''.join(difflib.unified_diff(original[name].splitlines(True), actual.splitlines(True), fromfile='p1/' + name, tofile='probe/' + name))
    source_checks.append({'path': name, 'only_expected_append_or_instrumentation': True, 'diff': diff})
manifest = json.loads((ROOT / 'p1-freeze/source-freeze.json').read_text())
hash_paths = [row for row in manifest['files'] if row['path'].startswith('crates/engine-native/src/engine/hash')]
assert hash_paths
for row in hash_paths:
    assert hashlib.sha256((PROBE / row['path']).read_bytes()).hexdigest() == row['sha256']
source = (SNAPSHOT / paths[1]).read_text().split('const SCENARIOS:', 1)[1]
rows = json.loads((ROOT / 'golden-hash-attribution.json').read_text())
binding = json.loads((ROOT / 'golden-final-source-binding.json').read_text())
raw_log = (ROOT / 'affected-full-final.log').read_text()
actual_pairs = {name: [actual, expected] for name, actual, expected in re.findall(r'^([a-z_]+) actual=([0-9a-f]{64}) expected=([0-9a-f]{64})$', raw_log, re.MULTILINE)}
assert len(rows) == len(actual_pairs) == len(binding['values']) == 14
records = []
for row in rows:
    name = row['scenario']
    matched = re.search('"' + re.escape(name) + r'"[\s\S]*?"([0-9a-f]{64})"', source)
    assert matched
    original_hash = matched.group(1)
    lab = dict(line.split('=', 1) for line in (ROOT / 'golden-lab' / f'{name}.lab.txt').read_text().splitlines() if line)
    assert original_hash == lab['original330'] == row['original330'] == actual_pairs[name][1]
    assert lab == row['hashes']
    axis = 'version_and_original_budget' if name == 'suidun_window_matrix' else 'version_without_new_window' if name == 'rollback_invariants' else 'version_only'
    assert lab[axis] == original_hash
    assert actual_pairs[name] == binding['values'][name]
    records.append({'scenario': name, 'old_source_expected': original_hash, 'matching_axis': axis, 'recorded_probe_value': lab[axis], 'final_full_suite_actual': actual_pairs[name][0], 'final_binding_exact_raw_log': True})
report = {'all_assertions_passed': True, 'probe_source_exactly_two_expected_changes': source_checks, 'production_hash_files_match_p1_freeze': [row['path'] for row in hash_paths], 'records': records, 'no_binary_probe_executed': True, 'scope': 'Independent source-diff and recorded-vector/log identity validation; this does not claim a newly executed hash experiment.'}
(OUT / 'hash-probe-independent-results.json').write_text(json.dumps(report, indent=2) + '\n')
print('Exact private source changes and recorded14hashes/final-source-log binding verified')
