import datetime, hashlib, importlib.util, json, pathlib, re, subprocess
OUT = pathlib.Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('pa09_accept', OUT / 'accept.py')
assert spec is not None and spec.loader is not None
accept = importlib.util.module_from_spec(spec); spec.loader.exec_module(accept)
commands = json.loads((OUT / 'commands.json').read_text())
expected = {'targeted', 'frontend', 'typecheck', 'vite', 'sourcecap-unit', 'sourcecap', 'diffcheck', 'bindings'}
latest = {record['name']: record for record in commands}
assert set(latest) == expected and all(x['exit'] == 0 for x in latest.values())
with (OUT / 'exit.log').open('x') as log:
    subprocess.run(['python3', '-B', str(OUT / 'accept.py'), 'exit'], cwd=accept.ROOT, env=accept.ENV, stdout=log, stderr=subprocess.STDOUT, check=True)
counts = {}
for name in ['targeted', 'frontend']:
    text = (OUT / (name + '.log')).read_text()
    counts[name] = {}
    for key in ['tests', 'pass', 'fail', 'cancelled', 'skipped', 'todo']:
        match = re.search(r'ℹ ' + key + r' (\d+)', text)
        assert match is not None, f'{name}: missing {key}'
        counts[name][key] = int(match.group(1))
assert counts['targeted']['pass'] == 20 and counts['frontend']['pass'] == 175
state = json.loads((OUT / 'exit.json').read_text())
binding = json.loads((OUT / 'bindings-result.json').read_text())
products = json.loads((OUT / 'production-products-sha256.json').read_text())
initial_products = json.loads((OUT / 'initial-products-sha256.json').read_text())
assert products['files'] == initial_products, 'Evidence-hook correction must not alter product output'
summary = {'utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'head': state['head'], 'versions': '330/105/758/WS6; desktop transport default WASM', 'scope': 'Main JS admission three-file integration only', 'counts': counts, 'commands_passed': sum(x['exit'] == 0 for x in commands), 'commands_recorded': len(commands), 'latest_command_groups_passed': len(latest), 'bundle_hook_correction_products_identical': True, 'no_staged_files': not state['staged'], 'source3_sha256': state['source3'], 'module_count': len(state['modules']), 'unchanged_main_dist_file_count': len(state['dist']), 'protected_count': len(state['protected']), 'narrow_comparison': json.loads((OUT / 'narrow-comparison.json').read_text()), 'production_root': products['root'], 'production_file_count': len(products['files']), 'production_files_digest': products['canonical_files_digest'], 'transport_chunks': binding['transport_chunks'], 'inherited_actual_webview': 'Existing private WK same-event-turn admission/lifecycle is inherited through identical source3 + eight source seams + unchanged a1f-to-Main Rust/Cargo. No new Main App/GUI run.', 'retained_attempts': json.loads((OUT / 'retained-setup-failures.json').read_text()), 'deferred': ['Parent fullpreflight after docs closure', 'new physical input or latency', 'new Main GUI/actual WK', 'Rust/native/WASM/Swift rebuild', 'fullmedia/resource resync', 'default transport switch', 'PA09 six-conditions closure'], 'warnings': 'Node localStorage ExperimentalWarning retained in targeted/frontend raw logs; no warning filtering. This run has no deprecated/6hint output; original historical evidence remains untouched.'}
accept.save('main-summary.json', summary)
# Every evidence file is indexed after the summary; the SHA index excludes itself.
index = {str(p.relative_to(OUT)): {'sha256': accept.sha(p), 'bytes': p.stat().st_size} for p in sorted(OUT.rglob('*')) if p.is_file() and p.name != 'artifact-sha256.json'}
accept.save('artifact-sha256.json', {'scope': 'Main acceptance evidence only; self excluded', 'files': index, 'external_products_manifest': str(OUT / 'production-products-sha256.json'), 'external_inputs_manifest': str(OUT / 'owninput-sha256.json')})
assert all(accept.sha(OUT / p) == record['sha256'] for p, record in index.items())
print(json.dumps({'summary': str(OUT / 'main-summary.json'), 'evidence_files': len(index), 'all_checks_passed': True, 'no_staged_files': not state['staged'], 'production_digest': products['canonical_files_digest']}, ensure_ascii=False))
