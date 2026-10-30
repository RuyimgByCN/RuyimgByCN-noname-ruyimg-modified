import json
import os
from pathlib import Path
import subprocess
import time

root = Path(__file__).resolve().parent.parent / 'worktree'
evidence = root.parent / 'evidence'
environment = os.environ.copy()
environment['CARGO_TARGET_DIR'] = str(root.parent / 'target')
commands = [
    (f'revival-gate-{package}.log', ['env', 'RUSTFLAGS=-D warnings', 'cargo', 'check', '--offline', '-p', package, '--all-targets'])
    for package in ['engine-native-kernel', 'engine-native-content', 'engine-native']
]
commands.append(('wasm-build.log', ['wasm-pack', 'build', 'crates/engine-native-wasm', '--target', 'nodejs', '--release', '--no-opt', '--out-dir', str(evidence / 'nc10-node-pkg')]))
records = []
for name, command in commands:
    queued = time.time()
    with (evidence / name).open('w') as output:
        result = subprocess.run(['python3', '/tmp/noname-run-serialized.py', *command], cwd=root, env=environment, stdout=output, stderr=subprocess.STDOUT)
    records.append({'log': name, 'command': command, 'exit': result.returncode, 'wall_seconds_including_lock_wait': time.time() - queued})
    (evidence / 'verification-results.json').write_text(json.dumps(records, indent=2))
    if name == 'wasm-build.log' and result.returncode == 0:
        smoke_env = environment.copy()
        smoke_env['PKG'] = str(evidence / 'nc10-node-pkg')
        with (evidence / 'wasm-smoke.log').open('w') as output:
            result = subprocess.run(['python3', '/tmp/noname-run-serialized.py', 'node', 'scripts/native_wasm_smoke.mjs'], cwd=root, env=smoke_env, stdout=output, stderr=subprocess.STDOUT)
        records.append({'log': 'wasm-smoke.log', 'command': ['node', 'scripts/native_wasm_smoke.mjs'], 'exit': result.returncode})
        (evidence / 'verification-results.json').write_text(json.dumps(records, indent=2))
