from pathlib import Path
from http.server import ThreadingHTTPServer, BaseHTTPRequestHandler
import datetime, hashlib, json, mimetypes, os, plistlib, shutil, subprocess, tarfile, threading, time
from urllib.parse import unquote, urlsplit
out = Path('/tmp/pa09-webview-5b7ad3ab').resolve()
evidence = out / 'evidence'
bundle = out / 'bundle'
identifier = 'org.noname.pa09.jsbound5b7ad3ab'
manifest = json.loads((evidence / 'native-build-result.json').read_text())
assert manifest['exit'] == 0
source = Path(manifest['binary'])
assert hashlib.sha256(source.read_bytes()).hexdigest() == manifest['binary_sha256']
app = out / 'products/PA09 JS Admission.app'
contents = app / 'Contents'
macos = contents / 'MacOS'
macos.mkdir(parents=True, exist_ok=False)
binary = macos / 'noname-next-desktop'
shutil.copy2(source, binary)
plist = {'CFBundleIdentifier': identifier, 'CFBundleName': 'PA09 JS Admission', 'CFBundleDisplayName': 'PA09 JS Admission', 'CFBundleExecutable': 'noname-next-desktop', 'CFBundlePackageType': 'APPL', 'CFBundleVersion': '1', 'CFBundleShortVersionString': '0.1.0', 'NSHighResolutionCapable': True, 'NSPrincipalClass': 'NSApplication', 'LSUIElement': True, 'NSAppTransportSecurity': {'NSAllowsLocalNetworking': True}}
(contents / 'Info.plist').write_bytes(plistlib.dumps(plist))
sign = subprocess.run(['codesign', '--force', '--sign', '-', str(app)], capture_output=True, text=True)
(evidence / 'codesign.log').write_text(sign.stdout + sign.stderr)
assert sign.returncode == 0
bundlefiles = {str(p.relative_to(bundle)): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(bundle.rglob('*')) if p.is_file()}
bundle_digest = hashlib.sha256(json.dumps(bundlefiles, sort_keys=True).encode()).hexdigest()
identity = {'app': str(app), 'binary': str(binary), 'source_binary_sha256': manifest['binary_sha256'], 'packaged_binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(), 'bundle': str(bundle), 'bundle_sha256': bundle_digest, 'bundle_files': bundlefiles, 'compile_time_config_sha256': manifest['config_sha256'], 'plist': plist}
(evidence / 'runtime-product-identity.json').write_text(json.dumps(identity, indent=2))
completed = threading.Event()
lock = threading.Lock()
http = []
report = None
class Handler(BaseHTTPRequestHandler):
    def log_message(self, *args): pass
    def send(self, body, code=200, kind='application/json'):
        self.send_response(code)
        self.send_header('Content-Type', kind)
        self.send_header('Content-Length', str(len(body)))
        self.send_header('Cache-Control', 'no-store')
        self.end_headers()
        self.wfile.write(body)
    def do_GET(self):
        name = unquote(urlsplit(self.path).path)
        path = bundle / ('index.html' if name in ('/', '/index.html') else name.lstrip('/'))
        if not path.resolve().is_relative_to(bundle) or not path.is_file():
            self.send(b'not found', 404, 'text/plain'); return
        data = path.read_bytes()
        row = {'utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'path': self.path, 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest(), 'client': self.client_address}
        with lock:
            http.append(row)
            (evidence / 'http-served-assets.json').write_text(json.dumps(http, indent=2))
        self.send(data, kind=mimetypes.guess_type(str(path))[0] or 'application/octet-stream')
    def do_POST(self):
        global report
        size = int(self.headers.get('Content-Length', 0))
        assert 0 <= size <= 10_000_000
        data = self.rfile.read(size)
        if self.path != '/report':
            self.send(b'not found', 404); return
        report = json.loads(data)
        (evidence / 'actual-webview-report.json').write_bytes(data)
        self.send(b'{"received":true}')
        completed.set()
server = ThreadingHTTPServer(('127.0.0.1', 19439), Handler)
threading.Thread(target=server.serve_forever, daemon=True).start()
statepaths = [Path('/Users/ruyimg/Library') / sub / identifier for sub in ['Application Support', 'WebKit', 'Caches']]
assert not any(p.exists() for p in statepaths)
started = time.monotonic()
process = None
try:
    env = os.environ.copy()
    env.pop('TAURI_CONFIG', None)
    with (evidence / 'app.log').open('w') as log:
        process = subprocess.Popen([str(binary)], cwd=out, env=env, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        owner = {**identity, 'driver_pid': os.getpid(), 'server_pid': os.getpid(), 'app_pid': process.pid, 'argv': [str(binary)], 'cwd': str(out), 'port': server.server_port, 'bind': '127.0.0.1', 'hidden_window': True, 'runtime_tauri_config': None, 'launch_utc': datetime.datetime.now(datetime.timezone.utc).isoformat()}
        (evidence / 'run-owner.json').write_text(json.dumps(owner, indent=2))
        deadline = time.monotonic() + 120
        while not completed.wait(0.25):
            if process.poll() is not None: raise RuntimeError('owned app exited before report: ' + str(process.returncode))
            if time.monotonic() > deadline: raise RuntimeError('actual WKWebView fixture timed out after 120s')
        assert report is not None
        served = {row['path'].split('?')[0].lstrip('/'): row['sha256'] for row in http}
        assert served.get('index.html') == bundlefiles['index.html'] or served.get('') == bundlefiles['index.html']
        assert all(value == bundlefiles[name or 'index.html'] for name, value in served.items())
        assert any(name.endswith('.js') for name in served)
        (evidence / 'served-bundle-verification.json').write_text(json.dumps({'passed': True, 'served': served, 'loaded_bundle_sha256': bundle_digest, 'assertions': 'Every served asset SHA equals the recorded production bundle; index and JS requested by the owned actual app.'}, indent=2))
except Exception as error:
    (evidence / 'run-failure.json').write_text(json.dumps({'error': str(error), 'app_poll': process.poll() if process else None, 'wall_seconds': time.monotonic() - started}, indent=2))
    raise
finally:
    cleanup = {'wall_seconds': time.monotonic() - started, 'server_pid': os.getpid(), 'app_pid': process.pid if process else None}
    if process and process.poll() is None:
        process.terminate()
        try: process.wait(timeout=10)
        except subprocess.TimeoutExpired: process.kill(); process.wait(timeout=5)
    cleanup['app_exit'] = process.returncode if process else None
    cleanup['owned_app_waited'] = process is None or process.poll() is not None
    server.shutdown(); server.server_close()
    cleanup['server_closed'] = True
    archived = []
    for i, path in enumerate(statepaths):
        if path.exists():
            archive = evidence / ('owned-state-' + str(i) + '.tar.gz')
            with tarfile.open(archive, 'w:gz') as tar: tar.add(path, arcname=path.name)
            archived.append({'path': str(path), 'archive': str(archive), 'sha256': hashlib.sha256(archive.read_bytes()).hexdigest()})
            shutil.rmtree(path)
    cleanup['owned_state_archived_then_removed'] = archived
    cleanup['disk_free_bytes'] = shutil.disk_usage(out).free
    (evidence / 'cleanup.json').write_text(json.dumps(cleanup, indent=2))
if not report.get('passed'):
    raise SystemExit('Actual fixture failed; original report retained')
print(json.dumps({'passed': report['passed'], 'jsBounded': report['jsBounded'], 'assertions': len(report['assertions']), 'binary_sha256': identity['packaged_binary_sha256'], 'bundle_sha256': bundle_digest}))
