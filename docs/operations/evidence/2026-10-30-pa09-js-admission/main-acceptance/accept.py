import datetime, hashlib, json, os, pathlib, shutil, subprocess, sys, tempfile, time
ROOT = pathlib.Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
OUT = ROOT / 'docs/operations/evidence/2026-10-30-pa09-js-admission/main-acceptance'
WEB = ROOT / 'apps/web-client'
ENV = {**os.environ, 'GIT_OPTIONAL_LOCKS': '0', 'GIT_NO_LAZY_FETCH': '1', 'CARGO_INCREMENTAL': '0', 'CARGO_BUILD_JOBS': '2'}
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def save(n, v): (OUT / n).write_text(json.dumps(v, ensure_ascii=False, indent=2) + '\n')
def git(*args): return subprocess.check_output(['git', '--no-optional-locks', *args], cwd=ROOT, env=ENV)
def load(p): return json.loads((ROOT / p).read_text())
def files(d): return {str(p.relative_to(ROOT)): sha(p) for p in (ROOT / d).rglob('*') if p.is_file()}
def snapshot():
    admission = load('docs/operations/evidence/2026-10-30-pa09-js-admission/parent-main-source-admission.json')
    compat = load('docs/operations/evidence/2026-10-30-up37-i02-audio-compat/parent-main-source-admission.json')['source14']
    a53 = load('docs/operations/evidence/2026-10-30-adr0787-timeout/parent-source-admission.json')['runtime_paths_applied']
    ci = load('docs/operations/evidence/2026-10-30-native-ci-corrections/parent-source-admission.json')['paths']
    protected = set(compat) | set(a53) | {p for p in ci if not p.startswith('docs/')}
    protected |= {'apps/web-client/src/engine/client.ts', 'apps/web-client/src/engine/desktopHost.ts', 'apps/web-client/src/engine/autoDrive.ts', 'apps/web-client/src/engine/nativeOutcome.ts', 'apps/web-client/scripts/pa09-native-host-smoke.mjs', 'apps/web-client/scripts/pa09-desktop-webview.ts', 'apps/web-client/scripts/pa09-document-reload.ts', 'apps/web-client/vite.config.ts', 'apps/web-client/scripts/ts-register.mjs', 'apps/web-client/scripts/ts-hook.mjs', 'content/original/catalog.json', 'Cargo.lock', 'Cargo.toml', '.pi-lens.json'}
    for base in [ROOT, WEB]:
        protected.update(str(p.relative_to(ROOT)) for pattern in ['*lock*', '*package.json', 'pnpm-workspace.yaml'] for p in base.glob(pattern) if p.is_file())
    # Main client is src/engine/client.ts; no mainClient.ts exists.
    client_callers = subprocess.check_output(['rg', '-l', 'from ["\'].*/(engine/)?client["\']|new EngineClient|DesktopEngineTransport|desktopEngineChoice', 'apps/web-client/src', 'apps/web-client/scripts'], cwd=ROOT, env=ENV, text=True).splitlines()
    protected.update(client_callers)
    protected.update(admission['source3'])
    index_path = pathlib.Path(git('rev-parse', '--git-path', 'index').decode().strip())
    if not index_path.is_absolute(): index_path = ROOT / index_path
    state = {'utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'head': git('rev-parse', 'HEAD').decode().strip(), 'tree': git('rev-parse', 'HEAD^{tree}').decode().strip(), 'index_sha256': sha(index_path), 'staged': git('diff', '--cached', '--name-only').decode().splitlines(), 'tracked_status': git('status', '--porcelain=v1', '--untracked-files=no').decode(), 'source3': {p: sha(ROOT / p) for p in admission['source3']}, 'protected': {p: sha(ROOT / p) for p in sorted(protected)}, 'groups': {'compat14': list(compat), 'A53': list(a53), 'CI4_groups_paths': [p for p in ci if not p.startswith('docs/')], 'client_and_transport_callers': client_callers}, 'modules': {}, 'dist': files('apps/web-client/dist'), 'guard': json.loads((ROOT / '.pi-lens.json').read_text()), 'frozen_package_diff_sha256': hashlib.sha256(git('diff', 'HEAD', '--', 'content/original/ported', 'content/original/legacy-source', 'packages/extensions', 'Cargo.lock')).hexdigest(), 'package_git_trees': {p: git('rev-parse', f'HEAD:{p}').decode().strip() for p in ['content/original/ported', 'content/original/legacy-source', 'packages/extensions']}, 'free_disk_bytes': shutil.disk_usage(ROOT).free}
    for d in ['apps/web-client/src/worker/native-pkg', 'apps/web-client/src/wasm-pkg', 'apps/web-client/public/pkg']: state['modules'].update(files(d))
    assert state['source3'] == admission['source3']
    assert state['head'] == admission['base'] and not state['staged']
    assert state['free_disk_bytes'] >= 3 * 1024**3
    g = state['guard']; assert not g['autofix']['enabled'] and not g['format']['enabled'] and not g['actionableWarnings']['autoFix']['enabled']
    assert state['modules']['apps/web-client/src/worker/native-pkg/engine_native_wasm_bg.wasm'] == '2292d7ab1c04b05724e49eb750962bb78af119dac80011524b18574742ed189b'
    assert state['modules']['apps/web-client/src/wasm-pkg/engine_wasm_bg.wasm'] == '2b1cc44e511484c804f10640429b32075fa525227f5d07468bdf5f9dc1fab355'
    return state
COMMANDS = {
    'targeted': (['node', '--import', './scripts/ts-register.mjs', '--test', 'scripts/pa09-desktop-transport.test.mjs', 'scripts/nc04-worker-failure.test.mjs'], WEB),
    'frontend': (['node', '--import', './scripts/ts-register.mjs', '--test'], WEB),
    'typecheck': (['node', 'node_modules/vue-tsc/bin/vue-tsc.js', '--noEmit'], WEB),
    'vite': (['node', str(OUT / 'vite-build.mjs')], WEB),
    'sourcecap-unit': (['python3', '-B', '-m', 'unittest', 'discover', '-s', 'scripts', '-p', 'test_check_source_lines.py'], ROOT),
    'sourcecap': (['python3', '-B', 'scripts/check-source-lines.py'], ROOT),
    'bindings': (['python3', '-B', str(OUT / 'check-bindings.py')], ROOT),
    'diffcheck': (['git', '--no-optional-locks', 'diff', '--check'], ROOT),
}
def run(name):
    cmd, cwd = COMMANDS[name]
    assert shutil.disk_usage(ROOT).free >= 3 * 1024**3
    actual = ['python3', '-B', '/tmp/noname-run-serialized.py', *cmd]
    log_name = name + '.log'
    attempt = 1
    while (OUT / log_name).exists():
        attempt += 1
        log_name = f'{name}-attempt{attempt}.log'
    start = time.monotonic()
    with (OUT / log_name).open('x') as f:
        p = subprocess.run(actual, cwd=cwd, env=ENV, stdout=f, stderr=subprocess.STDOUT)
    wall = time.monotonic() - start
    last = json.loads((OUT / log_name).read_text().strip().splitlines()[-1])
    record = {'name': name, 'attempt': attempt, 'argv': actual, 'cwd': str(cwd), 'exit': p.returncode, 'wall_seconds': wall, 'locked_seconds': last['locked_seconds'], 'queue_plus_wrapper_seconds': max(0, wall - last['locked_seconds']), 'disk_free_bytes': last['disk_free_bytes'], 'stdout_stderr': log_name, 'log_sha256': sha(OUT / log_name)}
    records = json.loads((OUT / 'commands.json').read_text()); records.append(record); save('commands.json', records)
    print(json.dumps(record), flush=True)
    admission = load('docs/operations/evidence/2026-10-30-pa09-js-admission/parent-main-source-admission.json')
    assert {p: sha(ROOT / p) for p in admission['source3']} == admission['source3']
    assert shutil.disk_usage(ROOT).free >= 3 * 1024**3
    if p.returncode: raise SystemExit(p.returncode)
if __name__ == '__main__':
    if sys.argv[1] == 'entry':
        assert not (OUT / 'entry.json').exists()
        state = snapshot(); save('entry.json', state); save('commands.json', [])
        private = pathlib.Path(tempfile.mkdtemp(prefix='pa09-main-js-29aa9984-')).resolve()
        paths = {'private': str(private), 'products': str(private / 'production-dist'), 'owninput': str(private / 'owninput')}; save('paths.json', paths)
        for src, dest in [('src/worker/native-pkg', 'native-pkg'), ('src/wasm-pkg', 'wasm-pkg'), ('public/pkg', 'public/pkg')]: shutil.copytree(WEB / src, private / 'owninput' / dest)
        save('owninput-sha256.json', {str(p.relative_to(private / 'owninput')): sha(p) for p in (private / 'owninput').rglob('*') if p.is_file()})
        save('toolchain.json', {x: subprocess.check_output([x, '--version'], env=ENV, text=True).strip() for x in ['node', 'pnpm', 'python3']})
        save('native-inheritance.json', {'baseline': 'a1f7250815eee803d931301ab46207719cb9a15b', 'main': state['head'], 'rust_cargo_changed_paths': git('diff', '--name-only', 'a1f7250815eee803d931301ab46207719cb9a15b', state['head'], '--', '*.rs', '**/Cargo.toml', 'Cargo.toml', 'Cargo.lock').decode().splitlines(), 'actual_webview_source3': state['source3'], 'actual_evidence': '/tmp/pa09-webview-5b7ad3ab/evidence', 'new_main_gui': False})
        print(json.dumps({'entry_ok': True, 'modules': len(state['modules']), 'dist': len(state['dist']), 'private': str(private)}))
    elif sys.argv[1] == 'exit':
        state = snapshot(); save('exit.json', state); before = json.loads((OUT / 'entry.json').read_text())
        invariant = ['head', 'tree', 'index_sha256', 'staged', 'source3', 'protected', 'groups', 'modules', 'dist', 'guard', 'frozen_package_diff_sha256', 'package_git_trees']
        checks = {k: state[k] == before[k] for k in invariant}
        checks['groups'] = all(set(state['groups'][k]) == set(before['groups'][k]) for k in before['groups'])
        save('narrow-comparison.json', {'invariants': checks, 'tracked_status_entry': before['tracked_status'], 'tracked_status_exit': state['tracked_status'], 'parent_docs_delta': git('diff', '--name-only', '--', 'docs').decode().splitlines(), 'interpretation': 'Only parent-authorized docs/evidence deltas are allowed; no restore performed.'})
        assert all(checks.values()), checks
        print(json.dumps(checks))
    else: run(sys.argv[1])
