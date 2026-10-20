#!/usr/bin/env python3
"""把上游扩展里的 `lib.skill.X = {...}` 定义**按语义分类**（规则 / UI / 标记 / 重复）。

动机（docs/operations/evidence/2026-10-19-unconverted-sources/）：
G0 台账里的「N 个技能」**不能直接当「待迁移技能数」**——抽样发现 L14 的最小三个「技能」
全是 `ui.autonode.hide()` / `game.as_removeImage()` / 操作 `playertip` 的 **UI 钩子**，
L08 的最小条目是 34–50 字节的**标记桩**，且同一技能在多个文件里**重复定义**。

用法：
    python3 scripts/classify_upstream_skills.py <扩展目录> [...]

输出：每个来源的 `总定义数 / 去重后 / 规则 / UI / 标记 / 其它`，以及规则技能的名单（前若干个）。
**口径**（全部基于文本特征，**不是**语义理解）：
- UI：块内出现 `ui.` / `document.` / `getElementsByClassName` / `innerHTML` / `game.as_removeImage`
      / `playertip` / `style.` / `appendChild`
- 标记：块长 < 120 字节，或出现 `mark:true` / `markimage` / `intro:`
- 规则：块内出现 `content:` 且出现 `player.` / `game.` / `effect` / `damage` / `draw` / `recover`
- 其它：以上都不满足
"""

import os
import re
import sys

UI_TOKENS = ("ui.", "document.", "getElementsByClassName", "innerHTML",
             "game.as_removeImage", "playertip", "style.", "appendChild")
MARK_TOKENS = ("mark:true", "markimage", "intro:")
RULE_TOKENS = ("content:", "player.", "game.", "effect", "damage", "draw", "recover")


def blocks(text):
    """产出 (name, body)。用**括号配平**截取块体，避免固定长度截断。"""
    for m in re.finditer(r"lib\.skill\.([A-Za-z0-9_$\u4e00-\u9fff]+)\s*=\s*", text):
        i = m.end()
        if i >= len(text) or text[i] != "{":
            continue
        depth = 0
        j = i
        while j < len(text):
            c = text[j]
            if c == "{":
                depth += 1
            elif c == "}":
                depth -= 1
                if depth == 0:
                    j += 1
                    break
            j += 1
        yield m.group(1), text[i:j]


def classify(body):
    if any(t in body for t in UI_TOKENS):
        return "ui"
    if len(body) < 120 or any(t in body for t in MARK_TOKENS):
        return "mark"
    if "content:" in body and any(t in body for t in RULE_TOKENS[1:]):
        return "rule"
    return "other"


def scan(root):
    total = 0
    seen = {}
    for dp, _, fs in os.walk(root):
        for f in fs:
            if not f.endswith(".js"):
                continue
            p = os.path.join(dp, f)
            try:
                s = open(p, encoding="utf-8", errors="replace").read()
            except OSError:
                continue
            for name, body in blocks(s):
                total += 1
                key = (name, re.sub(r"\s+", "", body))
                if key in seen:
                    seen[key]["copies"] += 1
                else:
                    seen[key] = {"kind": classify(body), "copies": 1,
                                 "file": os.path.relpath(p, root), "len": len(body)}
    return total, seen


def main(argv):
    if not argv:
        print(__doc__)
        return 2
    for root in argv:
        if not os.path.isdir(root):
            print(f"{root}: 不是目录")
            continue
        total, seen = scan(root)
        kinds = {"rule": 0, "ui": 0, "mark": 0, "other": 0}
        for v in seen.values():
            kinds[v["kind"]] += 1
        print(f"== {os.path.basename(root.rstrip('/'))}")
        print(f"   总定义数={total}  去重后={len(seen)}  规则={kinds['rule']}  "
              f"UI={kinds['ui']}  标记={kinds['mark']}  其它={kinds['other']}")
        rules = [(k[0], v) for k, v in seen.items() if v["kind"] == "rule"]
        rules.sort(key=lambda kv: kv[1]["len"])
        for name, v in rules[:5]:
            print(f"   规则样例: {name} ({v['len']}B, {v['file']}, 副本 {v['copies']})")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
