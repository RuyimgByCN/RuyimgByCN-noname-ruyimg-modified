#!/usr/bin/env python3
"""上游技能块 vs 原生宿主行为：**单技能事实检查器**（第 1017 轮）。

为什么需要它：本会话三次用「文本窗口」做审计都产生了大量假信号
（字符窗口溢到相邻技能、同名跨源/跨区段），结论全部作废。
唯一可靠的做法是：

1. **大括号配对**取上游技能块（跳过字符串字面量），而不是字符窗口；
2. **来源限定**——先「技能名 → 包 → 来源 ID → 上游目录」，只在该目录里找；
3. 原生侧按 `HostBehavior::` **切块归属**（整文件正则会跨技能）。

审计四类标志：

| 类 | 上游 | 原生 |
|---|---|---|
| 可选性 | `forced: true` / `direct: true` / 都没有 | `TriggerMeta.optional` |
| 触发事件 | `trigger: { player/global/...: "X" }` | `TriggerMeta.event`（经映射表）|
| 每阶段一次 | `usable: 1` + `enable:` | `host.rs` 的 `once_per_turn` 允许表 |
| 每局一次 | `limited: true` | behavior 块里的 once/used 痕迹 |

用法：
    python3 -B scripts/audit_upstream_skill_flags.py <skill> [behavior]

**只打印事实**（上游块 + 原生元数据 + 是否在 once 允许表），**不下判定**。
理由：同名跨源/跨区段在本仓极常见（`gaizao`/`baiyi`/`datong`/`senluo` 都踩过），
批量判定会产生大量假信号；结论必须由人逐条核对行为后给出。
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ARCH = ROOT.parent / "archive/upstream"

#: 上游 `usable: 1` 但原生走 **viewAs**（transform）路径、不需要主动限次的技能
VIEWAS_SKILLS = {"huopu", "fengze", "yeying", "luweiyan"}

#: 已核实为「同名不同技」的 behavior（上游同名块属于另一个来源/区段）
NAME_COLLISIONS = {
    "Gaizao": "原生是「李香龙：自己的杀被闪避后」，上游 character/swd 的 gaizao 是装备改造",
    "BossBaiyi": "原生是 boss「败移」PhaseBegin 弃翼牌，上游 character/mobile 的 baiyi 是交换座位",
}


def skill_blocks(txt: str) -> dict[str, list[str]]:
    """按「技能键 + 大括号配对」取块（字符串字面量内的括号不算）。"""
    out: dict[str, list[str]] = {}
    for m in re.finditer(r"\n\t([A-Za-z_][A-Za-z0-9_]*)\s*:\s*\{", txt):
        start = m.end() - 1
        depth, i, in_str = 0, start, None
        while i < len(txt):
            ch = txt[i]
            if in_str:
                if ch == "\\":
                    i += 2
                    continue
                if ch == in_str:
                    in_str = None
            elif ch in "\"'`":
                in_str = ch
            elif ch == "{":
                depth += 1
            elif ch == "}":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        out.setdefault(m.group(1), []).append(txt[m.start() : i + 1])
    return out


def source_dirs() -> dict[str, str]:
    ledger = (ROOT / "docs/migration/upstream-extension-reconstruction.md").read_text(encoding="utf-8")
    out = {}
    for m in re.finditer(
        r"\|\s*([UL]\d{2})\s*\[[^\]]*\]\(<?\.\./\.\./\.\./archive/upstream/([^)>]+)", ledger
    ):
        sid, rel = m.group(1), m.group(2)
        if "/apps/core/extension/" in rel:
            base = rel.split("/apps/core/extension/")[0]
            sub = rel.split("/apps/core/extension/")[1].split("/")[0]
            out.setdefault(sid, f"{base}/apps/core/extension/{sub}")
        else:
            out.setdefault(sid, rel.rsplit("/", 1)[0])
    return out


def skill_to_source() -> dict[str, str]:
    sys.path.insert(0, str(ROOT / "scripts"))
    import report_source_coverage as R  # noqa: E402

    pkg2sid = {}
    for row in R.rows():
        for p in row[1].split(","):
            p = p.strip()
            if p:
                pkg2sid[p] = row[0]
    skill_sid = {}
    for d in sorted((ROOT / "packages/extensions").iterdir()):
        if not d.is_dir():
            continue
        sid = pkg2sid.get(d.name)
        if not sid:
            continue
        for f in (d / "assets").glob("*.json"):
            try:
                v = json.loads(f.read_text(encoding="utf-8"))
            except Exception:
                continue

            def walk(o):
                if isinstance(o, dict):
                    n = o.get("name")
                    if isinstance(n, str) and re.fullmatch(r"[a-z0-9_]{2,40}", n):
                        skill_sid.setdefault(n, sid)
                    for x in o.values():
                        walk(x)
                elif isinstance(o, list):
                    for x in o:
                        walk(x)

            walk(v)
    return skill_sid


NATIVE_EVENT_MAP = {
    "phaseEnd": "PhaseEnd",
    "phaseBegin": "PhaseBegin",
    "phaseZhunbei": "PhaseBegin",
    "phaseJudge": "PhaseBegin",
    "phaseDraw": "PhaseBegin",
    "phaseUse": "PhaseBegin",
    "phaseDiscard": "PhaseBegin",
    "damageBegin1": "DamageBefore",
    "damageBegin2": "DamageBefore",
    "damageBegin3": "DamageBefore",
    "damageBegin4": "DamageBefore",
    "damageEnd": "Damage",
    "dieBegin": "Death",
    "die": "Death",
    "useCard": "CardUsed",
    "useCardToBefore": "CardUsed",
    "useCardToBegin": "CardUsed",
    "useCardToTargeted": "CardUsed",
    "shaBegin": "CardUsed",
    "shaAfter": "CardUsed",
    "loseHpEnd": "LoseHp",
    "recoverEnd": "Recover",
    "drawBegin": "CardsDrawn",
    "drawEnd": "CardsDrawn",
    "equipEnd": "Equipped",
    "linkEnd": "Linked",
    "turnOver": "TurnOvered",
}


def native_metas() -> dict[str, dict]:
    """按 HostBehavior 切块归属（整文件正则会跨技能）。"""
    out: dict[str, dict] = {}
    for f in sorted((ROOT / "crates/engine-native-kernel/src/vocabulary/host/metadata").glob("*.rs")):
        t = f.read_text(encoding="utf-8")
        marks = [m.start() for m in re.finditer(r"HostBehavior::(\w+)\s*=>", t)]
        for i, st in enumerate(marks):
            end = marks[i + 1] if i + 1 < len(marks) else len(t)
            blk = t[st:end]
            beh = re.match(r"HostBehavior::(\w+)", blk).group(1)
            ev = re.search(r"event:\s*EventKind::(\w+)", blk)
            opt = re.search(r"optional:\s*(true|false)", blk)
            if not ev or beh in out:
                continue
            out[beh] = {"event": ev.group(1), "optional": opt.group(1) == "true" if opt else None}
    return out


def native_blocks() -> dict[str, str]:
    out: dict[str, str] = {}
    for sub in ("filtering", "step", "activation"):
        for f in sorted((ROOT / "crates/engine-native-content/src/content/host" / sub).glob("*.rs")):
            t = f.read_text(encoding="utf-8")
            for m in re.finditer(r"HostBehavior::(\w+)\s*=>", t):
                nxt = re.search(r"HostBehavior::\w+\s*=>", t[m.end() :])
                end = m.end() + (nxt.start() if nxt else len(t) - m.end())
                out[m.group(1)] = out.get(m.group(1), "") + t[m.start() : end]
    return out


def host_keys() -> dict[str, str]:
    t = (ROOT / "crates/engine-native-kernel/src/vocabulary/host/keys.rs").read_text(encoding="utf-8")
    return {
        beh: name
        for beh, _k, name in re.findall(r'HostBehavior::(\w+)\s*=>\s*"(host:([a-z0-9_]+))"', t)
    }


def once_allowlist() -> set[str]:
    t = (ROOT / "crates/engine-native-content/src/content/host.rs").read_text(encoding="utf-8")
    m = re.search(r"let limits = match behavior \{(.*?)\n    \};", t, re.S)
    return set(re.findall(r"HostBehavior::(\w+)", m.group(1))) if m else set()


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    skill = sys.argv[1]
    sid = skill_to_source().get(skill)
    sdirs = source_dirs()
    print(f"技能: {skill}  来源: {sid or '(未解析)'}")
    if sid and sid in sdirs:
        d = ARCH / sdirs[sid]
        for f in d.rglob("skill.js"):
            try:
                txt = f.read_text(encoding="utf-8", errors="ignore")
            except Exception:
                continue
            for blk in skill_blocks(txt).get(skill, []):
                print(f"\n--- 上游块 {f.relative_to(ARCH)} ---")
                print(" ".join(blk.split())[:1200])
    metas = native_metas()
    for beh, name in host_keys().items():
        if name != skill:
            continue
        m = metas.get(beh)
        print(f"\n--- 原生 {beh} ---")
        if m:
            print(f"  event={m['event']}  optional={m['optional']}")
        else:
            print("  (无显式 TriggerMeta)")
        print(f"  in once_per_turn 允许表: {beh in once_allowlist()}")
        blk = native_blocks().get(beh, "")
        if blk:
            print(f"  原生块长度 {len(blk)} 字符；含 once/used 痕迹: {bool(re.search(r'once|_used', blk))}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
