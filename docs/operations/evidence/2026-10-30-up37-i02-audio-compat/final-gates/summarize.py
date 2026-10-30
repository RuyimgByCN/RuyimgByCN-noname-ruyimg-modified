"""Summarize only this run; never modify functional acceptance evidence."""
import ast
import datetime
import hashlib
import json
from pathlib import Path

OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[4]

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def read(name):
    return json.loads((OUT / name).read_text())

def main():
    commands = read('commands.json')
    before, after = read('narrow-entry.json'), read('narrow-exit.json')
    comparison = read('source-comparison.json')
    assert len(commands) == 4 and all(c['status'] != 'running' for c in commands)
    assert (OUT / 'driver.stdout.raw.txt').read_text().endswith('DONE\n')
    assert not (OUT / 'driver.stderr.raw.txt').read_text()
    preflight = (OUT / 'full-preflight.stdout.raw.txt').read_text()
    checks = [line for line in preflight.splitlines() if line.startswith(('✅', '❌', '⏭'))]
    tree = ast.parse((ROOT / 'scripts/preflight.py').read_text())
    configured = next(node.value for node in tree.body if isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name) and node.target.id == 'CHECKS')
    assert isinstance(configured, ast.List) and len(configured.elts) == 21
    assert commands[-1]['argv'] == ['python3', '-B', 'scripts/preflight.py']
    assert not any(line.startswith('⏭') for line in checks)
    passed = comparison['all_equal'] and all(c['status'] == 'PASS' for c in commands) and len(checks) == 21 and all(line.startswith('✅') for line in checks)
    admission = json.loads((OUT.parent / 'parent-main-source-admission.json').read_text())
    temporary_patch = Path(admission['source_only_patch_read'])
    summary = {
        'result': 'PASS' if passed else 'FAIL',
        'completed_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'head': after['head'], 'versions': {'rules': 330, 'archive': 105, 'ABI': 758, 'WS': 6, 'asset_schema': 2, 'catalog_schema': 4},
        'scope': 'compat_slice only; does not close whole I02 / 37 sources / Goal / default switch',
        'commands': 'commands.json', 'mandatory_gates': 4, 'actual_gates': len(commands),
        'passed_gates': sum(c['status'] == 'PASS' for c in commands),
        'failed_gates': [c['name'] for c in commands if c['status'] != 'PASS'],
        'preflight': {'default_argv': commands[-1]['argv'], 'configured_checks': 21, 'reported_checks': len(checks), 'checks': checks, 'skips': 0, 'retries': 0,
                      'raw_stdout': 'full-preflight.stdout.raw.txt', 'raw_stderr': 'full-preflight.stderr.raw.txt',
                      'nested_capture_limit': 'Unmodified preflight.py captures child stdout/stderr internally and prints only check status/failure tails; this run preserves all bytes that its default interface actually emitted, without claiming suppressed child output was collected.'},
        'source_comparison': 'source-comparison.json', 'entry': 'narrow-entry.json', 'exit': 'narrow-exit.json',
        'identity_equal': comparison['all_equal'], 'source14_exact': after['source14'] == admission['source14'],
        'source14_aggregate_sha256': after['source14_aggregate_sha256'],
        'admitted_total_patch_sha256': admission['patch_sha256'],
        'temporary_patch_pointer': {'path': str(temporary_patch), 'sha256_observed': digest(temporary_patch) if temporary_patch.exists() else None,
                                    'qualification': 'Temporary source-only patch pointer is not the immutable total patch record. Current SHA differs from admitted total patch SHA; source14 individual SHA and aggregate are the authority for this run. No patch file was modified.'},
        'protected_files': len(after['protected']), 'module_files': len(after['modules']), 'default_dist_files': len(after['dist']),
        'native_wasm_sha256': after['modules']['apps/web-client/src/worker/native-pkg/engine_native_wasm_bg.wasm'],
        'legacy_wasm_sha256': after['modules']['apps/web-client/src/wasm-pkg/engine_wasm_bg.wasm'],
        'parent_docs_baseline': 'Current entry-to-exit equality; earlier authorized parent docs changes are recorded in narrow-entry.json and are not treated as new drift.',
        'historical_parent_docs_deltas': after['accepted_prior_parent_docs_delta'],
        'functional_artifacts_reverified_readonly': len(after['functional_artifacts']),
        'functional_artifact_index_unchanged': before['functional_artifact_index_sha256'] == after['functional_artifact_index_sha256'],
        'functional_execution_repeated': False,
        'source_line_tests': {'passed': 2, 'failed': 0}, 'source_line_files': 8992,
        'strict_docrefs': {'references': 1065, 'missing_file_hint_classes': 6, 'out_of_range': 0, 'stderr_preserved': 'strict-docrefs.stderr.raw.txt'},
        'warnings_and_errors': {'all_emitted_stdout_stderr_preserved': True, 'strict_missing_hints_retained': True,
                                'no_normalization_or_retry_after_failure': True, 'earlier_resource_warnings_ignored_tests_transport_queue_errors': 'Historical raw evidence remains untouched. No old failure or ignored test is relabeled as passed by this run.'},
        'timing': {'locked_seconds_total': sum(c['locked_seconds'] or 0 for c in commands), 'wall_seconds_total': sum(c['wall_seconds'] for c in commands),
                   'queue_estimate_seconds_total': sum(c['queue_estimate_seconds'] or 0 for c in commands), 'queue_estimate_includes_wrapper_startup': True,
                   'hard_driver_deadline_seconds': 1200},
        'global_wrapper': {'path': '/tmp/noname-run-serialized.py', 'sha256': digest(Path('/tmp/noname-run-serialized.py')), 'CARGO_INCREMENTAL': '0', 'CARGO_BUILD_JOBS': '2'},
        'free_disk_min_observed_bytes': min([before['free_disk_bytes'], after['free_disk_bytes']] + [c[k] for c in commands for k in ['free_disk_before_bytes','free_disk_after_bytes']]),
        'minimum_required_free_bytes': 3 * 1024**3,
        'index_empty': not after['index'], 'index_bytes_unchanged': before['index_sha256'] == after['index_sha256'],
        'no_source_or_docs_body_edits': True, 'guard_unchanged_and_disabled': before['guard'] == after['guard'],
        'no_new_build_or_media_collection': True,
        'not_run': ['Cargo build/native3/workspace tests', 'Rust/Swift/WASM/ffmpeg compilation', '12000 codec audit', 'new media/HTTP/browser collection', 'extra 17000-source or all-asset body hash scan', 'new remote CI', 'stage/commit/push/default switch'],
        'inherited_limits': ['decoded bytes=0 was read after decodeAudioData detached the ArrayBuffer; pre-decode assertions plus independent17HTTP identities establish real lengths; raw unchanged', 'Original raw5 failures retained; no media rerun', 'Chrome extra2112 frames/44ms versus WK remains; no cross-decoder PCM/physical listening claim', 'Original generated resource tree relink authorization remains valid; no relink performed by this task'],
        'review_gate': 'independent reviewer required',
    }
    (OUT / 'summary.json').write_text(json.dumps(summary, ensure_ascii=False, indent=2) + '\n')
    artifacts = {str(p.relative_to(OUT)): digest(p) for p in sorted(OUT.rglob('*')) if p.is_file() and p.name != 'artifact-sha256.json'}
    (OUT / 'artifact-sha256.json').write_text(json.dumps(artifacts, ensure_ascii=False, indent=2) + '\n')
    assert all(digest(OUT / p) == h for p,h in artifacts.items())
    print(json.dumps({'result': summary['result'], 'commands': len(commands), 'preflight_checks': len(checks), 'identity_equal': comparison['all_equal'], 'artifacts': len(artifacts)}, ensure_ascii=False))

if __name__ == '__main__':
    main()
