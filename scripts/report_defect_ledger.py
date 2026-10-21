#!/usr/bin/env python3
"""Emit the machine-readable defect ledger.

Every rule difference found in this session lives in prose somewhere; this script
produces a single TSV so the list can be sorted, counted, and handed to whoever
has write access to `content/host/**` and `kernel/**`.

Sources
-------
* `report_phase_arms.py`  -> G14 / G20 / G15 / 自加分支 / 语义选择
* `report_event_arms.py`  -> damage / card / death families (需档 / 第N次)
* G23                      -> `PhaseEnd` meta without a phase pin (over-fires)
* MANUAL                   -> defects found by hand (5b, G19, kuwu, ...)

Columns: id, class, behavior, upstream, native, severity, fix, permission.

`permission` is `blocked` for every row whose fix sits in untracked host/kernel
files (all of them today), so the ledger states the blocker per row instead of
in a footnote.
"""

from __future__ import annotations

import argparse
import importlib.util
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
HOST = ROOT / "crates" / "engine-native" / "src" / "content" / "host"


def _load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


phase = _load("report_phase_arms", ROOT / "scripts" / "report_phase_arms.py")
event = _load("report_event_arms", ROOT / "scripts" / "report_event_arms.py")

#: Hand-found defects that no enumeration produces (each with its evidence pointer).
MANUAL: list[tuple[str, str, str, str, str, str]] = [
    # id, class, behavior, upstream, native, note
    ("5b", "A", "Reguhuo", "reguhuo_phase 标记（每回合一次）", "无标记，可无限次", "EPIC_REGUHUO_KEY 定义未接线"),
    ("G19", "A", "SwdZhuhai", "global: phaseEnd", "filter 含 t != owner", "技能主被排除"),
    ("G19", "A", "SwdTiebi", "global: useCardToBegin", "filter 含 t != owner", "技能主被排除"),
    ("G19", "A", "SwdRexue", "global: phaseEnd", "filter 含 actor != owner", "技能主被排除"),
    ("5c", "B", "ShengkeCheck", "target: useCardToBegin", "EventKind::CardUsed", "需 CardTargeted 档"),
    ("凭空", "A", "BossBaiyi", "enable: phaseUse, limited", "第 5 轮判定阶段触发", "凭空新增触发"),
    ("自加", "A", "SwdHuajin2", "source: damageBegin", "Damage OR 判定阶段分支", "自加分支"),
    ("5a", "A", "HuanleKuwu", "sha && notLink && countCards>0", "仅 is_sha_like", "少 2 个 filter 条件"),
    ("6", "A", "GujianFoodRangjie", "分支前无条件 draw()", "仅分支 0 摸牌", "分支 1 不摸牌"),
    ("G23b", "A", "SwdJiguanyuanGain", "damageEnd + damageSource", "EventKind::PhaseEnd", "事件种类完全错"),
    ("失效", "A", "SwdBaishouzhihuHeld", "PhaseEnd(begin:false)", "filter 要求 *begin", "永不触发"),
    ("G16", "A", "GwentCiguState", "phaseDrawBegin 减 event.num", "Draw, begin:false", "摸牌已结束，效果失效"),
    ("B", "B", "Tianxiang", "player: damageBegin3", "EventKind::DamageBefore", "需第 3 次档"),
]


def g23_rows() -> list[tuple[str, str, str, str, str]]:
    """PhaseEnd metas without a phase pin, whose upstream is phaseEnd."""
    meta = event.metadata_events()
    pinned = set()
    for path in HOST.rglob("*.rs"):
        if "filtering" not in str(path):
            continue
        text = path.read_text(encoding="utf-8")
        for m in re.finditer(r"HostBehavior::([A-Za-z0-9_]+)\s*=>", text):
            body = text[m.end() : m.end() + 900]
            nxt = body.find("HostBehavior::")
            body = body if nxt < 0 else body[:nxt]
            if re.search(r"phase:\s*crate::event::Phase::\w+", body) or re.search(
                r"\*phase\s*==\s*crate::event::Phase::\w+", body
            ):
                pinned.add(m.group(1))
    keys = phase.host_keys()
    skills = phase.skill_names()
    rows = []
    for behavior, kind in sorted(meta.items()):
        if kind != "PhaseEnd" or behavior in pinned:
            continue
        key = keys.get(behavior, "")
        trigger = phase.resolve_trigger(behavior, key, skills.get(key, "") or "")
        if "phaseEnd" not in trigger:
            continue
        rows.append((behavior, trigger, "PhaseEnd, 未钉阶段", "高", "把 `..` 钉成 End, begin: false"))
    return rows


def build() -> list[list[str]]:
    keys = phase.host_keys()
    skills = phase.skill_names()
    rows: list[list[str]] = []

    for behavior, p, begin, _loc, _line in phase.arms():
        key = keys.get(behavior, "")
        trigger = phase.resolve_trigger(behavior, key, skills.get(key, "") or "")
        verdict = phase.classify(behavior, p, begin, trigger)
        if verdict.startswith("OK") or verdict.startswith("REVIEW") or verdict in ("语义选择",):
            continue
        cls = "A" if not verdict.startswith(("G15",)) else "B"
        fix = {
            "G14(钉死阶段)": "filter 改为 `begin: true, ..`（不钉阶段）",
            "G20(早一步)": "filter 改为 `Phase::End, begin: false`",
        }.get(verdict, "按对照结论修改 filter / 删除分支")
        rows.append([verdict.split("(")[0], cls, behavior, trigger, f"{p}, begin:{begin}", "中", fix, "blocked"])

    for behavior, trigger, note, severity, fix in g23_rows():
        rows.append(["G23", "A", behavior, trigger, note, severity, fix, "blocked"])

    for behavior, which, _loc, _skill, trigger, verdict in [
        (b, w, loc, s, t, v) for b, w, loc, s, t, v in event_out()
    ]:
        if verdict.startswith("OK") or verdict.startswith("REVIEW"):
            continue
        cls = "A" if verdict.startswith("G21") else "B"
        rows.append([verdict.split("(")[0], cls, behavior, trigger, which, "高", "修触发时机/补档位", "blocked"])

    for rid, cls, behavior, upstream, native, note in MANUAL:
        rows.append([rid, cls, behavior, upstream, native, "高" if cls == "A" else "中", note, "blocked"])

    # 去重：同一 (id, behavior) 只保留一条（相位面与手工表会重复记同一个缺陷）
    seen: set[tuple[str, str]] = set()
    unique: list[list[str]] = []
    for row in rows:
        key = (row[0], row[2])
        if key in seen:
            continue
        seen.add(key)
        unique.append(row)
    unique.sort(key=lambda r: (r[1], r[0], r[2]))
    return unique


def event_out():
    keys = phase.host_keys()
    skills = phase.skill_names()
    for family, classifier in (
        ("damage", lambda w, t: event.classify(w, t)),
        ("card", lambda w, t: event.classify_card(w, t)),
        ("death", lambda w, t: event.classify_death(w, t)),
    ):
        meta = event.metadata_events()
        if family == "damage":
            candidates = event.arms()
        elif family == "card":
            candidates = [(b, e, "meta", 0) for b, e in sorted(meta.items()) if e in {"CardUsed", "CardResponded"}]
        else:
            candidates = [(b, e, "meta", 0) for b, e in sorted(meta.items()) if e in {"Death", "Dying"}]
        for behavior, which, loc, line in candidates:
            key = keys.get(behavior, "")
            trigger = phase.resolve_trigger(behavior, key, skills.get(key, "") or "")
            # 时序权威是 TriggerMeta（过滤臂的匹配模式会产生假阳性，见枚举证据 §1）
            native = meta.get(behavior, which)
            yield behavior, native, f"{loc}:{line}", skills.get(key, "") or "", trigger, classifier(native, trigger)


def source_of(behavior: str) -> str:
    """behavior -> 来源 id（经 host 键 -> 包 -> 来源），查不到返回 `?`。"""
    try:
        mapper = _load("map_package_sources", ROOT / "scripts" / "map_package_sources.py")
        by_src, _name = mapper.mapping()
    except Exception:
        return "?"
    keys = phase.host_keys()
    key = keys.get(behavior, "").removeprefix("host:")
    if not key:
        return "?"
    for source, packages in by_src.items():
        for package in packages:
            path = ROOT / "packages" / "extensions" / package / "assets"
            if not path.is_dir():
                continue
            for asset in path.glob("*.json"):
                try:
                    doc = json.loads(asset.read_text(encoding="utf-8"))
                except Exception:
                    continue
                for skill in doc.get("skills") or []:
                    if isinstance(skill, dict) and skill.get("mapping") == f"host:{key}":
                        return source
    return "?"


def by_source(rows: list[list[str]]) -> dict[str, list[str]]:
    out: dict[str, list[str]] = {}
    for row in rows:
        out.setdefault(source_of(row[2]), []).append(row[0])
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--write", type=Path, help="write TSV to this path")
    ap.add_argument("--by-source", action="store_true", help="also print per-source counts")
    args = ap.parse_args()

    rows = build()
    header = ["id", "class", "behavior", "upstream", "native", "severity", "fix", "permission"]
    lines = ["\t".join(header)] + ["\t".join(r) for r in rows]
    text = "\n".join(lines) + "\n"
    if args.write:
        args.write.parent.mkdir(parents=True, exist_ok=True)
        args.write.write_text(text, encoding="utf-8")
        print(f"wrote {args.write} ({len(rows)} rows)", file=sys.stderr)
    else:
        print(text, end="")

    by_class: dict[str, int] = {}
    for r in rows:
        by_class[r[1]] = by_class.get(r[1], 0) + 1
    print(f"合计 {len(rows)} 条：{by_class}", file=sys.stderr)

    if args.by_source:
        counts = by_source(rows)
        print("\n来源\t条数\t缺陷族", file=sys.stderr)
        for source, ids in sorted(counts.items(), key=lambda kv: (-len(kv[1]), kv[0])):
            print(f"{source}\t{len(ids)}\t{','.join(sorted(set(ids)))}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
