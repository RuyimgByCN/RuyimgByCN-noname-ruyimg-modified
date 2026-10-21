#!/usr/bin/env python3
"""G14 守卫：列出「原生挂 `Phase::Judge, begin=true`」的行为，并回查上游 `trigger`。

**背景**：上游 `phaseBegin`（任意阶段开始）被系统性映射成 `Phase::Judge`（判定阶段），
本会话确认 8 条。此脚本把「查哪些、上游是什么」变成**可复现的证据**，
避免结论只存在于会话记录里。

链路：`filtering*.rs` 的 filter → `keys.rs` 的 `host:` 键 → 包数据 `skills[].mapping`
→ 上游 `skill.js`/`mode/*.js` 的 `trigger`。

用法：
    python3 -B scripts/report_g14.py            # 打印表
    python3 -B scripts/report_g14.py --tsv F    # 写 TSV
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
HOST = ROOT / "crates/engine-native/src/content/host"
UPSTREAM = ROOT.parent / "archive/upstream"
PACKAGES = ROOT / "packages/extensions"

PHASE = re.compile(r"phase:\s*crate::event::Phase::Judge,\s*begin:\s*true")
BEHAVIOR = re.compile(r"HostBehavior::(\w+)\s*=>")
KEY = re.compile(r'HostBehavior::(\w+)\s*=>\s*"host:([^"]+)"')
TRIGGER = re.compile(r"trigger\s*:\s*\{([^}]*)\}")

# 人工已核（key 名与上游名不同 / 由 case-by-case 代码授予，自动链路查不到）。
# 值：(上游技能名, trigger 原文, 依据)
MANUAL: dict[str, tuple[str, str, str]] = {
    "diyguhuo": ("diyguhuo", 'player: "phaseBegin"', "v574 手工核对"),
    "gujian_food_yougeng": ("yougeng", 'player: "phaseBegin"', "v574 手工核对（卡牌效果宿主）"),
    "huanle_xuelunyang": ("xuelunyang", 'player: "phaseBegin"', "v574 手工核对"),
    "baiyi": ("baiyi", "(无 trigger，enable 主动技)", "v587 手工核对"),
    # 第 721 轮更正：`lexue` 上游是复合技（`group:["lexue1","lexue2"]`），父技能无
    # trigger，但子技能 `lexue1: { player: "phaseBegin" }` 才是真实触发面 ⇒ 归 G14。
    "lexue": ("lexue1", 'player: "phaseBegin"', "v721 更正（复合技，子技能 lexue1）"),
    "zhenjiu2": ("zhenjiu2", 'player: "phaseBegin"', "v587 手工核对"),
    "huajin2": ("huajin2", 'player: "damageBegin"', "v587 手工核对"),
    "hlss_fengqi": ("fengqi", "player: ['phaseZhunbeiBegin','phaseJieshuBegin']", "v608 手工核对（mode/boss.js）"),
}


def judge_behaviors() -> list[str]:
    out: list[str] = []
    files = sorted(HOST.glob("filtering*.rs")) + sorted(HOST.glob("filtering/*.rs"))
    for f in files:
        text = f.read_text(encoding="utf-8")
        marks = [(m.start(), m.group(1)) for m in BEHAVIOR.finditer(text)]
        for i, (pos, beh) in enumerate(marks):
            end = marks[i + 1][0] if i + 1 < len(marks) else len(text)
            if PHASE.search(text[pos:end]):
                out.append(beh)
    return out


def host_keys() -> dict[str, str]:
    return {m.group(1): m.group(2) for m in KEY.finditer((HOST / "keys.rs").read_text(encoding="utf-8"))}


def package_skill_names() -> dict[str, str]:
    """host:key -> 包内 skills[].name"""
    out: dict[str, str] = {}
    for p in PACKAGES.glob("*/assets/*.json"):
        try:
            data = json.loads(p.read_text(encoding="utf-8"))
        except Exception:
            continue

        def walk(node: object) -> None:
            if isinstance(node, dict):
                mp, nm = node.get("mapping"), node.get("name")
                if isinstance(mp, str) and isinstance(nm, str) and mp.startswith("host:"):
                    out.setdefault(mp[5:], nm)
                for v in node.values():
                    walk(v)
            elif isinstance(node, list):
                for v in node:
                    walk(v)

        walk(data)
    return out


def upstream_triggers() -> dict[str, str]:
    """上游技能名 -> trigger 原文（扫 skill.js 与 mode/*.js）"""
    out: dict[str, str] = {}
    pats = ("**/skill.js", "**/mode/*.js", "**/extension.js")
    for pat in pats:
        for f in UPSTREAM.glob(pat):
            text = f.read_text(encoding="utf-8", errors="ignore")
            for m in re.finditer(r"\n\s*([A-Za-z_]\w*)\s*:\s*\{", text):
                name = m.group(1)
                if name in out:
                    continue
                i = m.end() - 1
                depth = 0
                k = i
                while k < len(text):
                    if text[k] == "{":
                        depth += 1
                    elif text[k] == "}":
                        depth -= 1
                        if depth == 0:
                            k += 1
                            break
                    k += 1
                tr = TRIGGER.search(text[i:k])
                if tr:
                    out[name] = re.sub(r"\s+", " ", tr.group(1).strip())
    return out


def main(argv: list[str]) -> int:
    keys, names, triggers = host_keys(), package_skill_names(), upstream_triggers()
    rows = []
    for beh in judge_behaviors():
        key = keys.get(beh, "")
        skill = names.get(key, "")
        trig = triggers.get(skill, "") if skill else ""
        src = "自动"
        if key in MANUAL and not trig:
            skill, trig, src = MANUAL[key][0], MANUAL[key][1], MANUAL[key][2]
        if not trig:
            verdict = "未定位"
        elif trig.startswith("(无"):
            verdict = "不符(凭空新增触发)"  # 上游无 trigger，原生却挂了判定阶段触发
        elif "phaseBegin" in trig:
            verdict = "G14(不符)"  # 上游任意阶段开始，原生却挂判定阶段
        elif "phaseZhunbeiBegin" in trig:
            verdict = "不符(准备阶段)"  # 原生无 Prepare 档（G18）
        else:
            verdict = "不符(非阶段类)"  # 上游是 damageBegin 等非阶段触发
        rows.append((beh, key, skill, trig, verdict))

    tsv = ["行为\tkey\t包内技能名\t上游trigger\t判定"]
    tsv += ["\t".join(r) for r in rows]
    text = "\n".join(tsv) + "\n"
    if "--tsv" in argv:
        Path(argv[argv.index("--tsv") + 1]).write_text(text, encoding="utf-8")
        print(f"写入 {argv[argv.index('--tsv') + 1]}")
        return 0
    print(text)
    from collections import Counter

    c = Counter(r[4] for r in rows)
    print(f"Judge,begin=true 共 {len(rows)} 条：{dict(c)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
