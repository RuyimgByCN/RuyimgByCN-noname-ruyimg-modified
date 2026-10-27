#!/usr/bin/env python3
"""Scan actual instruction boundaries in runtime npack modules, plus preserved old packages."""
import json
from pathlib import Path

NO_OPERAND = {0x00, 0x01, 0x22, 0x23, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35,
              0x36, 0x37, 0x38, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59,
              0x5b, 0x5c, 0x7f}
OPERANDS = {0x10: 4, 0x11: 4, 0x12: 4, 0x20: 1, 0x21: 1, 0x40: 2, 0x41: 2,
            0x50: 1, 0x51: 1, 0x52: 1, 0x5a: 2}

def tags(code):
    found, i = set(), 0
    while i < len(code):
        opcode = code[i]; i += 1
        if opcode in NO_OPERAND:
            continue
        size = OPERANDS.get(opcode)
        if size is None or i + size > len(code):
            raise ValueError(f"invalid instruction {opcode:#x} at {i-1}")
        if opcode == 0x50:
            found.add(code[i])
        elif opcode == 0x5a:
            found.add(code[i] | code[i+1] << 8)
        i += size
    return found

files = sorted(Path("content/original/ported").rglob("*.npack.json"))
files += sorted(Path("packages").rglob("*.npack.json"))
files += [Path("docs/operations/evidence/2026-10-24-up37-n01-zhuhuan/onlyOL-0.1.2.npack.json"),
          Path("docs/operations/evidence/2026-10-27-pa12-tag656-fix/onlyOL-0.1.4.npack.json")]
modules = matches = 0
for file in files:
    package = json.loads(file.read_text())
    abi = package.get("manifest", {}).get("rule_vm_abi_version")
    for module in package.get("modules", []):
        modules += 1
        try:
            found = tags(module["code"]) & {656, 772}
        except ValueError as e:
            raise SystemExit(f"{file}: {module['name']}: {e}")
        if found:
            matches += 1
            print(f"{file} ABI={abi} module={module['name']} tags={sorted(found)}")
print(f"Scanned {len(files)} packages, {modules} modules, {matches} matches")
