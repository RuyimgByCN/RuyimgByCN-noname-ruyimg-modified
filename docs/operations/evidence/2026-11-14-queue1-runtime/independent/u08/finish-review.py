#!/usr/bin/env python3
"""Validate the exclusive evidence directory and publish its final hash manifest."""
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
from urllib.parse import unquote

BASE = Path(__file__).resolve().parent
REPO = BASE.parents[5]
ORIGINAL = BASE.parents[1] / 'U08'


def digest(path):
    data = Path(path).read_bytes()
    return {'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()}


def command(*args):
    return subprocess.check_output(args, cwd=REPO, text=True).strip()


def save(stream, value):
    json.dump(value, stream, ensure_ascii=False, indent=2)
    stream.write('\n')
    stream.flush()


# Reserve new output files in this single finalization; no earlier evidence is overwritten.
with (BASE / 'validation-final.json').open('x') as validation_stream, (BASE / 'freeze-context.json').open('x') as context_stream, (BASE / 'frozen-file-manifest.json').open('x') as manifest_stream:
    initial = json.loads((BASE / 'input-context-first.json').read_text())
    product_sources = [{'path': r['path'], **digest(r['path']), 'matches_start': digest(r['path'])['sha256'] == r['sha256']}
                       for r in initial['product_rust_sources']]
    initial_sources = [{'path': r['path'], 'matches_start': digest(r['path'])['sha256'] == r['sha256']}
                       for r in initial['source_inputs']]
    source_anchors = json.loads((BASE / 'source-anchors.json').read_text())
    anchor_checks = []
    for key, row in source_anchors.items():
        lines = Path(row['path']).read_text().splitlines()
        anchor_checks.append({'key': key, 'path': row['path'], 'line': row['line'],
                              'in_bounds': 1 <= row['line'] <= len(lines),
                              'source_text_matches': lines[row['line'] - 1] == row['source_text'],
                              'source_hash_matches': digest(row['path'])['sha256'] == row['sha256']})
    markdown = sorted(BASE.rglob('*.md'))
    link_checks = []
    for document in markdown:
        for match in re.finditer(r'(?<!!)\[[^\]\n]+\]\(([^)\n]+)\)', document.read_text()):
            target = match.group(1).strip('<>')
            if target.startswith(('https://', 'http://', 'mailto:', '#')):
                continue
            location, _, fragment = target.partition('#')
            resolved = (document.parent / unquote(location)).resolve()
            exists = resolved.exists()
            line = int(fragment[1:]) if re.fullmatch(r'L\d+', fragment) else None
            in_bounds = None
            if line is not None:
                in_bounds = exists and line <= len(resolved.read_text().splitlines())
            link_checks.append({'document': str(document.relative_to(BASE)), 'target': target,
                                'exists': exists, 'line': line, 'line_in_bounds': in_bounds})
    native = json.loads((ORIGINAL / 'native-round2/native-results.json').read_text())
    upstream = json.loads((ORIGINAL / 'browser-round4/results.json').read_text())
    per_case_inputs = []
    for group, rows, prefix in [('native-round2', native, 'native'), ('browser-round4', upstream, 'upstream')]:
        for row in rows:
            path = ORIGINAL / group / f'{prefix}-{row["case"]}.json'
            per_case_inputs.append({'path': str(path), 'aggregate_matches_individual': json.loads(path.read_text()) == row})
    original_manifest_path = ORIGINAL / 'manifest.json'
    original_manifest = json.loads(original_manifest_path.read_text())
    original_files = []
    for row in original_manifest['files']:
        actual = digest(ORIGINAL / row['path'])
        original_files.append({'path': row['path'], **actual, 'matches_frozen_manifest': all(actual[k] == row[k] for k in ('bytes', 'sha256'))})
    pending = json.loads((BASE / 'archive-replay02/pending-suffix-results.json').read_text())
    pending_pass = all(r['input_decoded_reencoded_bytes_exact'] and r['input_full_snapshot_equal_except_auth_generation']
                       and r['restored_choice_equals_recorded_choice'] and r['final_full_snapshot_equal_except_auth_generation']
                       and all(all(step[k] for k in ('request_reconstruction_exact', 'reply_exact', 'log_delta_exact', 'state_hash_exact')) for step in r['suffix_commands']) for r in pending)
    network = [json.loads(line) for line in (ORIGINAL / 'session-u08-1794594672096-network.jsonl').read_text().splitlines()]
    spine_urls = [{'line': i + 1, 'url': row.get('url')} for i, row in enumerate(network) if 'spine' in str(row.get('url', '')).lower()]
    all_links_pass = all(r['exists'] and r['line_in_bounds'] is not False for r in link_checks)
    owned_index = command('git', 'ls-files', '--stage', '--', str(BASE.relative_to(REPO)))
    product_status = command('git', 'status', '--short', '--untracked-files=no', '--', 'crates', 'packages', 'apps', 'scripts')
    global_command = ['python3', '-B', 'scripts/check_local_links.py', '--all']
    with (BASE / 'local-links-all-final.log').open('xb') as stream:
        checked = subprocess.run(global_command, cwd=REPO, stdout=stream, stderr=subprocess.STDOUT)
    validation = {
        'recorded_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'owned_markdown_files': len(markdown), 'owned_local_links': len(link_checks),
        'owned_line_anchors': sum(r['line'] is not None for r in link_checks),
        'owned_broken_links': [r for r in link_checks if not r['exists']],
        'owned_out_of_bounds_line_anchors': [r for r in link_checks if r['line_in_bounds'] is False],
        'link_checks': link_checks, 'source_anchor_checks': anchor_checks,
        'input_product_rust_source_count': len(product_sources),
        'all_product_rust_sources_unchanged': all(r['matches_start'] for r in product_sources),
        'all_initial_declared_sources_unchanged': all(r['matches_start'] for r in initial_sources),
        'all_source_anchor_texts_and_hashes_match': all(r['in_bounds'] and r['source_text_matches'] and r['source_hash_matches'] for r in anchor_checks),
        'all_62_individual_case_files_match_aggregates': all(r['aggregate_matches_individual'] for r in per_case_inputs),
        'per_case_input_checks': per_case_inputs,
        'original_manifest_sha256': digest(original_manifest_path)['sha256'],
        'original_manifest_sha256_matches_notified_freeze': digest(original_manifest_path)['sha256'] == '7934ad115c63fd0ace23d3b3800d1e2703e621554e9db6391035f09a1185205c',
        'original_frozen_file_count': len(original_files), 'all_original_frozen_files_unchanged': all(r['matches_frozen_manifest'] for r in original_files),
        'pending_independently_redecoded_and_suffix_replayed': len(pending),
        'suffix_commands_replayed': sum(len(r['suffix_commands']) for r in pending), 'all_pending_suffix_checks_pass': pending_pass,
        'network_request_count': len(network), 'spine_network_urls': spine_urls,
        'owned_git_index_entries': owned_index, 'tracked_product_status': product_status,
        'global_local_links_command': global_command, 'global_local_links_exit_code': checked.returncode,
        'global_local_links_log': 'local-links-all-final.log',
        'global_doc_line_refs_check_executed': False,
        'line_validation_scope': '本目录所有#L锚点，另逐个核39源码锚点的语义原文与整文件哈希；没有将其称全库file:line扫描。',
        'complete_preflight_executed': False,
    }
    validation['owned_scope_all_pass'] = all_links_pass and validation['all_product_rust_sources_unchanged'] and validation['all_initial_declared_sources_unchanged'] and validation['all_source_anchor_texts_and_hashes_match'] and validation['all_62_individual_case_files_match_aggregates'] and validation['all_original_frozen_files_unchanged'] and validation['original_manifest_sha256_matches_notified_freeze'] and pending_pass and not owned_index and not product_status and not spine_urls
    save(validation_stream, validation)
    binaries = [Path('/private/tmp/noname-u08-runtime-preserved-edoyg7eg/native-probe-round2'), REPO / 'target/debug/u08-independent-archive-review']
    context = {
        'frozen_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'head': command('git', 'rev-parse', 'HEAD'),
        'collection_head_preserved': '605201f3f49e10cea61c31d52a61bbb66c85599d', 'collection_source_snapshot_had_dirty_jiushi_changes': True,
        'independent_native_execution_and_freeze_head': '5c73d1d9158145e33f98f2018013914667d13964',
        'versions': initial['version_axes'], 'rustc': command('rustc', '--version'), 'cargo': command('cargo', '--version'),
        'tracked_product_status': product_status, 'all_worktree_status': command('git', 'status', '--short'),
        'source_hash_checks': initial_sources, 'product_rust_sources': product_sources,
        'binaries': [{'path': str(path), **digest(path)} for path in binaries],
        'private_cargo_target_created': False, 'shared_cargo_target': str(REPO / 'target'), 'shared_cargo_target_cleaned': False,
        'disk_free_bytes': shutil.disk_usage(REPO).free,
        'commands': [
            ['python3', '-B', str(BASE / 'write-review.py')],
            ['python3', '-B', str(BASE / 'finish-review.py')],
            global_command,
        ],
        'archive_compile_failure_preserved': {'log': 'archive-replay01.raw.log', 'source': 'archive-probe-first-source.rs', 'reason': 'json! macro requires parentheses around chained array expression'},
        'archive_compile_and_run_success': {'log': 'archive-replay02.raw.log', 'pending_archives': 76, 'suffix_commands': 134},
        'recompute_metadata_correction_preserved': {'original': 'recompute-first-source.py', 'corrected': 'recompute.py', 'detail': 'seat bool output collision corrected to seat index plus seat_matches; first outputs retained'},
        'original_frozen_inputs': {'manifest': str(original_manifest_path), **digest(original_manifest_path), 'files': len(original_files), 'all_match': all(r['matches_frozen_manifest'] for r in original_files)},
        'owned_git_index_entries': owned_index, 'new_equivalence_signoffs': 0,
        'not_executed': ['browser rerun', 'complete preflight', 'product three-crate/workspace acceptance', 'other platforms', 'product UI acceptance', 'Spine library access', 'inventory edit', 'product edit', 'Git stage/commit/push/ref mutation'],
        'freeze_boundary': '本目录现有文件全部冻结；manifest自身仅自排除。原U08与L10目录未由本agent写入。',
    }
    save(context_stream, context)
    assert validation['owned_scope_all_pass'], 'Owned scope validation failed; preserve outputs and coordinate before further writing.'
    rows = []
    for path in sorted(BASE.rglob('*')):
        if path.is_file() and path != BASE / 'frozen-file-manifest.json':
            rows.append({'path': str(path.relative_to(BASE)), **digest(path)})
    manifest = {'frozen_at_utc': context['frozen_at_utc'], 'head': context['head'], 'files': rows,
                'file_count': len(rows), 'total_bytes': sum(r['bytes'] for r in rows),
                'excluded_paths': ['frozen-file-manifest.json'], 'all_existing_files_frozen': True}
    save(manifest_stream, manifest)
    assert all(digest(BASE / r['path']) == {k: r[k] for k in ('bytes', 'sha256')} for r in rows)
    print(json.dumps({'head': context['head'], 'owned_scope_all_pass': validation['owned_scope_all_pass'],
                      'markdown': len(markdown), 'local_links': len(link_checks), 'line_anchors': validation['owned_line_anchors'],
                      'global_local_links_exit_code': checked.returncode, 'files': len(rows), 'bytes': manifest['total_bytes'],
                      'frozen_manifest_sha256': digest(BASE / 'frozen-file-manifest.json')['sha256']}, ensure_ascii=False))
