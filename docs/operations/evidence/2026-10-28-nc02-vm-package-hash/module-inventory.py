#!/usr/bin/env python3
"""Audit stored module fingerprints in tracked npack JSON files."""
import json
import subprocess
from pathlib import Path

import blake3

files = subprocess.check_output(["git", "ls-files", "-z", "*npack.json"]).decode().split("\0")
files = [name for name in files if name]
modules = 0
mismatches = []
for filename in files:
    package = json.loads(Path(filename).read_text(encoding="utf-8"))
    for module in package.get("modules", []):
        modules += 1
        name = module["name"].encode()
        code = bytes(module["code"])
        body = (
            b"noname-next/bytecode-module/v1"
            + len(name).to_bytes(4, "little")
            + name
            + bytes([module["local_count"]])
            + module["caps_mask"].to_bytes(4, "little")
            + module["entry_pc"].to_bytes(4, "little")
            + len(code).to_bytes(4, "little")
            + code
        )
        if blake3.blake3(body).digest() != bytes(module["logic_hash"]):
            mismatches.append((filename, module["name"]))
print(f"tracked npack files={len(files)} modules={modules} mismatches={len(mismatches)}")
for filename, name in mismatches:
    print(filename, name)
