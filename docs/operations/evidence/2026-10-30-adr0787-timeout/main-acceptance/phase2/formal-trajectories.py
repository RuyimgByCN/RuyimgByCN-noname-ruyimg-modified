#!/usr/bin/env python3
"""Run the real native vs generated-WASM trajectory comparison end to end.

The native side is the `nc08_cross_target_real` integration test (writes JSON via
NONAME_CROSS_TARGET_NATIVE_OUT); the WASM side loads the freshly built package at
apps/web-client/src/worker/native-pkg. Both use the same seed/command/choice policy
and a save/restore midpoint. The comparer rejects any revision/hash/event-code drift.
"""
from __future__ import annotations

import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
EVIDENCE = ROOT / "docs/operations/evidence/2026-10-24-native-wasm-cross-target"


def run(argv: list[str], env: dict[str, str] | None = None) -> None:
    completed = subprocess.run(argv, cwd=ROOT, env=env)
    if completed.returncode != 0:
        raise SystemExit(f"command failed ({completed.returncode}): {' '.join(argv)}")


def main() -> None:
    if True:
        tmp = '/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-a330-main-phase2-in4xi44k/formal'
        native_out = Path(tmp) / "native.json"
        wasm_out = Path(tmp) / "wasm.json"
        env = dict(os.environ)
        env["NONAME_CROSS_TARGET_NATIVE_OUT"] = str(native_out)
        env.setdefault("PATH", "/usr/local/bin:/usr/bin:/bin")
        run(
            [
                "cargo",
                "test",
                "--offline",
                "-p",
                "engine-native",
                "--test",
                "nc08",
                "nc08_cross_target_real::",
                "--",
                "--nocapture",
            ],
            env=env,
        )
        run(
            [
                "node",
                str(EVIDENCE / "wasm-trajectory.mjs"),
                str(wasm_out),
            ]
        )
        run(
            [
                sys.executable,
                str(EVIDENCE / "compare.py"),
                str(native_out),
                str(wasm_out),
            ]
        )

        # 第二阶段：正式安装包 huanle-cards，同一份 packs.json。
        packs = EVIDENCE / "huanle-packs.json"
        package_native = Path(tmp) / "native-package.json"
        package_wasm = Path(tmp) / "wasm-package.json"
        package_env = dict(env)
        package_env["NONAME_CROSS_TARGET_PACKS"] = str(packs)
        package_env["NONAME_CROSS_TARGET_NATIVE_OUT"] = str(package_native)
        run(
            [
                "cargo",
                "test",
                "--offline",
                "-p",
                "engine-native-wasm",
                "--test",
                "cross_target_package",
                "--",
                "--nocapture",
            ],
            env=package_env,
        )
        run(
            [
                "node",
                str(EVIDENCE / "wasm-package-trajectory.mjs"),
                str(packs),
                str(package_wasm),
            ]
        )
        run(
            [
                sys.executable,
                str(EVIDENCE / "compare.py"),
                str(package_native),
                str(package_wasm),
            ]
        )


if __name__ == "__main__":
    main()
