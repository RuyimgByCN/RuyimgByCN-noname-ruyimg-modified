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
MANUAL: list[tuple[str, str, str, str, str, str, str]] = [
    # id, class, behavior, upstream, native, note
    ("5b", "A", "Reguhuo", "L10", "reguhuo_phase 标记（每回合一次）", "无标记，可无限次", "EPIC_REGUHUO_KEY 定义未接线"),
    ("G19", "A", "SwdZhuhai", "U05", "global: phaseEnd", "filter 含 t != owner", "技能主被排除"),
    ("G19", "A", "SwdTiebi", "U05", "global: useCardToBegin", "filter 含 t != owner", "技能主被排除"),
    ("G19", "A", "SwdRexue", "U05", "global: phaseEnd", "filter 含 actor != owner", "技能主被排除"),
    ("5c", "B", "ShengkeCheck", "L06", "target: useCardToBegin", "EventKind::CardUsed", "需 CardTargeted 档"),
    ("5a", "A", "ShaDamageForceDiscard", "U06", "sha && notLink && countCards>0", "共享行为仅 is_sha_like", "共享行为未含苦无的 2 个条件（待复核）"),
    ("6", "A", "Rangjie", "L10", "分支前无条件 draw()", "仅分支 0 摸牌", "分支 1 不摸牌"),
    ("G23b", "A", "SwdJiguanyuanGain", "U05", "damageEnd + damageSource", "EventKind::PhaseEnd", "事件种类完全错"),
    ("G23b", "A", "SwdBaishouzhihuHeld", "U05", "player: 'discardEnd'（弃牌阶段结束）", "Phase { begin: true } 未钉阶段", "上游是弃牌阶段结束，原生每个阶段开始都触发"),
    ("L03c", "B", "HlssGaiming", "L03", "player: judgeBefore", "无 TriggerMeta",
     "技能完全不触发（缺 judge-before 幕）"),
    ("L20a", "B", "XiguaMopaiFanbei", "L20", 'player: phaseDrawBegin2', "无 TriggerMeta",
     "不触发 + 需「摸牌阶段第 2 次」档"),
    ("L20b", "B", "XiguaXueliangFanbei", "L20", 'global: gameStart', "EventKind::PhaseBegin",
     "事件错（应为 GameStart，缺档）"),
    ("U08c", "B", "YxsMenshen3", "U08", 'player: ["phaseBegin", "dieBegin"]', "EventKind::PhaseBegin",
     "少「死亡前」幕"),
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

    for behavior, p, begin, loc, line in phase.arms():
        key = keys.get(behavior, "")
        trigger = phase.resolve_trigger(behavior, key, skills.get(key, "") or "")
        verdict = phase.classify(behavior, p, begin, trigger)
        if (
            verdict.startswith("OK")
            or verdict.startswith("REVIEW")
            or verdict in ("语义选择", "相邻相位(可接受)")
        ):
            continue
        cls = "A" if not verdict.startswith(("G15", "G18")) else "B"
        fix = {
            "G14(钉死阶段)": "filter 改为 `begin: true, ..`（不钉阶段）",
            "G20(早一步)": "filter 改为 `Phase::End, begin: false`",
            "G22(晚一步)": "filter 从 `begin: false` 改为 `begin: true`（须在摸牌数确定前生效）",
            "G18(准备阶段缺档)": "需 `Phase::Prepare`（G18 缺档）",
            "凭空新增触发": "删除阶段触发（上游是主动技）",
        }.get(verdict, "按对照结论修改 filter / 删除分支")
        _t, path = phase.upstream_trigger_with_path(skills.get(key, "") or "", key.removeprefix("host:"))
        src = source_of_path(path) if path else source_of(behavior)
        rows.append([verdict.split("(")[0], cls, behavior, src, f"{loc}:{line}", trigger,
                     f"{p}, begin:{begin}", "中", fix, "blocked"])

    for behavior, trigger, note, severity, fix in g23_rows():
        _t, path = phase.upstream_trigger_with_path(skills.get(key_of(behavior), "") or "", key_of(behavior))
        rows.append(["G23", "A", behavior, source_of_path(path) if path else source_of(behavior), "filtering/*",
                     trigger, note, severity, fix, "blocked"])

    for behavior, which, _loc, _skill, trigger, verdict in [
        (b, w, loc, s, t, v) for b, w, loc, s, t, v in event_out()
    ]:
        if verdict.startswith("OK") or verdict.startswith("REVIEW"):
            continue
        # G21/G24 是「改触发时机」⇒ A 类；「需档」与「第 N 次」需扩枚举 ⇒ B 类
        cls = "A" if verdict.startswith(("G21", "G24")) else "B"
        rows.append([verdict.split("(")[0], cls, behavior, source_of(behavior), _loc,
                     trigger, which, "高", "修触发时机/补档位", "blocked"])

    for rid, cls, behavior, source, upstream, native, note in MANUAL:
        rows.append([rid, cls, behavior, source, "content/host/**", upstream, native,
                     "高" if cls == "A" else "中", note, "blocked"])

    # 去重：同一 (id, behavior) 只保留一条（相位面与手工表会重复记同一个缺陷）
    seen: set[tuple[str, str]] = set()
    unique: list[list[str]] = []
    for row in rows:
        key = (row[0], row[2])
        if key in seen:
            continue
        seen.add(key)
        unique.append(row)
    unique.sort(key=lambda r: (r[3], r[1], r[0], r[2]))
    return unique


#: 上游是**裸 `useCard`**（使用牌时）的行为：引擎已有 `CardUsed`，**不是缺档**。
#: 子帧先后（use 瞬间 vs after 窗口）属于口径问题 ⇒ 交 D 类判定，不计缺陷。
BARE_USE_CARD_REVIEW = {
    "SwdGongshen", "SwdLanzhi", "SwdLiuhong", "XianjianLiefeng", "NewZhixi",
}


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
            if behavior in BARE_USE_CARD_REVIEW:
                continue
            key = keys.get(behavior, "")
            trigger = phase.resolve_trigger(behavior, key, skills.get(key, "") or "")
            # 时序权威是 TriggerMeta（过滤臂的匹配模式会产生假阳性，见枚举证据 §1）
            native = meta.get(behavior, which)
            yield behavior, native, f"{loc}:{line}", skills.get(key, "") or "", trigger, classifier(native, trigger)


def package_upstreams() -> list[tuple[str, str]]:
    """[(upstream_path 片段, 来源 id)] —— 来自包数据的 `upstream_path` 字段。

    这是**证据型归因**：缺陷的触发器在哪个上游文件里找到，就归到
    「`upstream_path` 覆盖该文件的包」所属的来源。
    """
    try:
        mapper = _load("map_package_sources", ROOT / "scripts" / "map_package_sources.py")
        by_src, _names = mapper.mapping()
    except Exception:
        return []
    out: list[tuple[str, str]] = []
    for source, packages in by_src.items():
        for package in packages:
            for asset in (ROOT / "packages" / "extensions" / package / "assets").glob("*.json"):
                try:
                    doc = json.loads(asset.read_text(encoding="utf-8"))
                except Exception:
                    continue
                path = (doc.get("upstream_path") or "").strip()
                if path:
                    out.append((path.strip("/"), source))
    # 长的片段优先匹配（`extension/杀海拾遗/character/ow/` 先于 `extension/杀海拾遗/`）
    out.sort(key=lambda item: -len(item[0]))
    return out


def source_of_path(upstream_file: str) -> str:
    if not upstream_file:
        return "?"
    normalized = upstream_file.replace("\\", "/")
    for fragment, source in package_upstreams():
        if fragment and fragment in normalized:
            return source
    return "?"


def key_of(behavior: str) -> str:
    return phase.host_keys().get(behavior, "").removeprefix("host:")


def source_of(behavior: str) -> str:
    """behavior -> 来源 id（经 host 键 -> 包 -> 来源），查不到返回 `?`。"""
    try:
        mapper = _load("map_package_sources", ROOT / "scripts" / "map_package_sources.py")
        by_src, _name = mapper.mapping()
    except Exception:
        return "?"
    keys = phase.host_keys()
    key = keys.get(behavior, "").removeprefix("host:")
    # 1) 先按上游文件路径归因（证据型）
    _trigger, path = phase.upstream_trigger_with_path(
        phase.skill_names().get(f"host:{key}", "") or "", key
    )
    by_path = source_of_path(path)
    if by_path != "?":
        return by_path
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
        out.setdefault(row[3], []).append(row[0])
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--write", type=Path, help="write TSV to this path")
    ap.add_argument("--by-source", action="store_true", help="also print per-source counts")
    args = ap.parse_args()

    rows = build()
    header = ["id", "class", "behavior", "source", "location", "upstream", "native",
              "severity", "fix", "permission"]
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
