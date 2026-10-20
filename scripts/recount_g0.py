#!/usr/bin/env python3
"""对台账 §1 的 37 个来源**统一重跑**「定义/引用」统计，产出可对照的数字。

动机：台账里 L11 记「0 技能 / 0 武将」，实测却有 146 个 `lib.skill=` 定义 + 45 处 `lib.character`
（见 docs/operations/evidence/2026-10-19-unconverted-sources/l11-l12-recheck.md）。
**G0 数字口径不明，不可直接引用** ⇒ 本脚本用**同一口径**重跑全部来源。

口径（写死，便于对照）：
- **扫描范围**：台账 §1 行内链接指向的目录**整棵子树**（`*.js`）；
- **`lib.skill=`**：正则 `lib\\.skill\\.<name>\\s*=\\s*\\{` —— **定义**数；
- **`lib.card` / `lib.character` / `lib.translate`**：出现次数（**引用或定义都算**，故偏大）；
- 输出还包括 **文件数 / 总字节**，便于与 G0 的「MB」对照。

用法：
    python3 scripts/recount_g0.py            # 解析台账并统计
    python3 scripts/recount_g0.py --csv      # 输出 CSV

**注意**：`lib.card` 一类的计数**不区分定义与引用**，不能当「卡牌数」；`lib.skill=` 只匹配
「`lib.skill.X = {`」形式，`lib.skill.X = function(){…}` 之类**不计**。
"""

import json
import os
import re
import sys

LEDGER = "docs/migration/upstream-extension-reconstruction.md"
ROW = re.compile(r"^\| ?((?:U|L)\d+) \[([^\]]+)\]\(([^)]+)\)")
SKILL_DEF = re.compile(r"lib\.skill\.([A-Za-z0-9_$\u4e00-\u9fff]+)\s*=\s*\{")
OTHER = ("lib.card", "lib.character", "lib.translate")


def sources():
    out = []
    for line in open(LEDGER, encoding="utf-8"):
        m = ROW.match(line)
        if not m:
            continue
        sid, name, link = m.group(1), m.group(2), m.group(3).strip().strip("<>")
        path = os.path.normpath(os.path.join(os.path.dirname(LEDGER), link))
        # 台账链接多指向**单个文件**（`extension.js`）；G0 的统计显然是**整个扩展目录**，
        # 否则 L11 的 `extension_skill.js` 里 146 个定义会被漏掉（首版即错在此）。
        if os.path.isfile(path):
            path = os.path.dirname(path)
        out.append((sid, name, path))
    return out


def stats(path):
    """目录则扫子树；单文件则扫该文件。"""
    files = []
    if os.path.isdir(path):
        for dp, _, fs in os.walk(path):
            files += [os.path.join(dp, f) for f in fs if f.endswith(".js")]
    elif os.path.isfile(path):
        files = [path]
    total = 0
    skills = 0
    counts = {k: 0 for k in OTHER}
    for f in files:
        try:
            s = open(f, encoding="utf-8", errors="replace").read()
        except OSError:
            continue
        total += len(s.encode("utf-8", errors="replace"))
        skills += len(SKILL_DEF.findall(s))
        for k in OTHER:
            counts[k] += s.count(k)
    return len(files), total, skills, counts


def main(argv):
    as_csv = "--csv" in argv
    rows = []
    for sid, name, path in sources():
        if not os.path.exists(path):
            rows.append((sid, name, "缺失", "", "", "", "", "", ""))
            continue
        n, total, skills, c = stats(path)
        rows.append((sid, name, str(n), f"{total/1048576:.2f}M", str(skills),
                     str(c["lib.card"]), str(c["lib.character"]), str(c["lib.translate"]), path))
    if as_csv:
        print("id,name,files,bytes,skill_defs,lib.card,lib.character,lib.translate,path")
        for r in rows:
            print(",".join(r))
        return 0
    print(f"{'id':4s} {'名称':12s} {'文件':>5s} {'大小':>8s} {'skill=':>7s} {'card':>6s} {'char':>6s} {'translate':>9s}")
    for r in rows:
        print(f"{r[0]:4s} {r[1]:12s} {r[2]:>5s} {r[3]:>8s} {r[4]:>7s} {r[5]:>6s} {r[6]:>6s} {r[7]:>9s}")
    missing = [r[0] for r in rows if r[2] == "缺失"]
    print(f"\n来源数={len(rows)} 缺失={len(missing)} {missing}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
