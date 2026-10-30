from pathlib import Path
import datetime, hashlib, json, os, shutil, signal, subprocess, time
root = Path('/tmp/pa09-js-bound-4306bc00').resolve()
out = Path('/tmp/pa09-webview-5b7ad3ab').resolve()
evidence = out / 'evidence'
target = out / 'target'
assert not target.exists(), 'one fresh owned target only'
assert shutil.disk_usage(out).free >= 3 * 1024**3
config = (evidence / 'compile-time-tauri-config.json').read_text()
env = os.environ.copy()
env.update(CARGO_TARGET_DIR=str(target), CARGO_INCREMENTAL='0', CARGO_BUILD_JOBS='2', TAURI_CONFIG=config)
command = ['cargo', 'build', '--offline', '--locked', '--release', '-p', 'desktop-client', '--features', 'tauri-shell,tauri/custom-protocol', '--bin', 'noname-next-desktop', '-j', '2', '--message-format=json-render-diagnostics']
owner = {'utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'cwd': str(root), 'command': command, 'env_overrides': {k: env[k] for k in ['CARGO_TARGET_DIR', 'CARGO_INCREMENTAL', 'CARGO_BUILD_JOBS']}, 'config_sha256': hashlib.sha256(config.encode()).hexdigest(), 'serialized_by': '/tmp/noname-run-serialized.py', 'disk_before_bytes': shutil.disk_usage(out).free, 'driver_pid': os.getpid(), 'build_estimate': '5-10 minutes, under 3 GiB target; no shared target or WASM build'}
started = time.monotonic()
with (evidence / 'native-build.stdout.jsonl').open('w') as stdout, (evidence / 'native-build.stderr.log').open('w') as stderr:
    process = subprocess.Popen(['python3', '/tmp/noname-run-serialized.py', *command], cwd=root, env=env, stdout=stdout, stderr=stderr, start_new_session=True)
    owner['wrapper_pid'] = process.pid
    (evidence / 'native-build-owner.json').write_text(json.dumps(owner, indent=2))
    exceeded = None
    with (evidence / 'native-build-monitor.jsonl').open('w') as monitor:
        while process.poll() is None:
            size = sum(p.stat().st_size for p in target.rglob('*') if p.is_file()) if target.exists() else 0
            free = shutil.disk_usage(out).free
            row = {'elapsed_wall_seconds': time.monotonic() - started, 'target_bytes': size, 'free_bytes': free}
            monitor.write(json.dumps(row) + '\n'); monitor.flush()
            if free < 3 * 1024**3 or size > 5 * 1024**3 or time.monotonic() - started > 1200:
                exceeded = row
                os.killpg(process.pid, signal.SIGTERM)
                break
            time.sleep(5)
    try:
        result = process.wait(timeout=15)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL); result = process.wait(timeout=10)
owner.update(exit=result, wall_including_lock_seconds=time.monotonic() - started, exceeded=exceeded, disk_after_bytes=shutil.disk_usage(out).free)
binary = target / 'release/noname-next-desktop'
if result == 0 and binary.exists():
    owner['binary'] = str(binary)
    owner['binary_sha256'] = hashlib.sha256(binary.read_bytes()).hexdigest()
(evidence / 'native-build-result.json').write_text(json.dumps(owner, indent=2))
(out / 'native-build.done').write_text(str(result))
