#!/usr/bin/env python3
"""Regenerate docs/content-fidelity-ledger.md and the detail companion.

Classification (mutually exclusive for 近似/TODO):
  - TODO: file contains a `// TODO` comment
  - 近似: file contains a `// 近似` comment and no TODO
  - 完整: neither
INTENTIONAL_EMPTY_NOOP is a separate axis and may overlap the three classes.
"""

from __future__ import annotations

import argparse
import datetime as dt
import re
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PORTED = ROOT / "content" / "original" / "ported"
SUMMARY_OUT = ROOT / "docs" / "content-fidelity-ledger.md"
DETAIL_OUT = ROOT / "docs" / "content-fidelity-ledger-detail.md"

APPROX_RE = re.compile(r"//\s*近似")
TODO_RE = re.compile(r"//\s*TODO")
NOOP_RE = re.compile(r"INTENTIONAL_EMPTY_NOOP")
SKILL_NAME_RE = re.compile(r"skill\s+(\w+)\s*\(")
EMPTY_HINT_RE = re.compile(
    r"charlotte|audio-only|仅音频|空定义|无规则面|标记型技能", re.I
)


def first_comment_note(text: str, marker: str) -> str:
    for line in text.splitlines():
        if marker in line:
            note = line.split(marker, 1)[1].strip(" ：:")
            return note[:180]
    return ""


def classify(text: str) -> tuple[str, bool, bool]:
    has_todo = bool(TODO_RE.search(text))
    has_approx = bool(APPROX_RE.search(text))
    has_noop = bool(NOOP_RE.search(text))
    empty_upstream = bool(EMPTY_HINT_RE.search(text)) if has_noop else False
    if has_todo:
        kind = "todo"
    elif has_approx:
        kind = "approx"
    else:
        kind = "complete"
    return kind, has_noop, empty_upstream


def scan() -> list[dict]:
    rows = []
    for path in sorted(PORTED.rglob("*.skill")):
        # archive/ 子目录存放已退出编译池的历史遗留（rebuild-pack 的非递归
        # glob 不装入 npack），不属于保真度登记范围。
        if "archive" in path.relative_to(PORTED).parts:
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        kind, has_noop, empty_upstream = classify(text)
        rel = path.relative_to(ROOT).as_posix()
        pack = path.relative_to(PORTED).parts[0]
        m = SKILL_NAME_RE.search(text)
        name = m.group(1) if m else path.stem
        note = ""
        if kind == "todo":
            note = first_comment_note(text, "TODO")
        elif kind == "approx":
            note = first_comment_note(text, "近似")
        rows.append(
            {
                "pack": pack,
                "name": name,
                "path": rel,
                "kind": kind,
                "noop": has_noop,
                "empty_upstream": empty_upstream,
                "note": note,
            }
        )
    return rows


def write_summary(rows: list[dict], today: str) -> None:
    total = len(rows)
    kinds = Counter(r["kind"] for r in rows)
    complete, approx, todo = kinds["complete"], kinds["approx"], kinds["todo"]
    incomplete = approx + todo
    noop = sum(1 for r in rows if r["noop"])
    empty = sum(1 for r in rows if r["empty_upstream"])
    real_noop = noop - empty

    by_pack: dict[str, Counter] = defaultdict(Counter)
    for r in rows:
        by_pack[r["pack"]]["total"] += 1
        by_pack[r["pack"]][r["kind"]] += 1
        if r["noop"]:
            by_pack[r["pack"]]["noop"] += 1
        if r["empty_upstream"]:
            by_pack[r["pack"]]["empty"] += 1

    pack_rows = []
    for pack in sorted(by_pack):
        d = by_pack[pack]
        inc = d["approx"] + d["todo"]
        rate = (100.0 * inc / d["total"]) if d["total"] else 0.0
        pack_rows.append((rate, pack, d, inc))
    pack_rows.sort(key=lambda x: (-x[0], x[1]))

    lines = [
        "# 内容保真度清单（汇总）",
        "",
        "> 配套明细：[`content-fidelity-ledger-detail.md`](content-fidelity-ledger-detail.md)",
        "> （机器生成：`python3 scripts/gen-fidelity-ledger.py`；含路径与省略说明原文）。",
        ">",
        f"> 生成日期：{today}。扫描范围：`content/original/ported/**/*.skill`（排除 `archive/` 子目录）。",
        "",
        "## 背景",
        "",
        "架构计划 §47.4 要求内容例外「必须极少并走 ADR」。本清单按 `.skill` 源码注释",
        "登记当前保真度边界：`// 近似` / `// TODO` 在编译期被 lexer 跳过，不进入 IR，",
        "不影响 `required_rule_vm_abi`。`INTENTIONAL_EMPTY_NOOP` 是另一条轴——目录可",
        "标 `ported`、npack 可装入，但运行时无规则面。",
        "",
        "**本清单是纯登记，不改任何 .skill 内容、不动 ABI、不动 golden。**",
        "",
        "## 总览",
        "",
        "| 类别 | 数量 | 占比 | 性质 |",
        "|---|---|---|---|",
        f"| 完整 | {complete} | {100*complete/total:.1f}% | 无 `// 近似` / `// TODO` 注释 |",
        f"| 近似 | {approx} | {100*approx/total:.1f}% | 已实现但简化：旧 JS 某语义以更简单效果代替 |",
        f"| TODO | {todo} | {100*todo/total:.1f}% | 骨架占位：DSL 壳已建，核心逻辑待转换 |",
        f"| **合计** | **{total}** | **100%** | — |",
        "",
        f"近似与 TODO 互斥（交集 = 0）。不完整合计 **{incomplete}**（{100*incomplete/total:.1f}%）。",
        "",
        "## 空实现（独立轴，可与上表重叠）",
        "",
        f"含 `INTENTIONAL_EMPTY_NOOP` 的 `.skill`：**{noop}**。",
        f"其中约 **{empty}** 条注释标明上游 charlotte / audio-only / 空定义 / 无规则面；",
        f"其余约 **{real_noop}** 条是「上游有真实规则体，但当前 DSL 原语不足以精确建模」的诚实空实现。",
        "空实现**不能**计作可玩语义闭环；目录层 `migration_status: ported` 只表示清单闭包。",
        "",
        "## 两类不完整的本质区别",
        "",
        f"### 近似（{approx}）—— 已实现，有保真度差距",
        "",
        "这些技能有可运行的 DSL 实现，但用简化效果近似了旧 JS 的复杂语义。风险较低：",
        "技能能跑，只是行为与原版有差距。补全优先级取决于实战重要性。",
        "",
        f"### TODO（{todo}）—— 占位，核心未填",
        "",
        "DSL 骨架已建，但核心逻辑仍是 `// TODO` 注释。触发时可能无效果或不完整。",
        "",
        "## 按包分布（不完整率降序）",
        "",
        "| 包 | 总数 | 完整 | 近似 | TODO | 不完整率 | noop |",
        "|---|---|---|---|---|---|---|",
    ]
    for rate, pack, d, inc in pack_rows:
        lines.append(
            f"| {pack} | {d['total']} | {d['complete']} | {d['approx']} | {d['todo']} | "
            f"{rate:.0f}% | {d['noop']} |"
        )
    lines += [
        "",
        "## 建议的后续处理",
        "",
        "1. **本清单作为基线**：新移植技能若需近似，应走 ADR（§47.4），不得无声新增。",
        "   定期重跑 `python3 scripts/gen-fidelity-ledger.py` 核对数字变化。",
        "2. **TODO 专项**：已完成。清单内 `// TODO` 为 0；可运行合同用既有 Host 原语写完，",
        "   需要 chooseToUse / gaintag / expansion / 额外回合等新原语的改为诚实 noop。",
        "3. **诚实 noop + 近似（当前主线）**：上游空定义可维持；有真实规则体的 noop",
        "   与近似按缺失原语杠杆排序改写。",
        "4. **登记而非修改变更**：本脚本不触碰任何 `.skill` 内容。",
        "",
    ]
    SUMMARY_OUT.write_text("\n".join(lines), encoding="utf-8")


def write_detail(rows: list[dict], today: str) -> None:
    total = len(rows)
    kinds = Counter(r["kind"] for r in rows)
    incomplete = [r for r in rows if r["kind"] != "complete"]
    lines = [
        "# 内容保真度明细清单（机器生成，勿手改）",
        "",
        f"> 生成自 `python3 scripts/gen-fidelity-ledger.py`（{today}）。",
        "> 每个不完整 `.skill` 的 `// 近似` 或 `// TODO` 注释原文收录。complete 类只计数不列。",
        "",
        f"**总计**：{total} 个 `.skill`，其中近似 {kinds['approx']}、TODO {kinds['todo']}、"
        f"完整 {kinds['complete']}。",
        "",
    ]
    by_pack: dict[str, list[dict]] = defaultdict(list)
    for r in incomplete:
        by_pack[r["pack"]].append(r)
    pack_totals = Counter(r["pack"] for r in rows)
    for pack in sorted(by_pack):
        items = by_pack[pack]
        lines.append(f"## 包 `{pack}`（{pack_totals[pack]} 个，不完整 {len(items)}）")
        lines.append("")
        for kind, title in (("approx", "近似"), ("todo", "TODO")):
            subset = [r for r in items if r["kind"] == kind]
            if not subset:
                continue
            lines.append(f"### {title}（{len(subset)}）")
            lines.append("")
            lines.append("| 技能 | 路径 | 省略说明 |")
            lines.append("|---|---|---|")
            for r in subset:
                note = (r["note"] or "").replace("|", "\\|")
                lines.append(f"| `{r['name']}` | {r['path']} | {note} |")
            lines.append("")
    DETAIL_OUT.write_text("\n".join(lines), encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--date", default=dt.date.today().isoformat())
    args = parser.parse_args()
    rows = scan()
    write_summary(rows, args.date)
    write_detail(rows, args.date)
    kinds = Counter(r["kind"] for r in rows)
    print(
        f"wrote {SUMMARY_OUT.relative_to(ROOT)} and {DETAIL_OUT.relative_to(ROOT)}: "
        f"{len(rows)} files, complete={kinds['complete']} approx={kinds['approx']} "
        f"todo={kinds['todo']} noop={sum(1 for r in rows if r['noop'])}"
    )


if __name__ == "__main__":
    main()
