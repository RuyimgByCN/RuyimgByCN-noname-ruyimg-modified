#!/usr/bin/env python3
"""Read retained NC27 incremental, resource and Main results; execute no producers."""
from pathlib import Path
import hashlib
import json
import re
import subprocess

P = Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
E = Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-nc10-source27-pkix8fs6/evidence')
R = Path('/Users/ruyimg/.pi/agent/sessions/--Users-ruyimg-Documents-Claude-sanguosha--/subagent-artifacts/outputs/690c785a-d775-4f93-8ef9-371f54c28532/implementation/nc27-legacy-serde-p2-v3')
OUT = Path('/tmp/noname-parent-nc27-audit')
sha = lambda data: hashlib.sha256(data).hexdigest()
def load(path):
    return json.loads(path.read_bytes())
def check(path, record):
    data = path.read_bytes()
    assert sha(data) == record['sha256'], str(path)
    if 'bytes' in record:
        assert len(data) == record['bytes'], str(path)
    return data

mb = (R / 'source-manifest-v3.json').read_bytes()
assert sha(mb) == '6ebf6c33ec7cffc08b8084629ccdde75d277c62dadf72c37a5cecb6cf8e7cf19'
m = json.loads(mb)
assert sha((R / 'v3-delta.patch').read_bytes()) == '014197d181102ce8dd306cf27de42f215ed5ba471c8e4222baf5b8fbf40d2092'
assert len(m['files']) == 4
v2 = {x['path']: x for x in load(E / 'final-freeze-v2/source-manifest.json')['files']}
v3 = load(R / 'full-source-manifest-v3.json')
assert sha((R / 'full-source-manifest-v3.json').read_bytes()) == '4f81d2ec82329854df3f5662602bd6b985c1b69050c220e993f3da41da9f5b0e'
v3_files = {x['path']: x for x in v3['files']}
changed = {x['path'] for x in m['files']}
assert len(v3_files) == 43869
assert set(v3_files) == set(v2) | changed
assert {p for p in v3_files if p not in v2 or v3_files[p]['sha256'] != v2[p]['sha256']} == changed
for row in m['files']:
    check(R / 'source' / row['path'], row)
    check(E.parent / 'worktree' / row['path'], row)
    if row['path'] in v2:
        assert row['oldV2Sha256'] == v2[row['path']]['sha256']
red = load(R / 'red-matrix.json')
green = load(R / 'green-matrix.json')
assert len(red) == len(green) == 445
assert [(r['case'], r['raw']) for r in red] == [(r['case'], r['raw']) for r in green]
assert sum(r['old'] != r['bytes'] or r['old'] != r['direct'] for r in red) == 18
assert all(r['old'] == r['bytes'] == r['direct'] for r in green)
results = load(R / 'results.json')
for r in results['runs']:
    check(Path(r['logPath']), {'sha256': r['logSha256']})
for r in results['products']:
    check(Path(r['path']), r)
assert results['coreFullCounts'] == {'passed': 3861, 'failed': 0, 'ignored': 1, 'suites': 13}
assert results['prepatchDifferences'] == 18 and results['postpatchDifferences'] == 0
oracle = (R / 'source/crates/engine-core/src/engine/source_tests/choice_request_4dd.rs').read_text().split('\n', 1)[1]
oracle = oracle.replace('use engine_api::{ChoiceId, PlayerId};', 'use crate::ids::{ChoiceId, PlayerId};')
old = subprocess.run(['git', '--no-optional-locks', '-C', str(P), 'show', '4dd0b286:crates/engine-api/src/choice.rs'], capture_output=True, check=True).stdout
assert oracle.encode() == old

resource = E / 'nc27-resource-gate-materialization'
index = load(resource / 'evidence-sha256.json')
for name, value in index.items():
    check(Path(name), {'sha256': value})
before, after = load(resource / 'before.json'), load(resource / 'after.json')
assert before['pins'] == after['pins']
assert before['ordered_relative_sha256_totals'] == after['ordered_relative_sha256_totals']
assert after['observed_trees'] == after['expected_trees'] and not after['differences']
assert before['differences']
for name in ['nc27-resource-gate-refresh', 'nc27-resource-gate-verify', 'nc27-original-sync-verify-repaired']:
    assert load(E / (name + '.json'))['exit'] == 0
assert load(E / 'v2-original-sync-verify.json')['exit'] == 1
assert sha((E / 'v2-original-sync-verify.log').read_bytes()) == '8b6b91d3e0a4512c733f601b206ca91d4ed4c3d51e13c6593303bb2fa6b16d43'
resource_pairs = {}
for label in ['before', 'after']:
    pairs = [json.loads(x) for x in (resource / (label + '-pairs.jsonl')).read_text().splitlines()]
    totals = {}
    for r in pairs:
        if 'source_tuple' not in r:
            continue
        assert r['source_tuple']['sha256'] == r['target_tuple']['sha256']
        row = totals.setdefault(r['tree'], [0, 0])
        row[0 if r['same_inode'] else 1] += 1
    resource_pairs[label] = totals
assert resource_pairs['before']['portraits'] == [0, 2707]
assert resource_pairs['before']['audio'] == [0, 9652]
assert resource_pairs['after']['portraits'] == [2707, 0]
assert resource_pairs['after']['audio'] == [9652, 0]

main = Path('/tmp/nc27-main-functional-458ed333')
mi = load(main / 'artifact-sha256.json')
assert sha((main / 'artifact-sha256.json').read_bytes()) == '94191f6b13c86a3011ef6ae81153be08c1619f475166b399424af94ffb2f906b'
for name, row in mi.items():
    check(main / name, row)
comparison = load(main / 'comparison.json')
assert comparison['all_equal'] and all(comparison['invariants'].values())
raw = (main / 'native-all-layers.stdout.log').read_bytes()
rows = re.findall(rb'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', raw)
assert tuple(sum(int(r[i]) for r in rows) for i in range(3)) == (1715, 0, 75)
assert len(rows) == 17
output = {'status': 'PASS retained incremental/resource/Main identity and result parsing', 'v3_changed_files': sorted(changed), 'v3_composed_source_count': len(v3_files), 'only_legacy_reader_production_delta': True, 'oracle_reverse_import_exact4dd': True, 'differential_cases': 445, 'prepatch_differences': 18, 'postpatch_differences': 0, 'differential_scope': 'Acceptance/error diagnostics including positions; successful rows do NOT prove all decoded values equal', 'core_full': results['coreFullCounts'], 'p2_logs_fresh_sha': len(results['runs']), 'p2_products_fresh_sha': len(results['products']), 'resource_evidence_fresh_sha': len(index), 'resource_link_pairs': resource_pairs, 'original_resource_exit1_preserved': True, 'resource_repair_three_exits': [0, 0, 0], 'MainV2_evidence_files_fresh_sha': len(mi), 'MainV2_invariants': comparison, 'MainV2_native_counts': [1715, 0, 75], 'qualification': 'No producer/product execution or new tests/build/network; v3 incremental proof is not new whole-workspace acceptance. MainV2 is not MainV3. Original root/NoCert/privacy/quality remain open.'}
(OUT / 'incremental-resource-main-results.json').write_text(json.dumps(output, ensure_ascii=False, indent=2) + '\n')
print(json.dumps({k: output[k] for k in ['status', 'v3_composed_source_count', 'differential_cases', 'prepatch_differences', 'postpatch_differences', 'core_full', 'resource_evidence_fresh_sha', 'MainV2_evidence_files_fresh_sha']}, indent=2))
