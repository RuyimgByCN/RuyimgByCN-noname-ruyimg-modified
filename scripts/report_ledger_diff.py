#!/usr/bin/env python3
"""对比台账两个版本的行为清单（移出/新增/保留）。

**为什么需要**：第 912 轮手算「54 → 40 的构成」怎么也对不上，改用 `git` 差分后才
发现 `SwdMufengHeld` 的**效果缺陷**随作用域修正一起消失了。手算会漏，机器不会。

用法：
    python3 -B scripts/report_ledger_diff.py            # 与 HEAD 的台账对比
    python3 -B scripts/report_ledger_diff.py --rev Xxx  # 与指定提交对比
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LEDGER = "docs/operations/evidence/2026-10-19-defect-ledger/ledger.tsv"


def behaviors(text: str) -> dict[str, str]:
    """行文本 → {行为名: id}（跳过表头）。"""
    out: dict[str, str] = {}
    for line in text.splitlines()[1:]:
        if not line.strip():
            continue
        parts = line.split("\t")
        if len(parts) >= 3:
            out[parts[2]] = parts[0]
    return out


def git_show(rev: str) -> str:
    result = subprocess.run(
        ["git", "show", f"{rev}:{LEDGER}"],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        return ""
    return result.stdout


def diff(old: dict[str, str], new: dict[str, str]) -> tuple[list[str], list[str], list[str]]:
    removed = sorted(set(old) - set(new))
    added = sorted(set(new) - set(old))
    kept = sorted(set(old) & set(new))
    return removed, added, kept


def main(argv: list[str] | None = None) -> int:
    args = sys.argv[1:] if argv is None else argv
    rev = "HEAD"
    if "--rev" in args:
        rev = args[args.index("--rev") + 1]
    old = behaviors(git_show(rev))
    current = (ROOT / LEDGER).read_text(encoding="utf-8")
    new = behaviors(current)
    if not old:
        print(f"⚠ 取不到 {rev} 的台账（可能路径或引用不存在）")
        return 1
    removed, added, kept = diff(old, new)
    print(f"{rev}: {len(old)} 行 → 现在: {len(new)} 行（保留 {len(kept)}）")
    print(f"移出 {len(removed)} 行：{' '.join(removed) if removed else '（无）'}")
    print(f"新增 {len(added)} 行：{' '.join(added) if added else '（无）'}")
    if removed:
        print("⚠ 移出的每一行都必须在台账 README 里写明原因（第 912 轮的教训）")
    return 0


if __name__ == "__main__":
    sys.exit(main())
