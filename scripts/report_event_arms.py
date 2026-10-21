#!/usr/bin/env python3
"""Enumerate damage-triggered filter arms and classify them against upstream triggers.

Companion to `report_phase_arms.py`.  Where that script audits phase pins, this
one audits the **damage** surface, which is the second largest source of timing
mismatches (upstream distinguishes `damageBegin` / `damageBefore` / `damageEnd` /
`damageBegin2` / `damageBegin3`, while the engine only has `DamageBefore` and
`Damage`).

For every filtering arm whose body matches `EventData::Damage` or
`EventData::DamageBefore`, resolve the upstream trigger and classify:

* upstream `damageBegin` / `damageBefore`  + native `DamageBefore` -> OK
* upstream `damageBegin` / `damageBefore`  + native `Damage`       -> G21 (前/后错位)
* upstream `damageEnd` / `damageAfter`     + native `Damage`       -> OK
* upstream `damageEnd` / `damageAfter`     + native `DamageBefore` -> G21
* upstream `damageBegin2` / `damageBegin3`                        -> 第 N 次（需档位）
* upstream has no damage trigger                                   -> REVIEW(非伤害触发)

Usage:
    python3 -B scripts/report_event_arms.py            # TSV + summary
    python3 -B scripts/report_event_arms.py --check    # exit 1 when REVIEW rows exist
"""

from __future__ import annotations

import argparse
import importlib.util
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
HOST = ROOT / "crates" / "engine-native" / "src" / "content" / "host"

BEH_RE = re.compile(r"HostBehavior::([A-Za-z0-9_]+)\s*=>")
DAMAGE_RE = re.compile(r"EventData::(DamageBefore|Damage)\b")


def _load_phase_helpers():
    spec = importlib.util.spec_from_file_location("report_phase_arms", ROOT / "scripts" / "report_phase_arms.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def metadata_events() -> dict[str, str]:
    """behavior -> declared trigger event (`TriggerMeta.event`).

    This is the authoritative timing surface: the filter arm may match
    `EventData::Damage` while the meta declares `EventKind::DamageBefore`, and
    the meta is what the dispatcher uses.  Enumerating the filter pattern alone
    produces false positives (e.g. `Xunmeng`).
    """
    out: dict[str, str] = {}
    for path in sorted(HOST.rglob("metadata*.rs")) + sorted(HOST.rglob("metadata/*.rs")):
        if not path.exists():
            continue
        text = path.read_text(encoding="utf-8")
        for m in re.finditer(r"HostBehavior::([A-Za-z0-9_]+)\s*=>\s*Some\(TriggerMeta\s*\{", text):
            tail = text[m.end() : m.end() + 400]
            hit = re.search(r"event:\s*EventKind::([A-Za-z0-9_]+)", tail)
            if hit:
                out.setdefault(m.group(1), hit.group(1))
    return out


def arms() -> list[tuple[str, str, str, int]]:
    """[(behavior, which, file, line)] for arms matching a damage event."""
    rows: list[tuple[str, str, str, int]] = []
    for path in sorted(HOST.rglob("*.rs")):
        if "filtering" not in str(path):
            continue
        text = path.read_text(encoding="utf-8")
        lines = text.splitlines()
        anchors = [(m.start(), m.group(1)) for m in BEH_RE.finditer(text)]
        for offset, behavior in anchors:
            line_no = text.count("\n", 0, offset) + 1
            tail = text[offset : offset + 600]
            # 从 anchor 之后开始找下一个 arm（anchor 自身就含 `HostBehavior::`）
            nxt = tail.find("HostBehavior::", len("HostBehavior::"))
            body = tail if nxt < 0 else tail[:nxt]
            hit = DAMAGE_RE.search(body)
            if not hit:
                continue
            rows.append((behavior, hit.group(1), path.name, line_no))
    return rows


def classify(which: str, trigger: str) -> str:
    """`which` is the native event kind (`DamageBefore` / `Damage` / `Death` ...)."""
    if not trigger:
        return "REVIEW(上游未定位)"
    if "damageBegin3" in trigger:
        return "第3次(需档位)"
    if "damageBegin2" in trigger:
        return "第2次(需档位)"
    if "damageBegin" in trigger or "damageBefore" in trigger:
        return "OK" if which == "DamageBefore" else "G21(前/后错位)"
    if "damageEnd" in trigger or "damageAfter" in trigger or "damageSource" in trigger:
        return "OK" if which == "Damage" else "G21(前/后错位)"
    if "Damage" in trigger:
        return "OK" if which == "Damage" else "REVIEW(上游 Damage)"
    return "REVIEW(非伤害触发)"


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--check", action="store_true", help="exit 1 when REVIEW rows exist")
    args = ap.parse_args()

    phase = _load_phase_helpers()
    meta = metadata_events()
    keys = phase.host_keys()
    skills = phase.skill_names()
    out = []
    counts: dict[str, int] = {}
    for behavior, which, file, line in arms():
        key = keys.get(behavior, "")
        skill = skills.get(key, "") or ""
        trigger = phase.upstream_trigger(skill, key.removeprefix("host:"))
        native = meta.get(behavior, which)
        verdict = classify(native, trigger)
        counts[verdict] = counts.get(verdict, 0) + 1
        out.append((behavior, native, f"{file}:{line}", skill, trigger, verdict))

    print("behavior\tnative\t位置\tskill\t上游trigger\t判定")
    for row in sorted(out, key=lambda r: (r[5], r[0])):
        print("\t".join(row))
    print(f"\n共 {len(out)} 个伤害触发臂", file=sys.stderr)
    for verdict, n in sorted(counts.items(), key=lambda kv: -kv[1]):
        print(f"  {verdict}: {n}", file=sys.stderr)

    review = [r for r in out if r[5].startswith("REVIEW")]
    if review:
        print(f"\n{len(review)} 行需人工复核", file=sys.stderr)
        return 1 if args.check else 0
    print("\n无 REVIEW 行", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
