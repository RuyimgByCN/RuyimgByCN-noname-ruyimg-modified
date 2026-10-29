#!/usr/bin/env python3
"""Verify every `#[ignore]` guard references a name that actually exists.

A guard that asserts on a **misspelled** behaviour or a non-existent event can
never go green, so it silently protects nothing — the same failure mode as the
ledger's wrong `behavior` names (第 784 轮).  This check parses the guard file
for quoted `CamelCase` identifiers and `HostBehavior::X` / `EventKind::X`
references, then verifies each against the engine sources.

Allowed name sources:

* `HostBehavior` variants — `crates/engine-native-kernel/src/vocabulary/host/keys.rs`
* `EventKind` variants — `crates/engine-native-kernel/src/event.rs`
* `Phase` variants — `crates/engine-native-kernel/src/event.rs`
* a small whitelist of capability names that do **not** exist yet and are the
  whole point of a guard (`CardTargeted`, `ShaBegin`, …)
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
GUARDS = ROOT / "crates" / "engine-native" / "tests" / "nc08_registered_defect_guards.rs"
KEYS = ROOT / "crates" / "engine-native-kernel" / "src" / "vocabulary" / "host" / "keys.rs"
EVENT = ROOT / "crates" / "engine-native-kernel" / "src" / "event.rs"

#: 守卫**故意**引用的「尚不存在」的能力名（补上后守卫转绿）。
MISSING_CAPABILITIES = {
    "CardTargeted", "ShaBegin", "ShaAfter", "ShaBefore", "DeathBefore", "GameStart",
    "JudgeBefore", "DamageBegin2", "DamageBegin4", "DrawBegin2", "LoseHpEnd",
    "PhaseBefore", "PhaseAfter", "EnterGame", "Prepare", "DrawBefore",
    "Damage4",  # 与 DamageBegin4 同义的候选名

}

#: 测试函数名/局部变量等会被误抓的常见词。
IGNORE = {"CARGO_MANIFEST_DIR", "Ok", "None", "Some", "String", "Path", "Vec", "Result"}

#: 守卫会引用的**常量名**（`content/host/constants.rs` 里存在）。
ALLOWED_CONSTANTS = {"EPIC_REGUHUO_KEY"}

QUOTED = re.compile(r'"([A-Z][A-Za-z0-9_]*)"')
PREFIXED = re.compile(r"(HostBehavior|EventKind|Phase)::([A-Z][A-Za-z0-9_]*)")


def variants(path: Path, enum: str) -> set[str]:
    src = path.read_text(encoding="utf-8")
    if enum == "HostBehavior":
        return set(re.findall(r"HostBehavior::([A-Za-z0-9_]+)", src))
    return set(re.findall(rf"{enum}::([A-Za-z0-9_]+)", src))


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--strict", action="store_true", help="exit 1 on unknown names")
    args = ap.parse_args()

    text = GUARDS.read_text(encoding="utf-8")
    known = {
        "HostBehavior": variants(KEYS, "HostBehavior"),
        "EventKind": variants(EVENT, "EventKind"),
        "Phase": variants(EVENT, "Phase"),
    }
    all_known = set().union(*known.values()) | MISSING_CAPABILITIES | IGNORE | ALLOWED_CONSTANTS

    unknown: list[tuple[str, str]] = []
    for kind, name in PREFIXED.findall(text):
        if name not in known[kind] and name not in MISSING_CAPABILITIES:
            unknown.append((kind, name))
    for name in QUOTED.findall(text):
        if name not in all_known:
            unknown.append(("quoted", name))

    print(f"HostBehavior {len(known['HostBehavior'])} / EventKind {len(known['EventKind'])} / Phase {len(known['Phase'])}")
    if unknown:
        print(f"守卫里引用了 {len(unknown)} 个未知名字：", file=sys.stderr)
        for kind, name in sorted(set(unknown)):
            print(f"  - {kind}: {name}", file=sys.stderr)
        return 1 if args.strict else 0
    print("守卫引用的名字全部存在（或属于待补能力白名单）✅")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
