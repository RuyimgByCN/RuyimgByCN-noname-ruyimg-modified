import datetime
import hashlib
import json
from pathlib import Path
import re

OUT = Path(__file__).resolve().parent
ANSI = re.compile(r'\x1b\[[0-?]*[ -/]*[@-~]')
RUST = re.compile(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;')
state = json.loads((OUT / 'observation-state.json').read_text())
assert state['completed'] and state['run_status'] == 'completed' and state['run_conclusion'] == 'success'
assert len(state['jobs']) == 10 and not state['pending_jobs'] and not state.get('error')
assert all(j['conclusion'] == 'success' and not j['failed_steps'] and 'raw_log' in j and not j.get('log_error') for j in state['jobs'])
assert next(c for c in state['commands'] if c['name'] == 'watch')['exit'] == 0
summary = {'run_id': state['run_id'], 'head_sha': state['head_sha'], 'run_url': state['run_url'], 'status': state['run_status'], 'conclusion': state['run_conclusion'], 'watch_exit': 0, 'jobs': [], 'artifacts': state['artifacts'], 'cleanup': state['cleanup'], 'failed_tests': [], 'transport_errors': [c for c in state['commands'] if c.get('exit') != 0]}

def section(lines, marker):
    start = next(i for i, line in enumerate(lines) if line.startswith('##[group]Run ' + marker))
    end = next((i for i in range(start + 1, len(lines)) if lines[i].startswith('##[group]Run ')), len(lines))
    return lines[start:end]

for job in state['jobs']:
    raw = (OUT / job['raw_log']).read_bytes()
    lines = [ANSI.sub('', line.split(' ', 1)[1]) for line in raw.decode(errors='replace').splitlines() if ' ' in line]
    results = [tuple(map(int, m.groups())) for line in lines if (m := RUST.search(line))]
    start = datetime.datetime.fromisoformat(job['started_at'].replace('Z', '+00:00'))
    end = datetime.datetime.fromisoformat(job['completed_at'].replace('Z', '+00:00'))
    item = {key: job[key] for key in ('id', 'name', 'html_url', 'status', 'conclusion', 'log_bytes', 'log_crlf_count', 'failed_steps', 'skipped_steps', 'raw_log')}
    item['duration_seconds'] = (end - start).total_seconds()
    item['raw_log_sha256'] = hashlib.sha256(raw).hexdigest()
    if results:
        item['rust_test_results'] = dict(zip(('passed', 'failed', 'ignored', 'measured', 'filtered_out'), map(sum, zip(*results))))
    if job['name'] == 'native-web':
        worker = section(lines, 'pnpm smoke:native')
        names = [line.split(' | ', 1)[0] for line in worker if line.startswith('PASS ')]
        assert len(names) == len(set(names)) == 82
        assert not any(line.startswith('FAIL ') for line in worker)
        item['worker_checks'] = {'passed': len(names), 'failed': 0, 'names': names}
        frontend = section(lines, 'pnpm test')
        item['frontend_tap_summary'] = [line for line in frontend if re.match(r'^# (tests|pass|fail|skipped|todo) ', line)]
        assert '# tests 163' in item['frontend_tap_summary'] and '# fail 0' in item['frontend_tap_summary']
        assert item['rust_test_results']['passed'] == 1815 and item['rust_test_results']['ignored'] == 75
    if job['name'] == 'gates-ubuntu':
        assert item['rust_test_results'] == {'passed': 6894, 'failed': 0, 'ignored': 77, 'measured': 0, 'filtered_out': 0}
        conformance = section(lines, 'cargo run -p conformance')
        item['conformance_cases'] = [line.strip() for line in conformance if re.match(r'^\s*OK\s', line)]
        assert len(item['conformance_cases']) == 19 and 'ALL PASS' in conformance
        cross = section(lines, 'bash scripts/cross-target-hash.sh')
        assert 'ALL CROSS-TARGET CHECKS PASSED' in cross
        item['cross_target_result'] = 'ALL CROSS-TARGET CHECKS PASSED'
        verify = section(lines, 'node --test scripts/test_original_content_order.mjs scripts/test_original_resources.mjs')
        item['original_verify_summary'] = [line for line in verify if re.match(r'^# (tests|pass|fail|skipped) ', line)]
        at = verify.index('original content verified:')
        item['original_content_verified'] = verify[at:at + 7]
        assert '# tests 2' in item['original_verify_summary'] and '# fail 0' in item['original_verify_summary']
    if job['name'].startswith('build-'):
        assert item['rust_test_results'] == {'passed': 838, 'failed': 0, 'ignored': 0, 'measured': 0, 'filtered_out': 0}
    if job['name'].startswith('perf-'):
        metrics = []
        for line in lines:
            match = re.search(r'✓\s+(start-hash-p50|distance-query-p95|snapshot-restore|observer-projection)\s+(\d+) / (\d+) us', line)
            if match:
                metrics.append({'name': match[1], 'actual_us': int(match[2]), 'budget_us': int(match[3])})
        assert len(metrics) == 4 and all(m['actual_us'] <= m['budget_us'] for m in metrics) and 'result=PASS' in lines
        item['tier'] = 'server-baseline'
        item['metrics'] = metrics
    summary['jobs'].append(item)
assert sorted(a['name'] for a in summary['artifacts']['artifacts']) == ['installer-macos-latest', 'installer-ubuntu-latest', 'installer-windows-latest']
assert all(a['workflow_run']['head_sha'] == state['head_sha'] and a['workflow_run']['id'] == state['run_id'] and not a['expired'] for a in summary['artifacts']['artifacts'])
assert sum(len(j['skipped_steps']) for j in summary['jobs']) == 2
assert all(s['name'] == 'Install Linux Tauri dependencies' for j in summary['jobs'] for s in j['skipped_steps'])
assert not summary['transport_errors']
with (OUT / 'verified-ci-summary.json').open('x') as f:
    json.dump(summary, f, indent=2)
    f.write('\n')
manifest = [{'path': p.name, 'bytes': p.stat().st_size, 'sha256': hashlib.sha256(p.read_bytes()).hexdigest()} for p in sorted(OUT.iterdir()) if p.is_file()]
with (OUT / 'evidence-file-manifest.json').open('x') as f:
    json.dump(manifest, f, indent=2)
    f.write('\n')
print(json.dumps({'run': state['run_id'], 'jobs': len(summary['jobs']), 'passed_jobs': 10, 'failed_jobs': 0, 'skipped_steps': 2, 'artifacts': len(summary['artifacts']['artifacts']), 'raw_log_bytes': sum(j['log_bytes'] for j in summary['jobs']), 'file_count': len(manifest)}, indent=2))
