#!/usr/bin/env python3
"""比对 host 行为的「过滤臂事件」与「TriggerMeta 事件种类」是否自相矛盾。

**为什么要**：第 928 轮发现 `WudianZiyu` 的 `TriggerMeta.event = PhaseBegin`，
而过滤臂被我改成 `begin: false`（阶段结束）——**过滤臂与元数据打架时该行为永远不会触发**，
而且全量测试只会表现为「某个测试不通过」，不会指出根因。

用法：
    python3 -B scripts/report_filter_meta_mismatch.py [--strict]
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
HOST = ROOT / "crates/engine-native/src/content/host"

META_RE = re.compile(
    r"HostBehavior::(\w+)\s*=>\s*Some\(TriggerMeta\s*\{[^}]*?event:\s*EventKind::(\w+)",
    re.S,
)
ARM_RE = re.compile(r"HostBehavior::(\w+)\s*=>")


def meta_events(base: Path = HOST) -> dict[str, str]:
    """HostBehavior → 声明的 EventKind（取第一处声明）。"""
    out: dict[str, str] = {}
    for path in sorted(base.glob("metadata/*.rs")):
        text = path.read_text(encoding="utf-8")
        for name, kind in META_RE.findall(text):
            out.setdefault(name, kind)
    return out


def arm_bodies(base: Path = HOST) -> dict[str, str]:
    """HostBehavior → 过滤臂源码（取第一处非委托实现）。"""
    out: dict[str, str] = {}
    for path in sorted(base.glob("filtering/**/*.rs")):
        text = path.read_text(encoding="utf-8")
        for mt in ARM_RE.finditer(text):
            nxt = ARM_RE.search(text, mt.end())
            body = text[mt.end(): nxt.start() if nxt else len(text)]
            if "filter_common(" in body:
                continue
            out.setdefault(mt.group(1), body)
    return out


def arm_family(body: str) -> str:
    """把过滤臂归到一个粗粒度事件族。"""
    if "EventData::Phase {" in body:
        begin = "begin: true" in body
        begin_false = "begin: false" in body
        if begin and not begin_false:
            return "PhaseBegin"
        if begin_false and not begin:
            return "PhaseEnd"
        return "Phase"
    for kind, needle in (
        ("CardUsed", "EventData::CardUsed"),
        ("CardResponded", "EventData::CardResponded"),
        ("DamageBefore", "EventData::DamageBefore"),
        ("Damage", "EventData::Damage("),
        ("Death", "EventData::Death"),
        ("Dying", "EventData::Dying"),
        ("CardsDiscarded", "EventData::CardsDiscarded"),
        ("CardLost", "EventData::CardsLost"),
        ("Recover", "EventData::Recover"),
        ("CardsDrawn", "EventData::CardsDrawn"),
        ("TurnOvered", "EventData::TurnOvered"),
        ("Equipped", "EventData::Equipped"),
        ("Linked", "EventData::Linked"),
    ):
        if needle in body:
            return kind
    return "?"


def mismatches(metas: dict[str, str], arms: dict[str, str]) -> list[tuple[str, str, str]]:
    """返回 (行为, 元数据事件, 过滤臂族) —— 只在两者明显互斥时计入。"""
    out: list[tuple[str, str, str]] = []
    for name, kind in sorted(metas.items()):
        body = arms.get(name)
        if body is None:
            continue
        family = arm_family(body)
        if family == "?":
            continue
        # 只有「明确互斥」才算矛盾：PhaseBegin 臂 vs PhaseEnd 元数据（或反之）。
        # 过滤臂未写明 begin（family=="Phase"）时不算矛盾——它由元数据决定派发。
        opposites = {("PhaseBegin", "PhaseEnd"), ("PhaseEnd", "PhaseBegin")}
        if (kind, family) in opposites:
            out.append((name, kind, family))
    return out


def main(argv: list[str] | None = None) -> int:
    args = sys.argv[1:] if argv is None else argv
    metas = meta_events()
    arms = arm_bodies()
    bad = mismatches(metas, arms)
    print(f"元数据声明 {len(metas)} 项，过滤臂 {len(arms)} 项，矛盾 {len(bad)} 项")
    for name, kind, family in bad:
        print(f"  ❌ {name}: meta={kind} filter={family}")
    if "--strict" in args and bad:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
