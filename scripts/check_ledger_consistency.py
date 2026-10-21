#!/usr/bin/env python3
"""Cross-check the defect ledger against the enumerations that produced it.

The ledger (`report_defect_ledger.py`) merges three sources: the phase-arm
enumeration, the event-family enumeration, and a hand-written table.  Each has
been corrected several times, so a row can drift out of sync with its source:

* a ledger row whose `id` is a **phase verdict** must appear in
  `report_phase_arms.py` output with that same verdict;
* a row whose `id` is an **event verdict** must appear in
  `report_event_arms.py` (`--family damage|card|death`) with that verdict;
* hand-written rows (`5b`, `G19`, `L03a`, …) are exempt — they have no enumeration.

Usage:
    python3 -B scripts/check_ledger_consistency.py            # report mismatches
    python3 -B scripts/check_ledger_consistency.py --strict    # exit 1 on mismatch
"""

from __future__ import annotations

import argparse
import importlib.util
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

PHASE_IDS = {"G14", "G18", "G20", "G22", "G15", "凭空新增触发", "自加分支"}
EVENT_IDS = {"需档", "第3次", "G24"}
G23_ID = "G23"
HANDWRITTEN = {"5a", "5b", "5c", "6", "G19", "G23b", "失效", "L03a", "L03b", "L03c",
               "L20a", "L20b", "L20c", "L20d", "L27a", "L27b", "L29a", "L29b", "B",
               "L07a", "U08a", "U08b", "U08c"}


def _load(name: str, filename: str):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / filename)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def phase_rows() -> list[tuple[str, str]]:
    """[(behavior, verdict)] from the phase enumeration."""
    mod = _load("report_phase_arms", "report_phase_arms.py")
    keys = mod.host_keys()
    skills = mod.skill_names()
    out = []
    for behavior, phase, begin, _loc, _line in mod.arms():
        key = keys.get(behavior, "")
        trigger = mod.resolve_trigger(behavior, key, skills.get(key, "") or "")
        out.append((behavior, mod.classify(behavior, phase, begin, trigger)))
    return out


def event_rows() -> list[tuple[str, str]]:
    """[(behavior, verdict)] from the three event families."""
    mod = _load("report_event_arms", "report_event_arms.py")
    phase = _load("report_phase_arms", "report_phase_arms.py")
    keys = phase.host_keys()
    skills = phase.skill_names()
    meta = mod.metadata_events()
    out = []
    for behavior, which, _file, _line in mod.arms():
        key = keys.get(behavior, "")
        trigger = phase.resolve_trigger(behavior, key, skills.get(key, "") or "")
        out.append((behavior, mod.classify(meta.get(behavior, which), trigger)))
    for family, classifier in (("card", mod.classify_card), ("death", mod.classify_death)):
        pool = (
            [(b, e) for b, e in sorted(meta.items()) if e in ({"CardUsed", "CardResponded"} if family == "card" else {"Death", "Dying"})]
        )
        for behavior, event in pool:
            key = keys.get(behavior, "")
            trigger = phase.resolve_trigger(behavior, key, skills.get(key, "") or "")
            out.append((behavior, classifier(event, trigger)))
    return out


def ledger() -> list[list[str]]:
    path = ROOT / "docs" / "operations" / "evidence" / "2026-10-19-defect-ledger" / "ledger.tsv"
    lines = path.read_text(encoding="utf-8").splitlines()
    return [line.split("\t") for line in lines[1:] if line.strip()]


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--strict", action="store_true", help="exit 1 when any mismatch is found")
    args = ap.parse_args()

    phase = phase_rows()
    event = event_rows()
    ledger_mod = _load("report_defect_ledger", "report_defect_ledger.py")
    g23_behaviors = {row[0] for row in ledger_mod.g23_rows()}
    # 行为必须在 keys.rs 的枚举里出现（有些行为没有 `host:` 键，如卡片全局技能）
    keys_src = (ROOT / "crates/engine-native/src/content/host/keys.rs").read_text(encoding="utf-8")
    known_behaviors = set(re.findall(r"HostBehavior::([A-Za-z0-9_]+)", keys_src))
    phase_pairs = {(b, v) for b, v in phase}
    event_pairs = {(b, v) for b, v in event}
    phase_behaviors = {b for b, _ in phase}
    event_behaviors = {b for b, _ in event}

    problems: list[str] = []
    for row in ledger():
        if row[2] not in known_behaviors:
            problems.append(f"{row[0]} {row[2]}: 不是已知的 HostBehavior 变体（名字写错了？）")
            continue
        rid, _cls, behavior = row[0], row[1], row[2]
        if rid in HANDWRITTEN:
            continue
        if rid == G23_ID:
            if behavior not in g23_behaviors:
                problems.append(f"{rid} {behavior}: 台账有，PhaseEnd 未钉阶段普查里没有")
            continue
        if rid in PHASE_IDS:
            if behavior not in phase_behaviors:
                problems.append(f"{rid} {behavior}: 台账有，相位枚举里没有")
            elif not any(b == behavior and v.startswith(rid) for b, v in phase):
                actual = {v for b, v in phase if b == behavior}
                problems.append(f"{rid} {behavior}: 台账记 {rid}，枚举记 {sorted(actual)}")
        elif rid in EVENT_IDS:
            if behavior not in event_behaviors:
                problems.append(f"{rid} {behavior}: 台账有，事件枚举里没有")
            elif not any(b == behavior and v.startswith(rid[:2]) for b, v in event):
                actual = {v for b, v in event if b == behavior}
                problems.append(f"{rid} {behavior}: 台账记 {rid}，枚举记 {sorted(actual)}")
        else:
            problems.append(f"{rid} {behavior}: 未归类的台账 id（请在 PHASE_IDS/EVENT_IDS/HANDWRITTEN 里登记）")

    print(f"台账 {len(ledger())} 行；相位枚举 {len(phase)} 行；事件枚举 {len(event)} 行")
    if problems:
        print(f"发现 {len(problems)} 处不一致：", file=sys.stderr)
        for problem in problems:
            print(f"  - {problem}", file=sys.stderr)
        return 1 if args.strict else 0
    print("台账与枚举一致 ✅")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
