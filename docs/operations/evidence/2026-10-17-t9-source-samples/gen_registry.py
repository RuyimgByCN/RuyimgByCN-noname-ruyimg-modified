"""重建 T9 来源行为样本登记表（依据台账 + 证据目录 + 测试文件）。

用法（在 noname-next/ 根目录）：
    python3 -B docs/operations/evidence/2026-10-17-t9-source-samples/gen_registry.py

口径：
- 台账状态按 `docs/migration/upstream-extension-reconstruction.md` 逐行实测分类：
  「八项整包」→ 整包；「部分重构」→ 部分；「G0 完成」→ G0；「待重构」→ 待重构。
- 动态样本 = 该来源的 `docs/operations/evidence/<dir>` 证据目录；
- 测试入口 = 覆盖该来源的测试文件（`crates/engine-native/tests`、`crates/content-native-loader/tests`）。
"""

import os
import re
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
LEDGER = ROOT / "docs/migration/upstream-extension-reconstruction.md"
EVIDENCE = ROOT / "docs/operations/evidence"

# 关键词 → 测试文件匹配（按来源 id）
KEYWORDS = {
    "U01": ["d3_"], "U02": ["boss_play"], "U03": ["cardpile"], "U04": ["coin"],
    "U05": ["swd_", "wudian"], "U06": ["huanle", "combo"], "U07": ["wudian"], "U08": ["u08_", "hero_kill"],
    "L01": ["ai_tuning"], "L02": ["l02_"], "L03": ["boss_play"], "L04": ["l04_", "cardpile"],
    "L05": ["l05_", "coin"], "L06": ["l06_", "wuxing"], "L07": ["l07_", "omnisearch"],
    "L08": [], "L09": ["qhly"], "L10": ["l10_", "epic_"], "L11": [], "L12": ["rzsh"],
    "L13": ["l13_"], "L14": [], "L15": ["l15_"], "L16": ["l16_", "drag_read"],
    "L17": ["l17_", "douzhuan"], "L18": ["l18_", "update_guard"], "L19": ["l19_", "doomsday"],
    "L20": ["l20_", "mark_supplement"], "L21": ["l21_", "martial"], "L22": ["l22_", "elo"],
    "L23": ["l23_", "desktop_adapt"], "L24": ["l24_", "skin_switch"], "L25": [], "L26": ["l26_", "fool"],
    "L27": ["l27_", "select_beautify"], "L28": ["thunder_loader"], "L29": ["l29_", "longxue"],
}


def ledger_rows():
    rows = []
    for line in LEDGER.read_text(encoding="utf-8").split("\n"):
        m = re.match(r"\|\s*(U\d+|L\d+)\s*\[([^\]]+)\]", line)
        if not m:
            continue
        sid, name = m.group(1), m.group(2)
        if "八项整包" in line:
            kind = "整包"
        elif "部分重构" in line:
            kind = "部分重构"
        elif "G0 完成" in line:
            kind = "G0 完成"
        elif "待重构" in line:
            kind = "待重构"
        else:
            kind = "其他"
        rows.append({"id": sid, "name": name, "kind": kind})
    return rows


def main() -> None:
    rows = ledger_rows()
    ev_dirs = sorted(d.name for d in EVIDENCE.iterdir() if d.is_dir())
    tests = sorted(
        p.name
        for base in ("crates/engine-native/tests", "crates/content-native-loader/tests")
        for p in (ROOT / base).glob("*.rs")
    )
    for row in rows:
        sid = row["id"].lower()
        row["evidence"] = sorted(d for d in ev_dirs if sid in d.lower())[-3:]
        hits = []
        for kw in KEYWORDS.get(row["id"], []):
            hits += [t for t in tests if kw in t.lower()]
        row["tests"] = sorted(set(hits))[:4]

    counts = {}
    for row in rows:
        counts[row["kind"]] = counts.get(row["kind"], 0) + 1
    with_sample = sum(1 for r in rows if r["evidence"] and r["tests"])

    lines = [
        "# T9 来源行为样本与接口登记表（实测重建）",
        "",
        f"- 口径：**{len(rows)} 来源**逐行登记「台账状态 / 动态样本证据目录 / 测试入口」。",
        "- 本表由台账 + 证据目录 + 测试文件**实测生成**（`gen_registry.py`，可复现）。",
        "- 计数：**整包 {整包} / 部分重构 {部分重构} / G0 完成 {G0 完成} / 待重构 {待重构}**（合计 {total}）；"
        "**{with_sample}/{total} 已有动态样本与测试入口**。".format(
            with_sample=with_sample, total=len(rows), **{
                k: counts.get(k, 0) for k in ("整包", "部分重构", "G0 完成", "待重构")
            }
        ),
        "",
        "| 来源 | 名称 | 台账状态 | 动态样本证据（目录） | 测试入口 |",
        "|---|---|---|---|---|",
    ]
    for row in sorted(rows, key=lambda r: r["id"]):
        ev = ", ".join("`%s`" % e for e in row["evidence"]) or "—（待补）"
        ts = ", ".join("`%s`" % t for t in row["tests"]) or "—（待补）"
        lines.append("| %s | %s | %s | %s | %s |" % (row["id"], row["name"], row["kind"], ev, ts))
    lines += [
        "",
        "## 说明",
        "",
        "- 「动态样本」= 该来源在原生侧的可复跑证据目录（含定向/八项用例日志）。",
        "- 「测试入口」= 覆盖该来源的测试文件（八项用例、批次用例或装载器用例）。",
        "- 「G0 完成 / 待重构」来源尚无原生实现，故样本与测试入口为空——G0 结论见台账各行。",
        "- 本表由脚本生成；台账状态变化后请重跑 `gen_registry.py`，不要手改计数。",
        "",
    ]
    (HERE / "README.md").write_text("\n".join(lines), encoding="utf-8")
    print("written", HERE / "README.md", "| counts:", counts, "| with_sample:", with_sample)


if __name__ == "__main__":
    main()
