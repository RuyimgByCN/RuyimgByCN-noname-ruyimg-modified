import hashlib, json, pathlib, subprocess
OUT = pathlib.Path(__file__).resolve().parent
ROOT = pathlib.Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
ACTUAL = pathlib.Path('/tmp/pa09-webview-5b7ad3ab')
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def load(p): return json.loads(p.read_text())
paths = load(OUT / 'paths.json')
freeze = load(ACTUAL / 'evidence/actual-freeze-manifest.json')
admission = load(OUT.parent / 'parent-main-source-admission.json')
assert {x['path']: x['sha256'] for x in freeze['source3']} == admission['source3']
source_git_blobs = {}
for row in freeze['source3']:
    p = row['path']
    original = subprocess.check_output(['git', '--no-optional-locks', 'rev-parse', 'HEAD:' + p], cwd=ROOT, text=True).strip()
    current = subprocess.check_output(['git', '--no-optional-locks', 'hash-object', '--no-filters', p], cwd=ROOT, text=True).strip()
    assert original == admission['original_blobs'][p] == row['old_git_blob_sha1']
    assert current == row['new_git_blob_sha1']
    source_git_blobs[p] = {'base': original, 'current': current}
source_bindings = {}
for p in ['src/engine/client.ts', 'src/engine/desktopEngineTransport.ts', 'src/engine/desktopHost.ts', 'src/engine/autoDrive.ts', 'src/engine/nativeOutcome.ts', 'src/engine/observerMetadata.ts', 'src/engine/perfProbe.ts', 'src/types/engine.ts']:
    main_sha = sha(ROOT / 'apps/web-client' / p)
    actual_sha = sha(ACTUAL / 'product-web' / p)
    expected = freeze['source_and_bundle_files_sha256']['product-web/' + p]
    assert main_sha == actual_sha == expected, p
    source_bindings[p] = main_sha
inherited_files = {}
for p in ['actual-webview-report.json', 'raw-ipc.ndjson', 'transport-admission.ndjson', 'pending-counts.ndjson', 'native-source-baseline-proof.json', 'bounded-result.json', 'retained-failures.json']:
    actual_sha = sha(ACTUAL / 'evidence' / p)
    assert actual_sha == freeze['evidence_sha256'][p], p
    inherited_files[p] = actual_sha
assert not load(OUT / 'native-inheritance.json')['rust_cargo_changed_paths']
products = pathlib.Path(paths['products'])
products_sha = {str(p.relative_to(products)): sha(p) for p in products.rglob('*') if p.is_file()}
owninput = pathlib.Path(paths['owninput'])
assert load(OUT / 'owninput-sha256.json') == {str(p.relative_to(owninput)): sha(p) for p in owninput.rglob('*') if p.is_file()}
assert products_sha['assets/engine_native_wasm_bg-u6T604pW.wasm'] == '2292d7ab1c04b05724e49eb750962bb78af119dac80011524b18574742ed189b'
assert products_sha['assets/engine_wasm_bg-Dcf9D3BU.wasm'] == '2b1cc44e511484c804f10640429b32075fa525227f5d07468bdf5f9dc1fab355'
table = load(OUT / 'bundle-module-table.json')
transport_chunks = [name for name, chunk in table['chunks'].items() if any(p.endswith('/src/engine/desktopEngineTransport.ts') for p in chunk.get('modules', []))]
assert len(transport_chunks) == 1
chunk_code = (products / transport_chunks[0]).read_text()
assert 'Desktop engine busy; request was not submitted' in chunk_code
for name, chunk in table['chunks'].items():
    if chunk['type'] == 'chunk': assert sha(products / name) == chunk['outputSha256']
for p, record in table['sourceModules'].items(): assert sha(ROOT / 'apps/web-client' / p) == record['sourceSha256'], p
result = {'source_git_blobs': source_git_blobs, 'actual_source3_matches_admission': True, 'inherited_source_bindings': source_bindings, 'inherited_evidence_sha256': inherited_files, 'private_actual_freeze_manifest_sha256': sha(ACTUAL / 'evidence/actual-freeze-manifest.json'), 'native_main_vs_a1f_changed_paths': [], 'transport_chunks': transport_chunks, 'production_files': len(products_sha), 'production_source_modules': len(table['sourceModules']), 'native_wasm_exact': True, 'legacy_wasm_exact': True, 'owninput_unchanged': True, 'new_main_gui': False, 'qualification': 'Inherited private hidden actual WK same-event-turn admission/lifecycle only; not a fresh Main App/GUI run. Main compat changed originalResources and other non-seam production JS; those are built here but not rerun in WK.'}
(OUT / 'production-products-sha256.json').write_text(json.dumps({'root': str(products), 'files': products_sha, 'canonical_files_digest': hashlib.sha256(json.dumps(products_sha, sort_keys=True, separators=(',', ':')).encode()).hexdigest()}, indent=2) + '\n')
(OUT / 'bindings-result.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
print(json.dumps(result, ensure_ascii=False))
