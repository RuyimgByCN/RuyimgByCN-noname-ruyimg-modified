from pathlib import Path
import subprocess, json, os
root = Path(__file__).resolve().parents[1] / "worktree"
evidence = root.parent / "evidence"
target = str(root.parent / "target")
serial = ["python3", "/tmp/noname-run-serialized.py"]
heavy_env = ["env", "CARGO_TARGET_DIR=" + target]
steps = [
("final-all5", serial + heavy_env + ["cargo", "test", "--offline", "-p", "engine-native-kernel", "-p", "engine-native-content", "-p", "engine-native", "-p", "game-server", "-p", "engine-native-wasm", "--no-fail-fast"], root),
("final-strict-all3", serial + heavy_env + ["RUSTFLAGS=-D warnings", "cargo", "check", "--offline", "-p", "engine-native-kernel", "-p", "engine-native-content", "-p", "engine-native", "--all-targets"], root),
("final-wasm32-build", serial + heavy_env + ["wasm-pack", "build", "crates/engine-native-wasm", "--target", "web", "--release", "--no-opt", "--out-dir", "../../apps/web-client/src/worker/native-pkg"], root),
("final-emit-fixtures", serial + heavy_env + ["cargo", "run", "--offline", "-p", "game-server", "--example", "emit_projection_fixture", "--", str(evidence / "wasm32-fixtures.json")], root),
("final-wasm32-parity", ["node", "--import", "./scripts/ts-register.mjs", "scripts/native-server-projection-parity.mjs", str(evidence / "wasm32-fixtures.json")], root / "apps/web-client"),
("final-worker82", ["node", "--import", "./scripts/ts-register.mjs", "scripts/native-worker-smoke.mjs"], root / "apps/web-client"),
("final-frontend", ["node", "--import", "./scripts/ts-register.mjs", "--test"], root / "apps/web-client"),
("final-typecheck", ["node_modules/.bin/vue-tsc", "--noEmit"], root / "apps/web-client"),
("final-cross-target", serial + heavy_env + ["python3", "-B", "scripts/check-native-wasm-cross-target.py"], root),
("final-source-lines", ["python3", "-B", "scripts/check-source-lines.py"], root),
("final-source-line-units", ["python3", "-B", "-m", "unittest", "discover", "-s", "scripts", "-p", "test_check_source_lines.py"], root),
("final-preflight", ["python3", "-B", "-u", "scripts/preflight.py"], root),
]
results = []
for name, command, cwd in steps:
    print("START " + name, flush=True)
    with (evidence / (name + ".log")).open("w") as log:
        result = subprocess.run(command, cwd=cwd, stdout=log, stderr=subprocess.STDOUT)
    row = {"name": name, "command": command, "cwd": str(cwd), "exit": result.returncode}
    results.append(row)
    (evidence / "final-commands.json").write_text(json.dumps(results, indent=2) + "\n")
    print("FINISH " + name + " exit=" + str(result.returncode), flush=True)
