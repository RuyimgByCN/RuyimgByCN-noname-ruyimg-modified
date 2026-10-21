#!/usr/bin/env python3
"""在主稿里查找**已被更正但仍残留的旧数字**。

**为什么需要**：本会话发生过多次「改一半」——比如把「规则差异 5/32」改成 0/32 后，
其它文件里仍写着 5/32；或者「组合 13/32」只在摘要行改了、表格没改。
人工反查容易漏，做成脚本可复现。

**关键难点是「不该报的三种行」**（否则误报会淹没有效信息）：
1. **轮次日志**：`> **第 N 轮**：…` —— 当时确实是那个数，属历史快照；
2. **划线更正**：含 `~~旧值~~` 的行 —— 这正是更正记录本身；
3. **说明性更正**：含「已改为 / 原写 / 此前写 / 上一轮写 / 改为」的行。

用法：
    python3 -B scripts/check_stale_numbers.py            # 检查主稿
    python3 -B scripts/check_stale_numbers.py --list      # 只列出旧值表
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# 默认检查这 6 份主稿（与 check_local_links.CORE 同源）
CORE = [
    "docs/architecture/native-progress.md",
    "docs/architecture/native-rules-baseline.md",
    "docs/migration/upstream-extension-reconstruction.md",
    "docs/operations/evidence/2026-10-19-session-handover/README.md",
    "docs/operations/evidence/2026-10-19-rules-diff-handover/README.md",
    "docs/operations/evidence/2026-10-19-source-signoff-32/README.md",
]

# 旧值 -> (现值, 说明)
STALE: dict[str, tuple[str, str]] = {
    "规则差异 5/32": ("规则差异 0/32", "5/32 是生命周期证据覆盖数，已改四态实测"),
    "规则差异 **5/32": ("规则差异 0/32", "同上"),
    "组合 13/32": ("组合 11/32", "逐行统计后为 11（补 boss-zhugeliang 后 12）"),
    "未核 17": ("未核 1", "17 是算术错误（32+17≠52），且后续已核到 46"),
    "已核 32": ("已核 46", "52 处已核 46 / 不适用 5 / 未核 1"),
    "22 个有资产": ("15 个有资产", "22 是「可装载」，15 才是「有 cards/generals」"),
    "54 行有守卫": ("有守卫 53", "54 是台账总数，有守卫的是 53（缺 SwdBaishouzhihuHeld）"),
    "覆盖率 98.2%": ("覆盖率 98.1%", "98.2% 是把总数当有守卫数算出来的；脚本实测 98.1%"),
    "50 个 `#[ignore]`": ("51 个 `#[ignore]`", "第 888 轮新增 Tianxiang 效果守卫后为 51"),
    "50 failed": ("51 failed", "同上"),
    "等价 40": ("等价 42", "第 892 轮两条 GujianFood 升级为效果等价"),
    "形态 13": ("形态 10", "同上"),
    "等价 40 / 形态相符 13": ("等价 42 / 形态相符 10", "同上"),
}

# 这些子串出现时，该行属于「合法残留」
EXCUSE = ("~~", "已改为", "已改", "原写", "此前写", "上一轮写", "改为", "作废", "实际是", "当时", "现为")
# 轮次日志行
ROUND_LOG = re.compile(r"^>?\s*(?:-\s*)?\*\*第 \d+ 轮")


def violations(path: Path) -> list[tuple[int, str, str, str]]:
    """返回 (行号, 旧值, 现值, 行内容)。"""
    out: list[tuple[int, str, str, str]] = []
    for i, line in enumerate(path.read_text(encoding="utf-8", errors="ignore").splitlines(), 1):
        if ROUND_LOG.match(line.strip()):
            continue
        if any(x in line for x in EXCUSE):
            continue
        for old, (new, _why) in STALE.items():
            if old in line:
                out.append((i, old, new, line.strip()[:100]))
    return out


def main(argv: list[str]) -> int:
    if "--list" in argv:
        for old, (new, why) in STALE.items():
            print(f"{old!r} -> {new!r}  ({why})")
        return 0
    total = 0
    for rel in CORE:
        f = ROOT / rel
        if not f.exists():
            continue
        for ln, old, new, line in violations(f):
            total += 1
            print(f"{rel}:{ln}  {old!r} 应为 {new!r}")
            print(f"    {line}")
    print(f"残留旧值 {total} 处")
    return 1 if total else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
