from pathlib import Path
import collections
import hashlib
import json
import re

ROOT = Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-ci-da7e-observation-lkxhc3le')
OUT = Path('/tmp/parent-ci41082599800-review')
OUT.mkdir(exist_ok=True)
manifest = json.loads((ROOT / 'evidence-file-manifest.json').read_text())
assert len(manifest) == 43
actual_files = {path.name for path in ROOT.iterdir() if path.is_file()}
expected_files = {item['path'] for item in manifest}
assert len(actual_files) == 44 and actual_files == expected_files | {'evidence-file-manifest.json'}
checked = []
for item in manifest:
    path = ROOT / item['path']
    raw = path.read_bytes()
    actual = hashlib.sha256(raw).hexdigest()
    assert len(raw) == item['bytes'] and actual == item['sha256'], item['path']
    checked.append({'path': item['path'], 'bytes': len(raw), 'sha256': actual, 'equal': True})
data = json.loads((ROOT / 'jobs-final.stdout').read_text())
assert data['total_count'] == len(data['jobs']) == 10
assert len({job['id'] for job in data['jobs']}) == 10
pattern = re.compile(r'test result: ([\w-]+)\.\s+(\d+) passed;\s+(\d+) failed;\s+(\d+) ignored;\s+(\d+) measured;\s+(\d+) filtered out;')
jobs = []
for job in data['jobs']:
    assert job['run_id'] == 41082599800 and job['run_attempt'] == 1
    assert job['head_sha'] == 'da7e4c019461a6c467a9dd2de849f3b1a7494b86'
    assert job['head_branch'] == 'native-convergence'
    assert job['status'] == 'completed' and job['conclusion'] == 'success'
    assert all(step['status'] == 'completed' and step['conclusion'] in {'success', 'skipped'} for step in job['steps'])
    log = ROOT / f"job-{job['id']}-raw-log.stdout"
    raw = log.read_bytes()
    matches = pattern.findall(raw.decode('utf-8', errors='replace'))
    totals = dict(zip(['passed', 'failed', 'ignored', 'measured', 'filtered'], [sum(int(match[index]) for match in matches) for index in range(1, 6)]))
    assert totals['failed'] == 0 and all(match[0] == 'ok' for match in matches)
    jobs.append({'id': job['id'], 'name': job['name'], 'conclusion': job['conclusion'], 'step_counts': dict(collections.Counter(step['conclusion'] for step in job['steps'])), 'skipped_steps': [step['name'] for step in job['steps'] if step['conclusion'] == 'skipped'], 'rust_test_result_lines': len(matches), 'rust_totals': totals, 'raw_log_bytes': len(raw), 'raw_log_crlf_sequences': raw.count(b'\r\n')})
state = json.loads((ROOT / 'observation-state.json').read_text())
watch = [row for row in state['commands'] if row['name'] == 'watch']
assert len(watch) == 1 and watch[0]['exit'] == 0
assert watch[0]['command'] == ['gh', 'run', 'watch', '41082599800', '--repo', 'RuyimgByCN/RuyimgByCN-noname-ruyimg-modified', '--exit-status', '--interval', '30', '--compact']
report = {'read_only': True, 'no_live_api_build_or_producer_checker_run': True, 'manifest_entries': 43, 'actual_files': 44, 'hash_checks': checked, 'jobs': jobs, 'watch': watch[0], 'cleanup': state['cleanup'], 'all_assertions_passed': True, 'rust_count_scope': 'Each job independently sums its raw Rust test-result lines; jobs overlap and are not added as unique tests. JS/Node/Worker counts reviewed separately.'}
(OUT / 'independent-results.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps({'all_assertions_passed': True, 'manifest_entries': 43, 'actual_files': 44, 'jobs': jobs, 'watch': watch[0], 'cleanup': state['cleanup'], 'result': str(OUT / 'independent-results.json')}, indent=2))
